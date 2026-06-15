// -------------------------------------------------------------------------------------------------
//  Copyright (C) 2015-2026 Nautech Systems Pty Ltd. All rights reserved.
//  https://nautechsystems.io
//
//  Licensed under the GNU Lesser General Public License Version 3.0 (the "License");
//  You may not use this file except in compliance with the License.
//  You may obtain a copy of the License at https://www.gnu.org/licenses/lgpl-3.0.en.html
//
//  Unless required by applicable law or agreed to in writing, software
//  distributed under the License is distributed on an "AS IS" BASIS,
//  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//  See the License for the specific language governing permissions and
//  limitations under the License.
// -------------------------------------------------------------------------------------------------

//! HTTP client for the tastytrade REST API.
//!
//! Handles OAuth2 access-token refresh (with a freshness buffer and a refresh
//! lock to avoid thundering-herd refreshes), request construction, retry on
//! idempotent operations, and JSON response parsing. REST is the authoritative
//! source of truth for the adapter; the WebSocket streams are layered on top.

use std::{
    collections::HashMap,
    num::NonZeroU32,
    sync::{Arc, LazyLock},
    time::{Duration, Instant},
};

use arc_swap::{ArcSwap, ArcSwapOption};
use nautilus_core::consts::NAUTILUS_USER_AGENT;
use nautilus_network::{
    http::{HttpClient, HttpClientError, HttpResponse, Method, USER_AGENT},
    ratelimiter::quota::Quota,
    retry::{RetryConfig, RetryManager},
};
use serde_json::{Value, json};
use tokio_util::sync::CancellationToken;

use crate::{
    common::{
        consts::{
            API_QUOTE_TOKENS_PATH, CUSTOMER_ACCOUNTS_PATH, DEFAULT_HTTP_TIMEOUT_SECS,
            DEFAULT_REST_QUOTA_PER_SECOND, OAUTH_TOKEN_PATH, SESSION_TOKEN_TTL_SECS, SESSIONS_PATH,
            TOKEN_REFRESH_BUFFER_SECS,
        },
        credential::{AuthScheme, TastytradeCredential},
        enums::TastytradeEnvironment,
        urls,
    },
    http::{
        error::{Error, Result},
        models::{
            AccountsResponse, OAuthTokenResponse, QuoteTokenData, QuoteTokenResponse,
            SessionResponse,
        },
    },
};

/// Default tastytrade REST rate-limit budget (conservative placeholder).
static TASTYTRADE_REST_QUOTA: LazyLock<Quota> = LazyLock::new(|| {
    Quota::per_second(NonZeroU32::new(DEFAULT_REST_QUOTA_PER_SECOND).expect("non-zero"))
        .expect("valid quota")
});

/// Returns the default retry configuration for the tastytrade HTTP client.
#[must_use]
pub fn default_retry_config() -> RetryConfig {
    RetryConfig {
        max_retries: 3,
        initial_delay_ms: 100,
        max_delay_ms: 5_000,
        backoff_factor: 2.0,
        jitter_ms: 250,
        operation_timeout_ms: Some(60_000),
        immediate_first: false,
        max_elapsed_ms: Some(180_000),
    }
}

/// A cached OAuth access token with its monotonic expiry instant.
#[derive(Debug, Clone)]
struct TokenState {
    access_token: String,
    expires_at: Instant,
}

/// HTTP client for low-level tastytrade REST operations.
///
/// Manages OAuth2 token lifecycle and authenticated request execution. A single
/// instance is intended to be shared between the data and execution clients.
#[derive(Debug)]
pub struct TastytradeHttpClient {
    client: HttpClient,
    credential: Option<TastytradeCredential>,
    base_url: ArcSwap<String>,
    environment: TastytradeEnvironment,
    accept_version: Option<String>,
    token_refresh_buffer: Duration,
    access_token: ArcSwapOption<TokenState>,
    refresh_lock: tokio::sync::Mutex<()>,
    retry_manager: RetryManager<Error>,
    cancellation_token: CancellationToken,
}

