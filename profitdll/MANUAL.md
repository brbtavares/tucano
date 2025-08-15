# Manual da Crate `profitdll`

Integração isolada e opcional com a ProfitDLL, oferecendo:

- API unificada síncrona/assíncrona simplificada (`ProfitConnector`)
- Canal de eventos (`CallbackEvent`) com enumeração enriquecida (snapshot de ordem, trade, book, etc.)
- Mock cross‐platform para desenvolvimento (Linux/Mac) retornando `ProfitError`
- Bindings dinâmicos (Windows + feature `real_dll`) usando `libloading`
- Mapeamento completo de códigos `NL_*` → `ProfitError`

> Objetivo: permitir que o restante do workspace utilize eventos e ordens sem depender de cabeçalhos ou build steps proprietários enquanto mantém a superfície pronta para a DLL real em produção Windows.

---
## 1. Instalação

No workspace já incluso como membro. Em outro projeto:

```toml
[dependencies]
profitdll = { git = "https://github.com/<org>/tucano", package = "profitdll", default-features = false }
```

Para usar a DLL real (apenas Windows):

```toml
profitdll = { git = "https://github.com/<org>/tucano", package = "profitdll", features = ["real_dll" ] }
```

> Sem a feature `real_dll` a implementação mock é carregada; as funções retornam sucesso rápido e nenhum evento real é emitido.

---
## 2. Conceitos Principais

| Conceito | Descrição |
|----------|-----------|
| `ProfitConnector` | Fachada de alto nível para inicialização, login e operações de ordem. |
| `CallbackEvent` | Enum de eventos emitidos via `tokio::sync::mpsc::UnboundedReceiver`. |
| `AssetIdentifier` / `AccountIdentifier` | Identificadores de ativo e conta (ticker / exchange, account_id / broker). |
| `SendOrder` | Descreve ordem a ser enviada (market ou limit). |
| `OrderStatus` | Estados FIX + extensões proprietárias (mantidos como `#[non_exhaustive]`). |
| `ProfitError` | Mapeamento dos códigos numéricos `NL_*` + erros de carregamento. |

---
## 3. Fluxo de Uso (Mock ou DLL)

```rust
use profitdll::*;
use rust_decimal::Decimal;

# #[tokio::main]
# async fn main() -> Result<(), ProfitError> {
let connector = ProfitConnector::new(None)?; // ou Some("C:/caminho/ProfitDLL.dll")
let mut rx = connector.initialize_login("activation_key", "usuario", "senha").await?;

// subscrição (código de bolsa simplificado: "B"=Bovespa (ações), "F"=BM&F (derivativos))
connector.subscribe_ticker("PETR4", "B")?;

// envio de ordem
let order = SendOrder::new_market_order(
	AssetIdentifier::new("PETR4".into(), "B".into()),
	AccountIdentifier::new("ACC123".into(), "BRK".into()),
	OrderSide::Buy,
	Decimal::from(100)
);
connector.send_order(&order)?;

// processamento de eventos
while let Some(evt) = rx.recv().await {
	match evt {
		CallbackEvent::StateChanged { connection_type, result } => {
			println!("Estado: {:?} => {}", connection_type, result);
		}
		CallbackEvent::OrderSnapshot { order_id, status, filled, .. } => {
			println!("Snapshot ordem {order_id} status={:?} filled={}", status, filled);
		}
		_ => {}
	}
}
# Ok(()) }
```

---
## 4. Arquitetura Interna

### 4.1 Mock
Implementação em `src/mock.rs`. Mantém a mesma superfície para permitir alternância transparente. Gera eventos sintéticos (trades periódicos + atualizações simples de book) após `subscribe_ticker`. Usa os mesmos códigos de bolsa (`B` / `F`).

### 4.2 FFI Dinâmico
`src/ffi.rs` (ativado via `cfg(all(target_os = "windows", feature = "real_dll"))`).

Etapas no `new()`:
1. Carrega `Library` via `libloading`
2. Resolve símbolos obrigatórios (erro `MissingSymbol` se ausentes)
3. Cria canal unbounded para eventos
4. Armazena instância em `OnceCell` global

`initialize_login()`:
1. Chama `Initialize`
2. Registra callback de estado
3. Chama `Login`
4. Registra callback de ordem (se disponível)
5. Substitui o sender no struct global para devolver o `Receiver`

