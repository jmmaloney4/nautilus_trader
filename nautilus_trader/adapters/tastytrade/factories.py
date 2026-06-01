from __future__ import annotations

import asyncio
from functools import lru_cache

from nautilus_trader.cache.cache import Cache
from nautilus_trader.common.component import LiveClock, MessageBus
from nautilus_trader.live.factories import LiveDataClientFactory, LiveExecClientFactory

from .config import (
    TastytradeDataClientConfig,
    TastytradeExecClientConfig,
    TastytradeInstrumentProviderConfig,
)
from .data import TastytradeMarketDataClient
from .execution import TastytradeExecutionClient
from .http.client import TastytradeHttpClient
from .providers import TastytradeInstrumentProvider


def _resolve_base_urls(config_env: str) -> tuple[str, str]:
    if config_env == "live":
        base = "https://api.tastytrade.com"
        ws = "wss://streamer.tastytrade.com"
    else:
        base = "https://api.sandbox.tastytrade.com"
        ws = "wss://streamer.sandbox.tastytrade.com"
    return base, ws


@lru_cache(1)
def get_cached_tastytrade_http_client(
    loop: asyncio.AbstractEventLoop,
    clock: LiveClock,
    base_url: str,
    username: str | None,
    password: str | None,
    request_timeout: int,
) -> TastytradeHttpClient:
    return TastytradeHttpClient(
        loop=loop,
        clock=clock,
        base_url=base_url,
        username=username,
        password=password,
        request_timeout=request_timeout,
    )


@lru_cache(1)
def get_cached_tastytrade_instrument_provider(
    client: TastytradeHttpClient,
    clock: LiveClock,
    config: TastytradeInstrumentProviderConfig,
) -> TastytradeInstrumentProvider:
    return TastytradeInstrumentProvider(
        client=client,
        clock=clock,
        config=config,
    )


class TastytradeLiveDataClientFactory(LiveDataClientFactory):
    @staticmethod
    def create(
        loop: asyncio.AbstractEventLoop,
        name: str,
        config: TastytradeDataClientConfig,
        msgbus: MessageBus,
        cache: Cache,
        clock: LiveClock,
    ) -> TastytradeMarketDataClient:
        base_url_default, ws_url_default = _resolve_base_urls(config.environment)
        client = get_cached_tastytrade_http_client(
            loop=loop,
            clock=clock,
            base_url=config.base_url or base_url_default,
            username=config.username,
            password=config.password,
            request_timeout=config.request_timeout,
        )

        provider = get_cached_tastytrade_instrument_provider(
            client=client,
            clock=clock,
            config=config.instrument_provider,
        )

        return TastytradeMarketDataClient(
            loop=loop,
            msgbus=msgbus,
            cache=cache,
            clock=clock,
            instrument_provider=provider,
            config=config,
            name=name,
        )


class TastytradeLiveExecClientFactory(LiveExecClientFactory):
    @staticmethod
    def create(
        loop: asyncio.AbstractEventLoop,
        name: str,
        config: TastytradeExecClientConfig,
        msgbus: MessageBus,
        cache: Cache,
        clock: LiveClock,
    ) -> TastytradeExecutionClient:
        base_url_default, _ = _resolve_base_urls(config.environment)
        client = get_cached_tastytrade_http_client(
            loop=loop,
            clock=clock,
            base_url=config.base_url or base_url_default,
            username=config.username,
            password=config.password,
            request_timeout=60,
        )

        provider = get_cached_tastytrade_instrument_provider(
            client=client,
            clock=clock,
            config=config.instrument_provider,
        )

        return TastytradeExecutionClient(
            loop=loop,
            msgbus=msgbus,
            cache=cache,
            clock=clock,
            instrument_provider=provider,
            config=config,
            account_id=None,
            name=name,
        )


