/// Wrapper Rust para DLL da corretora brasileira
/// 
/// Este módulo abstrai as chamadas da DLL C/C++ fornecida pela corretora,
/// transformando-as em interfaces Rust seguras e assíncronas.

use crate::client::corretora_brasileira::types::*;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use std::{
    ffi::{CStr, CString},
    os::raw::{c_char, c_int, c_double, c_longlong},
    ptr,
    sync::Arc,
};
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};

// Definições das funções da DLL
extern "C" {
    // Funções de conexão
    fn dll_connect(login: *const c_char, password: *const c_char, server: *const c_char) -> c_int;
    fn dll_disconnect() -> c_int;
    fn dll_is_connected() -> c_int;
    
    // Funções de conta
    fn dll_get_balance_count() -> c_int;
    fn dll_get_balance(index: c_int, asset: *mut c_char, balance: *mut c_double) -> c_int;
    
    // Funções de ordens
    fn dll_place_order(
        symbol: *const c_char,
        side: c_int,
        quantity: c_double,
        price: c_double,
        order_type: c_int,
        order_id: *mut c_longlong
    ) -> c_int;
    
    fn dll_cancel_order(order_id: c_longlong) -> c_int;
    fn dll_get_open_orders_count() -> c_int;
    fn dll_get_open_order(
        index: c_int,
        order_id: *mut c_longlong,
        symbol: *mut c_char,
        side: *mut c_int,
        quantity: *mut c_double,
        price: *mut c_double
    ) -> c_int;
    
    // Funções de trades
    fn dll_get_trades_count(timestamp_since: c_longlong) -> c_int;
    fn dll_get_trade(
        index: c_int,
        trade_id: *mut c_longlong,
        symbol: *mut c_char,
        side: *mut c_int,
        quantity: *mut c_double,
        price: *mut c_double,
        timestamp: *mut c_longlong
    ) -> c_int;
    
    // Funções de callback (para streaming)
    fn dll_set_order_callback(callback: extern "C" fn(order_data: *const c_char));
    fn dll_set_balance_callback(callback: extern "C" fn(balance_data: *const c_char));
}

/// Resultado de operações da DLL
pub type DllResult<T> = Result<T, DllError>;

/// Erros específicos da DLL
#[derive(Debug, thiserror::Error)]
pub enum DllError {
    #[error("Connection failed: {0}")]
    Connection(String),
    
    #[error("Authentication failed")]
    Authentication,
    
    #[error("API error: {0}")]
    Api(String),
    
    #[error("Invalid data: {0}")]
    InvalidData(String),
    
    #[error("Network error: {0}")]
    Network(String),
    
    #[error("DLL internal error: {0}")]
    Internal(String),
}

/// Wrapper principal para a DLL da corretora
#[derive(Debug)]
pub struct CorretoraDll {
    connected: bool,
    last_error: Option<String>,
}

impl CorretoraDll {
    /// Cria nova instância do wrapper DLL
    pub fn new() -> DllResult<Self> {
        info!("Initializing corretora DLL wrapper");
        
        Ok(Self {
            connected: false,
            last_error: None,
        })
    }
    
    /// Conecta à corretora via DLL
    pub async fn connect(
        &mut self,
        login: &str,
        password: &str,
        server: &str,
    ) -> DllResult<bool> {
        debug!("Connecting to corretora: server={}", server);
        
        let c_login = CString::new(login)
            .map_err(|e| DllError::InvalidData(format!("Invalid login: {}", e)))?;
        let c_password = CString::new(password)
            .map_err(|e| DllError::InvalidData(format!("Invalid password: {}", e)))?;
        let c_server = CString::new(server)
            .map_err(|e| DllError::InvalidData(format!("Invalid server: {}", e)))?;
        
        // Executa em thread separada para não bloquear o runtime async
        let result = tokio::task::spawn_blocking(move || {
            unsafe {
                dll_connect(
                    c_login.as_ptr(),
                    c_password.as_ptr(),
                    c_server.as_ptr()
                )
            }
        }).await.map_err(|e| DllError::Internal(format!("Task join error: {}", e)))?;
        
        match result {
            0 => {
                self.connected = true;
                info!("Successfully connected to corretora");
                Ok(true)
            }
            1 => {
                self.last_error = Some("Invalid credentials".to_string());
                Err(DllError::Authentication)
            }
            2 => {
                self.last_error = Some("Network connection failed".to_string());
                Err(DllError::Network("Failed to connect to server".to_string()))
            }
            code => {
                let error_msg = format!("Unknown error code: {}", code);
                self.last_error = Some(error_msg.clone());
                Err(DllError::Internal(error_msg))
            }
        }
    }
    
