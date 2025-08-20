# Tucano Examples

Conjunto de binários de demonstração. Cada exemplo suporta backend real (Windows + `--features real_dll`) ou mock (qualquer SO / `PROFITDLL_FORCE_MOCK=1` / fallback automático se DLL indisponível).

## Binários
Todos localizados em `examples/src/`:
| Bin | Descrição |
|-----|-----------|
| `example_1_live_login` | Login + subscribe de um ticker e impressão dos primeiros eventos |
| `example_2_get_history_trades` | Solicita histórico de trades em um intervalo e imprime `HistoryTrade` |
| `mock_minimal` | Minimal: login (mock), subscribe, envia ordem simulada e lê alguns eventos |
| `tucano-examples` | Bin de índice: lista os exemplos disponíveis |

## Execução Rápida
Mock forçado (qualquer SO):
```bash
PROFITDLL_FORCE_MOCK=1 cargo run -p tucano-examples --bin example_1_live_login
```
Live (Windows + DLL):
```bash
cargo run -p tucano-examples --features real_dll --bin example_1_live_login
```
Histórico:
```bash
cargo run -p tucano-examples --features real_dll --bin example_2_get_history_trades
```
Minimal:
```bash
cargo run -p tucano-examples --bin mock_minimal
```

## Variáveis de Ambiente Principais
| Variável | Uso |
|----------|-----|
| `PROFIT_USER` / `PROFIT_PASSWORD` | Credenciais (mock aceita qualquer valor) |
| `PROFIT_ACTIVATION_KEY` | Opcional |
| `PROFITDLL_PATH` | Caminho da DLL real |
| `PROFITDLL_FORCE_MOCK=1` | Força backend mock |
| `PROFITDLL_DIAG=1` | Logs de diagnóstico do carregamento/callbacks |
| `PROFITDLL_STRICT=1` | Falha se não conseguir backend real (sem fallback para mock) |
| `EX1_TICKER` / `EX1_EXCHANGE` | Ticker do Example 1 (default PETR4 / B) |
| `HIST_TICKER` / `HIST_EXCHANGE` | Ticker do Example 2 (default PETR4 / B) |
| `INTERVAL_START` | Início intervalo Example 2 (ex: 2025-08-15T10:30) |
| `INTERVAL_MINUTES` | Duração em minutos (Example 2, default 5) |

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
| Sintoma | Possível causa | Mitigação |
|---------|----------------|-----------|
| "Backend não é real_dll" | Ausência de feature / não Windows / DLL faltando | Compilar com `--features real_dll` em Windows e conferir `PROFITDLL_PATH` |
| Sem eventos após subscribe | Credenciais/licença / ticker inválido | Verificar vars e usar `PROFITDLL_DIAG=1` |
| Histórico vazio | Intervalo sem trades / callback parcial | Ajustar `INTERVAL_START` / `INTERVAL_MINUTES` |
| MissingSymbol / símbolo ausente | Versão da DLL incompatível | Atualizar binding ou DLL |
| Fallback inesperado para mock | DLL não carregou e `PROFITDLL_STRICT` ausente | Definir `PROFITDLL_STRICT=1` para abortar |

## Roadmap
- [x] Mock mínimo
- [x] Login + subscribe
- [x] Histórico (pull)
- [ ] Envio de ordens (live) completo
- [ ] Book nível 2 avançado
- [ ] Métricas streaming (VWAP, agregações)
- [ ] Progresso / confirmação fim histórico
- [ ] CLI args para parametrizar todos exemplos

---
Licença: Apache-2.0 OR MIT.
