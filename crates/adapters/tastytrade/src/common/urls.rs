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

//! URL helpers for the tastytrade adapter.

use crate::common::enums::TastytradeEnvironment;

/// Returns the REST base URL for the given environment.
#[must_use]
pub const fn rest_url(environment: TastytradeEnvironment) -> &'static str {
    environment.rest_url()
}

/// Returns the account notification WebSocket URL for the given environment.
#[must_use]
pub const fn ws_account_url(environment: TastytradeEnvironment) -> &'static str {
    environment.ws_account_url()
}
