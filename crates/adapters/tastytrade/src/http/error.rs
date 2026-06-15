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

//! Error types for tastytrade REST operations.

use nautilus_network::http::HttpClientError;
use thiserror::Error;

/// Error type for tastytrade operations.
#[derive(Debug, Error)]
pub enum Error {
    /// Transport layer errors (network, connection issues).
    #[error("transport error: {0}")]
    Transport(String),

    /// JSON serialization/deserialization errors.
    #[error("serde error: {0}")]
    Serde(#[from] serde_json::Error),

    /// Authentication errors (missing/invalid credentials, expired token).
    #[error("auth error: {0}")]
    Auth(String),

    /// Rate limiting errors.
    #[error("rate limited (retry_after_ms={retry_after_ms:?})")]
    RateLimit { retry_after_ms: Option<u64> },

    /// Bad request errors (client-side invalid payload).
    #[error("bad request: {0}")]
    BadRequest(String),

    /// Server-side errors from the tastytrade API.
    #[error("server error: {0}")]
    Server(String),

    /// Request timeout.
    #[error("timeout")]
    Timeout,

    /// HTTP errors with status code.
    #[error("HTTP error {status}: {message}")]
    Http { status: u16, message: String },
}

impl Error {
    /// Creates a transport error.
    #[must_use]
    pub fn transport(msg: impl Into<String>) -> Self {
        Self::Transport(msg.into())
    }

    /// Creates an auth error.
    #[must_use]
    pub fn auth(msg: impl Into<String>) -> Self {
        Self::Auth(msg.into())
    }

    /// Creates a bad request error.
    #[must_use]
    pub fn bad_request(msg: impl Into<String>) -> Self {
        Self::BadRequest(msg.into())
    }

    /// Creates a server error.
    #[must_use]
    pub fn server(msg: impl Into<String>) -> Self {
        Self::Server(msg.into())
    }

    /// Creates an HTTP error.
    #[must_use]
    pub fn http(status: u16, message: impl Into<String>) -> Self {
        Self::Http {
            status,
            message: message.into(),
        }
    }

    /// Creates an error from an HTTP status code and response body.
    #[must_use]
    pub fn from_http_status(status: u16, body: &[u8]) -> Self {
        let message = String::from_utf8_lossy(body).to_string();
        match status {
            401 | 403 => Self::auth(format!("HTTP {status}: {message}")),
            400 | 422 => Self::bad_request(format!("HTTP {status}: {message}")),
            429 => Self::RateLimit {
                retry_after_ms: None,
            },
            500..=599 => Self::server(format!("HTTP {status}: {message}")),
            _ => Self::http(status, message),
        }
    }

    /// Maps a [`HttpClientError`] from the transport layer.
    #[expect(clippy::needless_pass_by_value)]
    #[must_use]
    pub fn from_http_client(error: HttpClientError) -> Self {
        Self::transport(format!("HTTP client error: {error}"))
    }

    /// Returns `true` if the error is retryable.
    #[must_use]
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::Transport(_) | Self::Timeout | Self::RateLimit { .. } | Self::Server(_) => true,
            Self::Http { status, .. } => *status >= 500,
            _ => false,
        }
    }

    /// Returns `true` if the error is due to authentication.
    #[must_use]
    pub fn is_auth_error(&self) -> bool {
        matches!(self, Self::Auth(_))
    }
}

/// Result type alias for tastytrade operations.
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case(401, true, false)]
    #[case(400, false, false)]
    #[case(429, false, true)]
    #[case(500, false, true)]
    #[case(404, false, false)]
    fn test_from_http_status_classification(
        #[case] status: u16,
        #[case] expect_auth: bool,
        #[case] expect_retryable: bool,
    ) {
        let err = Error::from_http_status(status, b"body");
        assert_eq!(err.is_auth_error(), expect_auth, "auth for {status}");
        assert_eq!(
            err.is_retryable(),
            expect_retryable,
            "retryable for {status}"
        );
    }
}
