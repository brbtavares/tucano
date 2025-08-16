# Manual - ProfitDLL

## Uso do Produto

###  Inicializando com Roteamento


Para utilizar a biblioteca é fundamental inicializar os serviços através das funções de inicialização. Mais especificamente, caso os serviços de roteamento sejam utilizados, deve-se utilizar a função ***DLLInitializeLogin***, que fará a conexão com os servidores de roteamento e market data.


Essa função é descrita na seção de funções expostas e requer um código de ativação fornecido no momento da contratação do produto, bem como nome de usuário e senha para efetuar o login no servidor de autenticação. Os outros parâmetros são callbacks obrigatórios (como ***StateCallback***, ***HistoryCallback***, ***OrderChangeCallback***, ***AccountCallback***, ***NewTradeCallback***, entre outros) que serão chamados pela DLL durante o uso e precisam ser especificados no momento da inicialização.


É importante notar que todos os callbacks ocorrem em uma thread chamada ***ConnectorThread*** e, portanto, ocorrem simultaneamente à aplicação cliente. A aplicação cliente deve processar os dados fornecidos através dos callbacks como dados a serem consumidos de outra thread. Sendo assim, caso necessário, devem tratar a escrita desses dados com seções críticas ou mutexes.


Os dados recebidos por meio de callbacks são armazenados em uma única fila de dados, portanto, qualquer processamento demorado dentro das funções de callback pode atrasar a fila de processamento de mensagens interna da DLL e causar atrasos no recebimento de trades ou outras informações. Para evitar isso, os dados devem ser processados e enviados para outras threads da aplicação imediatamente, ou realizar o mínimo de processamento possível. Acessos a banco de dados ou escritas em disco devem ser evitados durante o processamento de um callback.


Por fim, é importante ressaltar que os callbacks são projetados apenas para receber dados. Portanto, as funções de requisições à DLL ou qualquer outra função da interface da DLL (como ***SubscribeTicker***, ***SendBuyOrder***, ***GetAccount***, etc.) não devem ser chamadas dentro de um callback, pois isso pode causar exceções inesperadas e comportamento indefinido. Mais detalhes de implementação podem ser esclarecidos nos exemplos disponibilizados.

###  Inicializando com Market Data


O processo de inicialização do Market Data é análogo à inicialização com Roteamento, com a diferença do nome da função de inicialização ***DLLInitializeMarketLogin*** e redução de callbacks enviados por parâmetro, pois os mesmos são relacionados a ordens ou contas de roteamento.

### Tipos de dados


Todos os tipos citados nesse documento são tipos especificados na linguagem Delphi (como ***TAssetIDRec***, ***PWideChar***, ***Integer***, etc.), abaixo estão alguns links para conversão ou mapeamento desses tipos para as linguagens dos exemplos.

### Linkagem em 64 bits

Para utilizar a biblioteca em 64 bits a aplicação também deve ser compilada em 64 bits. A convenção de chamadas continua sendo ***stdcall***, assim como na versão 32 bits. Não existem problemas conhecidos para as linguagens de exemplo na versão 64 bits, portanto não há uma versão recomendada, é possível o uso das últimas versões de cada uma das linguagens.


A versão 64 bits não possui limitação de memória e portanto pode utilizar o máximo de memória disponível no sistema, podendo requisitar mais dados em uma requisição, limitado pela quantidade de RAM disponível.

## Interface da Biblioteca

A biblioteca expõe diversas funções chamadas diretamente pela aplicação cliente que realizam requisições para os servidores ou diretamente para os serviços e estruturas internas da DLL. Os tipos especificados nesta documentação estão codificados em Delphi, com exemplos específicos para outras linguagens de programação em seus respectivos arquivos de exemplo. Todas as estruturas necessárias para definir as funções da biblioteca são definidas a seguir:

