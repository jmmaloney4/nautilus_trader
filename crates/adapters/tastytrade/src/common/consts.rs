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

//! Constants for the tastytrade adapter.

use std::sync::LazyLock;

use nautilus_model::identifiers::{ClientId, Venue};
use ustr::Ustr;

/// Venue identifier string.
pub const TASTYTRADE: &str = "TASTYTRADE";

/// Static venue instance.
pub static TASTYTRADE_VENUE: LazyLock<Venue> = LazyLock::new(|| Venue::new(Ustr::from(TASTYTRADE)));

/// Static client ID instance.
pub static TASTYTRADE_CLIENT_ID: LazyLock<ClientId> =
    LazyLock::new(|| ClientId::new(Ustr::from(TASTYTRADE)));

// REST base URLs.
//
// Production is `api.tastytrade.com`; the sandbox ("certification") environment
// is `api.cert.tastyworks.com`. The sandbox resets every 24h and serves quotes
// delayed by 15 minutes; some services (real-time market data, NLV history,
// market metrics) are live-only.
pub const REST_URL: &str = "https://api.tastytrade.com";
pub const REST_URL_SANDBOX: &str = "https://api.cert.tastyworks.com";

// Account notification (alert) WebSocket URLs. The sandbox uses a distinct host.
pub const WS_ACCOUNT_URL: &str = "wss://streamer.tastyworks.com";
pub const WS_ACCOUNT_URL_SANDBOX: &str = "wss://streamer.cert.tastyworks.com";

/// OAuth token endpoint (relative to the REST base URL).
pub const OAUTH_TOKEN_PATH: &str = "/oauth/token";

/// DXLink quote-token endpoint (relative to the REST base URL).
pub const API_QUOTE_TOKENS_PATH: &str = "/api-quote-tokens";

/// Account discovery endpoint (relative to the REST base URL).
pub const CUSTOMER_ACCOUNTS_PATH: &str = "/customers/me/accounts";

/// Refresh the OAuth access token this many seconds before its stated expiry,
/// to avoid races where a token expires mid-flight.
pub const TOKEN_REFRESH_BUFFER_SECS: u64 = 60;

/// Default request timeout for REST calls, in seconds.
pub const DEFAULT_HTTP_TIMEOUT_SECS: u64 = 10;

/// Default REST rate-limit budget (requests per second).
///
/// tastytrade does not publish a clear rate-limit table; this is a conservative
/// placeholder. Note that repeated failed logins can get an IP blocked for
/// ~8 hours, so retry/backoff discipline matters more than raw throughput.
pub const DEFAULT_REST_QUOTA_PER_SECOND: u32 = 10;
