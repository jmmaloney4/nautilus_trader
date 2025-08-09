from __future__ import annotations

from typing import Any

from nautilus_trader.model.identifiers import InstrumentId, Symbol, Venue
from nautilus_trader.model.instruments import Equity
from nautilus_trader.model.objects import Currency, Price, Quantity

from ..common import TT_VENUE_DEFAULT_EQUITY
from .price_conversion import select_min_tick, tick_to_precision


def parse_equity_payload(payload: dict[str, Any]) -> Equity:
    symbol = payload["symbol"]
    venue_code = payload.get("listed-market") or TT_VENUE_DEFAULT_EQUITY
    instrument_id = InstrumentId(Symbol(symbol), Venue(venue_code))
    currency = Currency.from_str("USD")

    min_tick = select_min_tick(payload.get("tick-sizes") or [])
    precision = tick_to_precision(min_tick)
    price_increment = Price(min_tick, precision)

    is_fractional = bool(payload.get("is-fractional-quantity-eligible", True))
    # We record lot_size=1, size_precision is handled via separate quantity precision endpoint.
    lot_size = Quantity.from_int(1)

    ts = 0  # provider will fill real timestamps
    return Equity(
        instrument_id=instrument_id,
        raw_symbol=Symbol(symbol),
        currency=currency,
        price_precision=precision,
        price_increment=price_increment,
        lot_size=lot_size,
        ts_event=ts,
        ts_init=ts,
        info=payload,
    )


