<!--
	NOTA: Este arquivo foi reorganizado por tópicos funcionais a partir da versão linear auto‑convertida.
	O conteúdo bruto completo original foi preservado no final (Apêndice B) para auditoria.
-->

# Manual ProfitDLL (Reorganizado por Tópicos)

> Fonte: documentação oficial fornecida (última versão conhecida 4.0.0.30). Estrutura reagrupada por domínio funcional: visão geral, inicialização, autenticação, contas, posições, ordens, trades, book de ofertas, histórico, callbacks, funções utilitárias, tipos & estruturas, códigos de erro, mudanças de versão e apêndices.

## Índice
1. Visão Geral
2. Arquitetura & Threads
3. Inicialização e Login (Roteamento vs Market Data)
4. Callbacks: Modelo de Eventos
5. Contas e Subcontas
6. Tipos de Conta (AccountType)
7. Posições & Ativos da Posição
8. Ordens
9. Histórico de Ordens
10. Trades (Tempo Real & Histórico)
11. Livro de Ofertas (Offer Book)
12. Enumeração / Iteração (Assets, Orders, Positions)
13. Estruturas de Dados Principais
14. Tipos / Records (Referência Rápida)
15. Funções por Categoria
16. Códigos de Retorno & Erros
17. Considerações de Performance & Boas Práticas
18. Compatibilidade 32 vs 64 bits
19. Mapeamento de Tipos Delphi -> C / C# / Python
20. Changelog (Normalizado)
21. Apêndice A: Guia de Migração (Novos campos / callbacks)

---

## 1. Visão Geral
A ProfitDLL fornece uma interface de baixo nível (32 e 64 bits) para:
- Roteamento (envio/gestão de ordens e contas).
- Market Data (book, trades, cotações, posições).
Comunicação assíncrona baseada em callbacks (push) executados na thread interna `ConnectorThread`.

## 2. Arquitetura & Threads
- Thread interna: recebe eventos, normaliza e dispara callbacks.
- Aplicação cliente: deve copiar dados rapidamente e encaminhar para filas internas (evitar I/O bloqueante dentro de callbacks).
- Proibição: não chamar funções de requisição da DLL de dentro de callbacks (risco de deadlock / comportamento indefinido).

## 3. Inicialização e Login
| Cenário | Função | Requer | Callbacks iniciais |
|---------|--------|--------|--------------------|
| Roteamento + Market Data | `DLLInitializeLogin` | Código de ativação, usuário, senha | Set de callbacks obrigatórios (ordens, contas, MD) |
| Somente Market Data | `DLLInitializeMarketLogin` | Código de ativação, usuário, senha | Subconjunto (sem callbacks de roteamento) |

Boas práticas:
- Validar retorno; manter status de conexão.
- Re‑login controlado externamente (não em callback).

## 4. Callbacks: Modelo de Eventos
Categorias principais:
- Contas: lista, alterações, subcontas.
- Posições: atualização, zero position, assets vinculados.
- Ordens: criação, mudança de status, histórico, cancelamentos.
- Trades: tempo real (`SetTradeCallbackV2`), histórico (`SetHistoryTradeCallbackV2`).
- Livro de ofertas: callbacks V1 e V2 com flag adicional (desde 4.0.0.20).
- Asset Position List: iteração detalhada dos componentes de posição.

### Callbacks Introduzidos Recentemente
| Versão | Callback | Objetivo |
|--------|----------|----------|
| 4.0.0.28 | `SetAssetPositionListCallback` | Iterar ativos da posição |
| 4.0.0.28 | `SetBrokerAccountListChangedCallback` | Mudanças lista de contas |
| 4.0.0.28 | `SetBrokerSubAccountListChangedCallback` | Mudanças subcontas |
| 4.0.0.20 | `SetTradeCallbackV2`, `SetHistoryTradeCallbackV2` | Novo fluxo de trades |
| 4.0.0.20 | `SetOrderHistoryCallback` | Fluxo de histórico de ordens |

