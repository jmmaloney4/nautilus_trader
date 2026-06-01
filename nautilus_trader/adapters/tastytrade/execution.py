from __future__ import annotations

import asyncio

from nautilus_trader.cache.cache import Cache
from nautilus_trader.common.component import LiveClock, MessageBus
from nautilus_trader.live.execution_client import LiveExecutionClient
from nautilus_trader.model.enums import AccountType, OmsType
from nautilus_trader.model.identifiers import AccountId, ClientId

from .config import TastytradeExecClientConfig
from .providers import TastytradeInstrumentProvider


class TastytradeExecutionClient(LiveExecutionClient):
    """Skeleton execution client. Order flows will be wired in subsequent steps."""

    def __init__(
        self,
        loop: asyncio.AbstractEventLoop,
        msgbus: MessageBus,
        cache: Cache,
        clock: LiveClock,
        instrument_provider: TastytradeInstrumentProvider,
        config: TastytradeExecClientConfig,
        account_id: AccountId | None = None,
        name: str | None = None,
    ) -> None:
        super().__init__(
            loop=loop,
            client_id=ClientId(name or "TASTYTRADE"),
            venue=None,
            oms_type=OmsType.NETTING,
            instrument_provider=instrument_provider,
            account_type=AccountType.MARGIN,
            base_currency=None,
            msgbus=msgbus,
            cache=cache,
            clock=clock,
            config=config,
        )
        if account_id is not None:
            self._set_account_id(account_id)

    async def _connect(self) -> None:  # pragma: no cover - wired later
        await self.instrument_provider.initialize()
        self._set_connected(True)

    async def _disconnect(self) -> None:  # pragma: no cover - wired later
        self._set_connected(False)


