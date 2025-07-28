#!/usr/bin/env python3
"""
Demonstration of the new bar aggregation methods: 
TICK_IMBALANCE, VOLUME_IMBALANCE, VALUE_IMBALANCE, TICK_RUNS, VOLUME_RUNS, VALUE_RUNS

This script demonstrates how to use the newly implemented aggregation methods
that were previously missing from the Nautilus Trader platform.
"""

from nautilus_trader.model.data import BarSpecification, BarType
from nautilus_trader.model.enums import BarAggregation, PriceType
from nautilus_trader.test_kit.providers import TestInstrumentProvider


def demo_imbalance_bars():
    """Demonstrate imbalance bar aggregation methods."""
    print("=== Imbalance Bar Aggregators ===")
    
    # Get a test instrument
    instrument = TestInstrumentProvider.default_fx_ccy("EUR/USD")
    
    # Create different imbalance bar specifications
    tick_imbalance_spec = BarSpecification(
        step=10,
        aggregation=BarAggregation.TICK_IMBALANCE,
        price_type=PriceType.LAST
    )
    
    volume_imbalance_spec = BarSpecification(
        step=1000,
        aggregation=BarAggregation.VOLUME_IMBALANCE,
        price_type=PriceType.LAST
    )
    
    value_imbalance_spec = BarSpecification(
        step=10000,
        aggregation=BarAggregation.VALUE_IMBALANCE,
        price_type=PriceType.LAST
    )
    
    # Create bar types
    tick_imbalance_bar_type = BarType(instrument.id, tick_imbalance_spec)
    volume_imbalance_bar_type = BarType(instrument.id, volume_imbalance_spec)
    value_imbalance_bar_type = BarType(instrument.id, value_imbalance_spec)
    
    print(f"Tick Imbalance Bar Type: {tick_imbalance_bar_type}")
    print(f"Volume Imbalance Bar Type: {volume_imbalance_bar_type}")
    print(f"Value Imbalance Bar Type: {value_imbalance_bar_type}")
    
    print("\nImbalance bars trigger when the cumulative signed difference")
    print("between up and down movements reaches the threshold:")
    print("- Tick Imbalance: Counts +1/-1 for each up/down tick")
    print("- Volume Imbalance: Sums volume * tick_sign")
    print("- Value Imbalance: Sums (price * volume) * tick_sign")


def demo_runs_bars():
    """Demonstrate runs bar aggregation methods."""
    print("\n=== Runs Bar Aggregators ===")
    
    # Get a test instrument
    instrument = TestInstrumentProvider.default_fx_ccy("EUR/USD")
    
    # Create different runs bar specifications
    tick_runs_spec = BarSpecification(
        step=5,
        aggregation=BarAggregation.TICK_RUNS,
        price_type=PriceType.LAST
    )
    
    volume_runs_spec = BarSpecification(
        step=500,
        aggregation=BarAggregation.VOLUME_RUNS,
        price_type=PriceType.LAST
    )
    
    value_runs_spec = BarSpecification(
        step=5000,
        aggregation=BarAggregation.VALUE_RUNS,
        price_type=PriceType.LAST
    )
    
    # Create bar types
    tick_runs_bar_type = BarType(instrument.id, tick_runs_spec)
    volume_runs_bar_type = BarType(instrument.id, volume_runs_spec)
    value_runs_bar_type = BarType(instrument.id, value_runs_spec)
    
    print(f"Tick Runs Bar Type: {tick_runs_bar_type}")
    print(f"Volume Runs Bar Type: {volume_runs_bar_type}")
    print(f"Value Runs Bar Type: {value_runs_bar_type}")
    
    print("\nRuns bars trigger when the maximum consecutive runs")
    print("in either direction reaches the threshold:")
    print("- Tick Runs: Counts consecutive up or down ticks")
    print("- Volume Runs: Sums volume of consecutive runs")
    print("- Value Runs: Sums value of consecutive runs")


def demo_practical_usage():
    """Show practical usage scenarios for the new aggregators."""
    print("\n=== Practical Usage Scenarios ===")
    
    print("1. Tick Imbalance Bars:")
    print("   - Useful for detecting order flow imbalances")
    print("   - Small threshold (5-20) for high-frequency strategies")
    print("   - Larger threshold (50-200) for medium-frequency strategies")
    
    print("\n2. Volume Imbalance Bars:")
    print("   - Captures institutional order flow")
    print("   - Good for detecting large volume imbalances")
    print("   - Threshold based on typical volume patterns")
    
    print("\n3. Value Imbalance Bars:")
    print("   - Dollar-weighted order flow analysis")
    print("   - Normalizes across different price levels")
    print("   - Useful for cross-asset comparison")
    
    print("\n4. Tick Runs Bars:")
    print("   - Detects momentum and trend persistence")
    print("   - Small threshold (3-10) for quick reversal detection")
    print("   - Larger threshold (20-50) for trend confirmation")
    
    print("\n5. Volume Runs Bars:")
    print("   - Identifies sustained institutional interest")
    print("   - Helps detect accumulation/distribution patterns")
    print("   - Volume-weighted momentum analysis")
    
    print("\n6. Value Runs Bars:")
    print("   - Combines price and volume momentum")
    print("   - Best for identifying significant market moves")
    print("   - Cross-market comparison capabilities")


def demo_aggregator_usage():
    """Show how to use the aggregators in practice."""
    print("\n=== Using the Aggregators ===")
    
    print("In your trading strategy or data engine:")
    print("""
# Import the aggregators
from nautilus_trader.data.aggregation import (
    TickImbalanceBarAggregator,
    VolumeImbalanceBarAggregator,
    ValueImbalanceBarAggregator,
    TickRunsBarAggregator,
    VolumeRunsBarAggregator,
    ValueRunsBarAggregator,
)

# Create a bar handler
def handle_bar(bar):
    print(f"Received bar: {bar}")

# Create an aggregator
instrument = get_instrument("EUR/USD")
bar_spec = BarSpecification(10, BarAggregation.TICK_IMBALANCE, PriceType.LAST)
bar_type = BarType(instrument.id, bar_spec)

aggregator = TickImbalanceBarAggregator(
    instrument=instrument,
    bar_type=bar_type,
    handler=handle_bar,
)

# Feed data to the aggregator
aggregator.handle_trade_tick(tick)
""")


def main():
    """Run the demonstration."""
    print("Nautilus Trader - New Bar Aggregation Methods Demo")
    print("=" * 55)
    
    demo_imbalance_bars()
    demo_runs_bars()
    demo_practical_usage()
    demo_aggregator_usage()
    
    print("\n" + "=" * 55)
    print("The new aggregation methods provide advanced order flow")
    print("analysis capabilities for sophisticated trading strategies.")
    print("They complement the existing time, tick, volume, and value bars")
    print("with information-driven sampling approaches.")


if __name__ == "__main__":
    main()