```Delphi
TAssetIDRec = packed record 
    pwcTicker : PWideChar;  // Representa o nome do ativo ex.: "WDOFUT". 
    pwcBolsa  : PWideChar;  // Representa a bolsa que o ativo pertence ex. (para 
Bovespa): "B". 
    nFeed     : Integer;    
// Fonte dos dados 0 (Nelogica), 255 (Outro).
 end; 
PAssetIDRec = ^TAssetIDRec; 
TAccountRec = packed record 
    pwhAccountID     : PWideChar;  // Identificador da conta 
    pwhTitular       : PWideChar;  // Nome do titular da conta 
    pwhNomeCorretora : PWideChar;  // Nome da corretora 
    nCorretoraID     : Integer;    
// Identificador da corretora
 end; 
PAccountRec = ^TAccountRec; 
// Pointer Math 
PConnectorAccountIdentifierArrayOut = ^TConnectorAccountIdentifierOut; 
TConnectorOrderType = ( 
    cotMarket    = 1, 
    cotLimit     = 2, 
    cotStopLimit = 4 
); 
TConnectorOrderSide = ( 
    cosBuy  = 1, 
    cosSell = 2 
); 
TConnectorPositionType = ( 
    cptDayTrade     = 1, 
    cptConsolidated = 2 
); 
TConnectorOrderStatus = ( 
    cosNew                     = 0, 
    cosPartiallyFilled         = 1, 
    cosFilled                  = 2, 
    cosDoneForDay              = 3, 
    cosCanceled                = 4, 
    cosReplaced                = 5, 
    cosPendingCancel           = 6, 
    cosStopped                 = 7, 
    cosRejected                = 8, 
    cosSuspended               = 9, 
    cosPendingNew              = 10, 
    cosCalculated              = 11, 
    cosExpired                 = 12, 
    cosAcceptedForBidding      = 13, 
    cosPendingReplace          = 14, 
    cosPartiallyFilledCanceled = 15,
    cosReceived                = 16, 
    cosPartiallyFilledExpired  = 17, 
    cosPartiallyFilledRejected = 18, 
    cosUnknown                 = 200, 
    cosHadesCreated            = 201, 
    cosBrokerSent              = 202, 
    cosClientCreated           = 203, 
    cosOrderNotCreated         = 204, 
    cosCanceledByAdmin         = 205, 
    cosDelayFixGateway         = 206, 
    cosScheduledOrder          = 207 
); 
TConnectorAccountIdentifier = record 
    Version : Byte; 
// V0 
    BrokerID     : Integer; 
    AccountID    : PWideChar; 
    SubAccountID : PWideChar; 
    Reserved     : Int64; 
end; 
PConnectorAccountIdentifier = ^TConnectorAccountIdentifier; 
TConnectorAccountIdentifierOut = record 
    Version : Byte; 
// V0 
    BrokerID           : Integer; 
    AccountID          : TString0In; 
    AccountIDLength    : Integer; 
    SubAccountID       : TString0In; 
    SubAccountIDLength : Integer; 
    Reserved           : Int64; 
end; 
PConnectorAccountIdentifierOut = ^TConnectorAccountIdentifierOut; 
TConnectorAssetIdentifier = record 
    Version : Byte; 
// V0 
    Ticker   : PWideChar; 
    Exchange : PWideChar; 
    FeedType : Byte; 
end; 
TConnectorAssetIdentifierOut = record 
    Version : Byte; 
// V0 
    Ticker         : PWideChar; 
    TickerLength   : Integer; 
    Exchange       : PWideChar; 
    ExchangeLength : Integer; 
    FeedType : Byte; 
end;
TConnectorOrderIdentifier = record 
    Version : Byte; 
// V0 
    LocalOrderID : Int64; 
    ClOrderID    : PWideChar; 
end; 
TConnectorSendOrder = record 
    Version : Byte; 
// V0 
    AccountID : TConnectorAccountIdentifier; 
    AssetID   : TConnectorAssetIdentifier; 
    Password  : PWideChar; 
    OrderType : Byte; 
    OrderSide : Byte; 
    Price     : Double; 
    StopPrice : Double; 
    Quantity  : Int64; 
end; 
PConnectorSendOrder = ^TConnectorSendOrder; 
TConnectorChangeOrder = record 
    Version : Byte; 
// V0 
    AccountID : TConnectorAccountIdentifier; 
    OrderID   : TConnectorOrderIdentifier; 
    Password  : PWideChar; 
    Price     : Double; 
    StopPrice : Double; 
    Quantity  : Int64; 
end; 
PConnectorChangeOrder = ^TConnectorChangeOrder; 
TConnectorCancelOrder = record 
    Version : Byte; 
// V0 
    AccountID : TConnectorAccountIdentifier; 
    OrderID   : TConnectorOrderIdentifier; 
    Password  : PWideChar; 
end; 
PConnectorCancelOrder = ^TConnectorCancelOrder; 
TConnectorCancelOrders = record 
    Version : Byte; 
// V0 
    AccountID : TConnectorAccountIdentifier; 
    AssetID   : TConnectorAssetIdentifier; 
    Password  : PWideChar;
    end; 
PConnectorCancelOrders = ^TConnectorCancelOrders; 
TConnectorCancelAllOrders = record 
    Version : Byte; 
// V0 
    AccountID : TConnectorAccountIdentifier; 
    Password  : PWideChar; 
end; 
PConnectorCancelAllOrders = ^TConnectorCancelAllOrders; 
TConnectorZeroPosition = record 
    Version : Byte; 
// V0 
    AccountID : TConnectorAccountIdentifier; 
    AssetID   : TConnectorAssetIdentifier; 
    Password  : PWideChar; 
    Price     : Double; 
// V1 
    PositionType : Byte; 
end; 
PConnectorZeroPosition = ^TConnectorZeroPosition; 
TConnectorAccountType = ( 
  cutOwner       = 0, 
  cutAssessor    = 1, 
  cutMaster      = 2, 
  cutSubAccount  = 3, 
  cutRiskMaster  = 4, 
  cutPropOffice  = 5, 
  cutPropManager = 6 
); 
TConnectorTradingAccountOut = record 
    Version : Byte; 
// In Fields 
    AccountID : TConnectorAccountIdentifier; 
// Out fields 
    BrokerName         : PWideChar; 
    BrokerNameLength   : Integer; 
    OwnerName          : PWideChar; 
    OwnerNameLength    : Integer; 
    SubOwnerName       : PWideChar; 
    SubOwnerNameLength : Integer; 
    AccountFlags       : TFlags; 
// V1 
    AccountType        : Byte; // TConnectorAccountType
     end; 
PConnectorTradingAccountOut = ^TConnectorTradingAccountOut; 
TConnectorTradingAccountPosition = record 
    Version : Byte; 
// In Fields 
    AccountID : TConnectorAccountIdentifier; 
    AssetID   : TConnectorAssetIdentifier; 
// Out Fields 
    OpenQuantity           : Int64; 
    OpenAveragePrice       : Double; 
    OpenSide               : Byte; 
    DailyAverageSellPrice  : Double; 
    DailySellQuantity      : Int64; 
    DailyAverageBuyPrice   : Double; 
    DailyBuyQuantity       : Int64; 
    DailyQuantityD1        : Int64; 
    DailyQuantityD2        : Int64; 
    DailyQuantityD3        : Int64; 
    DailyQuantityBlocked   : Int64; 
    DailyQuantityPending   : Int64; 
    DailyQuantityAlloc     : Int64; 
    DailyQuantityProvision : Int64; 
    DailyQuantity          : Int64; 
    DailyQuantityAvailable : Int64; 
// V1 
    PositionType : Byte; 
// V2 
    EventID : Int64; 
end; 
PConnectorTradingAccountPosition = ^TConnectorTradingAccountPosition; 
TConnectorOrder = record 
    Version : Byte; 
    OrderID           : TConnectorOrderIdentifier; 
    AccountID         : TConnectorAccountIdentifier; 
    AssetID           : TConnectorAssetIdentifier; 
    Quantity          : Int64; 
    TradedQuantity    : Int64; 
    LeavesQuantity    : Int64; 
    Price             : Double; 
    StopPrice         : Double; 
    AveragePrice      : Double; 
    OrderSide         : Byte; // TConnectorOrderSide 
    OrderType         : Byte; // TConnectorOrderType 
    OrderStatus       : Byte; 
    ValidityType      : Byte; 
    Date              : TSystemTime; 
    LastUpdate        : TSystemTime; 
    CloseDate         : TSystemTime; 
    ValidityDate      : TSystemTime; 
    TextMessage       : PWideChar; 
// V1 
    EventID : Int64; 
end; 
PConnectorOrder = ^TConnectorOrder; 
TConnectorOrderOut = record 
    Version : Byte; 
// In Fields 
    OrderID           : TConnectorOrderIdentifier; 
// Out Fields 
    AccountID         : TConnectorAccountIdentifierOut; 
    AssetID           : TConnectorAssetIdentifierOut; 
    Quantity          : Int64; 
    TradedQuantity    : Int64; 
    LeavesQuantity    : Int64; 
    Price             : Double; 
    StopPrice         : Double; 
    AveragePrice      : Double; 
    OrderSide         : Byte; // TConnectorOrderSide 
    OrderType         : Byte; // TConnectorOrderType 
    OrderStatus       : Byte; 
    ValidityType      : Byte; 
    Date              : TSystemTime; 
    LastUpdate        : TSystemTime; 
    CloseDate         : TSystemTime; 
    ValidityDate      : TSystemTime; 
    TextMessage       : PWideChar; 
    TextMessageLength : Integer; 
// V1 
    EventID : Int64; 
end; 
PConnectorOrderOut = ^TConnectorOrderOut; 
TConnectorTrade = record 
    Version     : Byte; 
    TradeDate   : TSystemTime; 
    TradeNumber : Cardinal; 
    Price       : Double; 
    Quantity    : Int64;
     Volume      : Double; 
    BuyAgent    : Integer; 
    SellAgent   : Integer; 
    TradeType   : Byte; //TTradeType
 end; 
PConnectorTrade = ^TConnectorTrade; 
// Delegates 
TConnectorEnumerateOrdersProc = function( 
const a_Order : PConnectorOrder; 
const a_Param : LPARAM 
) : BOOL; stdcall; 
// TConnectorTradingAccountOut.Flags 
CA_IS_SUB_ACCOUNT : TFlags = 1; 
CA_IS_ENABLED     : TFlags = 2; 
// TConnectorMarketDataLibrary.Flags 
CM_IS_SHORT_NAME  : TFlags = 1; 
// Bolsas 
gc_bvBCB            = 65; // A 
gc_bvBovespa        = 66; // B 
gc_bvCambio         = 68; // D 
gc_bvEconomic       = 69; // E 
gc_bvBMF            = 70; // F 
gc_bvMetrics        = 75; // K 
gc_bvCME            = 77; // M 
gc_bvNasdaq         = 78; // N 
gc_bvOXR            = 79; // O 
gc_bvPioneer        = 80; // P 
gc_bvDowJones       = 88; // X 
gc_bvNyse           = 89; // Y 
// Status 
CONNECTION_STATE_LOGIN        = 0;  // Conexão com servidor de login 
CONNECTION_STATE_ROTEAMENTO   = 1;  // Conexão com servidor de roteamento 
CONNECTION_STATE_MARKET_DATA  = 2;  // Conexão com servidor de market data 
CONNECTION_STATE_MARKET_LOGIN = 3;  // Login com servidor market data 
LOGIN_CONNECTED    = 0;   
LOGIN_INVALID      = 1;   
LOGIN_INVALID_PASS = 2;   
LOGIN_BLOCKED_PASS = 3;   
LOGIN_EXPIRED_PASS = 4;   
// Servidor de login conectado 
// Login é inválido 
// Senha inválida 
// Senha bloqueada 
// Senha expirada 
LOGIN_UNKNOWN_ERR  = 200; // Erro interno de login 
ROTEAMENTO_DISCONNECTED        = 0; 
ROTEAMENTO_CONNECTING          = 1; 
ROTEAMENTO_CONNECTED           = 2; 
ROTEAMENTO_BROKER_DISCONNECTED = 3; 
ROTEAMENTO_BROKER_CONNECTING   = 4; 
ROTEAMENTO_BROKER_CONNECTED    = 5; 
MARKET_DISCONNECTED = 0;  // Desconectado do servidor de market data 
MARKET_CONNECTING   = 1;  // Conectando ao servidor de market data 
 MARKET_WAITING      = 2;  // Esperando conexão 
MARKET_NOT_LOGGED   = 3;  // Não logado ao servidor de market data 
MARKET_CONNECTED    = 4;  // Conectado ao market data 
CONNECTION_ACTIVATE_VALID   = 0;  // Ativação válida 
CONNECTION_ACTIVATE_INVALID = 1;  // Ativação inválida 
// Versões Antigas 
TConnectorOrderTypeV0 = ( 
    cotLimit  = 0, 
    cotStop   = 1, 
    cotMarket = 2 
); 
TConnectorOrderSideV0 = ( 
    cosBuy  = 0, 
    cosSell = 1 
); 
// TConnectorTradeCallback.Flags 
TC_IS_EDIT     : Cardinal = 1; 
TC_LAST_PACKET : Cardinal = 2;
```


### Códigos de erro

Tabela dos códigos (***NL_*** 2A) retornados pela DLL. Valores negativos seguem estilo ***HRESULT*** (bit de erro setado, exibido aqui em hex e decimal).

