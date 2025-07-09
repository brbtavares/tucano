# Toucan WebSocket Architecture - Separation of Responsibilities

## Overview

The Toucan project has a clear separation of responsibilities between public market data and private user data WebSocket streams across two different subcrates:

- **`toucan-data`**: Public market data WebSocket streams (order books, trades, liquidations)
- **`toucan-execution`**: Private user data WebSocket streams (account updates, order events)

## Detailed Separation

### toucan-data (Public Market Data)
**Location**: `/toucan-data/src/exchange/binance/`

**Purpose**: Handles public market data streams that don't require authentication

**Responsibilities**:
- Public trades streams (`@aggTrade`, `@trade`)
- Order book streams Level 1 (`@bookTicker`) 
- Order book streams Level 2 (`@depth`)
- Liquidation streams (`@forceOrder`)
- Kline/Candlestick streams (`@kline`)
- 24hr ticker statistics (`@ticker`)

**WebSocket Endpoints**:
- Spot: `wss://stream.binance.com:9443/ws/`
- Futures USD-M: `wss://fstream.binance.com/ws/`
- Futures COIN-M: `wss://dstream.binance.com/ws/`

**Current Implementation Status**: ✅ **FULLY IMPLEMENTED**
- Complete WebSocket integration
- All major exchanges supported (Binance, Coinbase, Kraken, etc.)
- Production-ready with examples and tests

### toucan-execution (Private User Data) 
**Location**: `/toucan-execution/src/client/binance/`

**Purpose**: Handles private account data streams that require authentication

**Responsibilities**:
- User Data Stream (account balance updates)
- Order execution events (NEW, FILLED, CANCELED, etc.)
- Trade execution notifications
- Account position updates
- Listen key management and refresh

**WebSocket Endpoints**:
- Spot: `wss://stream.binance.com:9443/ws/{listenKey}`
- Futures USD-M: `wss://fstream.binance.com/ws/{listenKey}`
- Futures COIN-M: `wss://dstream.binance.com/ws/{listenKey}`

**Current Implementation Status**: 🚧 **SKELETON IMPLEMENTED**
- Basic structure and ExecutionClient trait implementation
- Configuration and error handling 
- All methods return placeholder data with TODOs
- Ready for actual API integration

## Why This Separation Exists

### 1. **Authentication Requirements**
- **Public data**: No authentication required
- **Private data**: Requires API keys and listen key management

### 2. **Data Nature**
- **Public data**: Market-wide information, high frequency
- **Private data**: Account-specific information, lower frequency

### 3. **Connection Management**
- **Public data**: Can share connections across symbols
- **Private data**: Requires individual authenticated connections per account

### 4. **Error Handling**
- **Public data**: Connection issues, rate limits
- **Private data**: Authentication errors, account-specific issues

## No Code Duplication

The two implementations are **completely separate** and handle different data types:

### toucan-data WebSocket (Public)
```rust
// Example: Public trades stream
let streams = Streams::<PublicTrades>::builder()
    .subscribe([
        (BinanceSpot::default(), "btc", "usdt", MarketDataInstrumentKind::Spot, PublicTrades),
    ])
    .init()
    .await?;
```

### toucan-execution WebSocket (Private) 
```rust
// Example: Account data stream (when implemented)
let client = BinanceExecution::new_with_config(config);
let account_stream = client.account_stream().await?;

while let Some(event) = account_stream.next().await {
    match event {
        AccountEvent::BalanceUpdate(update) => { /* handle */ },
        AccountEvent::OrderUpdate(order) => { /* handle */ },
        AccountEvent::TradeUpdate(trade) => { /* handle */ },
    }
}
```

## Implementation Roadmap for toucan-execution

### Current Status: Ready for API Integration

The Binance client skeleton in `toucan-execution` is ready for completing the actual API implementation:

#### ✅ **Completed**
- [x] Project integration and trait implementation
- [x] Configuration management
- [x] Error handling patterns
- [x] Type system integration
- [x] Comprehensive test suite
- [x] Working examples

#### 🚧 **Next Steps (API Implementation)**
1. **HTTP Client Integration**
   ```toml
   # Add to Cargo.toml
   reqwest = { version = "0.11", default-features = false, features = ["json", "rustls-tls"] }
   hmac = "0.12"
   sha2 = "0.10"
   hex = "0.4"
   ```

2. **Authentication & Request Signing**
   - Implement HMAC-SHA256 signing for API requests
   - Handle timestamps and nonces
   - Manage API key and secret securely

3. **WebSocket User Data Stream** 
   - Create listen key via POST /api/v3/userDataStream
   - Connect to private WebSocket endpoint
   - Parse account events and map to internal types
   - Handle listen key renewal (every 30 minutes)

4. **REST API Endpoints**
   - Account information (`/api/v3/account`)
   - Open orders (`/api/v3/openOrders`) 
   - Order placement (`/api/v3/order`)
   - Order cancellation (`/api/v3/order`)
   - Trade history (`/api/v3/myTrades`)

## Key Takeaways

1. **No Overlap**: `toucan-data` and `toucan-execution` handle completely different WebSocket streams
2. **Clean Separation**: Public vs Private data with different authentication requirements
3. **Production Ready**: `toucan-data` is fully implemented, `toucan-execution` skeleton is ready for API integration
4. **No Duplication**: Each subcrate has distinct responsibilities and different connection patterns

The current implementation perfectly follows the architecture and there's no code duplication to worry about! 🎯
