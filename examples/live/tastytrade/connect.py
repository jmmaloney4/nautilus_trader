# Minimal example to test Tastytrade HTTP auth and list accounts
import asyncio
import os

from nautilus_trader.common.component import LiveClock
from nautilus_trader.adapters.tastytrade.http.client import TastytradeHttpClient


async def main() -> None:
    clock = LiveClock()
    loop = asyncio.get_event_loop()
    base_url = os.getenv("TT_BASE_URL", "https://api.sandbox.tastytrade.com")
    username = os.getenv("TT_USERNAME")
    password = os.getenv("TT_PASSWORD")

    client = TastytradeHttpClient(
        loop=loop,
        clock=clock,
        base_url=base_url,
        username=username,
        password=password,
        request_timeout=30,
    )

    await client.connect()
    accounts = await client.list_customer_accounts()
    print(accounts)
    await client.close()


if __name__ == "__main__":
    asyncio.run(main())
