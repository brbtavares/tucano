//! Sistema de callbacks para eventos do ProfitDLL

use std::collections::HashMap;
use chrono::{DateTime, Utc};
#[cfg(feature = "async")]
use tokio::sync::mpsc;
use parking_lot::Mutex;

// Eventos que podem ser gerados pelos callbacks
#[derive(Debug, Clone)]
pub enum CallbackEvent {
    StateChanged {
        connection_type: ConnectionState,
        result: i32,
    },
    ProgressChanged {
        ticker: String,
        exchange: String,
        feed_type: i32,
        progress: i32,
    },
    NewTrade {
        ticker: String,
        exchange: String,
        price: rust_decimal::Decimal,
        volume: rust_decimal::Decimal,
        timestamp: DateTime<Utc>,
        buy_agent: String,
        sell_agent: String,
        trade_id: i64,
        is_edit: bool,
    },
    DailySummary {
        ticker: String,
        exchange: String,
        open: rust_decimal::Decimal,
        high: rust_decimal::Decimal,
        low: rust_decimal::Decimal,
        close: rust_decimal::Decimal,
        volume: rust_decimal::Decimal,
        adjustment: rust_decimal::Decimal,
        max_limit: rust_decimal::Decimal,
        min_limit: rust_decimal::Decimal,
        trades_buyer: rust_decimal::Decimal,
        trades_seller: rust_decimal::Decimal,
    },
    PriceBookOffer {
        ticker: String,
        exchange: String,
        action: BookAction,
        price: rust_decimal::Decimal,
        position: i32,
    },
    OfferBookBid {
        ticker: String,
        exchange: String,
        action: BookAction,
        price: rust_decimal::Decimal,
        position: i32,
    },
    AccountChanged {
        account_id: String,
        account_holder: String,
        broker_name: String,
        broker_id: i32,
    },
    InvalidTicker {
        ticker: String,
        exchange: String,
        feed_type: i32,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum ConnectionState {
    Login = 0,
    Routing = 1,
    MarketData = 2,
    MarketLogin = 3,
}

#[derive(Debug, Clone, Copy)]
pub enum BookAction {
    New = 0,
    Edit = 1,
    Delete = 2,
}

// Handler de eventos usando async channels
#[cfg(feature = "async")]
pub struct ChannelEventHandler {
    sender: tokio::sync::mpsc::UnboundedSender<CallbackEvent>,
}

#[cfg(feature = "async")]
impl ChannelEventHandler {
    pub fn new() -> (Self, tokio::sync::mpsc::UnboundedReceiver<CallbackEvent>) {
        let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();
        (Self { sender }, receiver)
    }

    fn handle_event(&self, event: CallbackEvent) {
        if let Err(_) = self.sender.send(event) {
            tracing::error!("Failed to send callback event - receiver dropped");
        }
    }
}

pub trait EventHandler: Send + Sync {
    fn handle_event(&self, event: CallbackEvent);
}

#[cfg(feature = "async")]
impl EventHandler for ChannelEventHandler {
    fn handle_event(&self, event: CallbackEvent) {
        self.handle_event(event);
    }
}

// Gerenciador global de callbacks
struct CallbackManager {
    handlers: Mutex<Vec<Box<dyn EventHandler>>>,
}

impl CallbackManager {
    fn new() -> Self {
        Self {
            handlers: Mutex::new(Vec::new()),
        }
    }

    fn add_handler(&self, handler: Box<dyn EventHandler>) {
        self.handlers.lock().push(handler);
    }

    fn handle_event(&self, event: CallbackEvent) {
        let handlers = self.handlers.lock();
        for handler in handlers.iter() {
            handler.handle_event(event.clone());
        }
    }
}

// Instância global do callback manager
static CALLBACK_MANAGER: once_cell::sync::Lazy<CallbackManager> =
    once_cell::sync::Lazy::new(|| CallbackManager::new());

fn with_callback_manager<F>(f: F) 
where
    F: FnOnce(&CallbackManager),
{
    f(&CALLBACK_MANAGER);
}

pub fn add_event_handler(handler: Box<dyn EventHandler>) {
    CALLBACK_MANAGER.add_handler(handler);
}

// Funções callback específicas do Windows
#[cfg(windows)]
mod windows_callbacks {
    use super::*;

    #[no_mangle]
    pub extern "stdcall" fn state_callback(conn_state_type: i32, result: i32) {
        let connection_type = match conn_state_type {
            0 => ConnectionState::Login,
            1 => ConnectionState::Routing,
            2 => ConnectionState::MarketData,
            3 => ConnectionState::MarketLogin,
            _ => {
                tracing::warn!("Unknown connection state type: {}", conn_state_type);
                return;
            }
        };

        let event = CallbackEvent::StateChanged {
            connection_type,
            result,
        };

        with_callback_manager(|manager| manager.handle_event(event));
    }

    #[no_mangle]
    pub extern "stdcall" fn progress_callback(
        ticker: *const u16,
        exchange: *const u16,
        feed_type: i32,
        progress: i32,
    ) {
        let event = CallbackEvent::ProgressChanged {
            ticker: unsafe { wide_ptr_to_string(ticker) },
            exchange: unsafe { wide_ptr_to_string(exchange) },
            feed_type,
            progress,
        };

        with_callback_manager(|manager| manager.handle_event(event));
    }

    // Função utilitária para conversão de string wide (UTF-16)
    unsafe fn wide_ptr_to_string(ptr: *const u16) -> String {
        if ptr.is_null() {
            return String::new();
        }
        
        let mut len = 0;
        while *ptr.add(len) != 0 {
            len += 1;
        }
        
        let slice = std::slice::from_raw_parts(ptr, len);
        String::from_utf16_lossy(slice)
    }
}

// Funções stub para outras plataformas  
#[cfg(not(windows))]
mod stub_callbacks {
    use super::*;
    
    // Para compilação em outras plataformas, sem funcionalidade real
    pub fn state_callback(_conn_state_type: i32, _result: i32) {
        // Stub implementation
    }

    pub fn progress_callback(
        _ticker: *const u16,
        _exchange: *const u16,
        _feed_type: i32,
        _progress: i32,
    ) {
        // Stub implementation
    }
}

// Funções utilitárias para conversão de strings Windows
#[cfg(not(windows))]
unsafe fn wide_ptr_to_string(_ptr: *const u16) -> String {
    String::new()  // Stub implementation
}