## 5. Contas e Subcontas
Novos mecanismos (4.0.0.28):
- Enumeração por corretora (`GetAccountCountByBroker`, `GetAccountsByBroker`).
- Callbacks de atualização assíncrona das listas.
Estruturas: `TBrokerAccountListCallback`, `TBrokerSubAccountListCallback`.

## 6. Tipos de Conta (AccountType)
Campo `AccountType` adicionado em 4.0.0.30 à estrutura `TConnectorTradingAccountOut`.
Enum (exemplo extraído): `cutOwner`, `cutAssessor`, `cutMaster`, `cutSubAccount`, `cutRiskMaster`, `cutPropOffice`, `cutPropManager`.

## 7. Posições & Ativos da Posição
Extensão 4.0.0.28:
- Campo `EventID` adicionado em `TConnectorTradingAccountPosition` e `TConnectorOrder` / `TConnectorOrderOut` para vincular eventos.
- Iteração de ativos de uma posição via `EnumerateAllPositionAssets` e callback `TConnectorAssetPositionListCallback`.

## 8. Ordens
Fluxo inclui:
- Envio / Cancelamento.
- Atualizações (order change callbacks V1/V2).
- Flag adicional em callbacks de livro (4.0.0.20) impacta assinaturas de observabilidade.

## 9. Histórico de Ordens
4.0.0.20 introduz:
- Estruturas: `TConnectorOrder`, `TConnectorEnumerateOrdersProc`.
- Funções: `HasOrdersInInterval`, `EnumerateOrdersByInterval`, `EnumerateAllOrders`.
Callback: `SetOrderHistoryCallback`.

## 10. Trades
Versão 4.0.0.20 adiciona `TranslateTrade` e callbacks V2 para tempo real & histórico.
Estruturas: `TConnectorTrade`, `TConnectorTradeCallback`.

## 11. Livro de Ofertas (Offer Book)
- Callbacks V1 (`SetOfferBookCallback`) e V2 (`SetOfferBookCallbackV2`) com flag adicional (4.0.0.20).
- Otimização: tratar dados incrementalmente (mínimo trabalho no callback).

## 12. Enumeração / Iteração
| Domínio | Funções | Observações |
|---------|---------|-------------|
| Ativos da posição | `EnumerateAllPositionAssets` | Requer callback registrado |
| Ordens | `EnumerateOrdersByInterval`, `EnumerateAllOrders` | Usa janela temporal / completa |
| Contas | `GetAccountCountByBroker`, `GetAccountsByBroker` | Agrupado por corretora |

## 13. Estruturas de Dados Principais (Resumo)
| Estrutura | Categoria | Campos novos (últimas versões) |
|-----------|-----------|---------------------------------|
| `TConnectorTradingAccountOut` | Conta | `AccountType` (4.0.0.30) |
| `TConnectorTradingAccountPosition` | Posição | `EventID` (4.0.0.28) |
| `TConnectorOrder` / `TConnectorOrderOut` | Ordem | `EventID` (4.0.0.28) |
| `TConnectorTrade` | Trade | V2 callbacks (4.0.0.20) |

## 14. Tipos / Records (Referência Rápida)
Ver Apêndice B para definição completa linear (Delphi-like). Exemplos: `TAssetIDRec`, `TConnectorAccountIdentifier`, `TConnectorZeroPosition`, enums de status, etc.

