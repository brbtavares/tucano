// Mini-Disclaimer: Educational/experimental use; not investment advice or affiliation; see README & DISCLAIMER.
//! Event adapter for converting ProfitDLL events to Toucan AccountEvents

use crate::{balance::AssetBalance, error::AssetNameExchange, UnindexedAccountEvent};
use exchanges::temp::profitdll::{CallbackEvent, OrderSide};

use tucano_instrument::Side;

/// Convert ProfitDLL CallbackEvent to Toucan UnindexedAccountEvent
pub fn convert_callback_event(_event: CallbackEvent) -> Option<UnindexedAccountEvent> {
    None
}

/// Convert ProfitDLL OrderSide to Toucan Side
pub fn convert_order_side(side: OrderSide) -> Side {
    match side {
        OrderSide::Buy => Side::Buy,
        OrderSide::Sell => Side::Sell,
    }
}

/// Convert Toucan Side to ProfitDLL OrderSide
/*pub fn convert_to_profit_side(side: Side) -> OrderSide {
    match side {
        Side::Buy => OrderSide::Buy,
        Side::Sell => OrderSide::Sell,
    }
}*/

/// Create a balance snapshot from ProfitDLL data
pub fn create_balance_snapshot(
    _asset_data: &[(String, f64)],
) -> Vec<AssetBalance<AssetNameExchange>> {
    // Convert ProfitDLL balance data to Toucan format
    Vec::new()
}
