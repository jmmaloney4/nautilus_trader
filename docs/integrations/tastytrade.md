## Tastytrade adapter design and implementation plan

This document specifies a Python-first adapter for Tastytrade, modeled after the Interactive Brokers (IB) integration, with scope limited to Equities, Equity Options, and Futures Options. It defines architecture, symbology/venue mapping, data and execution capabilities, configuration, precision/rounding policy, testing, and a phased delivery plan.

### Goals and scope

- Live trading and data adapter for Tastytrade.
- Asset classes in scope:
  - Equities
  - Equity Options
  - Futures Options
- Live market data: L1 quotes and trades (no order book depth initially).
- Historical data: implement all historical endpoints exposed by Tastytrade relevant to the scoped asset classes (bars/quotes/trades where available).
- Multi-leg option orders (spreads/combos): later phase.
- OCO/OTO/brackets: later phase.

Out of scope for MVP:
- Order book depth (L2/L3), advanced order types beyond those explicitly supported by the API, complex risk management features.

### High-level architecture

Components mirror the IB adapter to fit Nautilus abstractions:

- `TastytradeInstrumentProvider`
  - Resolves instruments by symbol or identifier, loads option chains, maps venue-native JSON to Nautilus `Instrument` classes, and persists raw info for downstream components.

- `TastytradeMarketDataClient` (subclass of `LiveMarketDataClient`)
  - Streams L1 quotes and trades via the Tastytrade streamer (DXLink). Implements request handlers for historical quotes/trades/bars where supported.

- `TastytradeExecutionClient` (subclass of `LiveExecutionClient`)
  - Handles order submission/modify/cancel, consumes account/order/fill events (via account stream where available; otherwise polling fallback), and generates Nautilus reports and events.

- HTTP client and endpoint modules
  - Auth/session management, typed calls for instruments, accounts, orders, transactions, and historical data.

- WebSocket clients
  - Market streamer client (quotes/trades subscribe/unsubscribe).
  - Account streamer client (order status/fills/balances/positions updates), if available in target environment.

- Parsing/mapping utilities
  - Instrument symbol <-> `InstrumentId` conversions, order type/side/TIF/status mapping, price/quantity scaling helpers.

File layout (Python-first implementation):

- `nautilus_trader/adapters/tastytrade/`
  - `__init__.py`
  - `common.py` (constants/enums/utilities, venue codes)
  - `config.py` (InstrumentProvider, DataClient, ExecClient configs)
  - `providers.py` (`TastytradeInstrumentProvider`)
  - `data.py` (`TastytradeMarketDataClient`)
  - `execution.py` (`TastytradeExecutionClient`)
  - `factories.py` (helpers to wire HTTP/WS clients + provider/client instances)
  - `parsing/`
    - `instruments.py` (JSON -> Instrument; InstrumentId <-> symbol)
    - `data.py` (stream/historical -> QuoteTick/TradeTick/Bar)
    - `execution.py` (order/side/TIF/status mapping; effect derivation)
    - `price_conversion.py` (tick-size/display-factor scaling)
  - `http/`
    - `client.py` (auth/session, base request, rate limits)
    - `endpoints.py` (accounts, instruments, orders, market data)
    - `errors.py`
  - `websocket/`
    - `market.py` (DXLink client)
    - `account.py` (account streamer client)
    - `schemas.py` (message typing)

- `examples/live/tastytrade/`
  - `connect.py`, `subscribe_spy_quotes.py`, `place_cancel_equity_order.py`, `load_option_chain.py` …

### Symbology and venues

Nautilus uses `InstrumentId(Symbol, Venue)`.

- Equities
  - Symbol: e.g., `"AAPL"` from `/instruments/equities`.
  - Venue: prefer specific `listed-market` (e.g., `XNAS`, `ARCX`) when present; otherwise fall back to a default (e.g., `OTC`).

- Equity options
  - OCC symbol: `"SPY   230731C00393000"` from option chain endpoints.
  - Venue: prefer specific exchange if API supplies it; otherwise default to `OPRA`.

