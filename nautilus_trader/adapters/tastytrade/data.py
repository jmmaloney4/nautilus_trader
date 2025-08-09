from __future__ import annotations

import asyncio

from nautilus_trader.cache.cache import Cache
from nautilus_trader.common.component import LiveClock, MessageBus
from nautilus_trader.live.data_client import LiveMarketDataClient
from nautilus_trader.model.enums import BookType
from nautilus_trader.model.identifiers import ClientId
from nautilus_trader.data.messages import SubscribeOrderBook

from .config import TastytradeDataClientConfig
from .providers import TastytradeInstrumentProvider


class TastytradeMarketDataClient(LiveMarketDataClient):
    """Skeleton L1 market data client. Streaming will be wired in later steps."""

    def __init__(
        self,
        loop: asyncio.AbstractEventLoop,
        msgbus: MessageBus,
        cache: Cache,
        clock: LiveClock,
        instrument_provider: TastytradeInstrumentProvider,
        config: TastytradeDataClientConfig,
        name: str | None = None,
    ) -> None:
        super().__init__(
            loop=loop,
            client_id=ClientId(name or "TASTYTRADE"),
            venue=None,
            msgbus=msgbus,
            cache=cache,
            clock=clock,
            instrument_provider=instrument_provider,
            config=config,
        )

    async def _connect(self) -> None:  # pragma: no cover - wired later
        await self.instrument_provider.initialize()

    async def _disconnect(self) -> None:  # pragma: no cover - wired later
        return

    async def _subscribe_order_book_deltas(self, command: SubscribeOrderBook) -> None:
        if command.book_type in (BookType.L2_MBP, BookType.L3_MBO):
            self._log.error("Order book depth not supported by Tastytrade adapter at this time")
        return


