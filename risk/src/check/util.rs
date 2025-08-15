// Mini-Disclaimer: Uso educacional/experimental; sem recomendação de investimento ou afiliação; sem remuneração de terceiros; Profit/ProfitDLL © Nelógica; veja README & DISCLAIMER.
use rust_decimal::Decimal;
use std::collections::HashMap;

/// Calculate the absolute percentage difference between two decimal values.
pub fn calculate_abs_percent_difference(
    value1: Decimal,
    value2: Decimal,
) -> Result<Decimal, &'static str> {
    if value2.is_zero() {
        return Err("Cannot calculate percentage difference with zero denominator");
    }

    let diff = (value1 - value2).abs();
    Ok(diff / value2)
}

/// Calculate the notional value in quote currency.
pub fn calculate_quote_notional(
    quantity: Decimal,
    price: Decimal,
    contract_size: Decimal,
) -> Result<Decimal, &'static str> {
    let notional = quantity * price * contract_size;
    Ok(notional)
}

/// Utility function to calculate the maximum position size for a given instrument.
pub fn calculate_max_position_size<K, V>(positions: &HashMap<K, V>, instrument: &K) -> V
where
    K: std::hash::Hash + Eq,
    V: Clone + Default,
{
    positions.get(instrument).cloned().unwrap_or_default()
}

/// Utility function to calculate the current exposure for a given instrument.
pub fn calculate_current_exposure<K, V>(positions: &HashMap<K, V>, instrument: &K) -> V
where
    K: std::hash::Hash + Eq,
    V: Clone + Default,
{
    positions.get(instrument).cloned().unwrap_or_default()
}

/// Utility function to validate order size against position limits.
pub fn validate_order_size<T>(order_size: &T, max_size: &T) -> Result<(), &'static str>
where
    T: PartialOrd,
{
    if order_size > max_size {
        Err("Order size exceeds maximum allowed size")
    } else {
        Ok(())
    }
}

/// Utility function to validate order price against limits.
pub fn validate_order_price<T>(
    order_price: &T,
    min_price: &T,
    max_price: &T,
) -> Result<(), &'static str>
where
    T: PartialOrd,
{
    if order_price < min_price {
        Err("Order price below minimum allowed price")
    } else if order_price > max_price {
        Err("Order price above maximum allowed price")
    } else {
        Ok(())
    }
}

/// Utility function to check if an instrument is allowed for trading.
pub fn validate_instrument_allowed<T>(
    instrument: &T,
    allowed_instruments: &[T],
) -> Result<(), &'static str>
where
    T: PartialEq,
{
    if allowed_instruments.contains(instrument) {
        Ok(())
    } else {
        Err("Instrument not allowed for trading")
    }
}