### 14.1 Catálogo Exaustivo de Enums
| Enum | Valores (nome=valor) | Observações / Versões |
|------|----------------------|-----------------------|
| `TConnectorOrderType` | `cotMarket=1`, `cotLimit=2`, `cotStopLimit=4` | Valores >=4 introduzem stop limit. Versão principal >= 4.0.0.18 alinhada a V1. |
| `TConnectorOrderTypeV0` | `cotLimit=0`, `cotStop=1`, `cotStopLimit=2` | Usado antes da 4.0.0.18 (modo legado). |
| `TConnectorOrderSide` | `cosBuy=1`, `cosSell=2` | Versão atual (>=4.0.0.18). |
| `TConnectorOrderSideV0` | `cosBuy=0`, `cosSell=1` | Layout legado pré 4.0.0.18. |
| `TConnectorPositionType` | `cptDayTrade=1`, `cptConsolidated=2` | Campo `PositionType` V1 em `TConnectorTradingAccountPosition`. |
| `TConnectorOrderStatus` | `cosNew=0`, `cosPartiallyFilled=1`, `cosFilled=2`, `cosDoneForDay=3`, `cosCanceled=4`, `cosReplaced=5`, `cosPendingCancel=6`, `cosStopped=7`, `cosRejected=8`, `cosSuspended=9`, `cosPendingNew=10`, `cosCalculated=11`, `cosExpired=12`, `cosAcceptedForBidding=13`, `cosPendingReplace=14`, `cosPartiallyFilledCanceled=15`, `cosReceived=16`, `cosPartiallyFilledExpired=17`, `cosPartiallyFilledRejected=18`, `cosUnknown=200`, `cosHadesCreated=201`, `cosBrokerSent=202`, `cosClientCreated=203`, `cosOrderNotCreated=204`, `cosCanceledByAdmin=205`, `cosDelayFixGateway=206`, `cosScheduledOrder=207` | Estados completos (legado + internos); valores >=200 internos/diagnóstico. |
| `TConnectorAccountType` | `cutOwner`, `cutAssessor`, `cutMaster`, `cutSubAccount`, `cutRiskMaster`, `cutPropOffice`, `cutPropManager` | Adicionado em 4.0.0.30 (`AccountType` em `TConnectorTradingAccountOut`). |
| Flags `TConnectorTradingAccountOut.Flags` | `CA_IS_SUB_ACCOUNT=1` | Indica subconta. |
| Flags `TConnectorMarketDataLibrary.Flags` | `CM_IS_SHORT_NAME=1` | Nome curto de agente. |
| Flags `TConnectorTradeCallback.Flags` | `TC_IS_EDIT=1` | Trade editado (marcação). |

> Para qualquer enum não totalmente listado (p.ex. estados de ordem detalhados) consultar Apêndice B; manteremos esta tabela incremental até extrair todos os rótulos com precisão (TODO).

### 14.2 Identificadores & Tipos Base
| Record | Campos Principais | Notas |
|--------|-------------------|-------|
| `TConnectorAccountIdentifier` | `Version`, `AccountNumber`, ... | Identificador entrada (in). Existe também versão *Out*. |
| `TConnectorAccountIdentifierOut` | `Version`, `AccountNumber`, ... | Usado em estruturas *Out* (ex.: `TConnectorOrderOut`). |
| `TConnectorAssetIdentifier` | `Version`, `Ticker`, `Market` | Entrada. |
| `TConnectorAssetIdentifierOut` | `Version`, `Ticker`, `Market` | Saída. |
| `TConnectorOrderIdentifier` | `Version`, `LocalID`, ... | Para mapear ordem. |
| `TConnectorSendOrder` | `AccountID`, `AssetID`, `Quantity`, `Price`, `StopPrice`, `OrderType`, `OrderSide`, `ValidityType`, `ValidityDate` | Função `SendOrder` aceita ponteiro para este record. |
| `TConnectorChangeOrder` | `AccountID`, `OrderID`, campos de alteração (p.ex. `Price`, `Quantity`) | Usado em `SendChangeOrderV2`. |
| `TConnectorCancelOrder` | `AccountID`, `OrderID` | Cancelamento individual. |
| `TConnectorCancelOrders` | `AccountID`, `AssetID` | Cancelar todas as ordens de um ativo. |
| `TConnectorCancelAllOrders` | `AccountID`, `Password` | Cancelar todas as ordens da conta. |
| `TConnectorZeroPosition` | `AccountID`, `AssetID`, `PositionType` (V1), `EventID` (V2) | Zerar posição; V2 retorna `Int64` id local. |
| `TConnectorTradingAccountOut` | Identificação, nomes, `AccountFlags`, `AccountType` (V1) | Estrutura de conta agregada. |
| `TConnectorTradingAccountPosition` | Identificação, métricas diárias, `PositionType` (V1), `EventID` (V2) | Posição consolidada / intraday. |
| `TConnectorOrder` | Identificação, quantidades, preços, `OrderSide`, `OrderType`, `OrderStatus`, datas, `EventID` (V1) | Visão interna / histórica. |
| `TConnectorOrderOut` | Similar a `TConnectorOrder` porém com identificadores *_Out*, `TextMessageLength` | Saída detalhada via `GetOrderDetails`. |
| `TConnectorTrade` | `TradeDate`, `TradeNumber`, `Price`, `Quantity` | Traduzido via `TranslateTrade`. |
| `TConnectorEnumerateOrdersProc` | Callback de iteração: `(const a_Order: PConnectorOrder; a_Param: Pointer): Integer` | Retorno controla continuação. |
| `TConnectorAssetPositionListCallback` | `(AccountID, AssetID, EventID)` | Alterações granularizadas de posição. |
| `TConnectorOrderCallback` | `(a_OrderID)` | Eventos de criação/alteração/cancelamento. |
| `TConnectorAccountCallback` | `(a_AccountID)` | Fim do carregamento de histórico / listas. |
| `TConnectorTradeCallback` | `(a_Asset, a_pTrade)` | Recebe trade bruto (usar `TranslateTrade`). |

