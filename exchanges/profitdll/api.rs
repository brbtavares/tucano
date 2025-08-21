

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
/// - **activation_key**: Activation key provided by Nelogica.
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
    /// Parameters:
    /// - **creds**: [`Credentials`] containing activation_key, user and password.
    ///
    /// Returns: [`UnboundedReceiver<CallbackEvent>`] for asynchronous events.
    ///
    /// Errors: [`ProfitError`] as per NL_* codes.
    async fn initialize_login(
        &self,
        creds: &Credentials,
    ) -> Result<UnboundedReceiver<CallbackEvent>, ProfitError>;

    /// Requests subscription to a ticker (**SubscribeTicker**).
    ///
    /// Parameters:
    /// - **ticker**: Asset code.
    /// - **exchange**: Exchange.
    fn subscribe_ticker(&self, ticker: &str, exchange: &str) -> Result<(), ProfitError>;

    /// Cancels subscription to a ticker (**UnsubscribeTicker**).
    ///
    /// Parameters:
    /// - **ticker**: Asset code.
    /// - **exchange**: Exchange.
    fn unsubscribe_ticker(&self, ticker: &str, exchange: &str) -> Result<(), ProfitError>;

    /// Sends order (**SendOrder**).
    ///
    /// Parameters:
    /// - **order**: [`SendOrder`] with all required fields.
    fn send_order(&self, order: &SendOrder) -> Result<(), ProfitError>;

    /// Cancels existing order by ID (**CancelOrder**).
    ///
    /// Parameters:
    /// - **order_id**: Order identifier.
    fn cancel_order(&self, order_id: i64) -> Result<(), ProfitError>;

    /// Changes existing order (**ChangeOrder**).
    ///
    /// Parameters:
    /// - **order_id**: Order identifier.
    /// - **new_price**: New price (optional).
    /// - **new_qty**: New quantity (optional).
    fn change_order(
        &self,
        order_id: i64,
        new_price: Option<rust_decimal::Decimal>,
        new_qty: Option<rust_decimal::Decimal>,
    ) -> Result<(), ProfitError>;

    /// Requests trade history (**GetHistoryTrades**).
    ///
    /// Parameters:
    /// - **ticker**: Asset code.
    /// - **exchange**: Exchange.
    /// - **from_ms**: Initial timestamp (ms).
    /// - **to_ms**: Final timestamp (ms).
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

// ------------------ Mock implementation ------------------

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

// ------------------ Real DLL implementation ------------------

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

/// Backend selection strategy:
/// 1. If var `PROFITDLL_FORCE_MOCK=1` -> mock.
/// 2. Otherwise, on Windows + feature tries real DLL (path from `PROFITDLL_PATH` if set).
/// 3. Final fallback: mock.
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
            "[profitdll][DIAG] new_backend: returning mock (conditions for real not satisfied)"
        );
    }
    Ok(Box::new(mock::ProfitConnector::new(None)?))
}

/// Returns the concrete backend type for logging / diagnostics purposes.
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
