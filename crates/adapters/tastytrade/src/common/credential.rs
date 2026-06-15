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

//! Credentials for the tastytrade adapter.
//!
//! tastytrade supports two authentication mechanisms:
//!
//! - **OAuth2** (production-preferred): a long-lived `refresh_token` plus the
//!   application's `provider_secret` are exchanged at `POST /oauth/token` for a
//!   short-lived (~15 minute) bearer access token.
//! - **Session** (simple/sandbox): a `login` + `password` are posted to
//!   `POST /sessions` to obtain a `session-token` (~24h lifetime) used directly
//!   as the `Authorization` header value (no `Bearer` prefix).
//!
//! The short-lived token itself is managed by the HTTP client, not stored here.

use std::fmt::{Debug, Display};

use nautilus_core::env::resolve_env_var_pair;
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::http::error::{Error, Result};

/// Returns the `(provider_secret, refresh_token)` OAuth environment variable names.
#[must_use]
pub fn oauth_env_vars() -> (&'static str, &'static str) {
    ("TASTYTRADE_PROVIDER_SECRET", "TASTYTRADE_REFRESH_TOKEN")
}

/// Returns the `(login, password)` session environment variable names.
#[must_use]
pub fn session_env_vars() -> (&'static str, &'static str) {
    ("TASTYTRADE_LOGIN", "TASTYTRADE_PASSWORD")
}

/// The HTTP `Authorization` scheme used for a given credential type.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AuthScheme {
    /// OAuth2 bearer token: `Authorization: Bearer <token>`.
    Bearer,
    /// Session token: `Authorization: <token>` (no scheme prefix).
    Raw,
}

/// tastytrade credentials, zeroized on drop.
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub enum TastytradeCredential {
    /// OAuth2 refresh-token grant (production-preferred).
    OAuth {
        provider_secret: String,
        refresh_token: String,
    },
    /// Session login (simple; used for sandbox and personal access).
    Session { login: String, password: String },
}

impl TastytradeCredential {
    /// Creates an OAuth credential.
    #[must_use]
    pub fn oauth(provider_secret: String, refresh_token: String) -> Self {
        Self::OAuth {
            provider_secret,
            refresh_token,
        }
    }

    /// Creates a session credential.
    #[must_use]
    pub fn session(login: String, password: String) -> Self {
        Self::Session { login, password }
    }

    /// Resolves credentials from the environment, preferring OAuth over session.
    ///
    /// Returns `None` when neither a complete OAuth pair nor a complete session
    /// pair is present.
    #[must_use]
    pub fn resolve() -> Option<Self> {
        let (secret_var, refresh_var) = oauth_env_vars();
        if let Some((secret, refresh)) = resolve_env_var_pair(None, None, secret_var, refresh_var) {
            return Some(Self::oauth(secret, refresh));
        }
        let (login_var, password_var) = session_env_vars();
        resolve_env_var_pair(None, None, login_var, password_var)
            .map(|(login, password)| Self::session(login, password))
    }

    /// Loads credentials from environment variables.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Auth`] if neither credential pair is fully present.
    pub fn from_env() -> Result<Self> {
        let (secret_var, refresh_var) = oauth_env_vars();
        let (login_var, password_var) = session_env_vars();
        Self::resolve().ok_or_else(|| {
            Error::auth(format!(
                "set either ({secret_var}, {refresh_var}) for OAuth or \
                 ({login_var}, {password_var}) for session auth"
            ))
        })
    }

    /// Returns the `Authorization` scheme for this credential type.
    #[must_use]
    pub fn auth_scheme(&self) -> AuthScheme {
        match self {
            Self::OAuth { .. } => AuthScheme::Bearer,
            Self::Session { .. } => AuthScheme::Raw,
        }
    }

    /// Returns `true` if these are OAuth credentials.
    #[must_use]
    pub fn is_oauth(&self) -> bool {
        matches!(self, Self::OAuth { .. })
    }
}

impl Debug for TastytradeCredential {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let variant = match self {
            Self::OAuth { .. } => "OAuth",
            Self::Session { .. } => "Session",
        };
        f.debug_struct("TastytradeCredential")
            .field("kind", &variant)
            .field("secrets", &"***redacted***")
            .finish()
    }
}

impl Display for TastytradeCredential {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    fn test_auth_scheme() {
        let oauth = TastytradeCredential::oauth("s".into(), "r".into());
        assert_eq!(oauth.auth_scheme(), AuthScheme::Bearer);
        assert!(oauth.is_oauth());

        let session = TastytradeCredential::session("u".into(), "p".into());
        assert_eq!(session.auth_scheme(), AuthScheme::Raw);
        assert!(!session.is_oauth());
    }

    #[rstest]
    fn test_debug_redacts() {
        // Sentinel values that are not substrings of variant/field labels.
        let cred = TastytradeCredential::session("LOGINXYZ".into(), "PASSXYZ".into());
        let debug = format!("{cred:?}");
        assert!(debug.contains("redacted"));
        assert!(debug.contains("Session"));
        assert!(!debug.contains("LOGINXYZ"));
        assert!(!debug.contains("PASSXYZ"));
    }

    #[rstest]
    fn test_env_var_names() {
        assert_eq!(
            oauth_env_vars(),
            ("TASTYTRADE_PROVIDER_SECRET", "TASTYTRADE_REFRESH_TOKEN"),
        );
        assert_eq!(
            session_env_vars(),
            ("TASTYTRADE_LOGIN", "TASTYTRADE_PASSWORD"),
        );
    }
}
