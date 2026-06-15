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

//! Configuration types for the tastytrade adapter.

use serde::{Deserialize, Serialize};

use crate::common::{
    consts::{DEFAULT_HTTP_TIMEOUT_SECS, TOKEN_REFRESH_BUFFER_SECS},
    enums::TastytradeEnvironment,
};

/// Configuration for the tastytrade live data client.
#[derive(Debug, Clone, Serialize, Deserialize, bon::Builder)]
#[serde(default, deny_unknown_fields)]
#[cfg_attr(
    feature = "python",
    pyo3::pyclass(
        module = "nautilus_trader.core.nautilus_pyo3.tastytrade",
        from_py_object
    )
)]
#[cfg_attr(
    feature = "python",
    pyo3_stub_gen::derive::gen_stub_pyclass(module = "nautilus_trader.adapters.tastytrade")
)]
pub struct TastytradeDataClientConfig {
    /// OAuth provider (client) secret. Falls back to `TASTYTRADE_PROVIDER_SECRET`.
    pub provider_secret: Option<String>,
    /// OAuth refresh token. Falls back to `TASTYTRADE_REFRESH_TOKEN`.
    pub refresh_token: Option<String>,
    /// Overrides the REST base URL (defaults to the environment's URL).
    pub base_url_http: Option<String>,
    /// Overrides the account-stream WebSocket URL (defaults to the environment's URL).
    pub base_url_ws: Option<String>,
    /// Optional `Accept-Version` header value (date string) for API version pinning.
    pub accept_version: Option<String>,
    /// API environment (defaults to sandbox).
    #[builder(default)]
    pub environment: TastytradeEnvironment,
    /// REST request timeout, in seconds.
    #[builder(default = DEFAULT_HTTP_TIMEOUT_SECS)]
    pub http_timeout_secs: u64,
    /// Refresh the access token this many seconds before expiry.
    #[builder(default = TOKEN_REFRESH_BUFFER_SECS)]
    pub token_refresh_buffer_secs: u64,
    /// Interval for refreshing the instrument catalog, in minutes.
    #[builder(default = 60)]
    pub update_instruments_interval_mins: u64,
}

impl Default for TastytradeDataClientConfig {
    fn default() -> Self {
        Self::builder().build()
    }
}

/// Configuration for the tastytrade live execution client.
#[derive(Debug, Clone, Serialize, Deserialize, bon::Builder)]
#[serde(default, deny_unknown_fields)]
#[cfg_attr(
    feature = "python",
    pyo3::pyclass(
        module = "nautilus_trader.core.nautilus_pyo3.tastytrade",
        from_py_object
    )
)]
#[cfg_attr(
    feature = "python",
    pyo3_stub_gen::derive::gen_stub_pyclass(module = "nautilus_trader.adapters.tastytrade")
)]
pub struct TastytradeExecClientConfig {
    /// OAuth provider (client) secret. Falls back to `TASTYTRADE_PROVIDER_SECRET`.
    pub provider_secret: Option<String>,
    /// OAuth refresh token. Falls back to `TASTYTRADE_REFRESH_TOKEN`.
    pub refresh_token: Option<String>,
    /// The tastytrade account number this client trades (e.g. `5WT00001`).
    pub account_number: Option<String>,
    /// Overrides the REST base URL (defaults to the environment's URL).
    pub base_url_http: Option<String>,
    /// Overrides the account-stream WebSocket URL (defaults to the environment's URL).
    pub base_url_ws: Option<String>,
    /// Optional `Accept-Version` header value (date string) for API version pinning.
    pub accept_version: Option<String>,
    /// API environment (defaults to sandbox).
    #[builder(default)]
    pub environment: TastytradeEnvironment,
    /// REST request timeout, in seconds.
    #[builder(default = DEFAULT_HTTP_TIMEOUT_SECS)]
    pub http_timeout_secs: u64,
    /// Refresh the access token this many seconds before expiry.
    #[builder(default = TOKEN_REFRESH_BUFFER_SECS)]
    pub token_refresh_buffer_secs: u64,
    /// Require an order dry-run to succeed before submitting a live order.
    #[builder(default = true)]
    pub use_dry_run: bool,
}

impl Default for TastytradeExecClientConfig {
    fn default() -> Self {
        Self::builder().build()
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    fn test_data_config_defaults() {
        let config = TastytradeDataClientConfig::default();
        assert_eq!(config.environment, TastytradeEnvironment::Sandbox);
        assert_eq!(config.http_timeout_secs, DEFAULT_HTTP_TIMEOUT_SECS);
        assert_eq!(config.token_refresh_buffer_secs, TOKEN_REFRESH_BUFFER_SECS);
        assert!(config.provider_secret.is_none());
    }

    #[rstest]
    fn test_exec_config_defaults() {
        let config = TastytradeExecClientConfig::default();
        assert_eq!(config.environment, TastytradeEnvironment::Sandbox);
        assert!(config.use_dry_run);
        assert!(config.account_number.is_none());
    }
}
