// Mini-Disclaimer: Educational/experimental use; not investment advice or affiliation; see README & DISCLAIMER.
//! Neutral abstraction layer (Linux/Windows) for use in examples.
//!
//! Purpose: allow examples (and other crates) to use the real DLL when
//! available (Windows + feature `real_dll`) or automatically fall back to
//! mock on Linux / builds without the feature, without needing scattered `#[cfg]`.

use crate::{error::ProfitError, mock, CallbackEvent, SendOrder};
use core::any::{Any, TypeId};
use std::env;
use tokio::sync::mpsc::UnboundedReceiver;

/// Credentials structure for logging into the Profit DLL.
///
/// Parameters:
/// - **activation_key**: Activation key provided by Nelógica.
/// - **user**: Username registered on the platform.
/// - **password**: User password.
///
/// See [`InitializeLogin`](../MANUAL.md#initializelogin) for details.
#[derive(Debug, Clone)]
pub struct Credentials {
    /// Activation key (**ActivationKey**)
    pub activation_key: String,
    /// Username (**User**)
    pub user: String,
    /// Password (**Password**)
    pub password: String,
}

impl Credentials {
    /// Loads credentials from standard environment variables.
    ///
    /// Variables checked (in order):
    /// - PROFIT_USER (fallback to USER)
    /// - PROFIT_PASSWORD
    /// - PROFIT_ACTIVATION_KEY (if missing, uses empty string)
    pub fn from_env() -> Result<Self, ProfitError> {
        let user = env::var("PROFIT_USER")
            .or_else(|_| env::var("USER"))
            .map_err(|_| ProfitError::ConnectionFailed("PROFIT_USER not set".into()))?;
        let password = env::var("PROFIT_PASSWORD")
            .map_err(|_| ProfitError::ConnectionFailed("PROFIT_PASSWORD not set".into()))?;
        let activation_key = env::var("PROFIT_ACTIVATION_KEY").unwrap_or_default();
        Ok(Self {
            user,
            password,
            activation_key,
        })
    }
}

/// Abstract trait for Profit backend (real DLL or mock).
///
/// Implements the official Profit DLL interface, as described in [MANUAL.md](../MANUAL.md).
/// All methods and parameters follow the original DLL's naming and semantics.
#[async_trait::async_trait]
pub trait ProfitBackend: Send + Sync + Any {
    /// Initializes login in the DLL (**InitializeLogin**).
    ///
    /// Parâmetros:
    /// - **creds**: [`Credentials`] contendo activation_key, user e password.
    ///
    /// Retorna: [`UnboundedReceiver<CallbackEvent>`] para eventos assíncronos.
    ///
    /// Erros: [`ProfitError`] conforme códigos NL_*.
    async fn initialize_login(
        &self,
        creds: &Credentials,
    ) -> Result<UnboundedReceiver<CallbackEvent>, ProfitError>;

    /// Solicita inscrição em um ticker (**SubscribeTicker**).
    ///
    /// Parâmetros:
    /// - **ticker**: Código do ativo.
    /// - **exchange**: Bolsa.
    fn subscribe_ticker(&self, ticker: &str, exchange: &str) -> Result<(), ProfitError>;

    /// Cancela inscrição em um ticker (**UnsubscribeTicker**).
    ///
    /// Parâmetros:
    /// - **ticker**: Código do ativo.
    /// - **exchange**: Bolsa.
    fn unsubscribe_ticker(&self, ticker: &str, exchange: &str) -> Result<(), ProfitError>;

    /// Envia ordem (**SendOrder**).
    ///
    /// Parâmetros:
    /// - **order**: [`SendOrder`] com todos os campos obrigatórios.
    fn send_order(&self, order: &SendOrder) -> Result<(), ProfitError>;

    /// Cancela ordem existente pelo ID (**CancelOrder**).
    ///
    /// Parâmetros:
    /// - **order_id**: Identificador da ordem.
    fn cancel_order(&self, order_id: i64) -> Result<(), ProfitError>;

    /// Altera ordem existente (**ChangeOrder**).
    ///
    /// Parâmetros:
    /// - **order_id**: Identificador da ordem.
    /// - **new_price**: Novo preço (opcional).
    /// - **new_qty**: Nova quantidade (opcional).
    fn change_order(
        &self,
        order_id: i64,
        new_price: Option<rust_decimal::Decimal>,
        new_qty: Option<rust_decimal::Decimal>,
    ) -> Result<(), ProfitError>;

    /// Solicita histórico de trades (**GetHistoryTrades**).
    ///
    /// Parâmetros:
    /// - **ticker**: Código do ativo.
    /// - **exchange**: Bolsa.
    /// - **from_ms**: Timestamp inicial (ms).
    /// - **to_ms**: Timestamp final (ms).
    fn request_history_trades(
        &self,
        ticker: &str,
        exchange: &str,
        from_ms: i64,
        to_ms: i64,
    ) -> Result<(), ProfitError>;

    /// Finaliza backend e libera recursos (opcional).
    fn shutdown(&self) {}
}

// ------------------ Implementação para o mock ------------------

