# Phase 0 — validated sandbox payload schemas

Captured from the live certification/sandbox environment (`api.cert.tastyworks.com`)
via `examples/phase0_spike.rs`. **Types only — no values.** Field names are the
public API schema; all monetary fields are decimal **strings**. JSON keys are
`kebab-case` and responses are wrapped in a `{ "data": ..., "context": "<path>" }`
envelope. The sandbox resets every 24h, so positions/orders are empty unless an
order has been placed in the current window.

Auth used: session login (`POST /sessions`, login/password → bare `Authorization`
header). OAuth (`POST /oauth/token` → `Bearer`) is also implemented but not yet
exercised (no OAuth app credentials available).

## POST /sessions (auth)
`data.session-token` (string), `data.session-expiration` (string, optional),
`data.remember-token` (string, optional). Token used directly as the
`Authorization` header value (no `Bearer` prefix). ~24h lifetime.

## GET /customers/me/accounts
```
data.items[].account.{
  account-number, account-type-name, nickname, margin-or-cash,
  investment-objective, suitable-options-level,
  is-futures-approved, day-trader-status,
  is-closed, is-firm-error, is-firm-proprietary, is-foreign,
  opened-at, created-at
}
data.items[].authority-level
```
Modeled by `http::models::AccountsResponse` (decodes the account-number; other
fields ignored for now).

## GET /accounts/{account_number}/balances
Flat object of decimal **strings** under `data`. Key fields for the Phase 1
`AccountState` / margin mapping:
```
account-number, currency, snapshot-date, updated-at,
cash-balance, net-liquidating-value, equity-buying-power,
derivative-buying-power, used-derivative-buying-power,
day-trading-buying-power, cash-available-to-withdraw,
maintenance-requirement, maintenance-excess, margin-equity,
reg-t-margin-requirement, reg-t-call-value, maintenance-call-value,
long-equity-value, short-equity-value, long-derivative-value,
short-derivative-value, long-futures-value, short-futures-value,
futures-margin-requirement, cryptocurrency-margin-requirement,
pending-cash, buying-power-adjustment, ...
```
(~70 fields total; full list in the captured payload. Not yet typed.)

## GET /api-quote-tokens (DXLink)
```
data.{ token, dxlink-url, websocket-url, level, issued-at, expires-at }
```
Modeled by `http::models::QuoteTokenData`. Feeds the DXLink market-data stream.
`dxlink-url` is the endpoint to open; `token` (~24h TTL) authenticates the feed.

## POST /accounts/{account_number}/orders/dry-run
Validates an order and returns its impact **without** placing it. Validated
against sandbox (1-share AAPL limit). This is the canonical source for the order
and fee shapes (Phase 2 mapping).
```
data.order.{
  account-number, status, order-type, time-in-force, price, price-effect,
  size, cancellable, editable, edited, global-request-id,
  underlying-symbol, underlying-instrument-type, updated-at (number, epoch ms)
}
data.order.legs[].{
  action, instrument-type, symbol, quantity (number),
  remaining-quantity (number), fills[]
}
data.buying-power-effect.{
  change-in-buying-power(+ -effect), current-buying-power, new-buying-power,
  change-in-margin-requirement, isolated-order-margin-requirement,
  impact, is-spread
}
data.fee-calculation.{
  commission, regulatory-fees, clearing-fees, total-fees   (each with -effect),
  *-breakdown[].{ name, value, effect },
  rebates, proprietary-index-option-fees, per-quantity
}
data.warnings[].{ code, message }
data.notes[]
```
A **placed live order** (`GET /accounts/{n}/orders/live`, `items[]`) mirrors
`data.order` above, with `legs[].fills[]` populated as the order executes.

## GET /accounts/{account_number}/positions
`data.items[]` — **shape still pending**: empty in all captured runs (dry-run
does not open a position). Needs an actual fill in the sandbox to capture the
position leg/quantity/cost fields.
