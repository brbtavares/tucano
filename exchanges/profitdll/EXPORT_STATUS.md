# ProfitDLL Export Mapping Status

Gerado a partir de `profit.hpp` (C++ original) comparando com implementação atual em `profitdll/src/ffi.rs`.

Legenda:
- ✅ Implementado (ou equivalente) no Rust
- ⚠️ Parcial / Semântica reduzida
- ❌ Ausente
- 📝 Planejado (prioridade alta)

## 1. Inicialização / Sessão
| C++ Symbol | Status | Notas |
|------------|--------|-------|
| DLLInitializeLogin | ✅ (Initialize + Login sequência atual) | Carregado como `Initialize` + `Login` separados no Rust. Callbacks ainda reduzidos. |
| DLLInitializeMarketLogin | ❌ | Necessário segundo fluxo opcional (market). |
| DLLFinalize | ✅ (Finalize) | Método `Finalize` carregado mas não exposto publicamente ainda. |
| SetServerAndPort | ❌ | Config de endpoint. |
| GetServerClock | ❌ | Clock servidor (sincronização). |
| SetDayTrade | ❌ | Flag de day trade. |
| SetEnabledHistOrder | ❌ | Habilita histórico de ordens. |

## 2. Callbacks de Estado / Listas / Ajustes
| C++ Callback Setter | Status | Notas |
|---------------------|--------|-------|
| SetStateCallback | ✅ | Usado. |
| SetChangeCotationCallback | ❌ | Atualizações de cotação específicas. |
| SetAssetListCallback | ❌ | Lista básica de ativos. |
| SetAssetListInfoCallback | ❌ | Metadata ativo v1. |
| SetAssetListInfoCallbackV2 | ❌ | Metadata ativo v2 (setor / subsetor / segmento). |
| SetInvalidTickerCallback | ✅ (opcional) | Já carregado. |
| SetAdjustHistoryCallback | ❌ | Ajustes corporativos v1. |
| SetAdjustHistoryCallbackV2 | ⚠️ | Parsing heurístico inicial (layout suposto) + hexdump. |
| SetTheoreticalPriceCallback | ⚠️ | Placeholder registrado (campos parciais). |
| SeTConnectorBrokerAccountListChangedCallback | ❌ | Mudanças lista de contas. |
| SetBrokerSubAccountListChangedCallback | ❌ | Mudanças subcontas. |
| SetEnabledLogToDebug | ❌ | Ativar logs internos. |

## 3. Market Data Subscribe
| C++ Symbol | Status | Notas |
|------------|--------|-------|
| SubscribeTicker / UnsubscribeTicker | ✅ | Implementado. |
| SubscribePriceBook / UnsubscribePriceBook | ❌ | Book de preços (provavelmente snapshot Níveis). |
| SubscribeOfferBook / UnsubscribeOfferBook | ❌ | Book de ofertas detalhado. |
| SubscribeAdjustHistory / UnsubscribeAdjustHistory | ❌ | Stream de ajustes. |

## 4. Market Data Callbacks
| Callback | Status | Notas |
|----------|--------|-------|
| TNewTradeCallback | ⚠️ | Rust: TradeCallback(V1/V2) — sem parsing detalhado ainda (estrutura diferente). |
| THistoryTradeCallback | ⚠️ | Placeholder callback registrado (mesma struct de trade). |
| TNewDailyCallback | ⚠️ | Rust: DailySummary(V1/V2) — mapeado parcial. |
| TPriceBookCallback | ⚠️ | Rust: BookCallback(V1/V2) sem oferta detalhada / arrays. |
| TOfferBookCallback | ❌ | Ofertas (side + mudanças). |
| TNewTinyBookCallBack | ❌ | Nível reduzido. |
| TChangeStateTicker | ❌ | Estado de ticker. |
| TAdjustHistoryCallback / V2 | ⚠️ | Heurística parse (campos podem mudar quando layout confirmado). |
| TTheoreticalPriceCallback | ⚠️ | Placeholder (preço + qty). |
| TConnectorBrokerAccountListChangedCallback | ❌ | Lista de contas. |
| TConnectorBrokerSubAccountListChangedCallback | ❌ | Subcontas. |
| TProgressCallBack | ❌ | Progresso (ex: carregamento histórico). |
| TOrderChangeCallBack | ❌ | Atualizações ricas de ordem (temos só snapshot parcial via GetOrderDetails em callback de order). |
| THistoryCallBack (ordens) | ❌ | Histórico de ordens. |
| TAccountCallback | ✅ | Carregado como SetAccountCallback. |

