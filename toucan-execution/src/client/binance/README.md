# Binance Client Implementation

This document describes the current state of the Binance execution client and the roadmap for completing the implementation.

## Current Status

The Binance client (`BinanceExecution`) has been implemented as a skeletal framework that compiles and integrates with the Toucan execution system. Currently, all methods return placeholder data or empty results with appropriate TODO comments marking where actual API calls should be implemented.

### What's Implemented

1. **Basic Structure**: The `BinanceExecution` struct with proper configuration
2. **ExecutionClient Trait**: Complete implementation of all required methods
3. **Configuration**: `BinanceConfig` with fields for API credentials, testnet mode, and timeouts
4. **Error Handling**: Proper error types using the project's error system
5. **Logging**: Comprehensive logging using the `tracing` crate
6. **Type Safety**: All data structures properly typed with the project's type system

### What's NOT Implemented (TODOs)

1. **HTTP Client**: No actual HTTP requests to Binance API
2. **Authentication**: No API key signing/authentication logic
3. **WebSocket Streams**: No real-time data streaming
4. **Rate Limiting**: No rate limiting logic
5. **Response Parsing**: No actual parsing of Binance API responses

## Architecture

The implementation follows the project's patterns:

```
src/client/binance/
├── mod.rs          # Main client implementation
├── model.rs        # Binance-specific data structures (skeleton)
└── websocket.rs    # WebSocket handling (skeleton)
```

### Key Components

- **BinanceConfig**: Configuration struct for API credentials and settings
- **BinanceExecution**: Main client struct implementing `ExecutionClient` trait
- **Error Integration**: Uses project's `ConnectivityError` and `UnindexedClientError` types
- **Exchange Integration**: Uses `ExchangeId::BinanceSpot` identifier

## Usage Example

```rust
use toucan_execution::client::{binance::BinanceExecution, ExecutionClient};

let config = toucan_execution::client::binance::BinanceConfig {
    api_key: "your_api_key".to_string(),
    secret_key: "your_secret_key".to_string(),
    testnet: true,
    base_url: None,
    timeout_ms: 10000,
};

let client = BinanceExecution::new_with_config(config);

// All methods currently return placeholder data
let snapshot = client.account_snapshot(&assets, &instruments).await?;
let balances = client.fetch_balances().await?;
let orders = client.fetch_open_orders().await?;
```

## Next Steps for Implementation

### Phase 1: HTTP Client Integration

1. **Add HTTP Dependencies**: Integrate a pure-Rust HTTP client (avoid OpenSSL issues)
   ```toml
   # In Cargo.toml
   reqwest = { version = "0.11", default-features = false, features = ["json", "rustls-tls"] }
   ```

2. **Request Signing**: Implement Binance API authentication
   ```rust
   // Add crypto dependencies for HMAC-SHA256 signing
   hmac = "0.12"
   sha2 = "0.10"
   hex = "0.4"
   ```

### Phase 2: Core API Methods

1. **Account Information** (`/api/v3/account`)
   - Implement `account_snapshot()` method
   - Parse account balances and permissions

2. **Order Management**
   - `POST /api/v3/order` for `open_order()`
   - `DELETE /api/v3/order` for `cancel_order()`
   - `GET /api/v3/openOrders` for `fetch_open_orders()`

3. **Trade History** (`/api/v3/myTrades`)
   - Implement `fetch_trades()` method

### Phase 3: WebSocket Integration

1. **User Data Stream**
   - Implement `account_stream()` method
   - Listen for order updates, balance changes, etc.

2. **Connection Management**
   - Handle reconnections and keep-alive
   - Implement proper error recovery

### Phase 4: Production Readiness

1. **Rate Limiting**: Implement Binance rate limits
2. **Error Handling**: Proper mapping of Binance error codes
3. **Testing**: Add unit tests and integration tests
4. **Documentation**: Complete API documentation

## Development Guidelines

### HTTP Client Selection

Avoid OpenSSL dependencies to maintain compatibility:
- Use `rustls` instead of `openssl`
- Use `reqwest` with `rustls-tls` feature
- Or consider pure-Rust alternatives like `ureq`

### Authentication

Binance requires HMAC-SHA256 signatures:
```rust
fn sign_request(secret: &str, query_string: &str) -> String {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(query_string.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}
```

### Error Mapping

Map Binance errors to project types:
```rust
fn map_binance_error(error: BinanceError) -> UnindexedClientError {
    match error.code {
        -1021 => UnindexedClientError::Connectivity(ConnectivityError::Timeout),
        -1022 => UnindexedClientError::Api(ApiError::OrderRejected(error.msg)),
        _ => UnindexedClientError::Connectivity(ConnectivityError::Socket(error.msg)),
    }
}
```

## Testing

A complete example is available in `examples/binance_client_example.rs` that demonstrates:
- Client configuration
- Method calls (returning placeholder data)
- Error handling patterns
- Integration with project types

Run the example:
```bash
cargo run --example binance_client_example --package toucan-execution
```

## Dependencies Status

Current dependencies are minimal to avoid build issues:
- ✅ Basic Rust ecosystem crates
- ✅ Project workspace dependencies
- ❌ HTTP client (needs implementation)
- ❌ Crypto libraries (needs implementation)

The implementation is ready for incremental development of actual API integration.