    /// Desconecta da corretora
    pub async fn disconnect(&mut self) -> DllResult<()> {
        if !self.connected {
            return Ok(());
        }
        
        debug!("Disconnecting from corretora");
        
        let result = tokio::task::spawn_blocking(|| {
            unsafe { dll_disconnect() }
        }).await.map_err(|e| DllError::Internal(format!("Task join error: {}", e)))?;
        
        if result == 0 {
            self.connected = false;
            info!("Successfully disconnected from corretora");
            Ok(())
        } else {
            Err(DllError::Internal(format!("Disconnect failed with code: {}", result)))
        }
    }
    
    /// Verifica se está conectado
    pub async fn is_connected(&self) -> bool {
        if !self.connected {
            return false;
        }
        
        // Verifica status na DLL
        let result = tokio::task::spawn_blocking(|| {
            unsafe { dll_is_connected() }
        }).await.unwrap_or(0);
        
        result == 1
    }
    
    /// Busca balances da conta
    pub async fn get_account_balances(&self) -> DllResult<Vec<DllBalance>> {
        if !self.connected {
            return Err(DllError::Connection("Not connected".to_string()));
        }
        
        debug!("Fetching account balances from DLL");
        
        tokio::task::spawn_blocking(|| {
            let count = unsafe { dll_get_balance_count() };
            
            if count < 0 {
                return Err(DllError::Api("Failed to get balance count".to_string()));
            }
            
            let mut balances = Vec::with_capacity(count as usize);
            
            for i in 0..count {
                let mut asset_buffer = vec![0u8; 32]; // Buffer para asset symbol
                let mut balance_value: c_double = 0.0;
                
                let result = unsafe {
                    dll_get_balance(
                        i,
                        asset_buffer.as_mut_ptr() as *mut c_char,
                        &mut balance_value
                    )
                };
                
                if result == 0 {
                    // Converte C string para Rust string
                    let asset_cstr = unsafe { CStr::from_ptr(asset_buffer.as_ptr() as *const c_char) };
                    let asset = asset_cstr.to_string_lossy().to_string();
                    
                    let balance = DllBalance {
                        asset,
                        balance: Decimal::from_f64_retain(balance_value)
                            .unwrap_or(Decimal::ZERO),
                        timestamp: Utc::now(),
                    };
                    
                    balances.push(balance);
                } else {
                    warn!("Failed to get balance for index {}: error code {}", i, result);
                }
            }
            
            Ok(balances)
        }).await.map_err(|e| DllError::Internal(format!("Task join error: {}", e)))?
    }
    
    /// Coloca uma ordem
    pub async fn place_order(&self, order: DllOrderRequest) -> DllResult<DllOrderResponse> {
        if !self.connected {
            return Err(DllError::Connection("Not connected".to_string()));
        }
        
        debug!("Placing order via DLL: {:?}", order);
        
        let c_symbol = CString::new(order.symbol.clone())
            .map_err(|e| DllError::InvalidData(format!("Invalid symbol: {}", e)))?;
        
        tokio::task::spawn_blocking(move || {
            let mut order_id: c_longlong = 0;
            
            let result = unsafe {
                dll_place_order(
                    c_symbol.as_ptr(),
                    order.side as c_int,
                    order.quantity.to_f64().unwrap_or(0.0),
                    order.price.map(|p| p.to_f64().unwrap_or(0.0)).unwrap_or(0.0),
                    order.order_type as c_int,
                    &mut order_id
                )
            };
            
            match result {
                0 => {
                    info!("Order placed successfully: ID {}", order_id);
                    Ok(DllOrderResponse {
                        order_id: order_id.to_string(),
                        timestamp: Utc::now(),
                        status: DllOrderStatus::Submitted,
                    })
                }
                1 => Err(DllError::Api("Insufficient funds".to_string())),
                2 => Err(DllError::Api("Invalid symbol".to_string())),
                3 => Err(DllError::Api("Invalid quantity".to_string())),
                4 => Err(DllError::Api("Invalid price".to_string())),
                5 => Err(DllError::Api("Market closed".to_string())),
                code => Err(DllError::Api(format!("Unknown error code: {}", code))),
            }
        }).await.map_err(|e| DllError::Internal(format!("Task join error: {}", e)))?
    }
    