- Futures options
  - Symbol: strings starting with `./` (e.g., `"./ESU3 E1DQ3 230803P3860"`).
  - Venue: use `streamer-exchange-code` from the instrument payload (e.g., `XCME`, `XNYM`).

Symbol conversion policies:

- InstrumentId -> venue symbol: validate and encode reserved characters in URL paths (`/` and `./` encoded for REST), use `streamer-symbol` for streamer subscriptions when available.
- Venue symbol -> InstrumentId: parse the appropriate Tastytrade instrument objects, assign venue per rules above.

### Instruments: mapping to Nautilus classes

Implement JSON-to-Instrument mapping with precision and increments derived from API-provided fields.

- Equity -> `Equity`
  - Fields: `symbol`, `currency`, `tick-sizes` (derive `price_increment`/`price_precision`), `is-fractional-quantity-eligible` (size precision), `listed-market`.
  - Lot size: default 1 share (may use `is-fractional-quantity-eligible` and quantity precision endpoint to set `size_precision`).

- Equity Option -> `OptionContract`
  - Fields: `symbol` (OCC), `underlying-symbol`, `strike-price`, `expiration-date`, `option-type` (C/P), `shares-per-contract` (multiplier/lot size), `currency`, tick sizes.

- Futures Option -> `OptionContract`
  - Fields: option symbol `./…`, `underlying-symbol` (future), `product-code`, `strike-price`, `expiration-date`, `option-type`, `exercise-style`, `multiplier` (often 1), `display-factor`, tick sizes.

Common policies:

- `price_increment`/`price_precision`: from `tick-sizes`. If multiple ranges exist, pick the lowest tick for precision and preserve the list in `info` for reference.
- `size_precision`: from quantity precision endpoint (`/instruments/quantity-decimal-precisions`) or conservative defaults per asset class.
- `info`: persist the raw instrument payload plus derived fields (`streamer-symbol`, `streamer-exchange-code`, `display-factor`, etc.).

### Live market data

Streamer (DXLink):

- Quotes (bid/ask) and trades supported for equities/options/futures options using `streamer-symbol` and `streamer-exchange-code` where applicable.
- Client responsibilities:
  - Auth/init
  - Subscribe/unsubscribe per instrument
  - Parse messages to Nautilus `QuoteTick` and `TradeTick` with correct precision.

Nautilus API coverage:

- Implement `_subscribe_quote_ticks`, `_unsubscribe_quote_ticks`, `_subscribe_trade_ticks`, `_unsubscribe_trade_ticks`.
- For `_subscribe_order_book_deltas` return an error/info (not supported initially).

### Historical data

Implement where exposed by API:

- Quotes/trades history: map to `RequestQuoteTicks` and `RequestTradeTicks` (instrument-scoped, time-bounded, with pagination handling).
- Bars/candles: if available for underlying assets, support `RequestBars` for time-based aggregations; otherwise keep disabled.

Policies:

- Rate limits: implement backoff/retry and pagination, surface request status over `msgbus` (similar to IB data client).
- For coarse-grained historical APIs returning aggregates, convert into Nautilus `Bar` if semantics align; otherwise return ticks.

### Execution

Connection workflow:

1) Authenticate HTTP client.
2) Resolve `account-number` via `/customers/me/accounts` when not explicitly configured.
3) Subscribe to account streamer (if available) for order state/fills; otherwise polling fallback for open orders and transactions.

Supported order types (subject to Tastytrade API capabilities):

- MARKET, LIMIT, STOP_MARKET, STOP_LIMIT
- TIF: DAY, GTC, GTD (with explicit `expire_at` when GTD)
- Side: BUY/SELL; options require action semantics (Buy/Sell to Open/Close) derived from position context (see below)

Order mapping

- Side and open/close:
  - Equities: BUY/SELL (no open/close).
  - Options: derive Open/Close from current position (in cache) and signed order quantity. If ambiguous, expose a config override and/or explicit tag on order.
  - Futures options: same policy as equity options.

