import asyncio
from typing import Any

import pytest

from nautilus_trader.common.component import LiveClock
from nautilus_trader.adapters.tastytrade.config import TastytradeInstrumentProviderConfig
from nautilus_trader.adapters.tastytrade.http.client import TastytradeHttpClient
from nautilus_trader.adapters.tastytrade.providers import TastytradeInstrumentProvider
from nautilus_trader.model.identifiers import InstrumentId, Symbol, Venue


class DummySession:
    def request(self, method: str, url: str, **kwargs: object) -> object:
        class _Resp:
            status = 200
            headers = {"Content-Type": "application/json"}

            async def __aenter__(self):
                return self

            async def __aexit__(self, exc_type, exc, tb):
                return False

            async def json(self) -> dict[str, Any]:
                assert method == "GET"
                assert url.endswith("/instruments/equities/AAPL")
                return {
                    "data": {
                        "symbol": "AAPL",
                        "listed-market": "XNAS",
                        "tick-sizes": [{"value": "0.01"}],
                        "is-fractional-quantity-eligible": True,
                    },
                }

            async def text(self) -> str:
                return "{}"

        return _Resp()


@pytest.mark.asyncio()
async def test_provider_loads_equity() -> None:
    loop = asyncio.get_event_loop()
    clock = LiveClock()

    client = TastytradeHttpClient(loop=loop, clock=clock, base_url="https://api", request_timeout=5)
    # Inject dummy session without calling connect/login
    client._session = DummySession()

    provider = TastytradeInstrumentProvider(
        client=client,
        clock=clock,
        config=TastytradeInstrumentProviderConfig(),
    )

    iid = InstrumentId(Symbol("AAPL"), Venue("XNAS"))
    await provider.load_async(iid)
    inst = provider.find(iid)
    assert inst is not None
    assert inst.id == iid
    assert inst.price_increment.as_double() == 0.01


