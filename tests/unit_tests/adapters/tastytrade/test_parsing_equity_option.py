from nautilus_trader.adapters.tastytrade.parsing.instruments import parse_equity_option_payload


def test_parse_equity_option_payload_basic() -> None:
    payload = {
        "symbol": "SPY   230731C00393000",
        "underlying-symbol": "SPY",
        "strike-price": "393.0",
        "expiration-date": "2023-07-31",
        "option-type": "C",
        "shares-per-contract": 100,
        "tick-sizes": [{"value": "0.01"}],
    }

    inst = parse_equity_option_payload(payload)
    assert inst.raw_symbol.value.strip() == "SPY   230731C00393000"
    assert inst.price_increment.as_double() == 0.01
    assert inst.lot_size.as_double() == 100