- Price fields:
  - LIMIT price → `limit-price`
  - STOP trigger → `stop-price`
  - STOP_LIMIT → both fields
  - No trailing variants initially (reject unsupported types with a clear reason).

- Time-in-force:
  - DAY/GTC/GTD; when GTD, include `gtd` timestamp in venue format (UTC ISO8601), and reject if missing.

- Price effect derivation (Debit/Credit/None):
  - Equities: BUY = Debit, SELL = Credit.
  - Options: Buy-to-Open/Close = Debit; Sell-to-Open/Close = Credit.
  - Futures options: same as options.
  - Validation: if the API requires a `price-effect` field, derive using the rules above. Provide a per-order override tag and a config default. If an API validation error indicates mismatch, surface a clear rejection and include the calculated effect.

  Note: For single-leg orders covered by MVP, the automatic derivation is unambiguous under the rules above. Net-effect issues arise only for multi-leg orders (phase 4).

Order lifecycle and events

- On submit: place order, emit `generate_order_submitted`.
- On venue ACK/state changes: map to Nautilus `OrderStatus` (SUBMITTED, ACCEPTED, PARTIALLY_FILLED, FILLED, PENDING_CANCEL, CANCELED, REJECTED).
- Fills: from account stream or transactions (type Trade). Emit `generate_order_filled` with price/quantity scaled via instrument precision.
- Modifications: allowed fields depend on Tastytrade (price, quantity; reject unsupported changes).
- Cancel single/all: direct endpoints or by order id.

Reports

- `generate_order_status_report(s)`: open orders mapping.
- `generate_fill_reports`: from transactions filtered by type Trade within time range.
- `generate_position_status_reports`: positions endpoint mapped to Nautilus positions.

### Configuration

`TastytradeInstrumentProviderConfig`

- `load_all: bool` (default False)
- `load_ids: frozenset[InstrumentId] | None`
- Filters: `active_only`, `asset_classes` (equity/equity_option/future_option), chain build toggles for options (by underlying/product), pagination policy

`TastytradeDataClientConfig`

- `base_url: str` (sandbox/live)
- `streamer_url: str`
- `username: str | None`, `password: str | None` or token-based auth per Tasty auth patterns
- `connection_timeout: int`, `request_timeout: int`
- `handle_revised_bars: bool` (if bars supported), `ignore_quote_tick_size_updates: bool`

`TastytradeExecClientConfig`

- `base_url: str`
- `account_number: str | None`
- `connection_timeout: int`
- `price_effect_default: Literal["Debit","Credit","None"] | None` (fallback only)
- `override_open_close_policy: Optional[Callable]` (advanced users can inject policy)

### Precision and rounding policy

Strict correctness is required; the adapter must not silently round or truncate values.

- Prices: validate against instrument `price_increment` (tick-size). If not aligned, reject the order with a descriptive error that includes the nearest valid increments.
- Sizes/quantities: validate against `size_precision` and any lot/multiplier constraints. If not aligned, reject with a descriptive error.
- Historical data: preserve precision provided by the API. Convert to Nautilus types with exact precision consistent with `Instrument` definitions.

### HTTP/WS details

- HTTP client: retry with exponential backoff on 5xx and 429 (respect `Retry-After` when present). Timeouts and circuit-breaker style suppression for repeated failures.
- Authentication: follow Tasty auth patterns (session + token), store securely in memory; refresh when needed.
- Pagination: iterate until complete; expose progress via `msgbus` for long-running requests.
- WebSocket: automatic reconnect with backoff and resubscribe logic.

### Error handling and logging

- Map API errors to clear adapter exceptions with the upstream payload attached for diagnostics.
- When rejecting an order client-side (precision/TIF/effect), include the computed rationale and hints to correct.
- Guard against nil/empty fields in streamed messages; drop invalid ticks with a warning counter.

### Implementation phases