### 14.3 Evolução de Versões de Records
| Record | V0 | V1 | V2 |
|--------|----|----|----|
| `TConnectorTradingAccountOut` | Sem `AccountType` | +`AccountType` | (igual) |
| `TConnectorTradingAccountPosition` | Sem `PositionType` | +`PositionType` | +`EventID` |
| `TConnectorOrder` | Sem `EventID` | +`EventID` | (igual) |
| `TConnectorOrderOut` | Sem `EventID` | +`EventID` | (igual) |
| `TConnectorZeroPosition` | Sem `PositionType` | +`PositionType` | +`EventID` (quando presente pela lógica de evento) |

### 14.4 Campos Críticos e Semântica
| Campo | Presente em | Semântica |
|-------|-------------|-----------|
| `EventID` | Posição (V2), Ordem (V1), ZeroPosition (V2) | Liga atualização a evento de lista de ativos da posição. |
| `PositionType` | Posição (V1), ZeroPosition (V1) | Diferencia DayTrade vs Consolidated. |
| `AccountType` | Conta (V1) | Classificação hierárquica de conta/subconta/risk/prop. |
| `OrderSide` | Ordem | Direção (compra/venda). Penetra diferença V0/V1. |
| `OrderType` | Ordem | Tipo (limite, mercado, stop limit). |
| `OrderStatus` | Ordem | Estado de ciclo de vida (novo, parcialmente executado, etc). |
| `ValidityType/ValidityDate` | Ordem | Regras de expiração (GTC, DAY, IOC...). |
| `TextMessage` | OrdemOut | Mensagem do broker / rejeição. |
| `OpenAveragePrice` | Posição | Preço médio da posição aberta. |
| `DailyQuantityAvailable` | Posição | Quantidade disponível para negociar. |

### 14.5 Mapeamento Rápido Ordem <-> Identificadores
| Objetivo | Estrutura | Observações |
|----------|-----------|-------------|
| Enviar | `TConnectorSendOrder` | Usa `AccountID` + `AssetID`.
| Alterar | `TConnectorChangeOrder` | Requer `OrderID` existente. |
| Cancelar única | `TConnectorCancelOrder` | `OrderID` obrigatório. |
| Cancelar por ativo | `TConnectorCancelOrders` | `AssetID` + conta. |
| Cancelar tudo | `TConnectorCancelAllOrders` | `AccountID` (+ senha). |
| Zerar posição | `TConnectorZeroPosition` | Gera ordem(s) compensatórias. |

