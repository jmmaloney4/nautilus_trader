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

//! Phase 0 auth + connectivity spike for the tastytrade adapter.
//!
//! Exercises the REST surface needed to bootstrap the adapter and captures raw
//! payloads to disk so the typed mappings can be designed against real data.
//!
//! Credentials (a local `.env` file is loaded if present). Provide EITHER an
//! OAuth pair OR a session pair; OAuth is preferred when both are present:
//!
//! - `TASTYTRADE_PROVIDER_SECRET` + `TASTYTRADE_REFRESH_TOKEN` — OAuth.
//! - `TASTYTRADE_LOGIN` + `TASTYTRADE_PASSWORD` — session login.
//! - `TASTYTRADE_ENV` — `sandbox` (default) or `production`.
//!
//! Run with:
//!
//! ```bash
//! cargo run -p nautilus-tastytrade --example tastytrade-phase0
//! ```
//!
//! Raw payloads are written to `./phase0_payloads/`.

use std::path::PathBuf;

use nautilus_tastytrade::{
    common::enums::TastytradeEnvironment, http::client::TastytradeHttpClient,
};
use serde_json::Value;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env if present; ignore if absent.
    let _ = dotenvy::dotenv();

    let environment = match std::env::var("TASTYTRADE_ENV").as_deref() {
        Ok("production" | "prod") => TastytradeEnvironment::Production,
        _ => TastytradeEnvironment::Sandbox,
    };
    println!("== tastytrade Phase 0 spike (environment: {environment:?}) ==");

    let out_dir = PathBuf::from("phase0_payloads");
    std::fs::create_dir_all(&out_dir)?;

    let client = TastytradeHttpClient::from_env(environment)?;

    // 1. Authenticate (OAuth refresh or session login) to validate the flow.
    println!("\n[1] Authenticating...");
    match client.refresh_access_token().await {
        Ok(_) => println!("    OK — access token acquired"),
        Err(e) => {
            eprintln!("    FAILED: {e}");
            eprintln!("    Cannot continue without an access token.");
            return Err(anyhow::anyhow!(e));
        }
    }

    // 2. Account discovery.
    println!("\n[2] Fetching accounts (/customers/me/accounts)...");
    let accounts_raw = capture(&client.get_accounts_raw().await, &out_dir, "accounts");
    let account_numbers = client.get_account_numbers().await.unwrap_or_default();
    if account_numbers.is_empty() {
        println!("    No account numbers decoded (inspect accounts.json).");
    } else {
        println!("    Accounts: {account_numbers:?}");
    }
    let _ = accounts_raw;

    // 3. Per-account balances / positions / live orders.
    if let Some(account) = account_numbers.first() {
        println!("\n[3] Fetching balances/positions/live-orders for {account}...");
        capture(
            &client.get_balances_raw(account).await,
            &out_dir,
            "balances",
        );
        capture(
            &client.get_positions_raw(account).await,
            &out_dir,
            "positions",
        );
        capture(
            &client.get_live_orders_raw(account).await,
            &out_dir,
            "live_orders",
        );

        // Dry-run a small limit order to capture the order/buying-power/fee
        // response shape without placing anything. Priced far from the market
        // so it is obviously non-marketable; dry-run never reaches a venue.
        println!("\n[3b] Dry-running a sample order to capture the order shape...");
        let order = serde_json::json!({
            "order-type": "Limit",
            "time-in-force": "Day",
            "price": "1.00",
            "price-effect": "Debit",
            "legs": [{
                "instrument-type": "Equity",
                "symbol": "AAPL",
                "quantity": 1,
                "action": "Buy to Open"
            }]
        });
        capture(
            &client.dry_run_order_raw(account, &order).await,
            &out_dir,
            "dry_run",
        );
    } else {
        println!("\n[3] Skipping account-scoped calls (no account number).");
    }

    // 4. DXLink quote token (may be unavailable on unfunded/sandbox accounts).
    println!("\n[4] Fetching DXLink quote token (/api-quote-tokens)...");
    capture(&client.get_quote_token_raw().await, &out_dir, "quote_token");

    println!("\nDone. Raw payloads written to {}/", out_dir.display());
    Ok(())
}

/// Logs the outcome of a step and, on success, writes the raw payload to disk.
fn capture(
    result: &nautilus_tastytrade::http::error::Result<Value>,
    out_dir: &std::path::Path,
    name: &str,
) -> bool {
    match result {
        Ok(value) => {
            let path = out_dir.join(format!("{name}.json"));
            match serde_json::to_string_pretty(value) {
                Ok(pretty) => {
                    if let Err(e) = std::fs::write(&path, pretty) {
                        eprintln!(
                            "    [{name}] OK but failed to write {}: {e}",
                            path.display()
                        );
                    } else {
                        println!("    [{name}] OK — wrote {}", path.display());
                    }
                    true
                }
                Err(e) => {
                    eprintln!("    [{name}] OK but failed to serialize: {e}");
                    false
                }
            }
        }
        Err(e) => {
            eprintln!("    [{name}] FAILED: {e}");
            false
        }
    }
}