Phase 1: Scaffolding, Instruments, L1 quotes/trades for Equities

- Create adapter package layout and configs.
- Implement HTTP auth and basic client.
- Implement `TastytradeInstrumentProvider` for Equities and Equity Options chain discovery (read-only first), and Futures Options discovery by product.
- Implement market streamer client; support `_subscribe_quote_ticks` and `_subscribe_trade_ticks` for equities.
- Examples: connect + subscribe AAPL/SPY.

Phase 2: Execution for Equities

- Implement order submit/modify/cancel for equities.
- Implement account resolution and account stream/polling for order status/fills.
- Implement reports: order status, fills, positions.
- Examples: place/cancel equity orders; basic e2e.

Phase 3: L1 and execution for Equity Options and Futures Options

- Extend instrument provider to full option chain retrieval and single-leg option instrument mapping.
- Extend market streamer to option/future option symbols.
- Implement single-leg option and futures option orders with open/close derivation and `price-effect` mapping.
- Historical endpoints for quotes/trades/bars as available for these asset classes.

Phase 4: Multi-leg options (combos/spreads)

- Add multi-leg order construction (legs with appropriate actions and ratios), net price effect derivation, and coherent fill event handling.
- Examples: submit vertical spread; cancel/replace.

Later: OCO/OTO/brackets

- Add when API surfaces reliable primitives; define clear mapping to Nautilus contingencies.

### Testing plan

- Unit tests: symbol parsing/formatting, instrument mappers, order mapping (type/side/TIF/effect), price/size validators, historical pagination.
- Integration tests (sandbox): connect, subscribe quotes/trades, instrument load, submit/modify/cancel equity and single-leg option/future option orders; verify events and reports.
- Resilience tests: WS reconnect and resubscribe, HTTP 429 backoff, timeout handling.

### Example usage (to be added under `examples/live/tastytrade/`)

- `connect.py`: authenticate, list accounts, resolve `account_number`.
- `subscribe_spy_quotes.py`: load SPY equity + subscribe quotes/trades.
- `load_option_chain.py`: fetch SPY equity option chain and emit instruments.
- `place_cancel_equity_order.py`: place limit, cancel, observe events.
- `place_option_order.py`: single-leg option limit order.

### Notes on automatic price-effect derivation

For MVP single-leg orders, automatic derivation is unambiguous under standard conventions:

- Equities: BUY = Debit; SELL = Credit.
- Options and Futures Options: Buy-to-Open/Close = Debit; Sell-to-Open/Close = Credit.

Ambiguities can arise for multi-leg combos (later phase) where the net effect depends on aggregated legs; for those the adapter will compute the net and allow explicit override.

To mitigate mismatches, the execution client will:

- Compute a proposed `price-effect`.
- Allow an explicit override via config or per-order tag.
- If the venue rejects due to effect mismatch, surface the error with the computed effect to guide correction.

### Open design decisions (resolved for MVP)

- Venues: prefer specific exchange mapping where available (equities: `listed-market`; futures options: `streamer-exchange-code`), else `OPRA` for equity options.
- Market data: L1 quotes and trades only.
- Historical data: implement all available via Tastytrade for the scoped assets.
- No implicit rounding/truncation; strict validation against tick-size/precision; reject on mismatch.

### Security

- Never log secrets; mask in repr/str.
- Store tokens in memory only; refresh on expiry.
- Support sandbox and live endpoints; default to sandbox for examples.

### Performance considerations

- Batch symbol requests where API supports arrays (e.g., `/instruments/*?symbol[]=`) to reduce round-trips.
- Use asyncio tasks and gather for concurrent fetches within API rate limits.
- Minimize JSON allocations; reuse parsers and caches; avoid copying large payloads in hot paths.

---

This design mirrors the proven IB adapter patterns while accommodating Tastytrade’s symbology, streamer model, and REST surface. It prioritizes correctness (no silent rounding), clear mappings, and phased functionality aligned with the requested scope.