    /// Busca ordens abertas
    pub async fn get_open_orders(&self) -> DllResult<Vec<DllOrder>> {
        if !self.connected {
            return Err(DllError::Connection("Not connected".to_string()));
        }
        
        debug!("Fetching open orders from DLL");
        
        tokio::task::spawn_blocking(|| {
            let count = unsafe { dll_get_open_orders_count() };
            
            if count < 0 {
                return Err(DllError::Api("Failed to get open orders count".to_string()));
            }
            
            let mut orders = Vec::with_capacity(count as usize);
            
            for i in 0..count {
                let mut order_id: c_longlong = 0;
                let mut symbol_buffer = vec![0u8; 32];
                let mut side: c_int = 0;
                let mut quantity: c_double = 0.0;
                let mut price: c_double = 0.0;
                
                let result = unsafe {
                    dll_get_open_order(
                        i,
                        &mut order_id,
                        symbol_buffer.as_mut_ptr() as *mut c_char,
                        &mut side,
                        &mut quantity,
                        &mut price
                    )
                };
                
                if result == 0 {
                    let symbol_cstr = unsafe { 
                        CStr::from_ptr(symbol_buffer.as_ptr() as *const c_char) 
                    };
                    let symbol = symbol_cstr.to_string_lossy().to_string();
                    
                    let order = DllOrder {
                        order_id: order_id.to_string(),
                        symbol,
                        side: match side {
                            0 => DllSide::Buy,
                            1 => DllSide::Sell,
                            _ => DllSide::Buy, // Default fallback
                        },
                        quantity: Decimal::from_f64_retain(quantity)
                            .unwrap_or(Decimal::ZERO),
                        price: Some(Decimal::from_f64_retain(price)
                            .unwrap_or(Decimal::ZERO)),
                        status: DllOrderStatus::Open,
                        timestamp: Utc::now(), // DLL não fornece timestamp histórico
                    };
                    
                    orders.push(order);
                } else {
                    warn!("Failed to get order for index {}: error code {}", i, result);
                }
            }
            
            Ok(orders)
        }).await.map_err(|e| DllError::Internal(format!("Task join error: {}", e)))?
    }
    
    /// Busca trades desde um timestamp
    pub async fn get_trades_since(&self, since: DateTime<Utc>) -> DllResult<Vec<DllTrade>> {
        if !self.connected {
            return Err(DllError::Connection("Not connected".to_string()));
        }
        
        debug!("Fetching trades since {} from DLL", since);
        
        let timestamp_since = since.timestamp();
        
        tokio::task::spawn_blocking(move || {
            let count = unsafe { dll_get_trades_count(timestamp_since) };
            
            if count < 0 {
                return Err(DllError::Api("Failed to get trades count".to_string()));
            }
            
            let mut trades = Vec::with_capacity(count as usize);
            
            for i in 0..count {
                let mut trade_id: c_longlong = 0;
                let mut symbol_buffer = vec![0u8; 32];
                let mut side: c_int = 0;
                let mut quantity: c_double = 0.0;
                let mut price: c_double = 0.0;
                let mut timestamp: c_longlong = 0;
                
                let result = unsafe {
                    dll_get_trade(
                        i,
                        &mut trade_id,
                        symbol_buffer.as_mut_ptr() as *mut c_char,
                        &mut side,
                        &mut quantity,
                        &mut price,
                        &mut timestamp
                    )
                };
                
                if result == 0 {
                    let symbol_cstr = unsafe { 
                        CStr::from_ptr(symbol_buffer.as_ptr() as *const c_char) 
                    };
                    let symbol = symbol_cstr.to_string_lossy().to_string();
                    
                    let trade = DllTrade {
                        trade_id: trade_id.to_string(),
                        symbol,
                        side: match side {
                            0 => DllSide::Buy,
                            1 => DllSide::Sell,
                            _ => DllSide::Buy,
                        },
                        quantity: Decimal::from_f64_retain(quantity)
                            .unwrap_or(Decimal::ZERO),
                        price: Decimal::from_f64_retain(price)
                            .unwrap_or(Decimal::ZERO),
                        timestamp: DateTime::from_timestamp(timestamp, 0)
                            .unwrap_or(Utc::now()),
                    };
                    
                    trades.push(trade);
                } else {
                    warn!("Failed to get trade for index {}: error code {}", i, result);
                }
            }
            
            Ok(trades)
        }).await.map_err(|e| DllError::Internal(format!("Task join error: {}", e)))?
    }
    
