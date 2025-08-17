// Mini-Disclaimer: For educational/experimental use only; no investment advice or affiliation; no third-party compensation; Profit/ProfitDLL © Nelógica; see README & DISCLAIMER.
//! Simple order book imbalance strategy.
//!
//! Reusable in both live and backtest modes.
//! Does not depend on technical indicators; only compares aggregated bid vs ask volume.

use rust_decimal::Decimal;
use tucano_execution::order::request::OrderRequestOpen;
use tucano_execution::{ExchangeIndex, InstrumentIndex};
use tucano_markets::Side;
use tucano_trader::AlgoStrategy;

/// Configuration for the imbalance strategy.
#[derive(Debug, Clone)]
pub struct OrderBookImbalanceConfig {
    /// Minimum imbalance percentage (0-1) to trigger buy/sell. E.g., 0.6 => 60%.
    pub threshold: Decimal,
    /// Base quantity to send per order when signal occurs.
    pub quantity: Decimal,
}

impl Default for OrderBookImbalanceConfig {
    fn default() -> Self {
        Self {
            threshold: Decimal::new(60, 2),
            quantity: Decimal::ONE,
        } // 0.60
    }
}

/// Optional volatile state (e.g., last triggered direction) to avoid over-trading.
#[derive(Debug, Default, Clone)]
pub struct OrderBookImbalanceState {
    #[allow(dead_code)]
    last_side: Option<Side>,
}

pub struct OrderBookImbalanceStrategy<C = OrderBookImbalanceConfig> {
    pub config: C,
    pub state: OrderBookImbalanceState,
}

impl<C> OrderBookImbalanceStrategy<C> {
    pub fn new(config: C) -> Self {
        Self {
            config,
            state: OrderBookImbalanceState::default(),
        }
    }
}

// Estrutura simplificada de snapshot de livro que o motor poderia expor.
#[derive(Debug, Clone)]
pub struct SimpleBook {
    pub instrument: InstrumentIndex,
    pub best_bid_volume: Decimal,
    pub best_ask_volume: Decimal,
}

impl<C: AsRef<OrderBookImbalanceConfig>> AlgoStrategy<ExchangeIndex, InstrumentIndex>
    for OrderBookImbalanceStrategy<C>
{
    type State = crate::shared::NoOpState; // reutiliza um estado vazio existente / placeholder

    fn generate_algo_orders(
        &self,
        _state: &Self::State,
    ) -> (
        impl IntoIterator<
            Item = tucano_execution::order::request::OrderRequestCancel<
                ExchangeIndex,
                InstrumentIndex,
            >,
        >,
        impl IntoIterator<Item = OrderRequestOpen<ExchangeIndex, InstrumentIndex>>,
    ) {
        // Sem acesso real ao livro aqui — retornar vazio até integração com dados.
        (Vec::new(), Vec::new())
    }
}