impl TastytradeHttpClient {
    /// Creates a new [`TastytradeHttpClient`] with the given credentials.
    ///
    /// # Errors
    ///
    /// Returns an error if the underlying HTTP client cannot be created.
    pub fn new(
        credential: Option<TastytradeCredential>,
        environment: TastytradeEnvironment,
        accept_version: Option<String>,
        timeout_secs: u64,
        token_refresh_buffer_secs: u64,
        retry_config: Option<RetryConfig>,
    ) -> std::result::Result<Self, HttpClientError> {
        Ok(Self {
            client: HttpClient::new(
                Self::default_headers(),
                vec![],
                vec![],
                Some(*TASTYTRADE_REST_QUOTA),
                Some(timeout_secs),
                None,
            )?,
            credential,
            base_url: ArcSwap::from_pointee(urls::rest_url(environment).to_string()),
            environment,
            accept_version,
            token_refresh_buffer: Duration::from_secs(token_refresh_buffer_secs),
            access_token: ArcSwapOption::empty(),
            refresh_lock: tokio::sync::Mutex::new(()),
            retry_manager: RetryManager::new(retry_config.unwrap_or_else(default_retry_config)),
            cancellation_token: CancellationToken::new(),
        })
    }

    /// Creates an authenticated client from environment variables.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Auth`] if the required environment variables are unset.
    pub fn from_env(environment: TastytradeEnvironment) -> Result<Self> {
        let credential = TastytradeCredential::from_env()?;
        Self::new(
            Some(credential),
            environment,
            None,
            DEFAULT_HTTP_TIMEOUT_SECS,
            TOKEN_REFRESH_BUFFER_SECS,
            None,
        )
        .map_err(|e| Error::auth(format!("Failed to create HTTP client: {e}")))
    }

    /// Returns the configured environment.
    #[must_use]
    pub fn environment(&self) -> TastytradeEnvironment {
        self.environment
    }

    /// Returns `true` if this client has credentials for authenticated requests.
    #[must_use]
    pub fn is_authenticated(&self) -> bool {
        self.credential.is_some()
    }

    /// Returns the cancellation token shared by in-flight requests.
    #[must_use]
    pub fn cancellation_token(&self) -> &CancellationToken {
        &self.cancellation_token
    }

    /// Overrides the base REST URL (for testing with mock servers).
    pub fn set_base_url(&self, url: String) {
        self.base_url.store(Arc::new(url));
    }

    fn default_headers() -> HashMap<String, String> {
        HashMap::from([
            (USER_AGENT.to_string(), NAUTILUS_USER_AGENT.to_string()),
            ("Content-Type".to_string(), "application/json".to_string()),
            ("Accept".to_string(), "application/json".to_string()),
        ])
    }

    fn build_url(&self, path: &str) -> String {
        format!("{}{path}", self.base_url.load())
    }

    // --- OAuth token lifecycle -------------------------------------------------

    /// Returns a valid access token, refreshing it if missing or near expiry.
    ///
    /// # Errors
    ///
    /// Returns an error if no credentials are configured or the refresh fails.
    pub async fn ensure_access_token(&self) -> Result<String> {
        if let Some(state) = self.access_token.load_full()
            && state.expires_at > Instant::now() + self.token_refresh_buffer
        {
            return Ok(state.access_token.clone());
        }
        self.refresh_access_token().await
    }

