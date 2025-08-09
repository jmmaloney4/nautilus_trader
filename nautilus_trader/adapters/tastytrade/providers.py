from __future__ import annotations

import asyncio
from typing import Any

from nautilus_trader.common.component import Clock
from nautilus_trader.common.providers import InstrumentProvider
from nautilus_trader.model.identifiers import InstrumentId
from nautilus_trader.model.instruments import Equity
from nautilus_trader.model.identifiers import Symbol, Venue
from nautilus_trader.model.objects import Currency, Price, Quantity

from .config import TastytradeInstrumentProviderConfig
from .http.client import TastytradeHttpClient
from .common import TT_VENUE_DEFAULT_EQUITY


class TastytradeInstrumentProvider(InstrumentProvider):
    """
    Minimal instrument provider that can load a single equity instrument by symbol.

    Additional asset classes and chains will be added in subsequent phases.
    """

    def __init__(
        self,
        client: TastytradeHttpClient,
        clock: Clock,
        config: TastytradeInstrumentProviderConfig,
    ) -> None:
        super().__init__(config=config)
        self._client = client
        self._clock = clock

    async def load_all_async(self, filters: dict | None = None) -> None:
        # Not implemented for Tastytrade (too large to load all)
        return

    async def load_ids_async(self, instrument_ids: list[InstrumentId], filters: dict | None = None) -> None:
        for instrument_id in instrument_ids:
            await self.load_async(instrument_id, filters)

    async def load_async(self, instrument_id: InstrumentId, filters: dict | None = None) -> None:
        # For now, support equities by symbol only.
        symbol = instrument_id.symbol.value
        data = await self._client._request("GET", f"/instruments/equities/{symbol}")
        instrument = self._parse_equity(data["data"])  # shape per docs
        self.add(instrument)

    def _parse_equity(self, payload: dict[str, Any]) -> Equity:
        # Derive venue and precision from payload
        venue_code = payload.get("listed-market") or TT_VENUE_DEFAULT_EQUITY
        venue = Venue(venue_code)
        instrument_id = InstrumentId(Symbol(payload["symbol"]), venue)
        currency = Currency.from_str("USD")  # Tastytrade equities are USD; adjust if payload supplies

        # Tick sizes may be a list; choose smallest for increment/precision
        tick_sizes = payload.get("tick-sizes") or []
        if tick_sizes:
            min_tick = min(float(t.get("value", 0.01)) for t in tick_sizes if t.get("value"))
        else:
            min_tick = 0.01
        price = Price(min_tick, precision=len(f"{min_tick:.10f}".split(".")[1].rstrip("0")))

        # Fractional eligibility can influence size precision
        is_fractional = bool(payload.get("is-fractional-quantity-eligible", True))
        size_precision = 6 if is_fractional else 0

        now = self._clock.timestamp_ns()
        equity = Equity(
            instrument_id=instrument_id,
            raw_symbol=Symbol(payload["symbol"]),
            currency=currency,
            price_precision=price.precision,
            price_increment=price,
            lot_size=Quantity.from_int(1),
            ts_event=now,
            ts_init=now,
            info=payload,
        )

        return equity


