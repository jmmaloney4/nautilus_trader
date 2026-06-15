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

//! [NautilusTrader](http://nautilustrader.io) integration adapter for the
//! [tastytrade](https://developer.tastytrade.com/) brokerage.
//!
//! The adapter follows the structural pattern of the Interactive Brokers adapter
//! (config, factories, shared transport, provider, data client, execution client)
//! but is tastytrade-native: REST + OAuth2 with an account notification WebSocket
//! and DXLink market data. REST is treated as the authoritative source of truth
//! and the notification stream as low-latency hints.
//!
//! # Feature flags
//!
//! - `python`: Enables Python bindings via PyO3.
//! - `extension-module`: Builds as a Python extension module (implies `python`).
//! - `high-precision`: Enables high-precision (128-bit) value types.

#![warn(rustc::all)]
#![deny(unsafe_code)]
#![deny(nonstandard_style)]
#![deny(missing_debug_implementations)]
#![deny(clippy::missing_errors_doc)]
#![deny(rustdoc::broken_intra_doc_links)]

pub mod common;
pub mod config;
pub mod http;

pub use crate::{
    config::{TastytradeDataClientConfig, TastytradeExecClientConfig},
    http::client::TastytradeHttpClient,
};