    /// Cancela uma ordem
    pub async fn cancel_order(&self, order_id: &str) -> DllResult<bool> {
        if !self.connected {
            return Err(DllError::Connection("Not connected".to_string()));
        }
        
        debug!("Cancelling order {} via DLL", order_id);
        
        let order_id_num = order_id.parse::<i64>()
            .map_err(|e| DllError::InvalidData(format!("Invalid order ID: {}", e)))?;
        
        tokio::task::spawn_blocking(move || {
            let result = unsafe { dll_cancel_order(order_id_num) };
            
            match result {
                0 => {
                    info!("Order {} cancelled successfully", order_id_num);
                    Ok(true)
                }
                1 => Err(DllError::Api("Order not found".to_string())),
                2 => Err(DllError::Api("Order already filled".to_string())),
                3 => Err(DllError::Api("Order already cancelled".to_string())),
                code => Err(DllError::Api(format!("Cancel failed with code: {}", code))),
            }
        }).await.map_err(|e| DllError::Internal(format!("Task join error: {}", e)))?
    }
    
    /// Configura callbacks para streaming (opcional)
    pub fn setup_callbacks(&self) {
        debug!("Setting up DLL callbacks for real-time updates");
        
        unsafe {
            dll_set_order_callback(order_callback);
            dll_set_balance_callback(balance_callback);
        }
    }
}

impl Drop for CorretoraDll {
    fn drop(&mut self) {
        if self.connected {
            warn!("CorretoraDll dropped while still connected - forcing disconnect");
            // Não podemos usar async em Drop, então fazemos disconnect síncrono
            unsafe {
                dll_disconnect();
            }
        }
    }
}

// Callbacks C para streaming em tempo real
extern "C" fn order_callback(order_data: *const c_char) {
    if order_data.is_null() {
        return;
    }
    
    let data_str = unsafe {
        CStr::from_ptr(order_data).to_string_lossy()
    };
    
    debug!("Received order update from DLL: {}", data_str);
    
    // TODO: Parse JSON e enviar para broadcast channel
    // Exemplo: {"order_id": "123", "status": "filled", "filled_qty": 100.0}
}

extern "C" fn balance_callback(balance_data: *const c_char) {
    if balance_data.is_null() {
        return;
    }
    
    let data_str = unsafe {
        CStr::from_ptr(balance_data).to_string_lossy()
    };
    
    debug!("Received balance update from DLL: {}", data_str);
    
    // TODO: Parse JSON e enviar para broadcast channel
    // Exemplo: {"asset": "BRL", "balance": 10000.50}
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_dll_creation() {
        // Este teste só funciona se a DLL estiver disponível
        if std::env::var("SKIP_DLL_TESTS").is_ok() {
            return;
        }
        
        let result = CorretoraDll::new();
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_connection_invalid_credentials() {
        if std::env::var("SKIP_DLL_TESTS").is_ok() {
            return;
        }
        
        let mut dll = CorretoraDll::new().unwrap();
        let result = dll.connect("invalid", "invalid", "demo.test.com").await;
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DllError::Authentication));
    }
}