### 14.6 Callbacks (Catálogo Consolidado)
| Categoria | Callback | Assinatura (simplificada) | Versão / Notas |
|-----------|----------|---------------------------|----------------|
| Posições (ativos) | `SetAssetPositionListCallback` | `(AccountID, AssetID, EventID)` | 4.0.0.28 |
| Ordens stream | `SetOrderCallback` | `(OrderID)` | Eventos individuais (se `OrderHistoryCallback` definido, apenas diffs). |
| Histórico ordens | `SetOrderHistoryCallback` | `(AccountID)` | Completa carregamento; 4.0.0.20. |
| Trades tempo real | `SetTradeCallbackV2` | `(AssetID, pTrade)` | 4.0.0.20 substitui V1. |
| Trades histórico | `SetHistoryTradeCallbackV2` | `(AssetID, pTrade)` | 4.0.0.20. |
| Book ofertas V1 | `SetOfferBookCallback` | `(AssetID, raw...)` | Legado; preferir V2. |
| Book ofertas V2 | `SetOfferBookCallbackV2` | `(AssetID, raw..., Flags)` | +flag adicional 4.0.0.20. |
| Price book V1 | `SetPriceBookCallback` | `(AssetID, raw...)` | Legado; preferir V2. |
| Price book V2 | `SetPriceBookCallbackV2` | `(AssetID, raw..., Flags)` | 4.0.0.20. |
| Trades (V1) | `SetTradeCallback` | `(AssetID, pTrade)` | Obsoleto (mantido compat). |
| Trades Hist (V1) | `SetHistoryTradeCallback` | `(AssetID, pTrade)` | Obsoleto. |
| Ordens mudança V1 | `SetOrderChangeCallback` | `(OrderID, change...)` | Pré unificação V2. |
| Ordens mudança V2 | `SetOrderChangeCallbackV2` | `(OrderID, change..., Flags)` | Preferido. |
| Asset list | `SetAssetListCallback` | `(AssetID...)` | Market data geral. |
| Asset list info V1 | `SetAssetListInfoCallback` | `(AssetID, info...)` | Legado. |
| Asset list info V2 | `SetAssetListInfoCallbackV2` | `(AssetID, info..., Flags)` | Atual. |
| Ajuste histórico V1 | `SetAdjustHistoryCallback` | `(AssetID, data...)` | Legado. |
| Ajuste histórico V2 | `SetAdjustHistoryCallbackV2` | `(AssetID, data..., Flags)` | Atual. |
| Tiny book | `SetTinyBookCallback` | `(AssetID, bids/asks compact)` | Otimização leve. |
| Daily | `SetDailyCallback` | `(AssetID, OHLC...)` | Série diária. |
| Theoretical price | `SetTheoreticalPriceCallback` | `(AssetID, price)` | Teórico de opções / derivativos. |
| Change cotation | `SetChangeCotationCallback` | `(AssetID, quote)` | Tick de cotação. |
| Change state ticker | `SetChangeStateTickerCallback` | `(AssetID, state)` | Estado (ex: halted). |
| Série progresso | `SetSerieProgressCallback` | `(AssetID, progress)` | Progresso carregamento histórico. |
| State | `SetStateCallback` | `(state)` | Estados globais da DLL. |
| Invalid ticker | `SetInvalidTickerCallback` | `(AssetID)` | Ticker rejeitado. |
| Conta | `SetAccountCallback` | `(AccountID)` | Eventos lista/conta. |
| Histórico genérico V1 | `SetHistoryCallback` | `(AssetID, data...)` | Legado. |
| Histórico genérico V2 | `SetHistoryCallbackV2` | `(AssetID, data..., Flags)` | Atual. |

