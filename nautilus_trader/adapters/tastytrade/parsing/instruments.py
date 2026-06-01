from __future__ import annotations

from typing import Any

from nautilus_trader.model.identifiers import InstrumentId, Symbol, Venue
from nautilus_trader.model.instruments import Equity, OptionContract
from nautilus_trader.model.enums import OptionKind
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


def parse_equity_option_payload(payload: dict[str, Any]) -> OptionContract:
    """Parse a Tastytrade equity option payload to Nautilus OptionContract.

    Payload example fields:
    - symbol (OCC): "SPY   230731C00393000"
    - underlying-symbol: "SPY"
    - strike-price: "393.0"
    - expiration-date: "2023-07-31"
    - option-type: "C" | "P"
    - shares-per-contract: 100
    - active, settlement-type, tick-sizes, etc.
    """
    occ = payload["symbol"]
    underlying = payload.get("underlying-symbol") or ""
    venue = Venue("OPRA")  # default; can be refined by provider
    instrument_id = InstrumentId(Symbol(occ.replace(" ", "")), venue)

    currency = Currency.from_str("USD")
    min_tick = select_min_tick(payload.get("tick-sizes") or [])
    precision = tick_to_precision(min_tick)
    price_increment = Price(min_tick, precision)

    kind = OptionKind.CALL if payload.get("option-type") == "C" else OptionKind.PUT
    multiplier = Quantity.from_int(int(payload.get("shares-per-contract", 100)))

    # Times will be set by provider
    ts = 0
    strike_val = float(payload.get("strike-price", 0.0))
    return OptionContract(
        instrument_id=instrument_id,
        raw_symbol=Symbol(occ),
        asset_class=None,  # inferred by core for options; leave None if allowed
        currency=currency,
        price_precision=precision,
        price_increment=price_increment,
        multiplier=multiplier,
        lot_size=multiplier,
        underlying=underlying,
        strike_price=Price(strike_val, precision),
        activation_ns=ts,
        expiration_ns=ts,
        option_kind=kind,
        ts_event=ts,
        ts_init=ts,
        info=payload,
    )


def parse_futures_option_payload(payload: dict[str, Any]) -> OptionContract:
    """Parse a Tastytrade futures option payload to Nautilus OptionContract.

    Fields include:
    - symbol: "./ESU3 E1DQ3 230803P3860"
    - underlying-symbol: "/ESU3"
    - product-code, exchange, streamer-symbol, etc.
    - strike-price, expiration-date, option-type
    - multiplier (often 1.0) and tick-sizes
    """
    sym = payload["symbol"]
    underlying = payload.get("underlying-symbol") or ""
    # Venue from streamer-exchange-code if available, else fallback to exchange
    venue_code = payload.get("streamer-exchange-code") or payload.get("exchange") or "XCME"
    instrument_id = InstrumentId(Symbol(sym.replace(" ", "")), Venue(venue_code))

    currency = Currency.from_str("USD")
    min_tick = select_min_tick(payload.get("tick-sizes") or [])
    precision = tick_to_precision(min_tick)
    price_increment = Price(min_tick, precision)

    kind = OptionKind.CALL if payload.get("option-type") == "C" else OptionKind.PUT
    try:
        mult = float(payload.get("multiplier", 1.0))
    except Exception:
        mult = 1.0
    multiplier = Quantity(mult, precision=0)

    strike_val = float(payload.get("strike-price", 0.0))
    ts = 0
    return OptionContract(
        instrument_id=instrument_id,
        raw_symbol=Symbol(sym),
        asset_class=None,
        currency=currency,
        price_precision=precision,
        price_increment=price_increment,
        multiplier=multiplier,
        lot_size=multiplier,
        underlying=underlying,
        strike_price=Price(strike_val, precision),
        activation_ns=ts,
        expiration_ns=ts,
        option_kind=kind,
        ts_event=ts,
        ts_init=ts,
        info=payload,
    )