| Nome | Valor (Hex) | Valor (Dec) | Significado |
|------|-------------|-------------|-------------|
| NL_OK | 0x00000000 | 0 | Sucesso |
| NL_INTERNAL_ERROR | 0x80000001 | -2147483647 | Erro interno |
| NL_NOT_INITIALIZED | 0x80000002 | -2147483646 | DLL não inicializada |
| NL_INVALID_ARGS | 0x80000003 | -2147483645 | Argumentos inválidos |
| NL_WAITING_SERVER | 0x80000004 | -2147483644 | Aguardando dados do servidor |
| NL_NO_LOGIN | 0x80000005 | -2147483643 | Nenhum login efetuado |
| NL_NO_LICENSE | 0x80000006 | -2147483642 | Sem licença |
| NL_OUT_OF_RANGE | 0x80000009 | -2147483639 | Parâmetro fora de faixa / limite excedido |
| NL_MARKET_ONLY | 0x8000000A | -2147483638 | Função requer roteamento/mercado |
| NL_NO_POSITION | 0x8000000B | -2147483637 | Posição inexistente |
| NL_NOT_FOUND | 0x8000000C | -2147483636 | Recurso não encontrado |
| NL_VERSION_NOT_SUPPORTED | 0x8000000D | -2147483635 | Versão não suportada |
| NL_OCO_NO_RULES | 0x8000000E | -2147483634 | OCO sem regras |
| NL_EXCHANGE_UNKNOWN | 0x8000000F | -2147483633 | Bolsa desconhecida |
| NL_NO_OCO_DEFINED | 0x80000010 | -2147483632 | OCO inexistente |
| NL_INVALID_SERIE | 0x80000011 | -2147483631 | Série inválida (Level/Offset/Factor) |
| NL_LICENSE_NOT_ALLOWED | 0x80000012 | -2147483630 | Recurso não liberado na licença |
| NL_NOT_HARD_LOGOUT | 0x80000013 | -2147483629 | Não está em HardLogout |
| NL_SERIE_NO_HISTORY | 0x80000014 | -2147483628 | Série sem histórico |
| NL_ASSET_NO_DATA | 0x80000015 | -2147483627 | Ativo sem dados (TData) |
| NL_SERIE_NO_DATA | 0x80000016 | -2147483626 | Série sem dados (count=0) |
| NL_HAS_STRATEGY_RUNNING | 0x80000017 | -2147483625 | Estratégia em execução |
| NL_SERIE_NO_MORE_HISTORY | 0x80000018 | -2147483624 | Sem mais histórico disponível |
| NL_SERIE_MAX_COUNT | 0x80000019 | -2147483623 | Série atingiu limite de dados |
| NL_DUPLICATE_RESOURCE | 0x8000001A | -2147483622 | Recurso duplicado |
| NL_UNSIGNED_CONTRACT | 0x8000001B | -2147483621 | Contrato não assinado |
| NL_NO_PASSWORD | 0x8000001C | -2147483620 | Senha ausente |
| NL_NO_USER | 0x8000001D | -2147483619 | Usuário ausente |
| NL_FILE_ALREADY_EXISTS | 0x8000001E | -2147483618 | Arquivo já existe |
| NL_INVALID_TICKER | 0x8000001F | -2147483617 | Ticker inválido |
| NL_NOT_MASTER_ACCOUNT | 0x80000020 | -2147483616 | Conta não é master |

 ## Funções expostas
 
 As declarações de todas as funções expostas se encontram nesta seção. Algumas funções recebem tipos
 contendo callback no nome, estas serão descritas na próxima subseção.

 ```Delphi
function DLLInitializeLogin( 
    const pwcActivationKey : PWideChar; 
    const pwcUser          : PWideChar; 
    const pwcPassword      : PWideChar; 
    StateCallback          : TStateCallback; 
    HistoryCallback        : THistoryCallback; 
    OrderChangeCallback    : TOrderChangeCallback; 
    AccountCallback        : TAccountCallback; 
    NewTradeCallback       : TNewTradeCallback; 
    NewDailyCallback       : TNewDailyCallback; 
    PriceBookCallback      : TPriceBookCallback; 
    OfferBookCallback      : TOfferBookCallback; 
    HistoryTradeCallback   : THistoryTradeCallback; 
    ProgressCallback       : TProgressCallback; 
    TinyBookCallback       : TTinyBookCallback) : Integer; stdcall;
    function DLLInitializeMarketLogin( 
const pwcActivationKey : PWideChar; 
const pwcUser          : PWideChar; 
const pwcPassword      : PWideChar; 
    StateCallback          : TStateCallback; 
    NewTradeCallback       : TNewTradeCallback; 
    NewDailyCallback       : TnewDailyCallback 
    PriceBookCallback      : TPriceBookCallback; 
    OfferBookCallback      : TOfferBookCallback; 
    HistoryTradeCallback   : THistoryTradeCallback; 
    ProgressCallback       : TProgressCallback; 
    TinyBookCallback       : TTinyBookCallback) : Integer; stdcall; 
function DLLFinalize: Integer; stdcall; 
function SubscribeTicker(pwcTicker : PWideChar; pwcBolsa : PWideChar) : Integer; 
stdcall; 
 function UnsubscribeTicker(pwcTicker : PWideChar; pwcBolsa : PWideChar) : Integer; 
stdcall; 
function SubscribePriceBook(pwcTicker : PWideChar; pwcBolsa : PWideChar) : Integer; 
stdcall; 
function UnsubscribePriceBook(pwcTicker : PWideChar; pwcBolsa : PWideChar) : Integer; 
stdcall; 
function SubscribeOfferBook(pwcTicker : PWideChar; pwcBolsa : PWideChar) : Integer; 
stdcall; 
function UnsubscribeOfferBook(pwcTicker : PWideChar; pwcBolsa : PWideChar) : Integer; 
stdcall; 
function GetAgentNameByID(nID : Integer) : PWideChar; stdcall; 
function GetAgentShortNameByID(nID : Integer) : PWideChar; stdcall; 
function GetAgentNameLength(nAgentID : Integer; nShortName : Cardinal): Integer; 
stdcall; 
function GetAgentName( 
    nCount : Integer;  
    nAgentID : Integer;  
    pwcAgent : PWideChar;  
    nShortName : Cardinal) : Integer; stdcall; 
function GetAccount : Integer; stdcall; 
function SendBuyOrder( 
    pwcIDAccount   : PWideChar; 
    pwcIDCorretora : PWideChar; 
    pwcSenha       : PWideChar; 
    pwcTicker      : PWideChar; 
    pwcBolsa       : PWideChar; 
    dPrice         : Double; 
     nAmount        : Integer) : Int64; stdcall; 
function SendSellOrder( 
    pwcIDAccount   : PWideChar; 
    pwcIDCorretora : PWideChar; 
    pwcSenha       : PWideChar; 
    pwcTicker      : PWideChar; 
    pwcBolsa       : PWideChar; 
    dPrice         : Double; 
    nAmount        : Integer) : Int64; stdcall; 
function SendMarketBuyOrder( 
    pwcIDAccount   : PWideChar; 
    pwcIDCorretora : PWideChar; 
    pwcSenha       : PWideChar; 
    pwcTicker      : PWideChar; 
    pwcBolsa       : PWideChar; 
    nAmount        : Integer) : Int64; stdcall; 
function SendMarketSellOrder( 
    pwcIDAccount   : PWideChar; 
    pwcIDCorretora : PWideChar; 
    pwcSenha       : PWideChar; 
    pwcTicker      : PWideChar; 
    pwcBolsa       : PWideChar; 
    nAmount        : Integer) : Int64; stdcall; 
function SendStopBuyOrder( 
    pwcIDAccount   : PWideChar; 
    pwcIDCorretora : PWideChar; 
    pwcSenha       : PWideChar; 
    pwcTicker      : PWideChar; 
    pwcBolsa       : PWideChar; 
    dPrice         : Double; 
    dStopPrice     : Double; 
    nAmount        : Integer) : Int64; stdcall; 
function SendStopSellOrder( 
    pwcIDAccount   : PWideChar; 
    pwcIDCorretora : PWideChar; 
    pwcSenha       : PWideChar; 
    pwcTicker      : PWideChar; 
    pwcBolsa       : PWideChar; 
    dPrice         : Double; 
    dStopPrice     : Double; 
    nAmount        : Integer) : Int64; stdcall; 
function SendChangeOrder( 
    pwcIDAccount   : PWideChar; 
    pwcIDCorretora : PWideChar; 
    pwcSenha       : PWideChar; 
    pwcstrClOrdID  : PWideChar; 
    dPrice         : Double; 
    nAmount        : Integer) : Integer; stdcall; 
 function SendCancelOrder( 
 pwcIDAccount   : PWideChar; 
    pwcIDCorretora : PWideChar; 
    pwcClOrdId     : PWideChar; 
    pwcSenha       : PWideChar) : Integer; stdcall; 
function SendCancelOrders( 
    pwcIDAccount   : PWideChar; 
    pwcIDCorretora : PWideChar; 
    pwcSenha       : PWideChar; 
    pwcTicker      : PWideChar; 
    pwcBolsa       : PWideChar) : Integer; stdcall; 
function SendCancelAllOrders( 
    pwcIDAccount   : PWideChar; 
    pwcIDCorretora : PWideChar; 
    pwcSenha       : PWideChar) : Integer; stdcall; 
function SendZeroPosition( 
    pwcIDAccount   : PWideChar; 
    pwcIDCorretora : PWideChar; 
    pwcTicker      : PWideChar; 
    pwcBolsa       : PWideChar; 
    pwcSenha       : PWideChar; 
    dPrice         : Double) : Int64; stdcall; 
function SendZeroPositionAtMarket( 
    pwcIDAccount   : PWideChar; 
    pwcIDCorretora : PWideChar; 
    pwcTicker      : PWideChar; 
    pwcBolsa       : PWideChar; 
    pwcSenha       : PWideChar) : Int64; stdcall; 
function GetOrders( 
    pwcIDAccount   : PWideChar; 
    pwcIDCorretora : PWideChar; 
    dtStart        : PWideChar; 
    dtEnd          : PWideChar) : Integer; stdcall; 
function GetOrder(pwcClOrdId : PWideChar) : Integer; stdcall; 
function GetOrderProfitID(nProfitID : Int64): Integer; stdcall; 
function GetPosition( 
    pwcIDAccount   : PWideChar; 
    pwcIDCorretora : PWideChar; 
    pwcTicker      : PWideChar; 
    pwcBolsa       : PWideChar) : Pointer; stdcall; 
function GetHistoryTrades( 
const pwcTicker : PWideChar; 
const pwcBolsa  : PWideChar; 
    dtDateStart     : PWideChar; 
    dtDateEnd       : PWideChar) : Integer; stdcall; 
function SendOrder (const a_SendOrder    : PConnectorSendOrder) : Int64; stdcall; 
 function SendChangeOrderV2     
Integer; stdcall; 
function SendCancelOrderV2     
Integer; stdcall; 
function SendCancelOrdersV2    
 (const a_ChangeOrder  : PConnectorChangeOrder)
 (const a_CancelOrder  : PConnectorCancelOrder)
 (const a_CancelOrder  : PConnectorCancelOrders)
     : 
     : 
    : 
Integer; stdcall; 
function SendCancelAllOrdersV2 (const a_CancelOrder  : PConnectorCancelAllOrders) : 
Integer; stdcall; 
function SendZeroPositionV2    
(const a_ZeroPosition : PConnectorZeroPosition)
 Int64;   
stdcall; 
function GetAccountCount : Integer; stdcall; 
function GetAccounts( 
const a_nStartSource : Integer; 
const a_nStartDest   : Integer; 
const a_nCount       : Integer; 
const a_arAccounts   : PConnectorAccountIdentifierArrayOut 
) : Integer; stdcall; 
function GetAccountDetails(var a_Account : TConnectorTradingAccountOut) : Integer; 
stdcall; 
    : 
function GetSubAccountCount(const a_MasterAccountID : PConnectorAccountIdentifier) : 
Integer; stdcall; 
function GetSubAccounts( 
const a_MasterAccountID : PConnectorAccountIdentifier; 
const a_nStartSource    : Integer; 
const a_nStartDest      : Integer; 
const a_nCount          : Integer; 
const a_arAccounts      : PConnectorAccountIdentifierArrayOut 
) : Integer; stdcall; 
function GetPositionV2(var a_Position : TConnectorTradingAccountPosition) : Integer; 
stdcall; 
function GetOrderDetails(var a_Order : TConnectorOrderOut) : Integer; stdcall; 
function HasOrdersInInterval(const a_AccountID : PConnectorAccountIdentifier; const 
a_dtStart : TSystemTime; const a_dtEnd : TSystemTime) : NResult; stdcall; 
function EnumerateOrdersByInterval( 
const a_AccountID    : PConnectorAccountIdentifier; 
const a_OrderVersion : Byte; 
const a_dtStart      : TSystemTime; 
const a_dtEnd        : TSystemTime; 
const a_Param        : LPARAM; 
const a_Callback     : TConnectorEnumerateOrdersProc 
) : NResult; stdcall; 
function EnumerateAllOrders( 
const a_AccountID    : PConnectorAccountIdentifier; 
const a_OrderVersion : Byte; 
const a_Param        : LPARAM; 
const a_Callback     : TConnectorEnumerateOrdersProc 
 ) : NResult; stdcall; 
 function TranslateTrade(const a_pTrade : Pointer; var a_Trade : TConnectorTrade) : 
NResult; stdcall; 
function SetServerAndPort(const strServer, strPort : PWideChar) : Integer; stdcall; 
function GetServerClock (var dtDate : Double; var nYear, nMonth, nDay, nHour, nMin, 
nSec, nMilisec: Integer) : Integer; stdcall; 
function SetDayTrade(bUseDayTrade : Integer): Integer; stdcall; forward; 
function SetEnabledHistOrder(bEnabled : Integer) : Integer; stdcall; forward; 
function SetEnabledLogToDebug(bEnabled : Integer) : Integer; stdcall; forward; 
function RequestTickerInfo(const pwcTicker : PWideChar; const pwcBolsa : PWideChar) : 
Integer; stdcall; forward; 
function SubscribeAdjustHistory(pwcTicker : PWideChar; pwcBolsa : PWideChar) : 
Integer; stdcall; 
function UnsubscribeAdjustHistory(pwcTicker : PWideChar; pwcBolsa : PWideChar) : 
Integer; stdcall; 
function GetLastDailyClose(const pwcTicker, pwcBolsa: var dClose : Double; bAdjusted 
: Integer) : Integer; stdcall; 
function SetStateCallback(const a_StateCallback : TStateCallback) : Integer; stdcall; 
function SetAssetListCallback(const a_AssetListCallback : TAssetListCallback) : 
Integer; stdcall; 
function SetAssetListInfoCallback(const a_AssetListInfoCallback : 
TAssetListInfoCallback) : Integer; stdcall; 
function SetAssetListInfoCallbackV2(const a_AssetListInfoCallbackV2 : 
TAssetListInfoCallbackV2) : Integer; stdcall; 
function SetInvalidTickerCallback(const a_InvalidTickerCallback : 
TInvalidTickerCallback) : Integer; stdcall; 
function SetTradeCallback(const a_TradeCallback : TTradeCallback) : Integer; stdcall; 
function SetHistoryTradeCallback(const a_HistoryTradeCallback : 
THistoryTradeCallback) : Integer; stdcall; 
function SetDailyCallback(const a_DailyCallback : TDailyCallback) : Integer; stdcall; 
function SetTheoreticalPriceCallback(const a_TheoreticalPriceCallback : 
TTheoreticalPriceCallback) : Integer; stdcall; 
function SetTinyBookCallback(const a_TinyBookCallback : TTinyBookCallback) : Integer; 
stdcall; 
 function SetChangeCotationCallback(const a_ChangeCotation : TChangeCotation) :
  Integer; stdcall; 
 function SetChangeStateTickerCallback(const a_ChangeStateTicker : TChangeStateTicker) 
: Integer; stdcall; 
function SetSerieProgressCallback(const a_SerieProgressCallback : TProgressCallback) 
: Integer; stdcall; 
function SetOfferBookCallback(const a_OfferBookCallback : TOfferBookCallback) : 
Integer; stdcall; 
function SetOfferBookCallbackV2(const a_OfferBookCallbackV2 : TOfferBookCallbackV2) : 
Integer; stdcall; 
function SetPriceBookCallback(const a_PriceBookCallback : TPriceBookCallback) : 
Integer; stdcall; 
function SetPriceBookCallbackV2(const a_PriceBookCallbackV2 : TPriceBookCallbackV2) : 
Integer; stdcall; 
function SetAdjustHistoryCallback(const a_AdjustHistoryCallback : 
TAdjustHistoryCallback) : Integer; stdcall; 
function SetAdjustHistoryCallbackV2(const a_AdjustHistoryCallbackV2 : 
TAdjustHistoryCallbackV2) : Integer; stdcall; 
function SetAssetPositionListCallback(const a_AssetPositionListCallback : 
TConnectorAssetPositionListCallback) : Integer; stdcall; 
function SetAccountCallback(const a_AccountCallback : TAccountCallback) : Integer; 
stdcall; 
function SetHistoryCallback(const a_HistoryCallback : THistoryCallback) : Integer; 
stdcall; 
function SetHistoryCallbackV2(const a_HistoryCallbackV2 : THistoryCallbackV2) : 
Integer; stdcall; 
function SetOrderChangeCallback(const a_OrderChangeCallback : TOrderChangeCallback) : 
Integer; stdcall; 
function SetOrderChangeCallbackV2(const a_OrderChangeCallbackV2 : 
TOrderChangeCallbackV2) : Integer; stdcall; 
function SetOrderCallback(const a_OrderCallback : TConnectorOrderCallback) : Integer; 
stdcall; 
function SetOrderHistoryCallback(const a_OrderHistoryCallback : 
TConnectorAccountCallback) : NResult; stdcall; 
function SetTradeCallbackV2(const a_TradeCallbackV2 : TConnectorTradeCallback) : 
NResult; stdcall; 
function SetHistoryTradeCallbackV2(const a_HistoryTradeCallbackV2 : 
TConnectorTradeCallback) : NResult; stdcall; 
```

