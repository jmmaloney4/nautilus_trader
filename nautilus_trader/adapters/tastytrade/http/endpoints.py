from __future__ import annotations

from typing import Any

from .client import TastytradeHttpClient


async def get_equity(client: TastytradeHttpClient, symbol: str) -> dict[str, Any]:
    """Fetch a single equity instrument by symbol.

    Returns the raw `data` object from the API.
    """
    payload = await client._request("GET", f"/instruments/equities/{symbol}")
    return payload["data"]


async def list_equities(
    client: TastytradeHttpClient,
    symbols: list[str] | None = None,
    active_only: bool | None = None,
    is_index: bool | None = None,
    is_etf: bool | None = None,
) -> list[dict[str, Any]]:
    params: dict[str, Any] = {}
    if symbols:
        # Tastytrade supports symbol[]=AAPL&symbol[]=SPY form
        for idx, s in enumerate(symbols):
            params[f"symbol[{idx}]"] = s
    if active_only is not None:
        params["active"] = str(active_only).lower()
    if is_index is not None:
        params["is-index"] = str(is_index).lower()
    if is_etf is not None:
        params["is-etf"] = str(is_etf).lower()

    payload = await client._request("GET", "/instruments/equities", params=params)
    return payload.get("data", {}).get("items", [])


async def get_equity_option(client: TastytradeHttpClient, occ_symbol: str) -> dict[str, Any]:
    """Fetch a single equity option by OCC symbol (with spaces)."""
    # The caller must URL-encode upstream; we keep this low-level helper simple
    payload = await client._request("GET", f"/instruments/equity-options/{occ_symbol}")
    return payload["data"]


async def list_equity_options(
    client: TastytradeHttpClient,
    symbols: list[str] | None = None,
    active: bool | None = None,
) -> list[dict[str, Any]]:
    params: dict[str, Any] = {}
    if symbols:
        for idx, s in enumerate(symbols):
            params[f"symbol[{idx}]"] = s
    if active is not None:
        params["active"] = str(active).lower()
    payload = await client._request("GET", "/instruments/equity-options", params=params)
    return payload.get("data", {}).get("items", [])


