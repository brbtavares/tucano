

// Minimal local definitions for B3/ProfitDLL integration
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone)]
pub struct ProfitConnector; // Stub, implement as needed

impl ProfitConnector {
    pub fn subscribe_ticker(
        &self,
        _symbol: &str,
        _market: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum CallbackEvent {
    NewTrade,
    StateChanged,
    DailySummary,
    // Add fields as needed for real event data
}

#[derive(Debug, Clone)]
pub enum ProfitError {
    ConnectionFailed(String),
    // Add other error variants as needed
}