### 14.7 Funções (Catálogo Agrupado)
| Grupo | Funções Principais |
|-------|-------------------|
| Inicialização | `DLLInitializeLogin`, `DLLInitializeMarketLogin`, `DLLFinalize`, `SetServerAndPort` |
| Config / Ambiente | `SetEnabledHistOrder`, `SetEnabledLogToDebug`, `SetDayTrade` |
| Subscrição MD | `SubscribeTicker`, `UnsubscribeTicker`, `SubscribeOfferBook`, `UnsubscribeOfferBook`, `SubscribePriceBook`, `UnsubscribePriceBook`, `SubscribeAdjustHistory`, `UnsubscribeAdjustHistory` |
| Assets Info | `RequestTickerInfo`, `GetLastDailyClose` |
| Callbacks Registro | Todos `Set*Callback` listados na seção 14.6 |
| Ordens Legado | `SendBuyOrder`, `SendSellOrder`, `SendMarketBuyOrder`, `SendMarketSellOrder`, `SendStopBuyOrder`, `SendStopSellOrder`, `SendChangeOrder`, `SendCancelOrder`, `SendCancelOrders`, `SendCancelAllOrders`, `SendZeroPosition`, `SendZeroPositionAtMarket` |
| Ordens Unificado V2 | `SendOrder`, `SendChangeOrderV2`, `SendCancelOrderV2`, `SendCancelOrdersV2`, `SendCancelAllOrdersV2`, `SendZeroPositionV2` |
| Consulta Ordem/Posição | `GetOrder`, `GetOrderProfitID`, `GetOrderDetails`, `GetPosition`, `GetPositionV2`, `GetAccountDetails` |
| Histórico Trades | `GetHistoryTrades`, `TranslateTrade` |
| Enumeração Ordens | `HasOrdersInInterval`, `EnumerateOrdersByInterval`, `EnumerateAllOrders` |
| Contas / Subcontas | `GetAccountCount`, `GetAccounts`, `GetSubAccountCount`, `GetSubAccounts`, `GetAccountCountByBroker`, `GetAccountsByBroker` (novas) |
| Agentes | `GetAgentNameByID`, `GetAgentShortNameByID`, `GetAgentNameLength`, `GetAgentName` |
| Tempo / Relógio | `GetServerClock` |

### 14.8 Notas de Deprecação
| Item | Substituído por | Estratégia |
|------|-----------------|-----------|
| `OrderTypeV0/OrderSideV0` | `OrderType` / `OrderSide` | Migrar ao atualizar para >=4.0.0.18. |
| Callbacks V1 (Trades, History, Offer/Price Book) | Correspondentes V2 | Registrar ambas temporariamente para fallback. |
| Funções envio específico (`SendBuyOrder` etc.) | `SendOrder` | Centralizar lógica; reduzir matriz de casos. |
| `GetOrders` antigo | `HasOrdersInInterval` + `Enumerate*` | Detecção + iteração incremental. |

### 14.9 Exemplo de Fluxo Completo (Resumo)
1. `DLLInitializeLogin` + registrar callbacks críticos (state, order, trade V2, offer book V2, asset position list).
2. Carregar contas: `GetAccountCount` / `GetAccounts` ou por corretora.
3. Subscrição ativos: `SubscribeTicker` + `SubscribeOfferBook` conforme necessidade.
4. Enviar ordem: montar `TConnectorSendOrder` e chamar `SendOrder`.
5. Reagir a `SetOrderCallback` para status; usar `GetOrderDetails` se campo adicional precisar.
6. Atualizar posição via `SetAssetPositionListCallback` + consultas `GetPositionV2` sob demanda.
7. Para histórico de ordens: `HasOrdersInInterval` -> `EnumerateOrdersByInterval`.
8. Finalizar: `DLLFinalize` (garantir descarte após flush de filas internas).

---

## 15. Funções por Categoria (Seleção)
| Categoria | Funções |
|-----------|---------|
| Inicialização | `DLLInitializeLogin`, `DLLInitializeMarketLogin` |
| Posições | `SendZeroPositionV2`, `GetPositionV2` (códigos ajustados 4.0.0.24) |
| Ordens | (envio/cancelamento), `HasOrdersInInterval`, `EnumerateAllOrders` |
| Trades | `TranslateTrade` |
| Contas | `GetAccountCountByBroker`, `GetAccountsByBroker` |
| Assets Posição | `EnumerateAllPositionAssets` |
| Agentes | `GetAgentNameLength`, `GetAgentName` (4.0.0.24) |

## 16. Códigos de Retorno & Erros (Exemplos)
- Não posição (GetPositionV2 / SendZeroPositionV2) códigos normalizados (4.0.0.24).
- Falhas de lista de contas: `GetAccounts` corrigida (4.0.0.28).

