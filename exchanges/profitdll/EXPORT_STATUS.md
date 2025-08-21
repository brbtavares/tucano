# ProfitDLL Export Mapping Status

Generated from `profit.hpp` (original C++) comparing with current implementation in `profitdll/src/ffi.rs`.

Legend:
- ✅ Implemented (or equivalent) in Rust
- ⚠️ Partial / Reduced semantics
- ❌ Missing
- 📝 Planned (high priority)

## 1. Initialization / Session
| C++ Symbol | Status | Notes |
|------------|--------|-------|
| DLLInitializeLogin | ✅ (Initialize + Login current sequence) | Loaded as `Initialize` + `Login` separately in Rust. Callbacks still reduced. |
| DLLInitializeMarketLogin | ❌ | Required for optional (market) flow. |
| DLLFinalize | ✅ (Finalize) | `Finalize` method loaded but not yet publicly exposed. |
| SetServerAndPort | ❌ | Endpoint config. |
| GetServerClock | ❌ | Server clock (synchronization). |
| SetDayTrade | ❌ | Day trade flag. |
| SetEnabledHistOrder | ❌ | Enables order history. |

## 2. State Callbacks / Lists / Adjustments
| C++ Callback Setter | Status | Notes |
|---------------------|--------|-------|
| SetStateCallback | ✅ | Used. |
| SetChangeCotationCallback | ❌ | Specific quote updates. |
| SetAssetListCallback | ❌ | Basic asset list. |
| SetAssetListInfoCallback | ❌ | Asset metadata v1. |
| SetAssetListInfoCallbackV2 | ❌ | Asset metadata v2 (sector / subsector / segment). |
| SetInvalidTickerCallback | ✅ (optional) | Already loaded. |
| SetAdjustHistoryCallback | ❌ | Corporate actions v1. |
| SetAdjustHistoryCallbackV2 | ⚠️ | Initial heuristic parsing (assumed layout) + hexdump. |
| SetTheoreticalPriceCallback | ⚠️ | Registered placeholder (partial fields). |
| SeTConnectorBrokerAccountListChangedCallback | ❌ | Account list changes. |
| SetBrokerSubAccountListChangedCallback | ❌ | Subaccount changes. |
| SetEnabledLogToDebug | ❌ | Enable internal logs. |

## 3. Market Data Subscribe
| C++ Symbol | Status | Notes |
|------------|--------|-------|
| SubscribeTicker / UnsubscribeTicker | ✅ | Implementado. |
| SubscribePriceBook / UnsubscribePriceBook | ❌ | Price book (probably snapshot Levels). |
| SubscribeOfferBook / UnsubscribeOfferBook | ❌ | Detailed offer book. |
| SubscribeAdjustHistory / UnsubscribeAdjustHistory | ❌ | Adjustments stream. |

## 4. Market Data Callbacks
| Callback | Status | Notas |
|----------|--------|-------|
| TNewTradeCallback | ⚠️ | Rust: TradeCallback(V1/V2) — sem parsing detalhado ainda (estrutura diferente). |
| THistoryTradeCallback | ⚠️ | Placeholder callback registrado (mesma struct de trade). |
| TNewDailyCallback | ⚠️ | Rust: DailySummary(V1/V2) — mapeado parcial. |
| TPriceBookCallback | ⚠️ | Rust: BookCallback(V1/V2) sem oferta detalhada / arrays. |
| TOfferBookCallback | ❌ | Offers (side + changes). |
| TNewTinyBookCallBack | ❌ | Reduced level. |
| TChangeStateTicker | ❌ | Ticker state. |
| TAdjustHistoryCallback / V2 | ⚠️ | Heuristic parse (fields may change when layout is confirmed). |
| TTheoreticalPriceCallback | ⚠️ | Placeholder (price + qty). |
| TConnectorBrokerAccountListChangedCallback | ❌ | Account list. |
| TConnectorBrokerSubAccountListChangedCallback | ❌ | Subaccounts. |
| TProgressCallBack | ❌ | Progress (e.g., historical loading). |
| TOrderChangeCallBack | ❌ | Rich order updates (we only have partial snapshot via GetOrderDetails in order callback). |
| THistoryCallBack (orders) | ❌ | Order history. |
| TAccountCallback | ✅ | Carregado como SetAccountCallback. |

