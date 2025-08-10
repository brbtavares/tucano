//! B3 (Brasil Bolsa Balcão) exchange integration via ProfitDLL
//!
//! This module provides integration with the Brazilian stock exchange B3
//! through the ProfitDLL library from Nelógica.
//!
//! ## Architecture
//!
//! - **Hybrid Design**: Uses markets abstractions with B3-specific implementations
//! - **Multiple Connectors**: ProfitDLL is one of the possible connectivity providers
//! - **Future-Ready**: Easy to add other B3 APIs (official REST/WebSocket, etc.)
//! - **Asset Integration**: Works with markets::b3 asset types for proper categorization

#![allow(async_fn_in_trait)] // suppress async fn in trait warnings for this integration while refactoring

pub mod exchange;
pub mod instrument;
pub mod types;

pub use exchange::B3Exchange;
use markets::b3::{B3AssetCategory, B3AssetFactory};
use markets::profit_dll::{CallbackEvent, ProfitConnector};
use tokio::sync::mpsc;
// Re-export only required symbols (avoid wildcard causing warnings)
pub use types::B3Instrument;

/// B3 exchange connector using ProfitDLL
///
/// This is one of the possible connectivity providers for B3.
/// Future implementations could include:
/// - B3 Official REST API
/// - B3 WebSocket feeds
/// - Other third-party providers
///
/// Now integrated with markets::b3 asset types for proper categorization.
pub struct B3ProfitConnector {
    profit_connector: Option<ProfitConnector>,
    event_receiver: Option<mpsc::UnboundedReceiver<CallbackEvent>>,
}

impl std::fmt::Debug for B3ProfitConnector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("B3ProfitConnector")
            .field("connected", &self.profit_connector.is_some())
            .field("has_event_receiver", &self.event_receiver.is_some())
            .finish()
    }
}

impl B3ProfitConnector {
    pub fn new() -> Self {
        Self {
            profit_connector: None,
            event_receiver: None,
        }
    }

    /// Initialize connection to B3 via ProfitDLL
    pub async fn initialize(
        &mut self,
        activation_key: &str,
        user: &str,
        password: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let connector = ProfitConnector::new(None)?;
        let events = connector
            .initialize_login(activation_key, user, password)
            .await?;

        self.profit_connector = Some(connector);
        self.event_receiver = Some(events);

        Ok(())
    }

    /// Subscribe to market data for a specific B3 instrument
    pub fn subscribe_instrument(
        &self,
        instrument: &B3Instrument,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(connector) = &self.profit_connector {
            connector.subscribe_ticker(&instrument.symbol, &instrument.market)?;
        }
        Ok(())
    }

    /// Subscribe to market data using asset symbol
    /// Automatically detects asset type and category
    pub fn subscribe_asset(&self, symbol: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(connector) = &self.profit_connector {
            // Create asset from symbol to determine proper market
            let _asset = B3AssetFactory::from_symbol(symbol)?;

            // Subscribe with appropriate market designation
            // Most B3 assets use "B" market, but this could be extended
            connector.subscribe_ticker(symbol, "B")?;
        }
        Ok(())
    }

    /// Get asset category from symbol
    pub fn get_asset_category(&self, symbol: &str) -> Result<B3AssetCategory, String> {
    let _asset = B3AssetFactory::from_symbol(symbol)?;

        // This requires implementing a method to get category from the trait object
        // For now, we'll use pattern matching on the symbol
        if symbol.len() >= 6 && symbol.ends_with("11") && !symbol.ends_with("11B") {
            Ok(B3AssetCategory::ETF)
        } else if symbol.ends_with("11B") {
            Ok(B3AssetCategory::REIT)
        } else if symbol.len() >= 5 && symbol.len() <= 6 {
            let (letters, numbers) = symbol.split_at(4);
            if letters.chars().all(|c| c.is_alphabetic()) && numbers.chars().all(|c| c.is_numeric())
            {
                Ok(B3AssetCategory::Stock)
            } else {
                Ok(B3AssetCategory::Stock) // Default
            }
        } else {
            Ok(B3AssetCategory::Stock) // Default
        }
    }

    /// Process incoming events from ProfitDLL
    pub async fn process_events(&mut self) -> Option<B3MarketEvent> {
        if let Some(receiver) = &mut self.event_receiver {
            if let Some(event) = receiver.recv().await {
                return Some(B3MarketEvent::from_callback_event(event));
            }
        }
        None
    }
}