### 16.1 Tabelas de Status de Conexão / Login
| Categoria | Constante | Valor | Significado |
|-----------|-----------|-------|------------|
| Tipo de Conexão | `CONNECTION_STATE_LOGIN` | 0 | Conexão servidor de login |
| Tipo de Conexão | `CONNECTION_STATE_ROTEAMENTO` | 1 | Conexão servidor de roteamento |
| Tipo de Conexão | `CONNECTION_STATE_MARKET_DATA` | 2 | Conexão servidor de market data |
| Tipo de Conexão | `CONNECTION_STATE_MARKET_LOGIN` | 3 | Login em servidor de market data |
| Login | `LOGIN_CONNECTED` | 0 | Login efetuado |
| Login | `LOGIN_INVALID` | 1 | Usuário inválido |
| Login | `LOGIN_INVALID_PASS` | 2 | Senha inválida |
| Login | `LOGIN_BLOCKED_PASS` | 3 | Senha bloqueada |
| Login | `LOGIN_EXPIRED_PASS` | 4 | Senha expirada |
| Login | `LOGIN_UNKNOWN_ERR` | 200 | Erro interno de login |
| Roteamento | `ROTEAMENTO_DISCONNECTED` | 0 | Desconectado |
| Roteamento | `ROTEAMENTO_CONNECTING` | 1 | Conectando |
| Roteamento | `ROTEAMENTO_CONNECTED` | 2 | Conectado |
| Roteamento | `ROTEAMENTO_BROKER_DISCONNECTED` | 3 | Broker desconectado |
| Roteamento | `ROTEAMENTO_BROKER_CONNECTING` | 4 | Broker conectando |
| Roteamento | `ROTEAMENTO_BROKER_CONNECTED` | 5 | Broker conectado |
| Market Data | `MARKET_DISCONNECTED` | 0 | MD desconectado |
| Market Data | `MARKET_CONNECTING` | 1 | MD conectando |
| Market Data | `MARKET_WAITING` | 2 | Esperando conexão |
| Market Data | `MARKET_NOT_LOGGED` | 3 | Não logado |
| Market Data | `MARKET_CONNECTED` | 4 | Conectado |
| Ativação | `CONNECTION_ACTIVATE_VALID` | 0 | Ativação válida |
| Ativação | `CONNECTION_ACTIVATE_INVALID` | 1 | Ativação inválida |

### 16.2 Códigos de Erro (NL_*)
| Nome | Hex | Dec | Significado |
|------|-----|-----|------------|
| `NL_OK` | `0x00000000` | 0 | Sucesso |
| `NL_INTERNAL_ERROR` | `0x80000001` | -2147483647 | Erro interno |
| `NL_NOT_INITIALIZED` | `0x80000002` | -2147483646 | DLL não inicializada |
| `NL_INVALID_ARGS` | `0x80000003` | -2147483645 | Argumentos inválidos |
| `NL_WAITING_SERVER` | `0x80000004` | -2147483644 | Aguardando dados servidor |
| `NL_NO_LOGIN` | `0x80000005` | -2147483643 | Sem login ativo |
| `NL_NO_LICENSE` | `0x80000006` | -2147483642 | Sem licença |
| `NL_OUT_OF_RANGE` | `0x80000009` | -2147483639 | Fora de faixa / contagem excedida |
| `NL_MARKET_ONLY` | `0x8000000A` | -2147483638 | Função requer roteamento |
| `NL_NO_POSITION` | `0x8000000B` | -2147483637 | Posição inexistente |
| `NL_NOT_FOUND` | `0x8000000C` | -2147483636 | Recurso não encontrado |
| `NL_VERSION_NOT_SUPPORTED` | `0x8000000D` | -2147483635 | Versão não suportada |
| `NL_OCO_NO_RULES` | `0x8000000E` | -2147483634 | OCO sem regras |
| `NL_EXCHANGE_UNKNOWN` | `0x8000000F` | -2147483633 | Bolsa desconhecida |
| `NL_NO_OCO_DEFINED` | `0x80000010` | -2147483632 | OCO inexistente |
| `NL_INVALID_SERIE` | `0x80000011` | -2147483631 | Série inválida |
| `NL_LICENSE_NOT_ALLOWED` | `0x80000012` | -2147483630 | Recurso não liberado |
| `NL_NOT_HARD_LOGOUT` | `0x80000013` | -2147483629 | Não está em HardLogout |
| `NL_SERIE_NO_HISTORY` | `0x80000014` | -2147483628 | Série sem histórico |
| `NL_ASSET_NO_DATA` | `0x80000015` | -2147483627 | Ativo sem dados |
| `NL_SERIE_NO_DATA` | `0x80000016` | -2147483626 | Série sem dados |
| `NL_HAS_STRATEGY_RUNNING` | `0x80000017` | -2147483625 | Estratégia rodando |
| `NL_SERIE_NO_MORE_HISTORY` | `0x80000018` | -2147483624 | Sem mais histórico |
| `NL_SERIE_MAX_COUNT` | `0x80000019` | -2147483623 | Série no limite |
| `NL_DUPLICATE_RESOURCE` | `0x8000001A` | -2147483622 | Recurso duplicado |
| `NL_UNSIGNED_CONTRACT` | `0x8000001B` | -2147483621 | Contrato não assinado |
| `NL_NO_PASSWORD` | `0x8000001C` | -2147483620 | Senha ausente |
| `NL_NO_USER` | `0x8000001D` | -2147483619 | Usuário ausente |
| `NL_FILE_ALREADY_EXISTS` | `0x8000001E` | -2147483618 | Arquivo já existe |
| `NL_INVALID_TICKER` | `0x8000001F` | -2147483617 | Ticker inválido |
| `NL_NOT_MASTER_ACCOUNT` | `0x80000020` | -2147483616 | Conta não é master |

