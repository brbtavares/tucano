// Mini-Disclaimer: Uso educacional/experimental; sem recomendação de investimento ou afiliação; sem remuneração de terceiros; Profit/ProfitDLL © Nelógica; veja README & DISCLAIMER.
//! (Deprecated) ProfitDLL types were moved to crate `tucano-profitdll`.
//! This file remains temporarily to avoid breaking external imports; it re-exports
//! the new locations. Will be removed in a future major release.

pub use profitdll::*;

// All previous definitions re-exported; no inline code retained.

/// Estados de conexão do ProfitDLL
#[derive(Debug, Clone, Copy)]
pub enum ConnectionState {
    Login = 0,
    Routing = 1,
    MarketData = 2,
    MarketLogin = 3,
}

/// Ações no book de ofertas
#[derive(Debug, Clone, Copy)]
pub enum BookAction {
    New = 0,
    Edit = 1,
    Delete = 2,
}

/// Resultado das operações da DLL
pub type NResult = i32;

// Constantes de resultado - Error Codes
pub const NL_OK: NResult = 0;
pub const NL_INTERNAL_ERROR: NResult = -2147483647;
pub const NL_NOT_INITIALIZED: NResult = -2147483646;
pub const NL_INVALID_ARGS: NResult = -2147483645;
pub const NL_WAITING_SERVER: NResult = -2147483644;
pub const NL_NO_LOGIN: NResult = -2147483643;
pub const NL_NO_LICENSE: NResult = -2147483642;

/// Estrutura mock do ProfitConnector para desenvolvimento
///
/// NOTA: Esta é uma versão simplificada para permitir compilação
/// sem a DLL real. Em produção, seria substituída pela implementação
/// real do ProfitDLL.
#[derive(Debug)]
pub struct ProfitConnector {
    _connected: bool,
}

impl ProfitConnector {
    pub fn new(_dll_path: Option<&str>) -> Result<Self, String> {
        Ok(Self { _connected: false })
    }

    pub async fn initialize_login(
        &self,
        _activation_key: &str,
        _user: &str,
        _password: &str,
    ) -> Result<tokio::sync::mpsc::UnboundedReceiver<CallbackEvent>, String> {
        let (_sender, receiver) = tokio::sync::mpsc::unbounded_channel();
        println!("🔄 ProfitConnector: Simulando login (versão mock)");
        Ok(receiver)
    }

    pub fn subscribe_ticker(&self, ticker: &str, exchange: &str) -> Result<(), String> {
    println!("📊 ProfitConnector: Subscribing to {ticker} on {exchange}");
        Ok(())
    }

    pub fn unsubscribe_ticker(&self, ticker: &str, exchange: &str) -> Result<(), String> {
    println!("📊 ProfitConnector: Unsubscribing from {ticker} on {exchange}");
        Ok(())
    }
}

/// Identificador de ativo para ProfitDLL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetIdentifier {
    pub ticker: String,
    pub exchange: String,
}

impl AssetIdentifier {
    pub fn new(ticker: String, exchange: String) -> Self {
        Self { ticker, exchange }
    }

    pub fn bovespa(ticker: &str) -> Self {
        Self {
            ticker: ticker.to_string(),
            exchange: "B".to_string(),
        }
    }

    pub fn ticker(&self) -> &str {
        &self.ticker
    }

    pub fn exchange(&self) -> &str {
        &self.exchange
    }
}

/// Identificador de conta
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountIdentifier {
    pub account_id: String,
    pub broker: String,
}

impl AccountIdentifier {
    pub fn new(account_id: String, broker: String) -> Self {
        Self { account_id, broker }
    }
}

/// Lado da ordem
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderSide {
    Buy = 0,
    Sell = 1,
}

/// Estrutura para envio de ordens
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendOrder {
    pub asset: AssetIdentifier,
    pub account: AccountIdentifier,
    pub side: OrderSide,
    pub quantity: Decimal,
    pub price: Option<Decimal>, // None para market order
    pub validity: OrderValidity,
}

impl SendOrder {
    pub fn new_market_order(
        asset: AssetIdentifier,
        account: AccountIdentifier,
        side: OrderSide,
        quantity: Decimal,
    ) -> Self {
        Self {
            asset,
            account,
            side,
            quantity,
            price: None,
            validity: OrderValidity::ImmediateOrCancel,
        }
    }

    pub fn new_limit_order(
        asset: AssetIdentifier,
        account: AccountIdentifier,
        side: OrderSide,
        quantity: Decimal,
        price: Decimal,
    ) -> Self {
        Self {
            asset,
            account,
            side,
            quantity,
            price: Some(price),
            validity: OrderValidity::Day,
        }
    }
}

/// Validade da ordem
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderValidity {
    Day,
    GoodTillCanceled,
    ImmediateOrCancel,
    FillOrKill,
}

/// Erro do ProfitDLL
#[derive(Debug, thiserror::Error)]
pub enum ProfitError {
    #[error("Conexão falhou: {0}")]
    ConnectionFailed(String),
    #[error("Erro interno: {0}")]
    InternalError(String),
    #[error("Argumentos inválidos: {0}")]
    InvalidArgs(String),
    #[error("Não inicializado")]
    NotInitialized,
    #[error("Sem login")]
    NoLogin,
    #[error("Sem licença")]
    NoLicense,
}
