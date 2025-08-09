from __future__ import annotations

import asyncio
from dataclasses import dataclass
from typing import Any

import aiohttp

from nautilus_trader.common.component import LiveClock


@dataclass
class TastytradeSession:
    token: str
    csrf: str | None


class TastytradeHttpClient:
    """
    Minimal HTTP client for Tastytrade REST API.

    Handles session-based authentication for sandbox/live, basic retries,
    and a single example call to validate auth (list customer accounts).
    """

    def __init__(
        self,
        loop: asyncio.AbstractEventLoop,
        clock: LiveClock,
        base_url: str,
        username: str | None = None,
        password: str | None = None,
        request_timeout: int = 30,
    ) -> None:
        self._loop = loop
        self._clock = clock
        self._base_url = base_url.rstrip("/")
        self._username = username
        self._password = password
        self._timeout = aiohttp.ClientTimeout(total=request_timeout)
        self._session: aiohttp.ClientSession | None = None
        self._auth: TastytradeSession | None = None

    async def connect(self) -> None:
        if self._session is None:
            self._session = aiohttp.ClientSession(timeout=self._timeout)

        if self._username and self._password:
            await self._login()

    async def close(self) -> None:
        if self._session is not None:
            await self._session.close()
            self._session = None

    async def _login(self) -> None:
        assert self._session is not None
        # Tastytrade sandbox uses session login at /sessions
        url = f"{self._base_url}/sessions"
        async with self._session.post(
            url,
            json={"login": self._username, "password": self._password},
        ) as resp:
            if resp.status >= 300:
                text = await resp.text()
                raise RuntimeError(f"Tastytrade login failed: {resp.status} {text}")
            data = await resp.json()
            token = data.get("data", {}).get("session-token") or data.get("session-token")
            csrf = data.get("data", {}).get("csrf-token") or data.get("csrf-token")
            if not token:
                raise RuntimeError("Tastytrade login response missing session token")
            self._auth = TastytradeSession(token=token, csrf=csrf)

    def _headers(self) -> dict[str, str]:
        headers: dict[str, str] = {"accept": "application/json"}
        if self._auth:
            headers["Authorization"] = f"Bearer {self._auth.token}"
            if self._auth.csrf:
                headers["X-CSRF-Token"] = self._auth.csrf
        return headers

    async def _request(self, method: str, path: str, **kwargs: Any) -> Any:
        assert self._session is not None
        url = f"{self._base_url}{path}"
        headers = kwargs.pop("headers", {})
        headers.update(self._headers())
        async with self._session.request(method, url, headers=headers, **kwargs) as resp:
            if resp.status >= 300:
                text = await resp.text()
                raise RuntimeError(f"HTTP {resp.status} {path}: {text}")
            if "application/json" in resp.headers.get("Content-Type", ""):
                return await resp.json()
            return await resp.text()

    # Public wrapper for testing/mocking
    async def request(self, method: str, path: str, **kwargs: Any) -> Any:
        return await self._request(method, path, **kwargs)

    async def list_customer_accounts(self) -> Any:
        # Auth scope: customer is always 'me'
        return await self._request("GET", "/customers/me/accounts")