> Observação: Códigos negativos seguem convenção de HRESULT signed; checar sempre antes de tratar como sucesso. Somente `NL_OK` (0) indica sucesso definitivo. `NL_WAITING_SERVER` implica operação assíncrona pendente.

### 16.3 Boas Práticas de Tratamento
1. Tratar imediatamente `NL_WAITING_SERVER` como estado pendente e não re-disparar a mesma requisição até callback sinalizar conclusão.
2. Diferenciar falhas recuperáveis (`NL_WAITING_SERVER`, ausência de histórico) de falhas definitivas (`NL_INVALID_ARGS`).
3. Logar códigos desconhecidos para futura atualização de tabela.
4. Encapsular conversão para enum Rust segurando fallback para valor bruto.

## 17. Performance & Boas Práticas
- Evitar trabalho pesado em callbacks (sem I/O bloqueante).
- Copiar dados e enfileirar para processamento posterior.
- Não invocar funções de API dentro do callback que originou o evento.

## 18. Compatibilidade 32 vs 64 bits
- Mesma convenção (stdcall).
- 32 bits: limitação 4GB; dividir requisições volumosas.
- 64 bits: sem limitação prática de memória da DLL (além do SO).

## 19. Mapeamento de Tipos (Links)
- Delphi → C: Embarcadero RAD Studio docs.
- C → Python: `ctypes` docs.
- Delphi → C#: mapeamentos públicos (ver links originais no Apêndice B).

## 20. Changelog Normalizado
| Versão | Categoria | Alteração |
|--------|-----------|-----------|
| 4.0.0.30 | Contas | Campo `AccountType` em `TConnectorTradingAccountOut` |
| 4.0.0.28 | Posições | Campo `EventID` e iteração de ativos (`EnumerateAllPositionAssets`) |
| 4.0.0.28 | Contas | Enumeração por corretora + callbacks listas |
| 4.0.0.24 | Agentes | Funções `GetAgentNameLength` / `GetAgentName` |
| 4.0.0.24 | Posições | Ajuste códigos GetPositionV2 / SendZeroPositionV2; remoção limite tamanho ticker |
| 4.0.0.21 | Ordens | Correção duplicidade notificação sem alteração |
| 4.0.0.20 | Trades | Callbacks V2 + `TranslateTrade` |
| 4.0.0.20 | Ordens | Novo mecanismo histórico + funções enumerate |
| 4.0.0.20 | Livro | Flag adicional callbacks de book |

## 21. Apêndice A: Guia de Migração
- Verificar dependência de campos adicionados (AccountType, EventID).
- Atualizar assinatura de callbacks para V2 onde disponível (trades, book, order history).
- Adotar enumeração programática (orders / assets) ao invés de paginação manual.
