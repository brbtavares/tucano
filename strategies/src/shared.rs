// Mini-Disclaimer: For educational/experimental use only; no investment advice or affiliation; no third-party compensation; Profit/ProfitDLL © Nelógica; see README & DISCLAIMER.
//! Shared modules (indicators, utilities) among strategies.

/// Empty state that can be reused by strategies that do not yet
/// require specific engine data. Serves as a placeholder.
#[derive(Debug, Default, Clone, Copy)]
pub struct NoOpState;
