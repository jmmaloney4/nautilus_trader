from nautilus_trader.adapters.tastytrade.parsing.instruments import parse_futures_option_payload


def test_parse_futures_option_payload_basic() -> None:
    payload = {
        "symbol": "./ESU3 E1DQ3 230803P3860",
        "underlying-symbol": "/ESU3",
        "product-code": "ES",
        "exchange": "CME",
        "streamer-exchange-code": "XCME",
        "strike-price": "3860.0",
        "expiration-date": "2023-08-03",
        "option-type": "P",
        "multiplier": "1.0",
        "tick-sizes": [{"value": "0.05"}, {"value": "0.25"}],
    }

    inst = parse_futures_option_payload(payload)
    assert inst.raw_symbol.value.startswith("./ESU3")
    assert inst.price_increment.as_double() == 0.05
    assert inst.lot_size.as_double() == 1