    /// Forces a token refresh, re-authenticating via the configured mechanism
    /// (`POST /oauth/token` for OAuth, `POST /sessions` for session auth).
    ///
    /// # Errors
    ///
    /// Returns an error if no credentials are configured or the request fails.
    pub async fn refresh_access_token(&self) -> Result<String> {
        let credential = self
            .credential
            .as_ref()
            .ok_or_else(|| Error::auth("No credentials configured"))?;

        // Serialize concurrent refreshes; re-check freshness after acquiring the
        // lock so only the first waiter performs the network round-trip.
        let _guard = self.refresh_lock.lock().await;
        if let Some(state) = self.access_token.load_full()
            && state.expires_at > Instant::now() + self.token_refresh_buffer
        {
            return Ok(state.access_token.clone());
        }

        let (token, ttl) = match credential {
            TastytradeCredential::OAuth {
                provider_secret,
                refresh_token,
            } => {
                let body = json!({
                    "grant_type": "refresh_token",
                    "refresh_token": refresh_token,
                    "client_secret": provider_secret,
                });
                let body_bytes = serde_json::to_vec(&body).map_err(Error::Serde)?;
                let url = self.build_url(OAUTH_TOKEN_PATH);
                let value = self
                    .send(
                        Method::POST,
                        url,
                        Self::default_headers(),
                        Some(body_bytes),
                        true,
                    )
                    .await?;
                let resp: OAuthTokenResponse =
                    serde_json::from_value(value).map_err(Error::Serde)?;
                (resp.access_token, Duration::from_secs(resp.expires_in))
            }
            TastytradeCredential::Session { login, password } => {
                let body = json!({ "login": login, "password": password });
                let body_bytes = serde_json::to_vec(&body).map_err(Error::Serde)?;
                let url = self.build_url(SESSIONS_PATH);
                let value = self
                    .send(
                        Method::POST,
                        url,
                        Self::default_headers(),
                        Some(body_bytes),
                        true,
                    )
                    .await?;
                let resp: SessionResponse = serde_json::from_value(value).map_err(Error::Serde)?;
                (
                    resp.data.session_token,
                    Duration::from_secs(SESSION_TOKEN_TTL_SECS),
                )
            }
        };

        self.access_token.store(Some(Arc::new(TokenState {
            access_token: token.clone(),
            expires_at: Instant::now() + ttl,
        })));
        log::debug!("Refreshed tastytrade token (ttl={}s)", ttl.as_secs());
        Ok(token)
    }

    fn authed_headers(&self, token: &str) -> HashMap<String, String> {
        let mut headers = Self::default_headers();
        // OAuth access tokens use the `Bearer` scheme; session tokens are sent
        // as the bare `Authorization` value.
        let scheme = self
            .credential
            .as_ref()
            .map_or(AuthScheme::Bearer, TastytradeCredential::auth_scheme);
        let value = match scheme {
            AuthScheme::Bearer => format!("Bearer {token}"),
            AuthScheme::Raw => token.to_string(),
        };
        headers.insert("Authorization".to_string(), value);
        if let Some(version) = &self.accept_version {
            headers.insert("Accept-Version".to_string(), version.clone());
        }
        headers
    }

    // --- Request execution -----------------------------------------------------

    fn parse_response(&self, response: &HttpResponse) -> Result<Value> {
        if !response.status.is_success() {
            return Err(Error::from_http_status(
                response.status.as_u16(),
                &response.body,
            ));
        }
        if response.body.is_empty() {
            return Ok(Value::Null);
        }
        serde_json::from_slice(&response.body).map_err(Error::Serde)
    }

    // Retries are gated to idempotent operations so that order-mutating POSTs
    // cannot be replayed. Headers are captured up-front (auth token already
    // resolved by the caller), so retries reuse the same valid token.
    async fn send(
        &self,
        method: Method,
        url: String,
        headers: HashMap<String, String>,
        body: Option<Vec<u8>>,
        idempotent: bool,
    ) -> Result<Value> {
        let operation_name = url.clone();

        let operation = || {
            let method = method.clone();
            let url = url.clone();
            let headers = headers.clone();
            let body = body.clone();

            async move {
                let response = self
                    .client
                    .request(method, url, None, Some(headers), body, None, None)
                    .await
                    .map_err(Error::from_http_client)?;
                self.parse_response(&response)
            }
        };

        let should_retry = move |err: &Error| idempotent && err.is_retryable();

        self.retry_manager
            .execute_with_retry_with_cancel(
                &operation_name,
                operation,
                should_retry,
                Error::transport,
                &self.cancellation_token,
            )
            .await
    }

