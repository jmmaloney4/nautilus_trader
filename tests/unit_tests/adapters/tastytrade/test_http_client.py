from __future__ import annotations

import asyncio
import json
from types import TracebackType
from typing import Any

import pytest

from nautilus_trader.common.component import LiveClock
from nautilus_trader.adapters.tastytrade.http.client import TastytradeHttpClient


class DummyResp:
    def __init__(self, status: int, data: dict[str, object], content_type: str = "application/json") -> None:
        self.status = status
        self._data = data
        self.headers = {"Content-Type": content_type}

    async def __aenter__(self) -> DummyResp:
        return self

    async def __aexit__(
        self,
        exc_type: type[BaseException] | None,
        exc: BaseException | None,
        tb: TracebackType | None,
    ) -> bool:
        return False

    async def json(self) -> dict[str, object]:
        return self._data

    async def text(self) -> str:
        return json.dumps(self._data)


class DummySession:
    def __init__(self) -> None:
        self.closed = False
        self.calls: list[tuple[str, str]] = []

    async def close(self) -> None:
        self.closed = True

    def request(self, method: str, url: str, **kwargs: object) -> DummyResp:
        self.calls.append((method, url))
        # Simulate /sessions then /customers/me/accounts
        if url.endswith("/sessions"):
            return DummyResp(200, {"data": {"session-token": "abc", "csrf-token": "xyz"}})
        elif url.endswith("/customers/me/accounts"):
            return DummyResp(200, {"data": {"items": [{"account": {"account-number": "TEST123"}}]}})
        return DummyResp(404, {"error": "not found"})


@pytest.mark.asyncio()
async def test_http_client_login_and_accounts(monkeypatch: pytest.MonkeyPatch) -> None:
    loop = asyncio.get_event_loop()
    clock = LiveClock()
    client = TastytradeHttpClient(
        loop=loop,
        clock=clock,
        base_url="https://api.sandbox.tastytrade.com",
        username="user",
        password="pass",
        request_timeout=5,
    )

    dummy = DummySession()

    async def _connect_patch(self: TastytradeHttpClient) -> None:
        self._session = dummy
        await self._login()

    monkeypatch.setattr(TastytradeHttpClient, "connect", _connect_patch)

    await client.connect()
    data: dict[str, Any] = await client.list_customer_accounts()
    assert "data" in data
    assert data["data"]["items"][0]["account"]["account-number"] == "TEST123"
    await client.close()


