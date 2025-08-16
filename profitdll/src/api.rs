// Mini-Disclaimer: Uso educacional/experimental; sem recomendação de investimento ou afiliação; sem remuneração de terceiros; Profit/ProfitDLL © Nelógica; veja README & DISCLAIMER.
//! Camada de abstração neutra (Linux/Windows) para uso nos exemplos.
//!
//! Objetivo: permitir que exemplos (e outros crates) utilizem a DLL real quando
//! disponível (Windows + feature `real_dll`) ou automaticamente caiam para o
//! mock em Linux / builds sem a feature, sem precisar de `#[cfg]` espalhado.

use crate::{error::ProfitError, mock, CallbackEvent, SendOrder};
use core::any::{Any, TypeId};
use std::env;
use tokio::sync::mpsc::UnboundedReceiver;

/// Credenciais de login (obtidas preferencialmente via variáveis de ambiente).
#[derive(Debug, Clone)]
pub struct Credentials {
    pub user: String,
    pub password: String,
    /// Chave de ativação / licença se for necessária em fluxo futuro. Mantida para estabilidade.
    pub activation_key: String,
}

impl Credentials {
    /// Carrega credenciais de variáveis de ambiente padrão.
    ///
    /// Variáveis consultadas (em ordem):
    /// - PROFIT_USER (fallback para USER)
    /// - PROFIT_PASSWORD
    /// - PROFIT_ACTIVATION_KEY (se ausente, usa string vazia)
    pub fn from_env() -> Result<Self, ProfitError> {
        let user = env::var("PROFIT_USER")
            .or_else(|_| env::var("USER"))
            .map_err(|_| ProfitError::ConnectionFailed("PROFIT_USER não definido".into()))?;
        let password = env::var("PROFIT_PASSWORD")
            .map_err(|_| ProfitError::ConnectionFailed("PROFIT_PASSWORD não definido".into()))?;
        let activation_key = env::var("PROFIT_ACTIVATION_KEY").unwrap_or_default();
        Ok(Self {
            user,
            password,
            activation_key,
        })
    }
}

/// Contrato mínimo para uso genérico das capacidades necessárias nos exemplos.
#[async_trait::async_trait]
pub trait ProfitBackend: Send + Sync + Any {
    async fn initialize_login(
        &self,
        creds: &Credentials,
    ) -> Result<UnboundedReceiver<CallbackEvent>, ProfitError>;
    fn subscribe_ticker(&self, ticker: &str, exchange: &str) -> Result<(), ProfitError>;
    fn unsubscribe_ticker(&self, ticker: &str, exchange: &str) -> Result<(), ProfitError>;
    fn send_order(&self, order: &SendOrder) -> Result<(), ProfitError>;
    fn cancel_order(&self, order_id: i64) -> Result<(), ProfitError>;
    fn change_order(
        &self,
        order_id: i64,
        new_price: Option<rust_decimal::Decimal>,
        new_qty: Option<rust_decimal::Decimal>,
    ) -> Result<(), ProfitError>;
    /// Solicita histórico de trades (pull). Backend mock gera imediatamente; real encaminha à DLL.
    fn request_history_trades(
        &self,
        _ticker: &str,
        _exchange: &str,
        _from_ms: i64,
        _to_ms: i64,
    ) -> Result<(), ProfitError> {
        Ok(())
    }
    /// Solicita encerramento limpo de quaisquer tarefas internas (mock generators, etc.).
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
    if env::var("PROFITDLL_FORCE_MOCK")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
    {
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
                    return Err(e);
                } else {
                    eprintln!("[profitdll] Falha carregando DLL real, caindo para mock: {e}");
                }
            }
        }
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