/// Market events from B3 via ProfitDLL
#[derive(Debug, Clone)]
pub enum B3MarketEvent {
    StateChanged {
        connection_type: String,
        result: i32,
    },
    NewTrade {
        trade: B3Trade,
    },
    DailySummary {
        instrument: B3Instrument,
        open: rust_decimal::Decimal,
        high: rust_decimal::Decimal,
        low: rust_decimal::Decimal,
        close: rust_decimal::Decimal,
        volume: rust_decimal::Decimal,
    },
    OrderBookUpdate {
        instrument: B3Instrument,
        side: B3BookSide,
        level: B3BookLevel,
    },
    AccountChanged {
        account: B3Account,
    },
    InvalidInstrument {
        instrument: B3Instrument,
    },
}

impl B3MarketEvent {
    fn from_callback_event(event: CallbackEvent) -> Self {
        match event {
            CallbackEvent::StateChanged {
                connection_type,
                result,
            } => B3MarketEvent::StateChanged {
                connection_type: format!("{:?}", connection_type),
                result,
            },
            CallbackEvent::NewTrade {
                ticker,
                exchange,
                price,
                volume,
                timestamp,
                buy_agent,
                sell_agent,
                trade_id,
                ..
            } => {
                let instrument = B3Instrument::new(ticker, exchange);
                let trade = B3Trade {
                    id: trade_id.to_string().into(),
                    instrument: instrument.clone(),
                    side: B3Side::Buy, // ProfitDLL doesn't specify side in trade events
                    quantity: volume,
                    price,
                    timestamp,
                    buyer_agent: Some(buy_agent.into()),
                    seller_agent: Some(sell_agent.into()),
                };

                B3MarketEvent::NewTrade { trade }
            }
            CallbackEvent::DailySummary {
                ticker,
                exchange,
                open,
                high,
                low,
                close,
                volume,
                ..
            } => B3MarketEvent::DailySummary {
                instrument: B3Instrument::new(ticker, exchange),
                open,
                high,
                low,
                close,
                volume,
            },
            CallbackEvent::PriceBookOffer {
                ticker,
                exchange,
                price,
                position,
                ..
            } => {
                B3MarketEvent::OrderBookUpdate {
                    instrument: B3Instrument::new(ticker, exchange),
                    side: B3BookSide::Ask,
                    level: B3BookLevel {
                        price,
                        quantity: rust_decimal::Decimal::ZERO, // ProfitDLL doesn't provide quantity
                        position,
                    },
                }
            }
            CallbackEvent::OfferBookBid {
                ticker,
                exchange,
                price,
                position,
                ..
            } => {
                B3MarketEvent::OrderBookUpdate {
                    instrument: B3Instrument::new(ticker, exchange),
                    side: B3BookSide::Bid,
                    level: B3BookLevel {
                        price,
                        quantity: rust_decimal::Decimal::ZERO, // ProfitDLL doesn't provide quantity
                        position,
                    },
                }
            }
            CallbackEvent::AccountChanged {
                account_id,
                account_holder,
                broker_name,
                broker_id,
            } => {
                let account = B3Account {
                    account_id: account_id.into(),
                    account_holder: account_holder.into(),
                    broker_name: broker_name.into(),
                    broker_id,
                    balances: Vec::new(), // Would be populated separately
                };

                B3MarketEvent::AccountChanged { account }
            }
            CallbackEvent::InvalidTicker {
                ticker, exchange, ..
            } => B3MarketEvent::InvalidInstrument {
                instrument: B3Instrument::new(ticker, exchange),
            },
            _ => {
                // Handle other events as generic state change
                B3MarketEvent::StateChanged {
                    connection_type: "unknown".to_string(),
                    result: 0,
                }
            }
        }
    }
}

/// B3 subscription types for different data feeds
#[derive(Debug, Clone)]
pub enum B3SubscriptionType {
    /// Tick-by-tick trades
    Trades,
    /// Order book depth
    OrderBook,
    /// Daily summary/candle data
    DailySummary,
    /// All data types
    All,
}

/// Future connector trait for B3 connectivity providers
///
/// This trait can be implemented by:
/// - B3ProfitConnector (current implementation)
/// - B3RestConnector (future official API)
/// - B3WebSocketConnector (future real-time feed)
/// - MockB3Connector (for testing)
pub trait B3Connector {
    type Error: std::error::Error + Send + Sync + 'static;

    /// Initialize the connection
    async fn connect(&mut self) -> Result<(), Self::Error>;

    /// Subscribe to instrument data
    async fn subscribe(
        &self,
        instrument: &B3Instrument,
        sub_type: B3SubscriptionType,
    ) -> Result<(), Self::Error>;

    /// Get next market event
    async fn next_event(&mut self) -> Option<B3MarketEvent>;

    /// Disconnect
    async fn disconnect(&mut self) -> Result<(), Self::Error>;
}

// Re-exports for convenience
pub use types::*;
