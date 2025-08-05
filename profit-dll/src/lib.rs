//! ProfitDLL Rust Wrapper
//! 
//! Este crate fornece um wrapper Rust para a ProfitDLL, permitindo
//! comunicação com servidores de roteamento e market data para 
//! desenvolvimento de aplicações de trading no mercado brasileiro.
//!
//! ## Características
//! 
//! - Wrapper seguro para funções da DLL
//! - Sistema de callbacks assíncronos
//! - Tipos Rust idiomáticos para estruturas da DLL
//! - Suporte completo a trading e market data
//! - Integração com o ecossistema Toucan
//!
//! ## Exemplo de uso
//!
//! ```rust,no_run
//! use profit_dll::{ProfitConnector, AssetIdentifier, AccountIdentifier};
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let connector = ProfitConnector::new(None)?;
//! 
//! let mut events = connector.initialize_login(
//!     "sua_chave_ativacao",
//!     "usuario", 
//!     "senha"
//! ).await?;
//! 
//! // Subscrever para receber cotações
//! connector.subscribe_ticker("PETR4", "B")?;
//! 
//! // Processar eventos
//! while let Some(event) = events.recv().await {
//!     match event {
//!         CallbackEvent::NewTrade(trade) => {
//!             println!("Novo trade: {:?}", trade);
//!         }
//!         _ => {}
//!     }
//! }
//! # Ok(())
//! # }
//! ```

pub mod types;
pub mod callbacks;
pub mod connector;
pub mod error;
pub mod utils;

pub use connector::ProfitConnector;
pub use error::{ProfitError, Result};
pub use types::*;
pub use callbacks::{CallbackEvent, EventHandler};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_identifier() {
        let asset = AssetIdentifier::new("PETR4", "B", 0);
        assert_eq!(asset.ticker(), "PETR4");
        assert_eq!(asset.exchange(), "B");
        assert_eq!(asset.feed_type, 0);
    }

    #[test]
    fn test_account_identifier() {
        let account = AccountIdentifier::new(
            12345,
            "123456".to_string(),
            "sub123".to_string()
        );
        assert_eq!(account.broker_id, 12345);
        assert_eq!(account.account_id, "123456");
        assert_eq!(account.sub_account_id, "sub123");
    }

    #[test]
    fn test_send_order_creation() {
        let account = AccountIdentifier::new(1, "acc".to_string(), "sub".to_string());
        let asset = AssetIdentifier::new("PETR4", "B", 0);
        
        let market_order = SendOrder::new_market_order(
            account.clone(),
            asset.clone(),
            "password".to_string(),
            OrderSide::Buy,
            100
        );
        
        assert_eq!(market_order.order_type, OrderType::Market);
        assert_eq!(market_order.order_side, OrderSide::Buy);
        assert_eq!(market_order.quantity, 100);
        assert_eq!(market_order.price, -1.0);
    }
}
