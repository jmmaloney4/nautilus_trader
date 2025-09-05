// -------------------------------------------------------------------------------------------------
//  Copyright (C) 2015-2025 Nautech Systems Pty Ltd. All rights reserved.
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

use std::fmt::{Debug, Display};

use nautilus_core::UnixNanos;
use serde::{Deserialize, Serialize};

use crate::identifiers::InstrumentId;

/// Represents a consolidated best bid and offer (CBBO) message.
#[repr(C)]
#[derive(Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
#[cfg_attr(
    feature = "python",
    pyo3::pyclass(get_all, set_all, module = "nautilus_trader.model.data")
)]
pub struct ConsolidatedBBO {
    /// The UNIX nanosecond timestamp of when the record was created.
    pub ts_init: UnixNanos,
    /// The UNIX nanosecond timestamp of the event.
    pub ts_event: UnixNanos,
    /// The UNIX nanosecond timestamp of when the record was ingested by the data source.
    pub ts_in_delta: UnixNanos,
    /// The UNIX nanosecond timestamp of when the record was received by the system.
    pub ts_recv: UnixNanos,
    /// The instrument ID.
    pub instrument_id: InstrumentId,
    /// The best bid price.
    pub bid_price: f64,
    /// The best ask price.
    pub ask_price: f64,
    /// The best bid quantity.
    pub bid_qty: f64,
    /// The best ask quantity.
    pub ask_qty: f64,
    /// The last trade price.
    pub trade_price: f64,
    /// The last trade quantity.
    pub trade_qty: f64,
}

impl Debug for ConsolidatedBBO {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConsolidatedBBO")
            .field("instrument_id", &self.instrument_id)
            .field("ts_init", &self.ts_init)
            .field("ts_event", &self.ts_event)
            .field("ts_in_delta", &self.ts_in_delta)
            .field("ts_recv", &self.ts_recv)
            .field("bid_price", &self.bid_price)
            .field("ask_price", &self.ask_price)
            .field("bid_qty", &self.bid_qty)
            .field("ask_qty", &self.ask_qty)
            .field("trade_price", &self.trade_price)
            .field("trade_qty", &self.trade_qty)
            .finish()
    }
}

impl Display for ConsolidatedBBO {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ConsolidatedBBO(instrument_id={}, ts_event={}, bid_price={}, ask_price={}, bid_qty={}, ask_qty={}, trade_price={}, trade_qty={})",
            self.instrument_id,
            self.ts_event,
            self.bid_price,
            self.ask_price,
            self.bid_qty,
            self.ask_qty,
            self.trade_price,
            self.trade_qty
        )
    }
}

impl ConsolidatedBBO {
    /// Creates a new [`ConsolidatedBBO`] instance.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        ts_init: UnixNanos,
        ts_event: UnixNanos,
        ts_in_delta: UnixNanos,
        ts_recv: UnixNanos,
        instrument_id: InstrumentId,
        bid_price: f64,
        ask_price: f64,
        bid_qty: f64,
        ask_qty: f64,
        trade_price: f64,
        trade_qty: f64,
    ) -> Self {
        Self {
            ts_init,
            ts_event,
            ts_in_delta,
            ts_recv,
            instrument_id,
            bid_price,
            ask_price,
            bid_qty,
            ask_qty,
            trade_price,
            trade_qty,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identifiers::Venue;
    use crate::instrument_id;

    #[test]
    fn test_consolidated_bbo_creation() {
        let instrument_id = instrument_id!("BTCUSD.BINANCE");
        let cbbo = ConsolidatedBBO::new(
            UnixNanos::new(1),
            UnixNanos::new(2),
            UnixNanos::new(3),
            UnixNanos::new(4),
            instrument_id,
            100.0,
            101.0,
            1.0,
            2.0,
            100.5,
            1.5,
        );

        assert_eq!(cbbo.ts_init, UnixNanos::new(1));
        assert_eq!(cbbo.ts_event, UnixNanos::new(2));
        assert_eq!(cbbo.ts_in_delta, UnixNanos::new(3));
        assert_eq!(cbbo.ts_recv, UnixNanos::new(4));
        assert_eq!(cbbo.instrument_id, instrument_id);
        assert_eq!(cbbo.bid_price, 100.0);
        assert_eq!(cbbo.ask_price, 101.0);
        assert_eq!(cbbo.bid_qty, 1.0);
        assert_eq!(cbbo.ask_qty, 2.0);
        assert_eq!(cbbo.trade_price, 100.5);
        assert_eq!(cbbo.trade_qty, 1.5);
    }

    #[test]
    fn test_consolidated_bbo_display() {
        let instrument_id = instrument_id!("BTCUSD.BINANCE");
        let cbbo = ConsolidatedBBO::new(
            UnixNanos::new(1),
            UnixNanos::new(2),
            UnixNanos::new(3),
            UnixNanos::new(4),
            instrument_id,
            100.0,
            101.0,
            1.0,
            2.0,
            100.5,
            1.5,
        );

        let expected = "ConsolidatedBBO(instrument_id=BTCUSD.BINANCE, ts_event=2, bid_price=100, ask_price=101, bid_qty=1, ask_qty=2, trade_price=100.5, trade_qty=1.5)";
        assert_eq!(format!("{}", cbbo), expected);
    }
}
