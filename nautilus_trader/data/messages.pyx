cdef class ConsolidatedBBO(Data):
    """
    Represents a consolidated best bid and offer (CBBO) message.

    Parameters
    ----------
    ts_init : uint64_t
        UNIX timestamp (nanoseconds) when the object was initialized.
    ts_event : uint64_t
        UNIX timestamp (nanoseconds) when the event occurred.
    ts_in_delta : uint64_t
        UNIX timestamp (nanoseconds) when the event was ingested by the data source.
    ts_recv : uint64_t
        UNIX timestamp (nanoseconds) when the event was received by the system.
    instrument_id : InstrumentId
        The instrument ID.
    bid_price : float
        The best bid price.
    ask_price : float
        The best ask price.
    bid_qty : float
        The best bid quantity.
    ask_qty : float
        The best ask quantity.
    trade_price : float
        The last trade price.
    trade_qty : float
        The last trade quantity.
    """

    def __init__(
        self,
        uint64_t ts_init,
        uint64_t ts_event,
        uint64_t ts_in_delta,
        uint64_t ts_recv,
        InstrumentId instrument_id not None,
        float bid_price,
        float ask_price,
        float bid_qty,
        float ask_qty,
        float trade_price,
        float trade_qty,
    ) -> None:
        super().__init__(ts_init, ts_event, ts_in_delta, ts_recv, instrument_id)
        self.bid_price = bid_price
        self.ask_price = ask_price
        self.bid_qty = bid_qty
        self.ask_qty = ask_qty
        self.trade_price = trade_price
        self.trade_qty = trade_qty

    def __str__(self) -> str:
        return (
            f"{type(self).__name__}(\n"
            f"  instrument_id={self.instrument_id},\n"
            f"  ts_init={self.ts_init},\n"
            f"  ts_event={self.ts_event},\n"
            f"  ts_in_delta={self.ts_in_delta},\n"
            f"  ts_recv={self.ts_recv},\n"
            f"  bid_price={self.bid_price},\n"
            f"  ask_price={self.ask_price},\n"
            f"  bid_qty={self.bid_qty},\n"
            f"  ask_qty={self.ask_qty},\n"
            f"  trade_price={self.trade_price},\n"
            f"  trade_qty={self.trade_qty}\n"
            f")"
        )

    def __repr__(self) -> str:
        return (
            f"{type(self).__name__}(\n"
            f"  instrument_id={self.instrument_id},\n"
            f"  ts_init={self.ts_init},\n"
            f"  ts_event={self.ts_event},\n"
            f"  ts_in_delta={self.ts_in_delta},\n"
            f"  ts_recv={self.ts_recv},\n"
            f"  bid_price={self.bid_price},\n"
            f"  ask_price={self.ask_price},\n"
            f"  bid_qty={self.bid_qty},\n"
            f"  ask_qty={self.ask_qty},\n"
            f"  trade_price={self.trade_price},\n"
            f"  trade_qty={self.trade_qty}\n"
            f")"
        )