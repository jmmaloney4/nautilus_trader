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

//! Typed models for tastytrade REST responses.
//!
//! Phase 0 keeps typed models intentionally minimal: only the shapes needed to
//! drive the auth/discovery flow are decoded, while full payloads are captured
//! raw for schema mapping. The exact field set is provisional pending validation
//! against live sandbox payloads.
//!
//! Note: most tastytrade REST responses wrap their payload in a `{ "data": ...,
//! "context": ... }` envelope and use `kebab-case` JSON keys. The OAuth token
//! endpoint is a standard (un-enveloped, snake_case) OAuth2 response.

use serde::{Deserialize, Serialize};

/// Standard OAuth2 token response from `POST /oauth/token`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthTokenResponse {
    /// Short-lived bearer access token (~15 minute lifetime).
    pub access_token: String,
    /// Token lifetime in seconds.
    pub expires_in: u64,
    /// Token type (expected `"Bearer"`).
    #[serde(default)]
    pub token_type: Option<String>,
    /// Granted scope.
    #[serde(default)]
    pub scope: Option<String>,
}

/// Generic tastytrade response envelope: `{ "data": <T>, "context": "<path>" }`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Envelope<T> {
    pub data: T,
    #[serde(default)]
    pub context: Option<String>,
}

/// `data` payload for `GET /customers/me/accounts`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct AccountsData {
    #[serde(default)]
    pub items: Vec<AccountItem>,
}

/// A single entry in the customer's account list.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct AccountItem {
    pub account: AccountInner,
    #[serde(default)]
    pub authority_level: Option<String>,
}

/// The nested account object carrying the account number.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct AccountInner {
    pub account_number: String,
    #[serde(default)]
    pub nickname: Option<String>,
    #[serde(default)]
    pub account_type_name: Option<String>,
}

/// Typed alias for the `GET /customers/me/accounts` response.
pub type AccountsResponse = Envelope<AccountsData>;

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    fn test_deserialize_oauth_token() {
        let raw = r#"{"access_token":"abc","expires_in":900,"token_type":"Bearer"}"#;
        let resp: OAuthTokenResponse = serde_json::from_str(raw).unwrap();
        assert_eq!(resp.access_token, "abc");
        assert_eq!(resp.expires_in, 900);
        assert_eq!(resp.token_type.as_deref(), Some("Bearer"));
    }

    #[rstest]
    fn test_deserialize_accounts_envelope() {
        let raw = r#"{
            "data": {"items": [
                {"account": {"account-number": "5WT00001", "nickname": "main"}, "authority-level": "owner"}
            ]},
            "context": "/customers/me/accounts"
        }"#;
        let resp: AccountsResponse = serde_json::from_str(raw).unwrap();
        assert_eq!(resp.data.items.len(), 1);
        assert_eq!(resp.data.items[0].account.account_number, "5WT00001");
    }
}
