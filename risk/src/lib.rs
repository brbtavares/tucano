// Mini-Disclaimer: Educational/experimental use; not investment advice or affiliation; see README & DISCLAIMER.
//! # 🛡️ Risk - Risk Management Module
//!
//! Comprehensive framework for risk management in algorithmic trading,
//! providing validations, limits, and controls to protect capital
//! and ensure regulatory compliance.
//!
//! ## 🎯 Main Objectives
//!
//! - **Capital Protection**: Prevent excessive losses
//! - **Exposure Control**: Limit positions per asset/market
//! - **Compliance**: Adhere to financial regulations
//! - **Performance**: Real-time validations with low latency
//!
//! ## 🏗️ System Components
//!
//! ### RiskManager
//! Main interface for reviewing and filtering orders:
//! ```text
//! use risk::{RiskManager, RiskApproved, RiskRefused};
//!
//! impl RiskManager for MyRiskManager {
//!     fn check_order(&self, order: &Order) -> Result<RiskApproved<Order>, RiskRefused<Order>> {
//!         // Implement specific validations
//!     }
//! }
//! ```
//!
//! ### Validation Types
//! - **Position Limits**: Maximum position limits per instrument
//! - **Exposure Limits**: Total exposure limits per market
//! - **Leverage Control**: Maximum leverage control
//! - **Concentration Risk**: Prevent excessive concentration
//! - **Market Hours**: Market hours validation
//! - **Circuit Breakers**: Automatic stop on excessive losses
//!
//! ## 🔍 Result Structures
//!
//! ### RiskApproved<T>
//! Represents an operation approved by the risk system:
//! ```text
//! let approved = RiskApproved::new(order);
//! let order = approved.into_item(); // Extract the approved item
//! ```
//!
//! ### RiskRefused<T>
//! Represents an operation rejected with a specific reason:
//! ```text
//! let refused = RiskRefused::new(order, "Exceeds position limit");
//! println!("Rejected: {}", refused.reason);
//! ```
//!
//! ## 🚨 Common Risk Scenarios
//!
//! ### Position Limits
//! ```rust,ignore
//! if position_size > max_position_limit {
//!     return Err(RiskRefused::new(order, "Exceeds maximum position limit"));
//! }
//! ```
//!
//! ### Exposure Control
//! ```text
//! let total_exposure = calculate_exposure(&portfolio);
//! if total_exposure > exposure_limit {
//!     return Err(RiskRefused::new(order, "Exceeds exposure limit"));
//! }
//! ```
//!
//! ### Market Hours
//! ```text
//! if !is_market_open(instrument.exchange()) {
//!     return Err(RiskRefused::new(order, "Market closed"));
//! }
//! ```
//!
//! ## 📊 Risk Metrics
//!
//! - **VaR (Value at Risk)**: Loss risk under normal conditions
//! - **CVaR (Conditional VaR)**: Loss risk in extreme scenarios
//! - **Maximum Drawdown**: Largest historical observed loss
//! - **Sharpe Ratio**: Risk-adjusted return
//! - **Beta**: Correlation with reference market
//!
//! ## 🔄 Integration with Engine
//!
//! The risk module integrates natively with the core engine:
//! ```text
//! use core::engine::Engine;
//! use risk::RiskManager;
//!
//! let engine = Engine::new(
//!     clock,
//!     state,
//!     execution_txs,
//!     strategy,
//!     risk_manager // <- Automatic integration
//! );
//! ```

/// Module containing risk check implementations.
///
/// Includes specific validators for different types of risk
/// such as position limits, exposure, market hours, etc.
pub mod check;

pub use check::*;

use derive_more::{Constructor, Display, From};
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, hash::Hash, marker::PhantomData};
use tucano_execution::{
    order::request::{OrderRequestCancel, OrderRequestOpen},
    ExchangeIndex, InstrumentIndex,
};

/// Approved result of a [`RiskManager`] check.
///
/// Wrapper indicating that an item (such as an order) has passed all
/// risk checks and is approved for execution.
///
/// # Example
/// ```rust,ignore
/// use risk::RiskApproved;
///
/// let approved_order = RiskApproved::new(order);
/// println!("Order approved: {}", approved_order);
/// ```
#[derive(
    Debug,
    Clone,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    Serialize,
    Deserialize,
    Display,
    From,
    Constructor,
)]
pub struct RiskApproved<T>(pub T);

impl<T> RiskApproved<T> {
    /// Extrai o item aprovado do wrapper.
    pub fn into_item(self) -> T {
        self.0
    }
}

/// Rejected result of a [`RiskManager`] check.
///
/// Contains the rejected item and the specific reason for rejection,
/// allowing detailed logging and corrective actions.
///
/// # Example
/// ```rust,ignore
/// use risk::RiskRefused;
///
/// let refused = RiskRefused::new(order, "Exceeds position limit");
/// println!("Order rejected: {}", refused.reason);
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct RiskRefused<T, Reason = String> {
    /// The item that was rejected
    pub item: T,
    /// Specific reason for rejection
    pub reason: Reason,
}

impl<T> RiskRefused<T> {
    /// Creates a new `RiskRefused` instance with the provided item and reason.
    pub fn new(item: T, reason: impl Into<String>) -> Self {
        Self {
            item,
            reason: reason.into(),
        }
    }
}

impl<T, Reason> RiskRefused<T, Reason> {
    /// Extracts the rejected item from the wrapper.
    pub fn into_item(self) -> T {
        self.item
    }
}

