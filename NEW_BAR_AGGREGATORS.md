# New Bar Aggregation Methods for Nautilus Trader

This document describes the implementation of six new bar aggregation methods that were previously missing from Nautilus Trader: `TICK_IMBALANCE`, `VOLUME_IMBALANCE`, `VALUE_IMBALANCE`, `TICK_RUNS`, `VOLUME_RUNS`, and `VALUE_RUNS`.

## Overview

These new aggregation methods implement information-driven bar sampling techniques that provide more sophisticated market microstructure analysis capabilities. Unlike fixed-time interval bars, these methods create bars based on market activity patterns and order flow characteristics.

## Implemented Aggregators

### Imbalance Bars

Imbalance bars are based on the cumulative signed difference between buy and sell activity. They trigger when the absolute value of the imbalance reaches a specified threshold.

#### 1. Tick Imbalance Bars (`TickImbalanceBarAggregator`)

- **Aggregation Type**: `BarAggregation.TICK_IMBALANCE`
- **Logic**: Tracks cumulative tick imbalance (up ticks = +1, down ticks = -1)
- **Trigger**: When `|cumulative_tick_imbalance| >= threshold`
- **Use Cases**: 
  - Order flow analysis
  - High-frequency trading strategies
  - Market microstructure research

```python
from nautilus_trader.data.aggregation import TickImbalanceBarAggregator
from nautilus_trader.model.data import BarSpecification, BarType
from nautilus_trader.model.enums import BarAggregation, PriceType

bar_spec = BarSpecification(10, BarAggregation.TICK_IMBALANCE, PriceType.LAST)
bar_type = BarType(instrument.id, bar_spec)
aggregator = TickImbalanceBarAggregator(instrument, bar_type, handler)
```

#### 2. Volume Imbalance Bars (`VolumeImbalanceBarAggregator`)

- **Aggregation Type**: `BarAggregation.VOLUME_IMBALANCE`
- **Logic**: Tracks cumulative volume imbalance (up_volume - down_volume)
- **Trigger**: When `|cumulative_volume_imbalance| >= threshold`
- **Use Cases**:
  - Institutional order flow detection
  - Large volume imbalance analysis
  - Smart money tracking

```python
from nautilus_trader.data.aggregation import VolumeImbalanceBarAggregator

bar_spec = BarSpecification(1000, BarAggregation.VOLUME_IMBALANCE, PriceType.LAST)
bar_type = BarType(instrument.id, bar_spec)
aggregator = VolumeImbalanceBarAggregator(instrument, bar_type, handler)
```

#### 3. Value Imbalance Bars (`ValueImbalanceBarAggregator`)

- **Aggregation Type**: `BarAggregation.VALUE_IMBALANCE`
- **Logic**: Tracks cumulative value imbalance (up_value - down_value)
- **Calculation**: `value = price * volume * tick_sign`
- **Trigger**: When `|cumulative_value_imbalance| >= threshold`
- **Use Cases**:
  - Dollar-weighted order flow analysis
  - Cross-asset comparison
  - Price-level normalized analysis

```python
from nautilus_trader.data.aggregation import ValueImbalanceBarAggregator

bar_spec = BarSpecification(10000, BarAggregation.VALUE_IMBALANCE, PriceType.LAST)
bar_type = BarType(instrument.id, bar_spec)
aggregator = ValueImbalanceBarAggregator(instrument, bar_type, handler)
```

### Runs Bars

Runs bars are based on consecutive sequences of buy or sell activity. They trigger when the maximum run length in either direction reaches the threshold.

#### 4. Tick Runs Bars (`TickRunsBarAggregator`)

- **Aggregation Type**: `BarAggregation.TICK_RUNS`
- **Logic**: Tracks maximum consecutive buy or sell ticks
- **Trigger**: When `max(buy_runs, sell_runs) >= threshold`
- **Use Cases**:
  - Momentum detection
  - Trend persistence analysis
  - Reversal pattern identification

```python
from nautilus_trader.data.aggregation import TickRunsBarAggregator

bar_spec = BarSpecification(5, BarAggregation.TICK_RUNS, PriceType.LAST)
bar_type = BarType(instrument.id, bar_spec)
aggregator = TickRunsBarAggregator(instrument, bar_type, handler)
```

#### 5. Volume Runs Bars (`VolumeRunsBarAggregator`)

- **Aggregation Type**: `BarAggregation.VOLUME_RUNS`
- **Logic**: Tracks maximum consecutive volume in buy or sell direction
- **Trigger**: When `max(buy_volume_runs, sell_volume_runs) >= threshold`
- **Use Cases**:
  - Sustained institutional interest detection
  - Accumulation/distribution pattern analysis
  - Volume-weighted momentum analysis