### DLLInitializeLogin

| Nome                | Tipo                  | Descrição                                         |
|---------------------|-----------------------|---------------------------------------------------|
| const pwcActivationKey    | PWideChar             | Chave de ativação fornecida para login            |
| const pwcUser             | PWideChar             | Usuário para login da conta correspondente à chave de ativação |
| const pwcPassword         | PWideChar             | Senha de login                                    |
| StateCallback       | TStateCallback        | Callback de estado da conexão                     |
| HistoryCallback     | THistoryCallback      | Callback de histórico de ordens                    |
| OrderChangeCallback | TOrderChangeCallback  | Callback de mudança no estado de uma ordem         |
| AccountCallback     | TAccountCallback      | Callback de informações da conta de roteamento     |
| NewTradeCallback    | TNewTradeCallback     | Callback de trades em tempo real                   |
| NewDailyCallback    | TNewDailyCallback     | Callback de dados diários agregados                |
| PriceBookCallback   | TPriceBookCallback    | Callback de informações do livro de preços         |
| OfferBookCallback   | TOfferBookCallback    | Callback de informações do livro de ofertas        |
| HistoryTradeCallback| THistoryTradeCallback | Callback de dados de histórico de trades           |
| ProgressCallback    | TProgressCallback     | Callback de progresso de alguma requisição de histórico |
| TinyBookCallback    | TTinyBookCallback     | Callback de topo de livro                          |

Função de inicialização dos serviços de Market Data e Roteamento da DLL. Ela irá inicializar conexão com todos  servidores e criar os serviços necessários para comunicação. Outras funções podem retornar o status de erro NL_ERR_INIT caso DLLInitializeLogin não seja bem sucedida.

### DLLInitializeMarketLogin

| Nome                | Tipo                  | Descrição                                         |
|---------------------|-----------------------|---------------------------------------------------|
| pwcActivationKey    | PWideChar             | Chave de ativação fornecida para login            |
| pwcUser             | PWideChar             | Usuário para login da conta correspondente à chave de ativação |
| pwcPassword         | PWideChar             | Senha de login                                    |
| StateCallback       | TStateCallback        | Callback de estado da conexão                     |
| NewTradeCallback    | TNewTradeCallback     | Callback de trades em tempo real                   |
| NewDailyCallback    | TNewDailyCallback     | Callback de dados diários agregados                |
| PriceBookCallback   | TPriceBookCallback    | Callback de informações do livro de preços         |
| OfferBookCallback   | TOfferBookCallback    | Callback de informações do livro de ofertas        |
| HistoryTradeCallback| THistoryTradeCallback | Callback de dados de histórico de trades           |
| ProgressCallback    | TProgressCallback     | Callback de progresso de alguma requisição de histórico |
| TinyBookCallback    | TTinyBookCallback     | Callback de topo de livro                          |

Equivalente à função DLLInitializeLogin, porém inicializa somente serviços de Market Data.

### DLLFinalize

Função utilizada para finalização dos serviços da DLL.

### SetServerAndPort

| Nome                | Tipo                  | Descrição                                         |
|---------------------|-----------------------|---------------------------------------------------|
| const strServer     | Double                | Endereço do servidor de Market Data               |
| const strPort       | Integer               |  Porta do servidor de Market Data                 |

É usado para conectar em servidores específicos do Market Data, precisa ser chamado antes da inicialização (DLLInitialize ou InitializeMarket).

*Importante:* apenas utilizar essa função com orientação da equipe de desenvolvimento, a DLL funciona da melhor maneira escolhendo os servidores internamente.

### GetServerClock

| Nome         | Tipo    | Descrição                |
|--------------|---------|--------------------------|
| dtDate       | Double  | Data codificada como Double |
| nYear        | Integer | Ano                      |
| nMonth       | Integer | Mês                      |
| nDay         | Integer | Dia                      |
| nHour        | Integer | Hora                     |
| nMin         | Integer | Minuto                   |
| nSec         | Integer | Segundo                  |
| nMilisec     | Integer | Milissegundo             |

