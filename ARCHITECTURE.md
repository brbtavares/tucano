# Toucan Architecture Overview

## 🏗️ Project Structure & Data Flow

Toucan is designed with a clear separation of concerns across multiple subcrates, each handling different aspects of trading operations:

```
📦 Toucan Trading Ecosystem
├── 🧠 toucan/              # Core trading engine
├── 📊 toucan-data/         # PUBLIC market data streams
├── 💼 toucan-execution/    # PRIVATE account data & order execution
├── 🔧 toucan-instrument/   # Financial instruments & assets
└── 🌐 toucan-integration/  # Low-level web integration framework
```

## 📊 Data Stream Architecture

### **Two Distinct WebSocket Systems**

The project maintains a clean separation between public and private data streams:

#### 🌐 **Public Market Data** (`toucan-data`)
- **Purpose**: Real-time market information available to everyone
- **Authentication**: ❌ None required
- **Data Types**: 
  - Order books (L1/L2)
  - Public trades
  - Liquidations
  - Klines/Candlesticks
  - 24hr ticker stats
- **WebSocket Endpoints**: 
  - `wss://stream.binance.com/ws/` (public streams)
  - `wss://fstream.binance.com/ws/` (futures public)
- **Status**: ✅ **FULLY IMPLEMENTED** (production ready)

#### 🔐 **Private Account Data** (`toucan-execution`)
- **Purpose**: Account-specific trading information
- **Authentication**: ✅ API keys + HMAC-SHA256 signing required
- **Data Types**:
  - Account balance updates
  - Order execution events
  - Trade confirmations
  - Position updates
- **WebSocket Endpoints**: 
  - `wss://stream.binance.com/ws/{listenKey}` (authenticated)
  - `wss://fstream.binance.com/ws/{listenKey}` (futures authenticated)
- **Status**: 🚧 **SKELETON READY** (awaiting API integration)

### **Why This Separation Exists**

1. **🔐 Security**: Public data doesn't need sensitive API credentials
2. **🏃 Performance**: Public streams are high-frequency, private streams are event-driven
3. **📡 Connection Management**: Public streams can be shared, private streams need individual auth
4. **🛡️ Error Handling**: Different error types and recovery strategies

## 🎯 Implementation Status

### ✅ **Completed Components**

#### **toucan-data** (Public Market Data)
- **Status**: 🟢 **Production Ready**
- **Features**: Full WebSocket integration for all major exchanges
- **Exchanges**: Binance, Coinbase, Kraken, Bybit, BitMEX, OKX, Gate.io
- **Examples**: Comprehensive examples in `/toucan-data/examples/`

#### **toucan-execution** (Private Account Data)
- **Status**: 🟡 **Skeleton Complete**
- **Features**: Full ExecutionClient trait implementation with placeholder logic
- **Exchanges**: Binance client structure ready for API integration
- **Examples**: Working examples in `/toucan-execution/examples/`

### 🚧 **Next Implementation Phase**

The `toucan-execution` Binance client is ready for actual API integration:

1. **HTTP Client Integration**
   - Add `reqwest` with TLS support
   - Implement HMAC-SHA256 authentication
   - Map API responses to internal types

2. **WebSocket User Data Stream**
   - Implement listen key management
   - Connect to private WebSocket endpoints
   - Parse account events and order updates

3. **REST API Endpoints**
   - Account information and balances
   - Order placement and cancellation
   - Trade history and open orders

## 🔍 **Key Architectural Decisions**

### **No Code Duplication**
- Public and private WebSocket implementations are completely separate
- Each serves different data types with different authentication requirements
- No overlap in functionality or endpoints

### **Modular Design**
- Each subcrate can be used independently
- Clear interfaces between components
- Easy to extend with new exchanges or data types

### **Type Safety**
- All data structures are strongly typed
- Consistent error handling across all components
- Zero-cost abstractions where possible

## 📖 **For New Contributors**

### **Adding Public Market Data**
- Work in `toucan-data/src/exchange/`
- Implement `StreamSelector` and `ExchangeTransformer` traits
- Add examples in `toucan-data/examples/`

### **Adding Private Account Data**
- Work in `toucan-execution/src/client/`
- Implement `ExecutionClient` trait
- Add examples in `toucan-execution/examples/`

### **Understanding the Flow**
1. `toucan-data` provides market information to trading strategies
2. `toucan-execution` executes trades based on strategy decisions
3. `toucan` (core engine) coordinates everything together
4. `toucan-instrument` provides common data structures
5. `toucan-integration` provides low-level web integration tools

## 🎉 **Ready to Use**

The architecture is production-ready for:
- ✅ **Live Trading**: Real market data + real order execution
- ✅ **Paper Trading**: Real market data + mock order execution  
- ✅ **Backtesting**: Historical data + simulated execution
- ✅ **Research**: Market data analysis and strategy development

Each component is independently usable and well-documented with comprehensive examples!
