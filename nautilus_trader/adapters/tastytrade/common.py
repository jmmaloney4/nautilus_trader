from __future__ import annotations

from dataclasses import dataclass


# Venue constants
TT_VENUE_DEFAULT_EQUITY = "OTC"  # Fallback when listed-market is unavailable
TT_VENUE_DEFAULT_OPTIONS = "OPRA"  # Equity options default


@dataclass(frozen=True)
class TastytradeEnv:
    base_url: str
    streamer_url: str


SANDBOX = TastytradeEnv(
    base_url="https://api.sandbox.tastytrade.com/",  # placeholder; adjust if needed
    streamer_url="wss://streamer.sandbox.tastytrade.com/",  # placeholder
)

LIVE = TastytradeEnv(
    base_url="https://api.tastytrade.com/",
    streamer_url="wss://streamer.tastytrade.com/",
)


