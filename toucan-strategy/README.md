# Toucan Strategy

Strategy interfaces and implementations for the Toucan trading ecosystem.

## Overview

This crate provides the core strategy traits and default implementations that can be used
with the Toucan trading engine. Each strategy type handles a specific aspect of trading logic.

## Strategy Interfaces

### `AlgoStrategy`

For generating algorithmic orders based on the current system state:

```rust
use toucan_strategy::AlgoStrategy;

impl AlgoStrategy for MyStrategy {
    type State = MyState;
    
    fn generate_algo_orders(&self, state: &Self::State) -> (CancelOrders, OpenOrders) {
        // Your algorithmic trading logic here
        // Return cancel and open order requests
    }
}
```

### `ClosePositionsStrategy`

For generating orders to close open positions:

```rust
use toucan_strategy::ClosePositionsStrategy;

impl ClosePositionsStrategy for MyStrategy {
    type State = MyState;
    
    fn close_positions_requests(&self, state: &Self::State, filter: &dyn Any) -> (CancelOrders, OpenOrders) {
        // Your position closing logic here
        // Return orders to close positions matching the filter
    }
}
```

### `OnDisconnectStrategy`

For handling exchange disconnection events:

```rust
use toucan_strategy::OnDisconnectStrategy;

impl OnDisconnectStrategy<MyEngine> for MyStrategy {
    type OnDisconnect = ();
    
    fn on_disconnect(engine: &mut MyEngine, exchange: ExchangeId) -> Self::OnDisconnect {
        // Handle exchange disconnection
        // e.g., cancel orders, close positions, etc.
    }
}
```

### `OnTradingDisabled`

For handling trading disabled events:

```rust
use toucan_strategy::OnTradingDisabled;

impl OnTradingDisabled<MyEngine> for MyStrategy {
    type OnTradingDisabled = ();
    
    fn on_trading_disabled(engine: &mut MyEngine) -> Self::OnTradingDisabled {
        // Handle trading disabled event
        // e.g., cancel all orders, close positions, etc.
    }
}
```

## Default Strategy

The crate provides a `DefaultStrategy` that implements all strategy interfaces with no-op behavior:

```rust
use toucan_strategy::DefaultStrategy;

let strategy = DefaultStrategy::default();
```

This is useful for:
- Testing and development
- Base implementations that can be customized
- Demonstration purposes

## Usage with Toucan Core

```rust
use toucan_core::Engine;
use toucan_strategy::{AlgoStrategy, DefaultStrategy};

// Use the default strategy
let engine = Engine::new(
    // ... other parameters
    DefaultStrategy::default(),
    // ... other parameters
);

// Or use your custom strategy
let engine = Engine::new(
    // ... other parameters
    MyCustomStrategy::new(),
    // ... other parameters
);
```

## Features

- **Modular Design**: Each strategy interface handles a specific aspect of trading
- **Generic Implementation**: Works with any state type and engine configuration
- **Type Safety**: All interfaces are strongly typed
- **Extensible**: Easy to implement custom strategies

## Examples

See the `examples/` directory for complete examples of how to implement and use strategies.

## Dependencies

- `toucan-execution` - For order types and execution interfaces
- `toucan-instrument` - For instrument and exchange types
- `rust_decimal` - For precise decimal arithmetic