## 5. Ordens / Execução
| C++ Symbol | Status | Notas |
|------------|--------|-------|
| SendBuyOrder / SendSellOrder | ❌ | Rust tem `SendOrder` genérico (não separado). |
| SendStopBuyOrder / SendStopSellOrder | ❌ | Faltam stop orders. |
| SendMarketBuyOrder / SendMarketSellOrder | ❌ | Market orders dedicadas. |
| SendZeroPosition | ❌ | Zerar posição. |
| SendCancelOrder | ❌ | Cancelar ordem específica (temos V2? não). |
| SendCancelOrders | ❌ | Cancelar por ticker. |
| SendCancelAllOrders | ❌ | Cancelar todas ordens. |
| SendChangeOrder | ⚠️ | Existe `SendChangeOrderV2` opcional; assinatura diferente. |
| GetOrder | ❌ | Query individual. |
| GetOrders | ❌ | Query lista. |
| GetOrderProfitID | ❌ | Lookup por ProfitID. |
| GetOrderDetails | ✅ (opcional) | Usado em callback de ordem para snapshot. |

## 6. Posições / Contas / Agents
| C++ Symbol | Status | Notas |
|------------|--------|-------|
| GetPosition | ❌ | Retorna blob; exige parsing (estrutura Position). |
| EnumerateAllPositionAssets | ❌ | Enumeração de ativos com posição. |
| GetAccount | ❌ | Enumerar contas. |
| GetAgentNameByID / GetAgentShortNameByID | ❌ | Identidade de agentes. |
| GetAgentNameLength / GetAgentName | ❌ | Versão segura para buffer. |

## 7. Histórico / Dados
| C++ Symbol | Status | Notas |
|------------|--------|-------|
| GetHistoryTrades | ⚠️ | Mock implementado + stub FFI; falta parse real. |
| GetLastDailyClose | ❌ | Fechamento diário. |

## 8. Infra / Utilitários
| C++ Symbol | Status | Notas |
|------------|--------|-------|
| FreePointer | ⚠️ | Wrapper ForeignBuffer criado; ainda sem parse real. |

## 9. Estruturas ausentes no Rust
Precisaremos mapear em `repr(C)` + conversões:
- TAssetID (wchar_t* ticker, bolsa, feed)
- TConnectorAccountIdentifier / TConnectorAssetIdentifier
- Position + sub strings (buffer packed)
- Trade / TradeCandle (para histórico e realtime se V2 não usado)
- BookOffer arrays (OfferBookCallback)

## 10. Prioridade de Implementação Proposta
1. Histórico & Ajustes:
   - GetHistoryTrades (pull) + THistoryTradeCallback (push incremental)
   - SubscribeAdjustHistory / callbacks de ajuste (V2 direto)
2. Market Data Profundidade:
   - OfferBookCallback (separar do Book V2 atual) + SubscribeOfferBook
3. Execução essencial:
   - SendBuyOrder / Sell / Market / Stop / CancelAll / ZeroPosition
   - GetPosition + FreePointer parsing
4. Metadados de Ativos:
   - SetAssetListInfoCallbackV2 + SetAssetListCallback
5. Posições / Contas:
   - EnumerateAllPositionAssets / GetAccount / Account & Broker callbacks
6. Utilidades:
   - GetServerClock / SetServerAndPort / GetLastDailyClose
7. Complementos:
   - AgentName APIs, TheoreticalPrice, ChangeStateTicker, ChangeCotation

## 11. Abordagem Técnica
- Adicionar módulo `ffi_types.rs` com structs/ conversões wide → UTF-8 (`widestring` crate) sob cfg windows + real_dll.
- Extender `ProfitRaw` com símbolos opcionais novos; manter gating incremental (não quebrar builds).
- Introduzir enum de eventos enriquecido (`CallbackEvent` expandido) para novos callbacks com feature flags (ex: feature `md_extended`).
- Buffers: usar `Vec<u8>` + ponteiros; liberar via `FreePointer` imediatamente após parse; garantir `unsafe` encapsulado.
- Wide Strings: converter via `U16CStr` -> `String` (lossy fallback).

## 12. Riscos / Cuidados
- Diferença semântica entre `InitializeLogin` e `InitializeMarketLogin` (ordem de callbacks e requisitos de progress callback).
- Potencial reentrância: documentação avisa para não chamar funções da DLL dentro dos callbacks; design: enfileirar dados e processar fora.
- Sincronização: ampliar `SenderState` ou criar múltiplos canais (ex: separar channel de ordem vs market) para backpressure.
- Memória: garantir que `FreePointer` seja chamado exatamente uma vez por buffer.

## 13. Próximos Passos Automatizáveis
Script (futuro) para validar exports vs mapping e produzir diff automático.

---
Gerado automaticamente – editar conforme novas funções forem adicionadas.
