# Toucan Examples

Set of demonstration binaries. Each example supports real backend (Windows + `--features real_dll`) or mock (any OS / `PROFITDLL_FORCE_MOCK=1` / automatic fallback if DLL unavailable).

## Binaries
All located in `examples/src/`:
| Bin | Description |
|-----|-------------|
| `example_1_live_login` | Login + subscribe to a ticker and print the first events |
| `example_2_get_history_trades` | Requests trade history in a range and prints `HistoryTrade` |
| `mock_minimal` | Minimal: login (mock), subscribe, sends simulated order and reads some events |
| `toucan-examples` | Index bin: lists available examples |

## Quick Execution
Forced mock (any OS):
```bash
PROFITDLL_FORCE_MOCK=1 cargo run -p toucan-examples --bin example_1_live_login
```
Live (Windows + DLL):
```bash
cargo run -p toucan-examples --features real_dll --bin example_1_live_login
```
History:
```bash
cargo run -p toucan-examples --features real_dll --bin example_2_get_history_trades
```
Minimal:
```bash
cargo run -p toucan-examples --bin mock_minimal
```

## Main Environment Variables
| Variable | Usage |
|----------|-------|
| `PROFIT_USER` / `PROFIT_PASSWORD` | Credentials (mock accepts any value) |
| `PROFIT_ACTIVATION_KEY` | Optional |
| `PROFITDLL_PATH` | Real DLL path |
| `PROFITDLL_FORCE_MOCK=1` | Forces mock backend |
| `PROFITDLL_DIAG=1` | Diagnostic logs for loading/callbacks |
| `PROFITDLL_STRICT=1` | Fails if real backend not available (no fallback to mock) |
| `EX1_TICKER` / `EX1_EXCHANGE` |  Example 1 Ticker (default PETR4 / B) |
| `HIST_TICKER` / `HIST_EXCHANGE` | Example 2 Ticker (default PETR4 / B) |
| `INTERVAL_START` | Example 2 interval start (e.g., 2025-08-15T10:30) |
| `INTERVAL_MINUTES` | Duration in minutes (Example 2, default 5) |

Exemplo `.env` (live):
```env
PROFIT_USER=seu_usuario
PROFIT_PASSWORD=sua_senha
PROFITDLL_PATH=C:\\path\\to\\ProfitDLL.dll
EX1_TICKER=PETR4
EX1_EXCHANGE=B
HIST_TICKER=PETR4
HIST_EXCHANGE=B
INTERVAL_START=2025-08-15T10:30
INTERVAL_MINUTES=5
```

## Troubleshooting
| Symptom | Possible cause | Mitigation |
|---------|----------------|-----------|
| "Backend is not real_dll" | Feature missing / not Windows / DLL missing | Build with `--features real_dll` on Windows and check `PROFITDLL_PATH` |
| No events after subscribe | Credentials/license / invalid ticker | Check vars and use `PROFITDLL_DIAG=1` |
| Empty history | Interval without trades / partial callback | Adjust `INTERVAL_START` / `INTERVAL_MINUTES` |
| MissingSymbol / missing symbol | Incompatible DLL version | Update binding or DLL |
| Unexpected fallback to mock | DLL did not load and `PROFITDLL_STRICT` missing | Set `PROFITDLL_STRICT=1` to abort |

## Roadmap
- [x] Minimal mock
- [x] Login + subscribe
- [x] History (pull)
- [ ] Full live order sending
- [ ] Advanced level 2 book
- [ ] Streaming metrics (VWAP, aggregations)
- [ ] Progress / end-of-history confirmation
- [ ] CLI args to parameterize all examples

---
License: Apache-2.0 OR MIT.
