from __future__ import annotations


def tick_to_precision(tick: float) -> int:
    tick_str = f"{tick:.10f}"
    return len(tick_str.partition(".")[2].rstrip("0"))


def select_min_tick(tick_sizes: list[dict]) -> float:
    if not tick_sizes:
        return 0.01
    values: list[float] = []
    for t in tick_sizes:
        v = t.get("value")
        if v is None:
            continue
        try:
            values.append(float(v))
        except Exception:
            continue
    return min(values) if values else 0.01


