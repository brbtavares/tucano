# Tucano Examples

Diretório com binários de exemplo (mock e live).

## Estrutura
Pastas:
- `mock/` (sem DLL / sem credenciais)
- `live/` (requer DLL real, Windows e credenciais)

Binários principais:
- `mock_minimal`
- `example_1_live_login`
- `example_2_get_history_trades`

Scripts auxiliares antigos removidos (uso direto de `cargo run`).

## Requisitos Live
1. Windows.
2. `--features real_dll`.
3. `ProfitDLL.dll` acessível (`PROFITDLL_PATH`).
4. `PROFIT_USER`, `PROFIT_PASSWORD` (+ opcional `PROFIT_ACTIVATION_KEY`).
5. `.env` opcional na raiz.

Exemplo `.env`:
```
PROFIT_USER=seu_usuario
PROFIT_PASSWORD=sua_senha
PROFITDLL_PATH=C:\\path\\to\\ProfitDLL.dll
LIVE_TICKER=PETR4
LIVE_EXCHANGE=B
HIST_TICKER=PETR4
HIST_EXCHANGE=B
```

## Como Executar
Mock:
```
cargo run -p tucano-examples --bin mock_minimal
```
Login (Example 1):
```
cargo run -p tucano-examples --features real_dll --bin example_1_live_login
```
Histórico (Example 2):
```
cargo run -p tucano-examples --features real_dll --bin example_2_get_history_trades
```

## Variáveis
| Variável | Descrição |
|----------|-----------|
| PROFIT_USER | Usuário Profit |
| PROFIT_PASSWORD | Senha |
| PROFIT_ACTIVATION_KEY | Chave de ativação opcional |
| PROFITDLL_PATH | Caminho DLL real |
| LIVE_TICKER / LIVE_EXCHANGE | Ativo Example 1 |
| HIST_TICKER / HIST_EXCHANGE | Ativo Example 2 |
| PROFITDLL_DIAG | Logs diagnósticos (=1) |
| PROFITDLL_STRICT | Falha se não conseguir backend real (=1) |

## Troubleshooting
| Sintoma | Causa provável | Ação |
|---------|---------------|-------|
| Backend não é real_dll | Faltou feature / não Windows | Adicionar feature e usar Windows |
| Sem eventos | Credencial/licença | Verificar vars; `PROFITDLL_DIAG=1` |
| Histórico vazio | Intervalo ou callback placeholder | Ajustar intervalo/ticker |
| MissingSymbol | DLL incompatível | Verificar versão/export names |

## Roadmap
- [x] Mock mínimo
- [x] Login + subscribe
- [x] Histórico simples
- [ ] Envio de ordens
- [ ] Book nível 2
- [ ] VWAP / métricas streaming
- [ ] Progresso fetch histórico

---
Licença: Apache-2.0 OR MIT.
