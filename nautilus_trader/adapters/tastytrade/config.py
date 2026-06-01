from __future__ import annotations

from typing import Literal

from nautilus_trader.config import InstrumentProviderConfig
from nautilus_trader.config import LiveDataClientConfig
from nautilus_trader.config import LiveExecClientConfig


class TastytradeInstrumentProviderConfig(InstrumentProviderConfig):
    """
    Configuration for `TastytradeInstrumentProvider`.

    Parameters
    ----------
    load_all : bool, default False
        Whether to load all instruments on start (not recommended).
    active_only : bool, default True
        Filter by active instruments only where supported by the API.
    asset_classes : frozenset[str] | None
        Asset classes to consider ("equity", "equity_option", "future_option").
    """

    active_only: bool = True
    asset_classes: frozenset[str] | None = None


class TastytradeDataClientConfig(LiveDataClientConfig):
    """
    Configuration for `TastytradeMarketDataClient`.

    Parameters
    ----------
    environment : Literal["sandbox", "live"], default "sandbox"
        Selects Tastytrade endpoints.
    base_url : str | None
        Override base URL (rarely needed).
    streamer_url : str | None
        Override streamer WS URL (rarely needed).
    username : str | None
        Username for session-based auth (sandbox or live).
    password : str | None
        Password for session-based auth.
    connection_timeout : int, default 60
        Seconds to wait for initial connectivity.
    request_timeout : int, default 30
        Seconds to wait for REST request completion.
    """

    environment: Literal["sandbox", "live"] = "sandbox"
    base_url: str | None = None
    streamer_url: str | None = None
    username: str | None = None
    password: str | None = None
    connection_timeout: int = 60
    request_timeout: int = 30


class TastytradeExecClientConfig(LiveExecClientConfig):
    """
    Configuration for `TastytradeExecutionClient`.

    Parameters
    ----------
    environment : Literal["sandbox", "live"], default "sandbox"
        Selects Tastytrade endpoints.
    base_url : str | None
        Override base URL.
    username : str | None
        Username for session-based auth.
    password : str | None
        Password for session-based auth.
    account_number : str | None
        Explicitly set account number; otherwise resolved via `/customers/me/accounts`.
    connection_timeout : int, default 60
        Seconds to wait for initial connectivity.
    """

    environment: Literal["sandbox", "live"] = "sandbox"
    base_url: str | None = None
    username: str | None = None
    password: str | None = None
    account_number: str | None = None
    connection_timeout: int = 60