/// RiskManager interface for reviewing and optionally filtering cancel and open orders generated by an [`AlgoStrategy`](trader::AlgoStrategy).
///
/// ## Main Responsibilities
///
/// A RiskManager can implement various checks such as:
/// - **Exposure Filter**: Reject orders that would result in excessive exposure
/// - **Position Limits**: Check if the order does not exceed limits per instrument
/// - **Margin Validation**: Ensure sufficient margin for new positions
/// - **Market Hours**: Validate if the market is open for trading
/// - **Circuit Breakers**: Stop operations in case of excessive losses
/// - **Compliance**: Check compliance with regulations
///
/// ## Implementation Example
/// ```rust,ignore
/// use risk::{RiskManager, RiskApproved, RiskRefused};
///
/// struct MyRiskManager {
///     max_position: f64,
///     max_exposure: f64,
/// }
///
/// impl RiskManager for MyRiskManager {
///     fn check_order(&self, order: &Order) -> Result<RiskApproved<Order>, RiskRefused<Order>> {
///         if order.quantity > self.max_position {
///             return Err(RiskRefused::new(order.clone(), "Position too large"));
///         }
///         Ok(RiskApproved::new(order.clone()))
///     }
/// }
/// ```
///
/// For example, a RiskManager implementation may wish to:
/// - Filter out orders that would result in too much exposure.
/// - Filter out orders that have a too high quantity.
/// - Adjust order quantities.
/// - Filter out orders that would cross the OrderBook.
/// - etc.
///
/// # Type Parameters
/// * `ExchangeKey` - Type used to identify an exchange (defaults to [`ExchangeIndex`]).
/// * `InstrumentKey` - Type used to identify an instrument (defaults to [`InstrumentIndex`]).
#[allow(clippy::type_complexity)]
pub trait RiskManager<ExchangeKey = ExchangeIndex, InstrumentKey = InstrumentIndex> {
    type State;

    fn check(
        &self,
        state: &Self::State,
        cancels: impl IntoIterator<Item = OrderRequestCancel<ExchangeKey, InstrumentKey>>,
        opens: impl IntoIterator<Item = OrderRequestOpen<ExchangeKey, InstrumentKey>>,
    ) -> (
        impl IntoIterator<Item = RiskApproved<OrderRequestCancel<ExchangeKey, InstrumentKey>>>,
        impl IntoIterator<Item = RiskApproved<OrderRequestOpen<ExchangeKey, InstrumentKey>>>,
        impl IntoIterator<Item = RiskRefused<OrderRequestCancel<ExchangeKey, InstrumentKey>>>,
        impl IntoIterator<Item = RiskRefused<OrderRequestOpen<ExchangeKey, InstrumentKey>>>,
    );
}

/// Pass-through risk manager that approves all requests.
#[derive(Debug, Clone, Default)]
pub struct NoRiskManager;

impl<ExchangeKey, InstrumentKey> RiskManager<ExchangeKey, InstrumentKey> for NoRiskManager {
    type State = ();

    fn check(
        &self,
        _state: &Self::State,
        cancels: impl IntoIterator<Item = OrderRequestCancel<ExchangeKey, InstrumentKey>>,
        opens: impl IntoIterator<Item = OrderRequestOpen<ExchangeKey, InstrumentKey>>,
    ) -> (
        impl IntoIterator<Item = RiskApproved<OrderRequestCancel<ExchangeKey, InstrumentKey>>>,
        impl IntoIterator<Item = RiskApproved<OrderRequestOpen<ExchangeKey, InstrumentKey>>>,
        impl IntoIterator<Item = RiskRefused<OrderRequestCancel<ExchangeKey, InstrumentKey>>>,
        impl IntoIterator<Item = RiskRefused<OrderRequestOpen<ExchangeKey, InstrumentKey>>>,
    ) {
        let approved_cancels: Vec<_> = cancels.into_iter().map(RiskApproved::new).collect();
        let approved_opens: Vec<_> = opens.into_iter().map(RiskApproved::new).collect();
        let refused_cancels: Vec<RiskRefused<OrderRequestCancel<ExchangeKey, InstrumentKey>>> =
            vec![];
        let refused_opens: Vec<RiskRefused<OrderRequestOpen<ExchangeKey, InstrumentKey>>> = vec![];

        (
            approved_cancels,
            approved_opens,
            refused_cancels,
            refused_opens,
        )
    }
}

/// Naive implementation of the [`RiskManager`] interface, approving all orders *without any
/// risk checks*.
///
/// *THIS IS FOR DEMONSTRATION PURPOSES ONLY, NEVER USE FOR REAL TRADING OR IN PRODUCTION*.
#[derive(Debug, Clone)]
pub struct DefaultRiskManager<State> {
    phantom: PhantomData<State>,
}

impl<State> Default for DefaultRiskManager<State> {
    fn default() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

#[allow(clippy::type_complexity)]
impl<State, ExchangeKey, InstrumentKey> RiskManager<ExchangeKey, InstrumentKey>
    for DefaultRiskManager<State>
{
    type State = State;

    fn check(
        &self,
        _: &Self::State,
        cancels: impl IntoIterator<Item = OrderRequestCancel<ExchangeKey, InstrumentKey>>,
        opens: impl IntoIterator<Item = OrderRequestOpen<ExchangeKey, InstrumentKey>>,
    ) -> (
        impl IntoIterator<Item = RiskApproved<OrderRequestCancel<ExchangeKey, InstrumentKey>>>,
        impl IntoIterator<Item = RiskApproved<OrderRequestOpen<ExchangeKey, InstrumentKey>>>,
        impl IntoIterator<Item = RiskRefused<OrderRequestCancel<ExchangeKey, InstrumentKey>>>,
        impl IntoIterator<Item = RiskRefused<OrderRequestOpen<ExchangeKey, InstrumentKey>>>,
    ) {
        (
            cancels.into_iter().map(RiskApproved::new),
            opens.into_iter().map(RiskApproved::new),
            std::iter::empty(),
            std::iter::empty(),
        )
    }
}
