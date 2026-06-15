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

//! Enumerations for the tastytrade adapter.

use serde::{Deserialize, Serialize};

use crate::common::consts::{REST_URL, REST_URL_SANDBOX, WS_ACCOUNT_URL, WS_ACCOUNT_URL_SANDBOX};

/// The tastytrade API environment.
///
/// Defaults to [`Sandbox`](TastytradeEnvironment::Sandbox) so that exploratory
/// and Phase 0 work cannot accidentally hit production. Set explicitly to
/// [`Production`](TastytradeEnvironment::Production) for live trading.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(
    feature = "python",
    pyo3::pyclass(
        module = "nautilus_trader.core.nautilus_pyo3.tastytrade",
        eq,
        from_py_object,
        rename_all = "SCREAMING_SNAKE_CASE"
    )
)]
#[cfg_attr(
    feature = "python",
    pyo3_stub_gen::derive::gen_stub_pyclass_enum(module = "nautilus_trader.adapters.tastytrade")
)]
pub enum TastytradeEnvironment {
    /// Production environment (`api.tastytrade.com`).
    Production,
    /// Certification/sandbox environment (`api.cert.tastyworks.com`).
    #[default]
    Sandbox,
}

impl TastytradeEnvironment {
    /// Returns the REST base URL for this environment.
    #[must_use]
    pub const fn rest_url(self) -> &'static str {
        match self {
            Self::Production => REST_URL,
            Self::Sandbox => REST_URL_SANDBOX,
        }
    }

    /// Returns the account notification (alert) WebSocket URL for this environment.
    #[must_use]
    pub const fn ws_account_url(self) -> &'static str {
        match self {
            Self::Production => WS_ACCOUNT_URL,
            Self::Sandbox => WS_ACCOUNT_URL_SANDBOX,
        }
    }

    /// Returns `true` if this is the sandbox environment.
    #[must_use]
    pub const fn is_sandbox(self) -> bool {
        matches!(self, Self::Sandbox)
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    fn test_default_is_sandbox() {
        assert_eq!(
            TastytradeEnvironment::default(),
            TastytradeEnvironment::Sandbox
        );
        assert!(TastytradeEnvironment::default().is_sandbox());
    }

    #[rstest]
    fn test_urls() {
        assert!(
            TastytradeEnvironment::Production
                .rest_url()
                .starts_with("https://")
        );
        assert!(TastytradeEnvironment::Sandbox.rest_url().contains("cert"));
        assert!(
            TastytradeEnvironment::Production
                .ws_account_url()
                .starts_with("wss://")
        );
        assert!(
            TastytradeEnvironment::Sandbox
                .ws_account_url()
                .contains("cert")
        );
    }
}
