# tastytrade Adapter

`nautilus-tastytrade` is the Rust adapter that connects NautilusTrader to the
[tastytrade](https://developer.tastytrade.com/) brokerage REST and streaming APIs.

It targets an options/equities retail-broker surface. The architecture mirrors
the *shape* of the Interactive Brokers adapter (separate config, factories,
shared transport, instrument provider, data client, execution client, and
reconnect reconciliation) but is tastytrade-native: there is no SMART routing,
exchange/MIC selection, BAG combo reconstruction, or gateway lifecycle.

## Transport model

- **REST + OAuth2.** Access tokens last ~15 minutes and are refreshed from a
  long-lived refresh token via `POST /oauth/token`. REST is the authoritative
  source of truth for order/account state.
- **Account notification WebSocket.** One-directional notification stream;
  treated as low-latency *hints*, with REST used to reconcile final state.
- **DXLink market data.** Requires a separate quote token from
  `/api-quote-tokens` (≈24h TTL); the returned `dxlink-url` hosts the feed.

## Status

Phase 0 (auth + connectivity spike). Implemented so far:

- `common`: venue constants, environment, credentials, URLs.
- `config`: data/exec client config.
- `http`: OAuth refresh + the Phase 0 REST endpoints (accounts, balances,
  positions, live orders, quote tokens), with raw-payload capture.
- `examples/phase0_spike.rs`: runnable sandbox spike.

Not yet implemented: WebSocket clients, instrument provider, data/execution
clients, factories, PyO3 bindings. See the feasibility research at
`cavinsresearch/zeus/docs/internal/research/hades/2026-05-28-tastytrade-adapter.md`.

> The exact request/response shapes (OAuth body field names, order-status
> enumeration, fee decomposition) are provisional until validated against live
> sandbox payloads — which is the purpose of the Phase 0 spike. Raw payloads are
> persisted so the typed mappings can be corrected without losing audit trail.
