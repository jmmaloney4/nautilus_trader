from __future__ import annotations

import asyncio
from typing import Any
from urllib.parse import quote

from nautilus_trader.common.component import Clock
from nautilus_trader.common.providers import InstrumentProvider
from nautilus_trader.model.identifiers import InstrumentId
from nautilus_trader.model.instruments import Equity
from nautilus_trader.model.identifiers import Symbol, Venue, InstrumentId
from nautilus_trader.model.objects import Currency, Price, Quantity

from .config import TastytradeInstrumentProviderConfig
from .http.client import TastytradeHttpClient
from .common import TT_VENUE_DEFAULT_EQUITY
from .parsing.instruments import (
    parse_equity_payload,
    parse_equity_option_payload,
    parse_futures_option_payload,
)


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
        instrument = parse_equity_payload(data["data"])  # shape per docs
        # stamp timestamps
        # Instruments created in parser use placeholder timestamps; replace here
        ts = self._clock.timestamp_ns()
        object.__setattr__(instrument, "ts_event", ts)
        object.__setattr__(instrument, "ts_init", ts)
        self.add(instrument)

    async def load_equity_option_occ_async(self, occ_symbol: str) -> None:
        """Load a single equity option by OCC symbol (with spaces)."""
        encoded = quote(occ_symbol, safe="")
        data = await self._client._request("GET", f"/instruments/equity-options/{encoded}")
        instrument = parse_equity_option_payload(data["data"])
        ts = self._clock.timestamp_ns()
        object.__setattr__(instrument, "ts_event", ts)
        object.__setattr__(instrument, "ts_init", ts)
        self.add(instrument)

    async def load_futures_option_symbol_async(self, fut_opt_symbol: str) -> None:
        """Load a single futures option by full symbol (e.g., "./ESU3 E1DQ3 230803P3860")."""
        encoded = quote(fut_opt_symbol, safe="")
        data = await self._client._request("GET", f"/instruments/future-options/{encoded}")
        instrument = parse_futures_option_payload(data["data"])
        ts = self._clock.timestamp_ns()
        object.__setattr__(instrument, "ts_event", ts)
        object.__setattr__(instrument, "ts_init", ts)
        self.add(instrument)