## 5. Orders / Execution
| C++ Symbol | Status | Notes |
|------------|--------|-------|
| SendBuyOrder / SendSellOrder | ❌ | Rust has generic `SendOrder` (not separated). |
| SendStopBuyOrder / SendStopSellOrder | ❌ | Stop orders missing. |
| SendMarketBuyOrder / SendMarketSellOrder | ❌ | Dedicated market orders. |
| SendZeroPosition | ❌ | Zero position. |
| SendCancelOrder | ❌ | Cancel specific order (do we have V2? no). |
| SendCancelOrders | ❌ | Cancel by ticker. |
| SendCancelAllOrders | ❌ | Cancel all orders. |
| SendChangeOrder | ⚠️ | There is an optional `SendChangeOrderV2`; different signature. |
| GetOrder | ❌ | Individual query. |
| GetOrders | ❌ | List query. |
| GetOrderProfitID | ❌ | Lookup by ProfitID. |
| GetOrderDetails | ✅ (optional) | Used in order callback for snapshot. |

## 6. Positions / Accounts / Agents
| C++ Symbol | Status | Notes |
|------------|--------|-------|
| GetPosition | ❌ | Returns blob; requires parsing (Position structure). |
| EnumerateAllPositionAssets | ❌ | Enumeration of assets with position. |
| GetAccount | ❌ | Enumerate accounts. |
| GetAgentNameByID / GetAgentShortNameByID | ❌ | Agent identity. |
| GetAgentNameLength / GetAgentName | ❌ | Safe version for buffer. |

## 7. History / Data
| C++ Symbol | Status | Notes |
|------------|--------|-------|
| GetHistoryTrades | ⚠️ | Mock implemented + FFI stub; real parse missing. |
| GetLastDailyClose | ❌ | Daily close. |

## 8. Infra / Utilities
| C++ Symbol | Status | Notes |
|------------|--------|-------|
| FreePointer | ⚠️ | Wrapper ForeignBuffer created; still without real parse. |

## 9. Structures missing in Rust
We will need to map in `repr(C)` + conversions:
- TAssetID (wchar_t* ticker, exchange, feed)
- TConnectorAccountIdentifier / TConnectorAssetIdentifier
- Position + sub strings (packed buffer)
- Trade / TradeCandle (for history and realtime if V2 not used)
- BookOffer arrays (OfferBookCallback)

## 10. Proposed Implementation Priority
1. History & Adjustments:
   - GetHistoryTrades (pull) + THistoryTradeCallback (incremental push)
   - SubscribeAdjustHistory / adjustment callbacks (direct V2)
2. Market Data Depth:
   - OfferBookCallback (separate from current Book V2) + SubscribeOfferBook
3. Essential Execution:
   - SendBuyOrder / Sell / Market / Stop / CancelAll / ZeroPosition
   - GetPosition + FreePointer parsing
4. Asset Metadata:
   - SetAssetListInfoCallbackV2 + SetAssetListCallback
5. Positions / Accounts:
   - EnumerateAllPositionAssets / GetAccount / Account & Broker callbacks
6. Utilities:
   - GetServerClock / SetServerAndPort / GetLastDailyClose
7. Complements:
   - AgentName APIs, TheoreticalPrice, ChangeStateTicker, ChangeCotation

## 11. Technical Approach
- Add `ffi_types.rs` module with structs/conversions wide → UTF-8 (`widestring` crate) under cfg windows + real_dll.
- Extend `ProfitRaw` with new optional symbols; keep incremental gating (do not break builds).
- Introduce enriched event enum (`CallbackEvent` expanded) for new callbacks with feature flags (e.g., feature `md_extended`).
- Buffers: use `Vec<u8>` + pointers; free via `FreePointer` immediately after parse; ensure `unsafe` is encapsulated.
- Wide Strings: convert via `U16CStr` -> `String` (lossy fallback).

## 12. Risks / Cautions
- Semantic difference between `InitializeLogin` and `InitializeMarketLogin` (order of callbacks and progress callback requirements).
- Potential reentrancy: documentation warns not to call DLL functions inside callbacks; design: queue data and process outside.
- Synchronization: expand `SenderState` or create multiple channels (e.g., separate order vs market channel) for backpressure.
- Memory: ensure `FreePointer` is called exactly once per buffer.

## 13. Next Automatable Steps
Script (future) to validate exports vs mapping and produce automatic diff.

---
Automatically generated – edit as new functions are added.