```python
from nautilus_trader.data.aggregation import VolumeRunsBarAggregator

bar_spec = BarSpecification(500, BarAggregation.VOLUME_RUNS, PriceType.LAST)
bar_type = BarType(instrument.id, bar_spec)
aggregator = VolumeRunsBarAggregator(instrument, bar_type, handler)
```

#### 6. Value Runs Bars (`ValueRunsBarAggregator`)

- **Aggregation Type**: `BarAggregation.VALUE_RUNS`
- **Logic**: Tracks maximum consecutive value in buy or sell direction
- **Calculation**: `value = price * volume`
- **Trigger**: When `max(buy_value_runs, sell_value_runs) >= threshold`
- **Use Cases**:
  - Combined price and volume momentum
  - Significant market move identification
  - Cross-market comparison

```python
from nautilus_trader.data.aggregation import ValueRunsBarAggregator

bar_spec = BarSpecification(5000, BarAggregation.VALUE_RUNS, PriceType.LAST)
bar_type = BarType(instrument.id, bar_spec)
aggregator = ValueRunsBarAggregator(instrument, bar_type, handler)
```

## Technical Implementation

### Tick Direction Detection

All aggregators use a consistent tick direction detection method:

```python
def _get_tick_sign(self, price):
    """Get the tick sign based on price movement."""
    if self._last_price is None:
        self._last_price = price
        return 0.0

    tick_sign = 0.0
    if price > self._last_price:
        tick_sign = 1.0  # Up tick
    elif price < self._last_price:
        tick_sign = -1.0  # Down tick
    # If price == last_price, tick_sign remains 0.0

    self._last_price = price
    return tick_sign
```

### Bar Triggering Logic

#### Imbalance Bars
- Accumulate signed values based on tick direction
- Trigger when absolute cumulative value reaches threshold
- Reset accumulator after bar creation

#### Runs Bars
- Track separate buy and sell run counters
- Trigger when maximum of either counter reaches threshold
- Reset both counters after bar creation

### Data Engine Integration

The new aggregators are integrated into the data engine alongside existing aggregators:

```python
# In nautilus_trader/data/engine.pyx
elif bar_type.spec.aggregation == BarAggregation.TICK_IMBALANCE:
    aggregator = TickImbalanceBarAggregator(
        instrument=instrument,
        bar_type=bar_type,
        handler=self.process,
    )
# ... similar blocks for other aggregators
```

## Usage Patterns

### High-Frequency Trading
- Small thresholds (5-20 for ticks, 100-1000 for volume)
- Focus on tick and volume imbalance bars
- Quick reaction to order flow changes

### Medium-Frequency Trading
- Moderate thresholds (20-100 for ticks, 1000-10000 for volume)
- Combination of imbalance and runs bars
- Trend and momentum detection

### Institutional Analysis
- Large thresholds for volume and value bars
- Focus on sustained patterns
- Accumulation/distribution detection

## Benefits

1. **Information-Driven Sampling**: Bars are created based on market activity rather than arbitrary time intervals
2. **Order Flow Analysis**: Direct measurement of buy/sell pressure and imbalances
3. **Adaptive Frequency**: More bars during active periods, fewer during quiet periods
4. **Statistical Properties**: Better statistical properties for modeling compared to time bars
5. **Market Microstructure**: Enhanced insight into market dynamics and liquidity

## Performance Considerations

- All aggregators use efficient Cython implementations
- Decimal arithmetic for precise financial calculations
- Minimal memory overhead with simple state tracking
- O(1) complexity for tick processing

## Testing

Comprehensive test suites are provided for all aggregators:

```python
# Run tests
python -m pytest tests/unit_tests/data/test_aggregation.py::TestTickImbalanceBarAggregator
python -m pytest tests/unit_tests/data/test_aggregation.py::TestVolumeImbalanceBarAggregator
# ... etc for other aggregators
```

## Examples

See `examples/demo_new_bar_aggregators.py` for practical usage examples and demonstrations of all new aggregation methods.

## References

1. López de Prado, M. (2018). *Advances in Financial Machine Learning*. Wiley.
2. Easley, D., López de Prado, M., & O'Hara, M. (2012). The volume clock: Insights into the high‐frequency paradigm. *The Journal of Portfolio Management*, 39(1), 19-29.
3. Cartea, Á., Jaimungal, S., & Penalva, J. (2015). *Algorithmic and High-Frequency Trading*. Cambridge University Press.

## Migration Notes

- Existing code using only time/tick/volume/value bars will continue to work unchanged
- New aggregators are opt-in and don't affect existing functionality
- Same handler patterns and APIs as existing aggregators
- Full compatibility with existing data infrastructure

---

*This implementation completes the bar aggregation suite in Nautilus Trader, providing traders and researchers with state-of-the-art order flow analysis capabilities.*