// Temporary stub to allow `pub mod profitdll;` in exchanges::temp
// This allows imports like exchanges::temp::profitdll::*
// Remove this file when the real module is migrated.

// --- Stubs to eliminate unresolved import errors ---
#[allow(dead_code)]
pub enum CallbackEvent {}

#[allow(dead_code)]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug)]
pub enum ProfitError {
    ConnectionFailed(String),
    // Add more variants as needed for stubbing
}