#[async_trait::async_trait]
impl ProfitBackend for mock::ProfitConnector {
    async fn initialize_login(
        &self,
        creds: &Credentials,
    ) -> Result<UnboundedReceiver<CallbackEvent>, ProfitError> {
        self.initialize_login(&creds.activation_key, &creds.user, &creds.password)
            .await
    }
    fn subscribe_ticker(&self, ticker: &str, exchange: &str) -> Result<(), ProfitError> {
        self.subscribe_ticker(ticker, exchange)
    }
    fn unsubscribe_ticker(&self, ticker: &str, exchange: &str) -> Result<(), ProfitError> {
        self.unsubscribe_ticker(ticker, exchange)
    }
    fn send_order(&self, order: &SendOrder) -> Result<(), ProfitError> {
        self.send_order(order)
    }
    fn cancel_order(&self, order_id: i64) -> Result<(), ProfitError> {
        self.cancel_order(order_id)
    }
    fn change_order(
        &self,
        order_id: i64,
        new_price: Option<rust_decimal::Decimal>,
        new_qty: Option<rust_decimal::Decimal>,
    ) -> Result<(), ProfitError> {
        self.change_order(order_id, new_price, new_qty)
    }
    fn request_history_trades(
        &self,
        ticker: &str,
        exchange: &str,
        from_ms: i64,
        to_ms: i64,
    ) -> Result<(), ProfitError> {
        // passo de 1s para mock
        self.get_history_trades(ticker, exchange, from_ms, to_ms, 1_000)
    }
    fn shutdown(&self) {
        self.shutdown_all();
    }
}

// ------------------ Implementação para a DLL real ------------------

#[cfg(all(target_os = "windows", feature = "real_dll"))]
#[async_trait::async_trait]
impl ProfitBackend for crate::ffi::ProfitConnector {
    async fn initialize_login(
        &self,
        creds: &Credentials,
    ) -> Result<UnboundedReceiver<CallbackEvent>, ProfitError> {
        self.initialize_login(&creds.activation_key, &creds.user, &creds.password)
            .await
    }
    fn subscribe_ticker(&self, ticker: &str, exchange: &str) -> Result<(), ProfitError> {
        self.subscribe_ticker(ticker, exchange)
    }
    fn unsubscribe_ticker(&self, ticker: &str, exchange: &str) -> Result<(), ProfitError> {
        self.unsubscribe_ticker(ticker, exchange)
    }
    fn send_order(&self, order: &SendOrder) -> Result<(), ProfitError> {
        self.send_order(order)
    }
    fn cancel_order(&self, order_id: i64) -> Result<(), ProfitError> {
        self.cancel_order(order_id)
    }
    fn change_order(
        &self,
        order_id: i64,
        new_price: Option<rust_decimal::Decimal>,
        new_qty: Option<rust_decimal::Decimal>,
    ) -> Result<(), ProfitError> {
        self.change_order(order_id, new_price, new_qty)
    }
    fn request_history_trades(
        &self,
        ticker: &str,
        exchange: &str,
        from_ms: i64,
        to_ms: i64,
    ) -> Result<(), ProfitError> {
        self.get_history_trades(ticker, exchange, from_ms, to_ms)
    }
    fn shutdown(&self) {}
}

/// Estratégia de seleção do backend:
/// 1. Se var `PROFITDLL_FORCE_MOCK=1` -> mock.
/// 2. Senão, em Windows + feature tenta DLL real (caminho de `PROFITDLL_PATH` se definido).
/// 3. Fallback final: mock.
pub fn new_backend() -> Result<Box<dyn ProfitBackend>, ProfitError> {
    let force_mock = env::var("PROFITDLL_FORCE_MOCK")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    if force_mock {
        if env::var("PROFITDLL_DIAG")
            .map(|v| v == "1")
            .unwrap_or(false)
        {
            eprintln!("[profitdll][DIAG] new_backend: FORCE_MOCK=1 -> usando mock");
        }
        return Ok(Box::new(mock::ProfitConnector::new(None)?));
    }
    #[cfg(all(target_os = "windows", feature = "real_dll"))]
    {
        let path = env::var("PROFITDLL_PATH").ok();
        match crate::ffi::ProfitConnector::new(path.as_deref()) {
            Ok(conn) => {
                if env::var("PROFITDLL_DIAG")
                    .map(|v| v == "1")
                    .unwrap_or(false)
                {
                    eprintln!("[profitdll][DIAG] Backend real instanciado.");
                }
                return Ok(Box::new(conn));
            }
            Err(e) => {
                if env::var("PROFITDLL_STRICT")
                    .map(|v| v == "1")
                    .unwrap_or(false)
                {
                    if env::var("PROFITDLL_DIAG")
                        .map(|v| v == "1")
                        .unwrap_or(false)
                    {
                        eprintln!(
                            "[profitdll][DIAG] new_backend: STRICT=1 e falhou carregar DLL: {e}"
                        );
                    }
                    return Err(e);
                } else {
                    eprintln!("[profitdll] Falha carregando DLL real, caindo para mock: {e}");
                }
            }
        }
    }
    if env::var("PROFITDLL_DIAG")
        .map(|v| v == "1")
        .unwrap_or(false)
    {
        eprintln!(
            "[profitdll][DIAG] new_backend: retornando mock (condições para real não satisfeitas)"
        );
    }
    Ok(Box::new(mock::ProfitConnector::new(None)?))
}

/// Retorna o tipo concreto do backend para fins de logging / diagnóstico.
pub fn backend_kind(b: &dyn ProfitBackend) -> &'static str {
    if b.type_id() == TypeId::of::<mock::ProfitConnector>() {
        return "mock";
    }
    #[cfg(all(target_os = "windows", feature = "real_dll"))]
    {
        if b.type_id() == TypeId::of::<crate::ffi::ProfitConnector>() {
            return "real_dll";
        }
    }
    "unknown"
}
