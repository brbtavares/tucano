
/// Compatibility type aliases during the ongoing markets → execution refactor.
///
/// Why aliases (pub type X = String)?
/// - Fast migration away from the previous strongly-typed index newtypes.
/// - Zero runtime cost / zero code churn in dependent crates.
/// - Easy rollback and incremental re-introduction of real newtypes later.
///
/// When to migrate to real newtypes again:
/// - After stabilizing engine + analytics behavior under string route.
/// - Once we want the compiler to prevent accidental cross-kind mixups (asset vs instrument).
///
/// Transitional plan (see long design note below): keep these aliases exported here;
/// begin introducing opt-in newtypes under `compat::typed` for modules that are
/// ready (e.g. indexing, persistence). Downstream crates should not assume these
/// stay aliases forever—treat them as semantic identifiers.
// Compatibility types for migration from the markets architecture
// We keep simple aliases to avoid breaking the build; we add an optional
// `typed` module with newtypes for progressive migration (opt-in).
pub type AssetIndex = String;
pub type InstrumentIndex = String;
pub type AssetNameExchange = String;
pub type InstrumentNameExchange = String;
pub type QuoteAsset = String;
pub type ExchangeIndex = String;
pub type ExchangeKey = String;
pub type AssetKey = String;
pub type InstrumentKey = String;
// New layer (Phase 1 of Exchange / Broker / Transport separation)
// BrokerId: identifies the broker (e.g., "XP", "CLEAR"). Initially alias = String.
// AccountId: identifies the account within the broker (e.g., account number / login).
pub type BrokerId = String;
pub type AccountId = String;

/// Experimental newtypes for future migration (not yet used). Provide greater safety
/// without breaking current code. When adopted, just replace imports:
/// `use execution::compat::typed::AssetIndex` etc.
#[allow(dead_code)]
pub mod typed {
    macro_rules! string_newtype {
        ($(#[$meta:meta])* $vis:vis struct $Name:ident;) => {
            $(#[$meta])*
            #[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, serde::Serialize, serde::Deserialize)]
            $vis struct $Name(String);
            impl $Name { pub fn new<S: Into<String>>(s: S) -> Self { Self(s.into()) } pub fn as_str(&self) -> &str { &self.0 } pub fn into_string(self) -> String { self.0 } }
            impl std::fmt::Debug for $Name { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, concat!(stringify!($Name), "(\"{}\")"), self.0) } }
            impl std::fmt::Display for $Name { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.write_str(&self.0) } }
            impl From<&str> for $Name { fn from(s: &str) -> Self { Self::new(s) } }
            impl From<String> for $Name { fn from(s: String) -> Self { Self::new(s) } }
            impl From<$Name> for String { fn from(v: $Name) -> Self { v.0 } }
            impl AsRef<str> for $Name { fn as_ref(&self) -> &str { self.as_str() } }
            impl std::ops::Deref for $Name { type Target = str; fn deref(&self) -> &Self::Target { self.as_str() } }
        }
    }
    string_newtype!(
        pub struct AssetIndex;
    );
    string_newtype!(
        pub struct InstrumentIndex;
    );
    string_newtype!(
        pub struct AssetNameExchange;
    );
    string_newtype!(
        pub struct InstrumentNameExchange;
    );
    string_newtype!(
        pub struct ExchangeIndex;
    );
    string_newtype!(
        pub struct QuoteAsset;
    );
}

// Re-export from markets - keeping ExchangeId as the original enum
pub use toucan_instrument::{ExchangeId, Side};

// Import required order types
use crate::order::OrderKey;

// Compatible response types
pub type UnindexedOrderKey = OrderKey<String>;

// For compatibility with old code that expected IndexError
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    thiserror::Error,
    serde::Serialize,
    serde::Deserialize,
)]
pub enum IndexError {
    #[error("Asset index error: {0}")]
    AssetIndex(String),
    #[error("Instrument index error: {0}")]
    InstrumentIndex(String),
    #[error("Exchange index error: {0}")]
    ExchangeIndex(String),
}

// Conversions from &str/String -> IndexError helpers (future ergonomics)
impl IndexError {
    pub fn asset<S: Into<String>>(s: S) -> Self {
        Self::AssetIndex(s.into())
    }
    pub fn instrument<S: Into<String>>(s: S) -> Self {
        Self::InstrumentIndex(s.into())
    }
    pub fn exchange<S: Into<String>>(s: S) -> Self {
        Self::ExchangeIndex(s.into())
    }
}

// -----------------------------------------------------------------------------
// Design notes / typing migration (summary)
// -----------------------------------------------------------------------------
// Alias = String vs Newtype:
//   pub type AssetIndex = String;  // alias: no new type, just another name.
//   pub struct AssetIndex(String); // newtype: distinct type, extra safety.
//
// Current situation:
//   We keep aliases for development speed and simple rollback.
//   ExchangeId remains canonical enum from `markets`.
//   AssetNameExchange / InstrumentNameExchange represent raw input symbols.
//   AssetIndex / InstrumentIndex represent internal identifiers (still Strings).
//   AssetKey / InstrumentKey are generic aliases to parameterize structures.
//   QuoteAsset identifies the quote/fees asset (still an alias).
//
// Motivations to migrate to newtypes in the future:
//   * Safety: prevent accidentally swapping InstrumentIndex for AssetIndex.
//   * Evolution: change internal representation (e.g., String -> u32) without breaking external API.
//   * Validation: normalize symbols (B3 vs crypto) in the constructor.
//   * Performance: cheaper hashing / interning / compact storage.
//
// Classification axes used for each type:
//   Dimension  : Exchange | Asset | Instrument | Quote
//   Nature     : External name (NameExchange) | Internal index (Index) | Enum ID | Generic key (Key)
//   Flow       : Input (APIs/streams) | Core (engine/state) | Output (analytics)
//
// Suggested migration phases:
//   F0: (current) aliases = String.
//   F1: Introduce *Symbol newtypes (AssetSymbol, InstrumentSymbol) for NameExchange.
//   F2: Introduce lightweight AssetIndex/InstrumentIndex newtypes (String inside) with From<&str>.
//   F3: Change internal maps to use newtypes as keys.
//   F4: Optimize representation (u32 / NonZeroU32) while keeping public API.
//   F5: (Optional) Scope by exchange: struct AssetId { exchange: ExchangeId, index: AssetIndex }.
//
// Criteria before migrating each alias:
//   1. Define uniqueness (global or per exchange).
//   2. Ensure round-trip (name -> index -> name) is tested.
//   3. Centralize conversions (map.rs / indexer).
//   4. Write invariants in doc comments.
//
// Next steps (when migration is decided):
//   * Create a definitive module (e.g., execution::types) with newtypes.
//   * Replace uses in maps and indexer first (smallest public surface).
//   * Adjust client traits (ExecutionClient) by adding generic bounds if needed.
//   * Gradually remove aliases from compat.rs when coverage >90%.
//
// This block serves as a quick reference for future refactors; keep updated as decisions are made.