    /// Sends an authenticated GET request, ensuring a fresh access token.
    ///
    /// # Errors
    ///
    /// Returns an error if authentication or the request fails.
    pub async fn get(&self, path: &str) -> Result<Value> {
        let token = self.ensure_access_token().await?;
        let headers = self.authed_headers(&token);
        let url = self.build_url(path);
        self.send(Method::GET, url, headers, None, true).await
    }

    /// Sends an authenticated POST request with a JSON body.
    ///
    /// `idempotent` controls whether the request is eligible for retry; set it
    /// to `false` for order-mutating endpoints.
    ///
    /// # Errors
    ///
    /// Returns an error if authentication or the request fails.
    pub async fn post(&self, path: &str, body: &Value, idempotent: bool) -> Result<Value> {
        let token = self.ensure_access_token().await?;
        let headers = self.authed_headers(&token);
        let url = self.build_url(path);
        let body_bytes = serde_json::to_vec(body).map_err(Error::Serde)?;
        self.send(Method::POST, url, headers, Some(body_bytes), idempotent)
            .await
    }

    // --- Phase 0 endpoints -----------------------------------------------------

    /// Fetches the customer's accounts (`GET /customers/me/accounts`).
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails.
    pub async fn get_accounts_raw(&self) -> Result<Value> {
        self.get(CUSTOMER_ACCOUNTS_PATH).await
    }

    /// Fetches and decodes the customer's account numbers.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the response cannot be decoded.
    pub async fn get_account_numbers(&self) -> Result<Vec<String>> {
        let value = self.get_accounts_raw().await?;
        let resp: AccountsResponse = serde_json::from_value(value).map_err(Error::Serde)?;
        Ok(resp
            .data
            .items
            .into_iter()
            .map(|item| item.account.account_number)
            .collect())
    }

    /// Fetches account balances (`GET /accounts/{account_number}/balances`).
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails.
    pub async fn get_balances_raw(&self, account_number: &str) -> Result<Value> {
        self.get(&format!("/accounts/{account_number}/balances"))
            .await
    }

    /// Fetches account positions (`GET /accounts/{account_number}/positions`).
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails.
    pub async fn get_positions_raw(&self, account_number: &str) -> Result<Value> {
        self.get(&format!("/accounts/{account_number}/positions"))
            .await
    }

    /// Fetches live (working) orders (`GET /accounts/{account_number}/orders/live`).
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails.
    pub async fn get_live_orders_raw(&self, account_number: &str) -> Result<Value> {
        self.get(&format!("/accounts/{account_number}/orders/live"))
            .await
    }

    /// Dry-runs an order (`POST /accounts/{account_number}/orders/dry-run`).
    ///
    /// Validates the order and returns its buying-power effect, fee calculation,
    /// and the resolved order structure **without** sending it to any venue.
    /// Treated as idempotent (it never mutates state) so it is retry-eligible.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails.
    pub async fn dry_run_order_raw(&self, account_number: &str, order: &Value) -> Result<Value> {
        self.post(
            &format!("/accounts/{account_number}/orders/dry-run"),
            order,
            true,
        )
        .await
    }

    /// Fetches a DXLink quote token (`GET /api-quote-tokens`).
    ///
    /// The response carries the `dxlink-url` and `token` used to open the market
    /// data stream; quote tokens expire after ~24 hours.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails. Note this endpoint requires a
    /// funded, registered account and is unavailable for some sandbox setups.
    pub async fn get_quote_token_raw(&self) -> Result<Value> {
        self.get(API_QUOTE_TOKENS_PATH).await
    }

    /// Fetches and decodes a DXLink quote token (`GET /api-quote-tokens`).
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the response cannot be decoded.
    pub async fn get_quote_token(&self) -> Result<QuoteTokenData> {
        let value = self.get_quote_token_raw().await?;
        let resp: QuoteTokenResponse = serde_json::from_value(value).map_err(Error::Serde)?;
        Ok(resp.data)
    }
}