Callbacks capturam lock leve (`Mutex`) para serializar acesso e publicam `CallbackEvent`.

### 4.3 Snapshots de Ordem
Callback de ordem tenta primeiro `GetOrderDetails`; se sucesso, emite `OrderSnapshot` com enriquecimento (side/type/status/filled etc.). Caso contrário, fallback para `OrderUpdated` minimalista.

### 4.4 Sentinels
É usado o valor constante `SENTINEL_MARKET_OR_KEEP = -1.0` (exposta pelo módulo FFI) para representar:
1. Ordem de mercado (price ausente)
2. “Manter” valor anterior em alteração de ordem quando `new_price` ou `new_quantity` não são fornecidos.
Futuro: separar em duas constantes distintas se a DLL diferenciar semanticamente.

---
## 5. Erros (`ProfitError`)

Todos os retornos da DLL passam por `map()` → `from_nresult()`. Códigos desconhecidos convertem para `ProfitError::Unknown(code)` garantindo forward compatibility.

Categoria adicional de erros (feature `real_dll`): `Load`, `MissingSymbol`.

---
## 6. Extensões Planejadas

| Item | Status | Notas |
|------|--------|-------|
| Callbacks Trade/Book V2 | Pendente | Estrutura já preparada (tipos e placeholders). |
| Account & InvalidTicker callbacks | Pendente | Necessário fila para eventos de conta. |
| Eventos sintéticos no mock | Parcial | Geração básica implementada (trades + book). |
| Sentinel constants | Pendente | Melhorar semântica (-1.0). |
| Documentar mapeamento completo de `OrderStatus` | Parcial | Enum presente; descrição textual futura. |
| Testes integração Windows com DLL real | Pendente | Requer ambiente e CI matrix. |

---
## 7. Boas Práticas de Uso

1. Sempre tratar `CallbackEvent::StateChanged` antes de enviar ordens para garantir sessão estável.
2. Não bloquear thread dentro de callbacks (canal já desacopla; processar no consumer). 
3. Validar preenchimento parcial via `OrderSnapshot` (comparando `filled` e `quantity`).
4. Usar feature gating no workspace para evitar builds Windows em ambientes Linux CI quando a DLL for necessária.

---
## 8. Exemplo Simplificado (Market e Cancel)

```rust
let order_id_to_cancel = 12345i64;
connector.cancel_order(order_id_to_cancel)?;
```

---
## 9. Compatibilidade e Evolução

O enum `#[non_exhaustive]` em eventos e status permite adicionar novos campos / variantes sem breaking changes. Consumidores devem usar `_` no `match`.

---
## 10. Troubleshooting

| Sintoma | Causa Provável | Ação |
|---------|----------------|------|
| `ProfitError::Load` | Caminho DLL incorreto | Verifique path passado em `new(Some(path))`. |
| `ProfitError::MissingSymbol` | Versão DLL antiga | Atualize Profit ou ative somente símbolos disponíveis. |
| Sem eventos após login | Falha de credenciais ou callback não registrado | Inspecione eventos de `StateChanged` e código retornado. |
| `Unknown(n)` | Novo código não mapeado | Atualizar crate, ou tratar fallback. |

---
## 11. Segurança & Concurrency

Callbacks sincronizados com `Mutex` simples (baixa contenção). Se carga alta de eventos de book gerar latência, avaliar substituição por `parking_lot` + loteamento.

---
## 12. Roadmap Resumido

- [ ] Implementar todos os trampolines faltantes (trade/book/account)
- [ ] Eventos sintéticos no mock
- [ ] Enriquecimento de mensagens de erro (incluir ticker em InvalidTicker)
- [ ] Benchmarks de throughput (canal vs crossbeam)
- [ ] CI matrix Windows para testes de smoke com DLL real

---
## 13. Licença

Segue a licença do repositório principal. Código FFI não inclui binários proprietários da Profit.

---
## 14. Histórico

Originado da crate `tucano-profitdll` renomeada para `profitdll` para manter convenção de nomes no workspace.

---
## 15. Glossário

| Termo | Definição |
|-------|-----------|
| FFI | Foreign Function Interface |
| Sentinel | Valor especial que sinaliza ausência ou manter valor |
| Snapshot | Captura completa do estado de uma ordem via `GetOrderDetails` |

