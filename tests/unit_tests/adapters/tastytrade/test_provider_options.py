import asyncio
from typing import Any

import pytest

from nautilus_trader.common.component import LiveClock
from nautilus_trader.adapters.tastytrade.config import TastytradeInstrumentProviderConfig
from nautilus_trader.adapters.tastytrade.http.client import TastytradeHttpClient
from nautilus_trader.adapters.tastytrade.providers import TastytradeInstrumentProvider
from nautilus_trader.model.identifiers import InstrumentId, Symbol, Venue


class DummySession:
    def __init__(self, path: str, payload: dict[str, Any]) -> None:
        self.path = path
        self.payload = payload

    def request(self, method: str, url: str, **kwargs: object) -> object:
        class _Resp:
            status = 200
            headers = {"Content-Type": "application/json"}

            def __init__(self, expected: str, payload: dict[str, Any]) -> None:
                self.expected = expected
                self._payload = payload

            async def __aenter__(self):
                return self

            async def __aexit__(self, exc_type, exc, tb):
                return False

            async def json(self) -> dict[str, Any]:
                assert url.endswith(self.expected)
                return self._payload

            async def text(self) -> str:
                return "{}"

        return _Resp(self.path, self.payload)


@pytest.mark.asyncio()
async def test_provider_loads_equity_option_occ() -> None:
    loop = asyncio.get_event_loop()
    clock = LiveClock()
    client = TastytradeHttpClient(loop=loop, clock=clock, base_url="https://api", request_timeout=5)
    payload = {
        "data": {
            "symbol": "SPY   230731C00393000",
            "underlying-symbol": "SPY",
            "strike-price": "393.0",
            "expiration-date": "2023-07-31",
            "option-type": "C",
            "shares-per-contract": 100,
            "tick-sizes": [{"value": "0.01"}],
        },
    }
    client._session = DummySession("/instruments/equity-options/SPY%20%20%20230731C00393000", payload)

    provider = TastytradeInstrumentProvider(client=client, clock=clock, config=TastytradeInstrumentProviderConfig())
    await provider.load_equity_option_occ_async("SPY   230731C00393000")

    # InstrumentId uses stripped spaces in symbol for internal consistency
    iid = InstrumentId(Symbol("SPY230731C00393000"), Venue("OPRA"))
    inst = provider.find(iid)
    assert inst is not None
    assert inst.price_increment.as_double() == 0.01


@pytest.mark.asyncio()
async def test_provider_loads_futures_option_symbol() -> None:
    loop = asyncio.get_event_loop()
    clock = LiveClock()
    client = TastytradeHttpClient(loop=loop, clock=clock, base_url="https://api", request_timeout=5)
    payload = {
        "data": {
            "symbol": "./ESU3 E1DQ3 230803P3860",
            "underlying-symbol": "/ESU3",
            "product-code": "ES",
            "exchange": "CME",
            "streamer-exchange-code": "XCME",
            "strike-price": "3860.0",
            "expiration-date": "2023-08-03",
            "option-type": "P",
            "multiplier": "1.0",
            "tick-sizes": [{"value": "0.05"}, {"value": "0.25"}],
        },
    }
    client._session = DummySession("/instruments/future-options/.%2FESU3%20E1DQ3%20230803P3860", payload)

    provider = TastytradeInstrumentProvider(client=client, clock=clock, config=TastytradeInstrumentProviderConfig())
    await provider.load_futures_option_symbol_async("./ESU3 E1DQ3 230803P3860")

    iid = InstrumentId(Symbol("./ESU3E1DQ3230803P3860"), Venue("XCME"))
    inst = provider.find(iid)
    assert inst is not None
    assert inst.price_increment.as_double() == 0.05