Retorna o horário do servidor de Market Data, pode ser chamado somente após inicialização. O parâmetro dtDate corresponde a uma referência para Double que segue o padrão [TDateTime do Delphi](descrito em http://docwiki.embarcadero.com/Libraries/Sydney/en/System.TDateTime). Os outros parâmetros também são passados por referência ao caller e somente representam os valores de data calendário do valor codificado no parâmetro dtDate.

### GetLastDailyClose

| Nome             | Tipo      | Descrição                                                        |
|------------------|-----------|------------------------------------------------------------------|
| const pwcTicker  | PWideChar | Ticker do ativo, por exemplo "PETR4"                             |
| const pwcBolsa   | PWideChar | Código da bolsa, por exemplo "B" para Bovespa                    |
| dClose           | Double*   | Ponteiro para Double onde será retornado o preço de fechamento   |
| bAdjusted        | Integer   | 1 para preço ajustado, 0 para preço não ajustado                 |

Retorna o preço de fechamento do ativo no último pregão. O valor é retornado por referência em `dClose`. O parâmetro `bAdjusted` indica se o valor retornado deve ser ajustado por proventos (1) ou não (0).

A função retorna o valor do fechamento (dClose) do candle anterior ao dia atual, de acordo com o parâmetro bAdjusted. Se bAdjusted for 0, o valor não ajustado é retornado; caso contrário, o valor ajustado é retornado. Para que a função retorne NL_OK com dados, é necessário que SubscribeTicker tenha sido previamente chamada para o mesmo ativo. Na primeira chamada da função, os dados são requisitados ao servidor e a função retorna NL_WAITING_SERVER. Todas as chamadas subsequentes para o mesmo ativo retornam os dados já carregados. Ativos inválidos retornam NL_ERR_INVALID_ARGS. Se os dados da série diária ou ajustes não estiverem previamente carregados, essa chamada irá carregá-los e, consequentemente, disparar os callbacks progressCallback e adjustHistoryCallback.

### SubscribeTicker

| Nome                | Tipo      | Descrição                |
|---------------------|-----------|--------------------------|
| const pwcTicker     | PWideChar | Ticker do ativo          |
| const pwcBolsa      | PWideChar | Bolsa do ativo           |

É usado para receber as cotações em tempo real de determinado ativo. As informações são recebidas posteriormente à inscrição assim que disponíveis pelo callback especificado no parâmetro NewTradeCallback da função de inicialização. Em caso de requisção de ticker inválido, um evento vai ser disparado na callback definida SetInvalidTickerCallback. UnsubscribeTicker desativa este serviço.

### UnsubscribeTicker

| Nome           | Tipo      | Descrição       |
|----------------|-----------|-----------------|
| pwcTicker      | PWideChar | Ticker do ativo |
| pwcBolsa       | PWideChar | Bolsa do ativo  |

Solicita ao serviço de Market Data que interrompa o envio de cotações em tempo real de um determinado ativo.

### SubscribeOfferBook

| Nome                | Tipo      | Descrição       |
|---------------------|-----------|-----------------|
| const pwcTicker     | PWideChar | Ticker do ativo |
| const pwcBolsa      | PWideChar | Bolsa do ativo  |

É usado para receber informações do livro de ofertas em tempo real. As informações são recebidas posteriormente à inscrição assim que disponíveis pelo callback especificado no parâmetro OfferBookCallback da função de inicialização. Em caso de requisção de ticker inválido, um evento vai ser disparado na callback definida SetInvalidTickerCallback. UnsubscribeOfferBook desativa esse serviço.

### UnsubscribeOfferBook

| Nome                | Tipo      | Descrição       |
|---------------------|-----------|-----------------|
| const pwcTicker     | PWideChar | Ticker do ativo |
| const pwcBolsa      | PWideChar | Bolsa do ativo  |

Solicita ao serviço de Market Data que interrompa o envio em tempo real do livro de ofertas de um determinado ativo.




### SubscribePriceBook

| Nome           | Tipo      | Descrição       |
|----------------|-----------|-----------------|
| const pwcTicker| PWideChar | Ticker do ativo |
| const pwcBolsa | PWideChar | Bolsa do ativo  |

É usado para receber informações do livro de preços em tempo real. As informações são recebidas posteriormente à inscrição assim que disponíveis pelo callback especificado no parâmetro PriceBookCallback da função de inicialização. Em caso de requisção de ticker inválido, um evento vai ser disparado na callback definida SetInvalidTickerCallback. UnsubscribePriceBook desativa esse serviço.

### UnsubscribePriceBook

| Nome           | Tipo      | Descrição       |
|----------------|-----------|-----------------|
| const pwcTicker| PWideChar | Ticker do ativo |
| const pwcBolsa | PWideChar | Bolsa do ativo  |

Solicita ao serviço de Market Data que interrompa o envio do livro de preços em tempo real de um determinado ativo.
As chamadas de Subscribe e Unsubscribe SubscribeTicker, UnsubscribeTicker, SubscribePriceBook, UnsubscribePriceBook, SubscribeOfferBook, UnsubscribeOfferBook recebe os seus parâmetros no seguinte padrão:
Ticker: PETR4, Bolsa: B Ticker: WINFUT, Bolsa: F
Mais exemplos de bolsas podem ser encontradas na seção de declarações.

### SubscribeAdjustHistory

| Nome           | Tipo      | Descrição       |
|----------------|-----------|-----------------|
| const pwcTicker| PWideChar | Ticker do ativo |
| const pwcBolsa | PWideChar | Bolsa do ativo  |

É utilizado para receber histórico de ajustes do ativo determinado ticker. É necessário fornecer a função de callback SetAdjustHistoryCallback ou SetAdjustHistoryCallbackV2 para utilizar esse subscribe. Em caso de requisção de ticker inválido, um evento vai ser disparado na callback definida SetInvalidTickerCallback.

### UnsubscribeAdjustHistory

| Nome           | Tipo      | Descrição       |
|----------------|-----------|-----------------|
| const pwcTicker| PWideChar | Ticker do ativo |
| const pwcBolsa | PWideChar | Bolsa do ativo  |

Solicita ao serviço de Market Data que interrompa o envio de informações de ajustes de um determinado ativo.

### GetAgentNameByID e GetAgentShortNameByID

| Nome | Tipo    | Descrição                  |
|------|---------|----------------------------|
| nID  | Integer | Identificador do agente negociante |

O valor retornado apresenta o nome completo e abreviado, respectivamente, deste agente.
Depreciada: Utilizada a função GetAgentName junto a GetAgentNameLength para buscar o nome do agente.

### GetAgentNameLength

| Nome       | Tipo     | Descrição                                      |
|------------|----------|------------------------------------------------|
| nID        | Integer  | Identificador do agente negociante              |
| nShortFlag | Cardinal | Seta a busca pelo nome completo ou abreviado    |

O valor retornado representa o tamanho do nome do agente, sendo o nome completo ou abreviado.

### GetAgentName

| Nome         | Tipo     | Descrição                                      |
|--------------|----------|------------------------------------------------|
| nAgentLength | Integer  | Tamanho da string do nome                      |
| nID          | Integer  | Identificador do agente negociante             |
| pwcAgent     | PWideChar| Ponteiro que receberá o nome do agente.        |
| nShortFlag   | Cardinal | Seta a busca pelo nome completo ou abreviado   |

O valor retornado apresenta o nome completo ou abreviado do agente de acordo com a Flag definida. É necessário enviar o tamanho da string, que pode ser adquirido pela função GetAgentNameLength

### GetHistoryTrades

| Nome         | Tipo      | Descrição                                      |
|--------------|-----------|------------------------------------------------|
| const pwcTicker | PWideChar | Ticker do ativo                             |
| const pwcBolsa  | PWideChar | Bolsa do ativo                              |
| dtDateStart     | PWideChar | Data de início da requisição no formato DD/MM/YYYY HH:mm:SS (mm minuto MM mês) |
| dtDateEnd       | PWideChar | Data do fim da requisição no formato DD/MM/YYYY HH:mm:SS (mm minuto MM mês)    |

É utilizado para solicitar as informações do histórico de um ativo a partir de uma data (pwcTicker = ‘PETR4’; dtDateStart = ‘06/08/2018 09:00:00’; dtDateEnd= ‘06/08/2018 18:00:00’). Retorno será dado na função de callback THistoryTradeCallback especificada por parâmetro para a função de inicialização. Em TProgressCallback será retornado o progresso de Download (1 até 100).

### SetDayTrade

| Nome        | Tipo    | Descrição                                      |
|-------------|---------|------------------------------------------------|
| bUseDayTrade| Integer | Indica se deve usar a flag de day trade (1 true, 0 false) |

Função disponível para clientes cujas corretoras tenham controle de risco DayTrade. Desta forma, as ordens são enviadas com a tag DayTrade. O parâmetro é um boleano (0 = False, 1 = True). Ao definí-lo como true, todas ordens serão enviadas com o modo DayTrade ativado. Para desativar, basta setar para falso.

### SetEnabledLogToDebug

| Nome     | Tipo    | Descrição                        |
|----------|---------|-----------------------------------|
| bEnabled | Integer | Indica se deve salvar logs de debug |

Função para definir uma se a DLL deve salvar logs para debug (1 = salvar / 0 = Não salvar).

### RequestTickerInfo

| Nome           | Tipo      | Descrição       |
|----------------|-----------|-----------------|
| const pwcTicker| PWideChar | Ticker do ativo |
| const pwcBolsa | PWideChar | Bolsa do ativo  |

É utilizado para buscar novas informações do ativo (ex. ISIN). A resposta é retornada nos callbacks TAssetListInfoCallback, TAssetListInfoCallbackV2 e TAssetListCallback, caso os mesmos tenham sido enviados à DLL por meio das funções SetAssetListInfoCallback, SetAssetListInfoCallbackV2 e SetAssetListCallback. Em caso de requisção de ticker inválido, um evento vai ser disparado na callback definida SetInvalidTickerCallback.


## Callbacks

Essa seção descreve como devem ser declaradas e o objetivo de cada função de callback da biblioteca.

**Importante:** Outras funções da DLL não devem ser utilizadas dentro de um callback.

Callbacks são chamados a partir da thread ***ConnectorThread*** e portanto estão em uma thread diferente da thread principal do programa do cliente.

As funções de callbacks devem ser todas declaradas com a convenção de chamadas [***stdcall***](https://en.wikipedia.org/wiki/X86_calling_conventions). Isso é válido para ambas versões, 32 e 64 bits.


### TAdjustHistoryCallback

| Nome         | Tipo         | Descrição                        |
|--------------|--------------|----------------------------------|
| rAssetID     | TAssetIDRec  | Ativo correspondente ao ajuste   |
| dValue       | Double       | Valor do ajuste                  |
| strAdjustType| PWideChar    | Tipo de ajuste *                 |
| strObserv    | PWideChar    | Observação                       |
| dtAjuste     | PWideChar    | Data do ajuste                   |
| dtDeliber    | PWideChar    | Data de deliberação              |
| dtPagamento  | PWideChar    | Data do pagamento                |
| nAffectPrice | Integer      | Indica se afeta ou não o preço   |

Corresponde ao callback de ajustes de um ativo. Para utilizar esse callback é necessário enviá-lo à DLL através da função SetAdjustHistoryCallback. Preferir utilizar a função SetAdjustHistoryCallbackV2, nela há uma descrição mais detalhada de como realizar o cálculo do ajuste.

### TAdjustHistoryCallbackV2

| Nome         | Tipo         | Descrição                                      |
|--------------|--------------|------------------------------------------------|
| rAssetID     | TAssetIDRec  | Ativo correspondente ao ajuste                 |
| dValue       | Double       | Valor do ajuste                                |
| strAdjustType| PWideChar    | Tipo de ajuste *                               |
| strObserv    | PWideChar    | Observação                                     |
| dtAjuste     | PWideChar    | Data do ajuste                                 |
| dtDeliber    | PWideChar    | Data de deliberação                            |
| dtPagamento  | PWideChar    | Data do pagamento                              |
| nFlags       | Cardinal     | Flag de soma (descrita abaixo)                 |
| dMult        | Double       | Multiplicador                                  |

Corresponde ao callback de ajustes de um ativo. Para utilizar esse callback é necessário enviá-lo à DLL através da função SetAdjustHistoryCallbackV2. nFlags é um campo de bits b0 a b31, onde o bit 0 (menos significativo) indica se o ajuste afeta o preço e o bit 1 indica se é um ajuste de Soma. dMult é o valor pré-computado que deve ser multiplicado pelo preço para realizar o ajuste, somente é utilizado caso o ajuste não seja um ajuste de soma e seja um ajuste que afeta preço, informação fornecida no campo nFlags. O valor -9999 de dMult indica que o mesmo é inválido e não deve ser utilizado. Caso o valor dMult seja inválido, utiliza-se dValue para realizar o cálculo, sendo uma subtração em caso de ajuste de soma e divisão caso contrário.
    nQtdSeller     : Integer; 
    rAssetID   : TAssetIDRec; 
    nAction    : Integer; 
    nPosition  : Integer; 
    nSide      : Integer; 
    nQtds      : Integer; 
    nCount     : Integer; 
    dPrice     : Double; 
    pArraySell : Pointer; 
    pArrayBuy  : Pointer) stdcall; 
TPriceBookCallbackV2 = procedure( 
    rAssetID   : TAssetIDRec;
     nAction    : Integer; 
    nPosition  : Integer; 
    nSide      : Integer; 
    nQtds      : Int64; 
    nCount     : Integer; 
    dPrice     : Double; 
    pArraySell : Pointer; 
    pArrayBuy  : Pointer) stdcall; 
TOfferBookCallback = procedure( 
    rAssetID    : TAssetIDRec ; 
    nAction     : Integer; 
    nPosition   : Integer; 
    Side        : Integer; 
    nQtd        : Integer; 
    nAgent      : Integer; 
    nOfferID    : Int64; 
    dPrice      : Double; 
    bHasPrice   : Char; 
    bHasQtd     : Char; 
    bHasDate    : Char; 
    bHasOfferID : Char; 
    bHasAgent   : Char; 
    pwcDate     : PWideChar; 
    pArraySell  : Pointer 
    pArrayBuy   : Pointer) stdcall; 
TOfferBookCallbackV2 = procedure( 
    rAssetID    : TAssetIDRec ; 
    nAction     : Integer; 
    nPosition   : Integer; 
    Side        : Integer; 
    nQtd        : Int64; 
    nAgent      : Integer; 
    nOfferID    : Int64; 
    dPrice      : Double; 
    bHasPrice   : Char; 
    bHasQtd     : Char; 
    bHasDate    : Char; 
    bHasOfferID : Char; 
    bHasAgent   : Char; 
    pwcDate     : PWideChar; 
    pArraySell  : Pointer 
    pArrayBuy   : Pointer) stdcall; 
TConnectorAssetPositionListCallback = procedure( 
    AccountID : TConnectorAccountIdentifier; 
    AssetID   : TConnectorAssetIdentifier; 
    EventID   : Int64) stdcall; forward; 
TAccountCallback = procedure( 
    nCorretora            : Integer; 
    CorretoraNomeCompleto : PWideChar; 
    AccountID             : PWideChar 
    NomeTitular           : PWideChar) stdcall; forward;
     TBrokerAccountListCallback = procedure( 
    BrokerID : Integer; 
    Changed  : Cardinal); stdcall; 
TBrokerSubAccountListCallback = procedure( 
    BrokerID    : Integer; 
    a_AccountID : TConnectorAccountIdentifier 
); stdcall; 
TOrderChangeCallback = procedure( 
    rAssetID    : TAssetIDRec; 
    nCorretora  : Integer; 
    nQtd        : Integer; 
    nTradedQtd  : Integer; 
    nLeavesQtd  : Integer; 
    nSide       : Integer; 
    dPrice      : Double; 
    dStopPrice  : Double; 
    dAvgPrice   : Double; 
    nProfitID   : Int64; 
    TipoOrdem   : PWideChar; 
    Conta       : PWideChar; 
    Titular     : PWideChar; 
    ClOrdID     : PWideChar; 
    Status      : PWideChar; 
    Date        : PWideChar; 
    TextMessage : PWideChar) stdcall; 
THistoryCallback = procedure( 
    rAssetID   : TAssetIDRec; 
    nCorretora : Integer; 
    nQtd       : Integer; 
    nTradedQtd : Integer; 
    nLeavesQtd : Integer; 
    nSide      : Integer; 
    dPrice     : Double; 
    dStopPrice : Double; 
    dAvgPrice  : Double; 
    nProfitID  : Int64; 
    TipoOrdem  : PWideChar; 
    Conta      : PWideChar; 
    Titular    : PWideChar; 
    ClOrdID    : PWideChar; 
    Status     : PWideChar; 
    Date       : PWideChar) stdcall; 
THistoryTradeCallback = procedure( 
    rAssetID     : TAssetIDRec; 
    pwcDate      : PWideChar; 
    nTradeNumber : Cardinal; 
    dPrice       : Double; 
    dVol         : Double; 
    nQtd         : Integer; 
    nBuyAgent    : Integer; 
    nSellAgent   : Integer; 
    nTradeType   : Integer) stdcall;
     TTinyBookCallback = procedure( 
    rAssetID : TAssetIDRec; 
    dPrice   : Double; 
    nQtd     : Integer; 
    nSide    : Integer) stdcall; 
TAssetListCallback = procedure( 
    rAssetID : TAssetIDRec; 
    pwcName  : PWideChar) stdcall; 
TAssetListInfoCallback = procedure( 
    rAssetID            : TAssetIDRec; 
    pwcName             : PWideChar; 
    pwcDescription      : PWideChar; 
    nMinOrderQtd        : Integer; 
    nMaxOrderQtd        : Integer; 
    nLote               : Integer; 
    stSecurityType      : Integer; 
    ssSecuritySubType   : Integer; 
    dMinPriceIncrement  : Double; 
    dContractMultiplier : Double; 
    strValidDate        : PWideChar; 
    strISIN             : PWideChar) stdcall; 
TAssetListInfoCallbackV2 = procedure( 
    rAssetID            : TAssetIDRec; 
    pwcName             : PWideChar; 
    pwcDescription      : PWideChar; 
    nMinOrderQtd        : Integer; 
    nMaxOrderQtd        : Integer; 
    nLote               : Integer; 
    stSecurityType      : Integer; 
    ssSecuritySubType   : Integer; 
    dMinPriceIncrement  : Double; 
    dContractMultiplier : Double; 
    strValidDate        : PWideChar; 
    strISIN             : PWideChar; 
    strSetor            : PWideChar; 
    strSubSetor         : PWideChar; 
    strSegmento         : PWideChar) stdcall; 
TChangeStateTicker = procedure( 
    rAssetID : TAssetIDRec; 
    pwcDate  : PWideChar; 
    nState   : Integer) stdcall; 
TInvalidTickerCallback = procedure( 
const AssetID : TConnectorAssetIdentifier 
) stdcall; 
TAdjustHistoryCallback = procedure( 
    rAssetID      : TAssetIDRec; 
    dValue        : Double; 
    strAdjustType : PWideChar; 
    strObserv     : PWideChar;
      dtAjuste      : PWideChar; 
    dtDeliber     : PWideChar; 
    dtPagamento   : PWideChar; 
    nAffectPrice  : Integer) stdcall; 
TAdjustHistoryCallbackV2 = procedure( 
    rAssetID      : TAssetIDRec; 
    dValue        : Double; 
    strAdjustType : PwideChar; 
    strObserv     : PwideChar; 
    dtAjuste      : PwideChar; 
    dtDeliber     : PwideChar; 
    dtPagamento   : PwideChar; 
    nFlags        : Cardinal; 
    dMult         : Double) stdcall; 
TTheoreticalPriceCallback = procedure( 
    rAssetID          : TAssetIDRec; 
    dTheoreticalPrice : Double; 
    nTheoreticalQtd   : Int64) stdcall; 
TChangeCotation = procedure( 
    rAssetID     : TAssetIDRec; 
    pwcDate      : PWideChar; 
    nTradeNumber : Cardinal; 
    dPrice       : Double) stdcall; 
THistoryCallbackV2 = procedure( 
    rAssetID     : TAssetIDRec; 
    nCorretora   : Integer; 
    nQtd         : Integer; 
    nTradedQtd   : Integer; 
    nLeavesQtd   : Integer; 
    nSide        : Integer; 
    nValidity    : Integer; 
    dPrice       : Double; 
    dStopPrice   : Double; 
    dAvgPrice    : Double; 
    nProfitID    : Int64; 
    TipoOrdem    : PWideChar; 
    Conta        : PWideChar; 
    Titular      : PWideChar; 
    ClOrdID      : PWideChar; 
    Status       : PWideChar; 
    LastUpdate   : PWideChar; 
    CloseDate    : PWideChar; 
    ValidityDate : PWideChar) stdcall; 
TOrderChangeCallbackV2 = procedure( 
    rAssetID     : TAssetIDRec; 
    nCorretora   : Integer; 
    nQtd         : Integer; 
    nTradedQtd   : Integer; 
    nLeavesQtd   : Integer; 
    nSide        : Integer; 
    nValidity    : Integer; 
    dPrice       : Double; 
    dStopPrice   : Double; 
    dAvgPrice    : Double; 
    nProfitID    : Int64; 
    TipoOrdem    : PWideChar; 
    Conta        : PWideChar; 
    Titular      : PWideChar; 
    ClOrdID      : PWideChar; 
    Status       : PWideChar; 
    LastUpdate   : PWideChar; 
    CloseDate    : PWideChar; 
    ValidityDate : PWideChar; 
    TextMessage  : PWideChar) stdcall; 
TConnectorOrderCallback = procedure( 
const a_OrderID : TConnectorOrderIdentifier 
); stdcall; 
TConnectorAccountCallback = procedure( 
const a_AccountID : TConnectorAccountIdentifier 
); stdcall; 
TConnectorTradeCallback = procedure( 
const a_Asset  : TConnectorAssetIdentifier; 
const a_pTrade : Pointer; 
const a_nFlags : Cardinal 
); stdcall; 
```


Corresponde ao callback para informar o estado de login, de conexão, de roteamento e de ativação do produto. De acordo com o tipo de ***nConnStateType*** informado, sendo eles:

```Delphi
 CONNECTION_STATE_LOGIN        = 0;  // Conexão com servidor de login 
CONNECTION_STATE_ROTEAMENTO   = 1;  // Conexão com servidor de roteamento 
CONNECTION_STATE_MARKET_DATA  = 2;  // Conexão com servidor de market data 
CONNECTION_STATE_MARKET_LOGIN = 3;  // Login com servidor market data 
LOGIN_CONNECTED    = 0;   
LOGIN_INVALID      = 1;   
LOGIN_INVALID_PASS = 2;   
LOGIN_BLOCKED_PASS = 3;   
LOGIN_EXPIRED_PASS = 4;   
// Servidor de login conectado 
// Login é inválido 
// Senha inválida 
// Senha bloqueada 
// Senha expirada 
LOGIN_UNKNOWN_ERR  = 200; // Erro interno de login 
ROTEAMENTO_DISCONNECTED        = 0; 
ROTEAMENTO_CONNECTING          = 1; 
ROTEAMENTO_CONNECTED           = 2; 
ROTEAMENTO_BROKER_DISCONNECTED = 3; 
ROTEAMENTO_BROKER_CONNECTING   = 4; 
ROTEAMENTO_BROKER_CONNECTED    = 5; 
 MARKET_DISCONNECTED = 0;  // Desconectado do servidor de market data 
MARKET_CONNECTING   = 1;  // Conectando ao servidor de market data 
MARKET_WAITING      = 2;  // Esperando conexão 
MARKET_NOT_LOGGED   = 3;  // Não logado ao servidor de market data 
MARKET_CONNECTED    = 4;  // Conectado ao market data 
CONNECTION_ACTIVATE_VALID   = 0;  // Ativação válida 
CONNECTION_ACTIVATE_INVALID = 1;  // Ativação inválida
```


Sendo o tipo ***nConnStateType*** recebido um dos valores de ***CONNECTION_STATE***, e ***Integer*** o estado de login do serviço específico.


Os valores corretos para uma conexão válida são:

* ***nConnStateType*** = ***CONNECTION_STATE_LOGIN***
***Integer*** = ***LOGIN_CONNECTED***
* ***nConnStateType*** = ***CONNECTION_STATE_ROTEAMENTO***
***Integer*** = ***ROTEAMENTO_CONNECTED***
* ***nConnStateType*** = ***CONNECTION_STATE_MARKET_DATA***
***Integer*** = ***MARKET_CONNECTED***
* ***nConnStateType*** = ***CONNECTION_STATE_MARKET_LOGIN***
***Integer*** = ***CONNECTION_ACTIVATE_VALID***

### TNewTradeCallback

### TNewTradeCallback

| Nome         | Tipo         | Descrição                                                                                   |
|--------------|--------------|---------------------------------------------------------------------------------------------|
| rAssetID     | TAssetIDRec  | Ativo ao qual o trade pertence                                                               |
| pwcDate      | PWideChar    | Data do trade no formato DD/MM/YYYY HH:mm:SS.ZZZ (mm minuto, MM mês e ZZZ milissegundo)      |
| nTradeNumber | Cardinal     | Número de série de um trade                                                                  |
| dPrice       | Double       | Preço de execução                                                                            |
| dVol         | Double       | Volume financeiro                                                                            |
| nQtd         | Integer      | Quantidade                                                                                   |
| nBuyAgent    | Integer      | Agente comprador                                                                             |
| nSellAgent   | Integer      | Agente vendedor                                                                              |
| nTradeType   | Integer      | Tipo do trade                                                                                |
| bEdit        | Char         | Indica se é uma edição                                                                       |


Corresponde ao callback para informar um novo trade, recebido após se inscrever para este mesmo ativo (segundo função ***SubscribeTicker*** já especificada). O ***nTradeNumber*** é o identificador único do trade por pregão. ***bEdit*** informa se o trade recebido é uma edição (informação da bolsa) ou uma adição. O ID para identificar um trade editado é o ***pwcDate***. ***tradeType*** indica o tipo de trade segundo a tabela abaixo:

```Delphi
1. Cross trade 
2. Compra agressão 
3. Venda agressão
4. Leilão 
5. Surveillance 
6. Expit 
7. Options Exercise 
8. Over the counter 
9. Derivative Term 
10. Index 
11. BTC 
12. On Behalf 
13. RLP 
32. Desconhecido
```

### TNewDailyCallback

| Nome             | Tipo         | Descrição                                                                                   |
|------------------|--------------|---------------------------------------------------------------------------------------------|
| rAssetID         | TAssetIDRec  | Ativo ao qual o trade pertence                                                               |
| pwcDate          | PWideChar    | Data do trade no formato DD/MM/YYYY HH:mm:SS.ZZZ (mm minuto, MM mês e ZZZ milissegundo)      |
| dOpen            | Double       | Preço do trade na abertura do mercado                                                        |
| dHigh            | Double       | Maior preço atingido                                                                         |
| dLow             | Double       | Menor preço atingido                                                                         |
| dClose           | Double       | Preço do último trade ocorrido                                                               |
| dVol             | Double       | Volume financeiro                                                                            |
| dAjuste          | Double       | Ajuste do preço                                                                              |
| dMaxLimit        | Double       | Limite superior de preço para ordem                                                          |
| dMinLimit        | Double       | Limite inferior de preço para ordem                                                          |
| dVolBuyer        | Double       | Volume de compradores                                                                        |
| dVolSeller       | Double       | Volume de vendedores                                                                         |
| nQtd             | Integer      | Quantidade                                                                                   |
| nNegocios        | Integer      | Número total de negócios ocorridos                                                           |
| nContratosOpen   | Integer      | Número de contratos abertos                                                                  |
| nQtdBuyer        | Integer      | Número de compradores                                                                        |
| nQtdSeller       | Integer      | Número de vendedores                                                                         |
| nNegBuyer        | Integer      | Número de negócios compradores                                                               |
| nNegSeller       | Integer      | Número de negócios vendedores                                                                |


Corresponde ao callback para informar uma nova cotação com informações agregadas do dia de pregão.

O campo ssSecuritySubType é uma especificação dentro do tipo e pode ser um dos abaixo:

```Delphi
** Subtipo do ativo
0.	ssFXSpot
1.	ssGold
2.	ssIndex
3.	ssInterestRate
4.	ssFXRate
5.	ssForeignDebt
6.	ssAgricultural
7.	ssEnergy
8.	ssEconomicIndicator
9.	ssStrategy
10.	ssFutureOption
11.	ssVolatility
12.	ssSwap
13.	ssMiniContract
14.	ssFinancialRollOver
15.	ssAgriculturalRollOver
16.	ssCarbonCredit
17.	ssUnknown
18.	ssFractionary
19.	ssStock
20.	ssCurrency
21.	ssOTC	// OTC MercadoBalcao
22.	ssFII	// FII Fundo de Investimento Imobiliario

// PUMA 2.0 -Equities
23.	ssOrdinaryRights	// DO
24.	ssPreferredRights	// DP
25.	ssCommonShares	// ON
26.	ssPreferredShares	// PN
27.	ssClassApreferredShares	// PNA
28.	ssClassBpreferredShares	// PNB
29.	ssClassCpreferredShares	// PNC
30.	ssClassDpreferredShares	// PND
31.	ssOrdinaryReceipts	// ON REC
32.	ssPreferredReceipts	// PN REC
33.	ssCommonForward
34.	ssFlexibleForward
35.	ssDollarForward
36.	ssIndexPointsForward
37.	ssNonTradeableETFIndex
38.	ssPredefinedCoveredSpread
39.	ssTraceableETF
40.	ssNonTradeableIndex
41.	ssUserDefinedSpread
42.	ssExchangeDefinedspread	// Não usado atualmente
43.	ssSecurityLoan
44.	ssTradeableIndex
45.	ssOthers
46.	ssBrazilianDepositaryReceipt	// BDR
47.	ssFund
48.	ssOtherReceipt
49.	ssOtherRight
50.	ssUNIT
51.	ssClassEPreferredShare	// PNE
52.	ssClassFPreferredShare	// PNF
53.	ssClassGPreferredShare	// PNG
54.	ssWarrant
55.	ssNonTradableSecurityLending
56.	ssForeignIndexETF
57.	ssGovernmentETF
58.	ssIpoOrFollowOn
59.	ssGrossAuction
60.	ssNetAuction
61.	ssTradableIndexInPartnership
62.	ssNontradableIndexInPartnership
63.	ssFixedIncomeETF
64.	ssNontradableFixedIncomeETF
65.	ssOutrightPurchase
66.	ssSpecificCollateralRepo
67.	ssDebenture
68.	ssRealStateReceivableCertificate
69.	ssAgribusinessReceivableCertificate
70.	ssPromissoryNote
71.	ssLetraFinanceira
72.	ssAmericanDepositaryReceipt
73.	ssUnitInvestmentFund
74.	ssReceivableInvestmentFund
75.	ssOutrightTPlus1
76.	ssRepoTPlus1
77.	ssNonTradableGrossSettlement
78.	ssNonTradableNetSettlement
79.	ssETFPrimaryMarket
80.	ssSharesPrimaryMarket
81.	ssRightsPrimaryMarket
82.	ssUnitPrimaryMarket
83.	ssFundPrimaryMarket
84.	ssForeignIndexETFPrimaryMarket
```


### TAssetListInfoCallbackV2

| Nome                | Tipo         | Descrição                                 |
|---------------------|--------------|-------------------------------------------|
| rAssetID            | TAssetIDRec  | Ativo ao qual a informação pertence       |
| pwcName             | PWideChar    | Nome do ativo                             |
| pwcDescription      | PWideChar    | Descrição do ativo                        |
| nMinOrderQtd        | Integer      | Mínima quantidade de ordens permitidas    |
| nMaxOrderQtd        | Integer      | Máxima quantidade de ordens permitidas    |
| nLote               | Integer      | Tamanho de um lote                        |
| stSecurityType      | Integer      | Tipo do ativo *                           |
| ssSecuritySubType   | Integer      | Subtipo do ativo **                       |
| dMinPriceIncrement  | Double       | Incremento mínimo de preço                |
| dContractMultiplier | Double       | Multiplicador do contrato                 |
| strValidDate        | PWideChar    | Data de validade caso expire              |
| strISIN             | PWideChar    | String ISIN do ativo                      |
| strSetor            | PWideChar    | Setor de atuação                          |
| strSubSetor         | PWideChar    | Subsetor dentro do setor                  |
| strSegmento         | PWideChar    | Segmento de atuação                       |


Extensão do callback ***TAssetListInfoCallback***, adicionando os campos de setor, subsetor e segmento para detalhamento adicional do ativo.

### TInvalidTickerCallback

Nome	Tipo	Descrição
AssetID	TConnectorAssetIdentifier	Ativo inválido


Corresponde ao callback para retorno de requisição do ticker inválido.

TTheoreticalPriceCallback

Nome	Tipo	Descrição
rAssetID	TAssetIDRec	Ativo ao qual a informação pertence
dTheoreticalPrice	Double	Preço teórico
nTheoreticalQtd	Int64	Quantidade teórica


Corresponde ao callback para retorno do preço e quantidades teóricas durante o leilão de um ativo.

  TAdjustHistoryCallback

Nome	Tipo	Descrição
rAssetID	TAssetIDRec	Ativo correspondente ao ajuste
dValue	Double	Valor do ajuste
strAdjustType	PWideChar	Tipo de ajuste *
strObserv	PWideChar	Observação
dtAjuste	PWideChar	Data do ajuste
dtDeliber	PWideChar	Data de deliberação
dtPagamento	PWideChar	Data do pagamento
nAffectPrice	Integer	Indica se afeta ou não o preço


Corresponde ao callback de ajustes de um ativo. Para utilizar esse callback é necessário enviá-lo à DLL através da função ***SetAdjustHistoryCallback***. Preferir utilizar a função ***SetAdjustHistoryCallbackV2***, nela há uma descrição mais detalhada de como realizar o cálculo do ajuste.

TAdjustHistoryCallbackV2


Nome	Tipo	Descrição
rAssetID	TAssetIDRec	Ativo correspondente ao ajuste
dValue	Double	Valor do ajuste
strAdjustType	PWideChar	Tipo de ajuste *
strObserv	PWideChar	Observação
dtAjuste	PWideChar	Data do ajuste
dtDeliber	PWideChar	Data de deliberação
dtPagamento	PWideChar	Data do pagamento
nFlags	Cardinal	Flag de soma (descrita abaixo)
dMult	Double	Multiplicador


Corresponde ao callback de ajustes de um ativo. Para utilizar esse callback é necessário enviá-lo à DLL através da função ***SetAdjustHistoryCallbackV2***. ***nFlags*** é um campo de bits b0 a b31, onde o bit 0 (menos significativo) indica se o ajuste afeta o preço e o bit 1 indica se é um ajuste de Soma. ***dMult*** é o valor pré-computado que deve ser multiplicado pelo preço para realizar o ajuste, somente é utilizado caso o ajuste não seja um ajuste de soma e seja um ajuste que afeta preço, informação fornecida no campo ***nFlags***. O valor -9999 de ***dMult*** indica que o mesmo é inválido e não deve ser utilizado. Caso o valor ***dMult*** seja inválido, utiliza-se ***dValue*** para realizar o cálculo, sendo uma subtração em caso de ajuste de soma e divisão caso contrário.
Para realizar o cálculo do ajuste é possível utilizar os parâmetros da seguinte forma:
- Quando ***dMult*** for um valor válido, o ajuste é feito multiplicando o preço por esse valor.
- Quando a flag de soma está setada, o valor de ajuste é subtraído do preço.
- Quando a flag de soma não está setada, o preço é dividido pelo valor de ajuste.


### TChangeCotation

| Nome         | Tipo        | Descrição                                         |
|--------------|-------------|---------------------------------------------------|
| rAssetID     | TAssetIDRec | Ativo em que ocorreu a mudança                    |
| pwcDate      | PWideChar   | Data da mudança na cotação                        |
| nTradeNumber | Cardinal    | Número sequencial do trade em que ocorreu a mudança|
| dPrice       | Double      | Preço novo                                        |


Este callback é usado para informar quando ocorrer uma modificação de preço no ativo, informando qual foi o último preço e hora negociado. Para utilizar esse callback é necessário enviá-lo à DLL através da função ***SetChangeCotationCallback***.


### TChangeStateTicker

| Nome      | Tipo        | Descrição                      |
|-----------|-------------|--------------------------------|
| rAssetID  | TAssetIDRec | Ativo em que ocorreu a mudança |
| pwcDate   | PWideChar   | Data da mudança de estado      |
| nState    | Integer     | Estado do ativo                |


Corresponde ao callback de identificação de alteração de estado do ativo. A data informada é a data em que houve modificação do estado, apenas alguns estados mostram a data. Os estados possíveis estão listados abaixo:


### THistoryCallbackV2

| Nome         | Tipo         | Descrição                                         |
|--------------|--------------|---------------------------------------------------|
| rAssetID     | TAssetIDRec  | Ativo ao qual o livro pertence                    |
| nCorretora   | Integer      | Identificador da corretora                        |
| nQtd         | Integer      | Quantidade da ordem                               |
| nTradedQtd   | Integer      | Quantidade já executada                           |
| nLeavesQtd   | Integer      | Quantidade pendente de execução                   |
| nSide        | Integer      | Lado da ordem (Compra=1, Venda=2)                 |
| nValidity    | Integer      | Tipo de validade da ordem*                        |
| dPrice       | Double       | Preço da ordem                                    |
| dStopPrice   | Double       | Preço de stop em caso de ordem stop               |
| dAvgPrice    | Double       | Média do preço executado                          |
| nProfitID    | Int64        | Identificador interno por sessão da ordem         |
| TipoOrdem    | PWideChar    | Tipo da ordem                                     |
| Conta        | PWideChar    | Identificador da conta                            |
| Titular      | PWideChar    | Titular da conta                                  |
| ClOrdID      | PWideChar    | Identificador único da ordem (permanente)         |
| Status       | PWideChar    | Status da ordem                                   |
| LastUpdate   | PWideChar    | Data da última atualização da ordem               |
| CloseDate    | PWideChar    | Data do fechamento da ordem, se já estiver fechada|
| ValidityDate | PWideChar    | Data de referência para a validade da ordem        |


Corresponde ao callback secundário (opcional) da solicitação de histórico de ordens. Para utilizar esse callback é necessário enviá-lo à DLL através da função ***SetHistoryCallbackV2***, sendo, então, chamado nas mesmas ocasiões que ***THistoryCallback*** for chamado. O histórico corresponde apenas às ordens do dia atual. O campo ***nValidity*** representa o tipo de validade da ordem retornada, que pode ser um dos valores abaixo:

### TOrderChangeCallbackV2

| Nome         | Tipo         | Descrição                                         |
|--------------|--------------|---------------------------------------------------|
| rAssetID     | TAssetIDRec  | Ativo ao qual o livro pertence                    |
| nCorretora   | Integer      | Identificador da corretora                        |
| nQtd         | Integer      | Quantidade da ordem                               |
| nTradedQtd   | Integer      | Quantidade já executada                           |
| nLeavesQtd   | Integer      | Quantidade pendente de execução                   |
| nSide        | Integer      | Lado da ordem (Compra=1, Venda=2)                 |
| nValidity    | Integer      | Tipo de validade da ordem                         |
| dPrice       | Double       | Preço da ordem                                    |
| dStopPrice   | Double       | Preço de stop em caso de ordem stop               |
| dAvgPrice    | Double       | Média do preço executado                          |
| nProfitID    | Int64        | Identificador interno por sessão da ordem         |
| TipoOrdem    | PWideChar    | Tipo da ordem                                     |
| Conta        | PWideChar    | Identificador da conta                            |
| Titular      | PWideChar    | Titular da conta                                  |
| ClOrdID      | PWideChar    | Identificador único da ordem (permanente)         |
| Status       | PWideChar    | Status da ordem                                   |
| LastUpdate   | PWideChar    | Data da última atualização da ordem               |
| CloseDate    | PWideChar    | Data do fechamento da ordem, se já estiver fechada|
| ValidityDate | PWideChar    | Data de referência para a validade da ordem        |
| TextMessage  | PWideChar    | Mensagem de informações extras                    |


Corresponde ao callback secundário (opcional) para informar as modificações de ordens enviadas por uma conta. Para utilizar esse callback é necessário enviá-lo à DLL através da função ***SetOrderChangeCallbackV2***, sendo, então, chamado nas mesmas ocasiões que ***TOrderChangeCallback*** for chamado. O campo ***nValidity*** representa o tipo de validade da ordem retornada e os valores possíveis podem ser checados na documentação de ***THistoryCallbackV2***.

### TBrokerAccountListCallback

| Nome       | Tipo     | Descrição                      |
|------------|----------|--------------------------------|
| nCorretora | Integer  | Identificador da corretora     |
| Changed    | Cardinal | Status de mudança das contas   |


Corresponde ao callback para informar as contas existentes já disponíveis para usar.

### TBrokerSubAccountListCallback

| Nome       | Tipo                        | Descrição                  |
|------------|-----------------------------|----------------------------|
| nCorretora | Integer                     | Identificador da corretora |
| AccountID  | TConnectorAccountIdentifier | Identificador da conta     |


Corresponde ao callback para informar as sub-contas existentes já disponíveis para usar.
