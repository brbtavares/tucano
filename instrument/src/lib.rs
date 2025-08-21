
#![forbid(unsafe_code)]
#![warn(
    unused,
    clippy::cognitive_complexity,
    unused_crate_dependencies,
    unused_extern_crates,
    clippy::unused_self,
    clippy::useless_let_if_seq,
    missing_debug_implementations,
    rust_2018_idioms,
    rust_2024_compatibility
)]
#![allow(clippy::type_complexity, clippy::too_many_arguments, type_alias_bounds)]

pub mod asset;
pub mod exchange;
pub mod index;
pub mod instrument;
pub mod types;

// Re-export key types for downstream crates at the crate root (barter-rs style)
pub use exchange::{ExchangeId, ExchangeIndex};
pub use index::{builder::IndexedInstrumentsBuilder, error::IndexError, IndexedInstruments};
pub use instrument::kind::future::FutureContract;
pub use instrument::kind::option::{OptionContract, OptionExercise, OptionKind};
pub use instrument::kind::perpetual::PerpetualContract;
pub use instrument::kind::InstrumentKind;
pub use instrument::market_data::kind::{
    MarketDataFutureContract, MarketDataInstrumentKind, MarketDataOptionContract,
};
pub use instrument::market_data::MarketDataInstrument;
pub use instrument::{Instrument, InstrumentId, InstrumentIndex};
pub use types::{Keyed, Side, Underlying};
