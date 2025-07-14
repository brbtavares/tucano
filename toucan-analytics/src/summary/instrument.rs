use crate::{
    metric::{
        calmar::CalmarRatio,
        drawdown::{
            Drawdown, DrawdownGenerator,
            max::{MaxDrawdown, MaxDrawdownGenerator},
            mean::{MeanDrawdown, MeanDrawdownGenerator},
        },
        profit_factor::ProfitFactor,
        rate_of_return::RateOfReturn,
        sharpe::SharpeRatio,
        sortino::SortinoRatio,
        win_rate::WinRate,
    },
    summary::pnl::PnLReturns,
    time::TimeInterval,
};
use chrono::{DateTime, TimeDelta, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

// Use the unified PositionExited from the parent module
use super::PositionExited;

/// TearSheet summarising the trading performance related to an instrument.
#[derive(Debug, Clone, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct TearSheet<Interval> {
    pub pnl: Decimal,
    pub pnl_return: RateOfReturn<Interval>,
    pub sharpe_ratio: SharpeRatio<Interval>,
    pub sortino_ratio: SortinoRatio<Interval>,
    pub calmar_ratio: CalmarRatio<Interval>,
    pub profit_factor: ProfitFactor,
    pub win_rate: WinRate,
    pub drawdown: Option<Drawdown>,
    pub drawdown_mean: Option<MeanDrawdown>,
    pub drawdown_max: Option<MaxDrawdown>,
}

impl<Interval: Default> Default for TearSheet<Interval> {
    fn default() -> Self {
        Self {
            pnl: Decimal::ZERO,
            pnl_return: RateOfReturn::default(),
            sharpe_ratio: SharpeRatio::default(),
            sortino_ratio: SortinoRatio::default(),
            calmar_ratio: CalmarRatio::default(),
            profit_factor: ProfitFactor::default(),
            win_rate: WinRate::default(),
            drawdown: None,
            drawdown_mean: None,
            drawdown_max: None,
        }
    }
}

/// Generator for an [`TearSheet`].
#[derive(Debug, Clone, PartialEq, PartialOrd, Default, Deserialize, Serialize)]
pub struct TearSheetGenerator {
    /// Time of the latest trading engine update.
    pub time_engine_now: DateTime<Utc>,

    /// PnL returns statistics for the session, including totals, wins and losses.
    pub pnl_returns: PnLReturns,

    /// Drawdown statistics
    pub pnl_drawdown: DrawdownGenerator,
    pub pnl_drawdown_mean: MeanDrawdownGenerator,
    pub pnl_drawdown_max: MaxDrawdownGenerator,
}

impl TearSheetGenerator {
    /// Initialise a [`TearSheetGenerator`] using a time that acts as both the start & now time.
    pub fn init(time_engine_start: DateTime<Utc>) -> Self {
        Self {
            time_engine_now: time_engine_start,
            pnl_returns: PnLReturns::default(),
            pnl_drawdown: DrawdownGenerator::init(Decimal::ZERO, time_engine_start),
            pnl_drawdown_mean: MeanDrawdownGenerator::default(),
            pnl_drawdown_max: MaxDrawdownGenerator::default(),
        }
    }

    /// Update the [`TearSheetGenerator`] from the next [`PositionExited`].
    pub fn update_from_position<AssetKey, InstrumentKey>(
        &mut self,
        position: &PositionExited,
    ) {
        self.time_engine_now = position.time_exit;
        self.pnl_returns.update::<AssetKey, InstrumentKey>(position);

        if let Some(next_drawdown) = self
            .pnl_drawdown
            .update(position.pnl_realised, self.time_engine_now)
        {
            self.pnl_drawdown_mean.update(&next_drawdown);
            self.pnl_drawdown_max.update(&next_drawdown);
        }
    }

    /// Generate the latest [`TearSheet`] at the specific [`TimeInterval`].
    ///
    /// For example, pass [`Annual365`](super::super::time::Annual365) to generate a crypto-centric
    /// (24/7 trading) annualised [`TearSheet`].
    pub fn generate<Interval>(&mut self, interval: Interval) -> TearSheet<Interval>
    where
        Interval: TimeInterval,
    {
        let risk_free_return = Decimal::ZERO; // Default risk-free return
        
        TearSheet {
            pnl: self.pnl_returns.pnl_raw,
            pnl_return: RateOfReturn::calculate(
                self.pnl_returns.total.mean,
                interval,
            ),
            sharpe_ratio: SharpeRatio::calculate(
                risk_free_return,
                self.pnl_returns.total.mean,
                self.pnl_returns.total.dispersion.std_dev,
                interval,
            ),
            sortino_ratio: SortinoRatio::calculate(
                risk_free_return,
                self.pnl_returns.total.mean,
                self.pnl_returns.losses.dispersion.std_dev,
                interval,
            ),
            calmar_ratio: CalmarRatio::calculate(
                risk_free_return,
                self.pnl_returns.total.mean,
                self.pnl_drawdown_max.max.as_ref().map(|max| max.0.value).unwrap_or(Decimal::ZERO),
                interval,
            ),
            profit_factor: ProfitFactor::calculate(
                self.pnl_returns.total.sum.max(Decimal::ZERO),
                self.pnl_returns.losses.sum.abs(),
            ).unwrap_or_default(),
            win_rate: WinRate::calculate(
                (self.pnl_returns.total.count - self.pnl_returns.losses.count).into(),
                self.pnl_returns.total.count.into(),
            ).unwrap_or_default(),
            drawdown: self.pnl_drawdown_mean.mean_drawdown.as_ref().map(|md| Drawdown {
                value: md.mean_drawdown,
                time_start: DateTime::<Utc>::MIN_UTC, // Placeholder
                time_end: DateTime::<Utc>::MIN_UTC,   // Placeholder
            }),
            drawdown_mean: self.pnl_drawdown_mean.mean_drawdown.clone(),
            drawdown_max: self.pnl_drawdown_max.max.clone(),
        }
    }

    /// Reset the internal state, using a new starting `DateTime<Utc>` as seed.
    pub fn reset(&mut self, time_engine_start: DateTime<Utc>) {
        *self = Self::init(time_engine_start);
    }
}
