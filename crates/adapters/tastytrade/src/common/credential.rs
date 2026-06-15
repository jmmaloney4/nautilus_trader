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

//! OAuth2 credentials for the tastytrade adapter.

use std::fmt::{Debug, Display};

use nautilus_core::env::resolve_env_var_pair;
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::http::error::{Error, Result};

/// Returns the `(provider_secret, refresh_token)` environment variable names.
#[must_use]
pub fn credential_env_vars() -> (&'static str, &'static str) {
    ("TASTYTRADE_PROVIDER_SECRET", "TASTYTRADE_REFRESH_TOKEN")
}

/// tastytrade OAuth2 credentials, zeroized on drop.
///
/// tastytrade uses an OAuth2 refresh-token grant: the long-lived `refresh_token`
/// plus the application's `provider_secret` (client secret) are exchanged at
/// `POST /oauth/token` for a short-lived (~15 minute) access token. The access
/// token itself is managed by the HTTP client, not stored here.
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct TastytradeCredential {
    provider_secret: String,
    refresh_token: String,
}

impl TastytradeCredential {
    /// Creates a new [`TastytradeCredential`] instance.
    #[must_use]
    pub fn new(provider_secret: String, refresh_token: String) -> Self {
        Self {
            provider_secret,
            refresh_token,
        }
    }

    /// Resolves credentials from provided values or [`credential_env_vars`],
    /// returning `None` when neither yields a complete pair.
    #[must_use]
    pub fn resolve(provider_secret: Option<&str>, refresh_token: Option<&str>) -> Option<Self> {
        let (secret_var, refresh_var) = credential_env_vars();
        let (secret, refresh) = resolve_env_var_pair(
            provider_secret
                .filter(|s| !s.trim().is_empty())
                .map(String::from),
            refresh_token
                .filter(|s| !s.trim().is_empty())
                .map(String::from),
            secret_var,
            refresh_var,
        )?;
        Some(Self::new(secret, refresh))
    }

    /// Loads credentials from environment variables.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Auth`] if the environment variables are unset or empty.
    pub fn from_env() -> Result<Self> {
        let (secret_var, refresh_var) = credential_env_vars();
        Self::resolve(None, None).ok_or_else(|| {
            Error::auth(format!(
                "{secret_var} and {refresh_var} environment variables are required"
            ))
        })
    }

    /// Returns the OAuth provider (client) secret.
    #[must_use]
    pub fn provider_secret(&self) -> &str {
        &self.provider_secret
    }

    /// Returns the OAuth refresh token.
    #[must_use]
    pub fn refresh_token(&self) -> &str {
        &self.refresh_token
    }
}

impl Debug for TastytradeCredential {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(stringify!(TastytradeCredential))
            .field("provider_secret", &"***redacted***")
            .field("refresh_token", &"***redacted***")
            .finish()
    }
}

impl Display for TastytradeCredential {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TastytradeCredential(***redacted***)")
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    fn test_resolve_with_explicit_values() {
        let cred = TastytradeCredential::resolve(Some("secret"), Some("refresh"))
            .expect("both explicit values must resolve");
        assert_eq!(cred.provider_secret(), "secret");
        assert_eq!(cred.refresh_token(), "refresh");
    }

    #[rstest]
    fn test_debug_redacts() {
        // Use sentinel values that are not substrings of the field names, so the
        // assertion checks that the secret *values* (not labels) are hidden.
        let cred = TastytradeCredential::new("PSVALUE123".to_string(), "RTVALUE456".to_string());
        let debug = format!("{cred:?}");
        assert!(debug.contains("redacted"));
        assert!(!debug.contains("PSVALUE123"));
        assert!(!debug.contains("RTVALUE456"));
    }

    #[rstest]
    fn test_env_var_names() {
        assert_eq!(
            credential_env_vars(),
            ("TASTYTRADE_PROVIDER_SECRET", "TASTYTRADE_REFRESH_TOKEN"),
        );
    }
}
