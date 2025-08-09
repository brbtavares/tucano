use markets::{InstrumentKind, Keyed, MarketDataInstrument};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// Instrument related data that defines an associated unique `Id`.
///
/// Verbose `InstrumentData` is often used to subscribe to market data feeds, but it's unique `Id`
/// can then be used to key consumed [MarketEvents](crate::event::MarketEvent), significantly reducing
/// duplication in the case of complex instruments (eg/ options).
pub trait InstrumentData: Debug + Clone + Eq + Send + Sync {
    type Key: Debug + Clone + Eq + Send + Sync;
    fn key(&self) -> &Self::Key;
    fn kind(&self) -> &InstrumentKind;
}

impl<InstrumentKey> InstrumentData for Keyed<InstrumentKey, MarketDataInstrument>
where
    InstrumentKey: Debug + Clone + Eq + Send + Sync,
{
    type Key = InstrumentKey;

    fn key(&self) -> &Self::Key {
        &self.key
    }

    fn kind(&self) -> &InstrumentKind {
        &self.value.kind
    }
}

impl InstrumentData for MarketDataInstrument {
    type Key = Self;

    fn key(&self) -> &Self::Key {
        self
    }

    fn kind(&self) -> &InstrumentKind {
        &self.kind
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize, Serialize)]
pub struct MarketInstrumentData<InstrumentKey> {
    pub key: InstrumentKey,
    pub name_exchange: String,
    pub kind: InstrumentKind,
}

impl<InstrumentKey> InstrumentData for MarketInstrumentData<InstrumentKey>
where
    InstrumentKey: Debug + Clone + Eq + Send + Sync,
{
    type Key = InstrumentKey;

    fn key(&self) -> &Self::Key {
        &self.key
    }

    fn kind(&self) -> &InstrumentKind {
        &self.kind
    }
}

impl<InstrumentKey> std::fmt::Display for MarketInstrumentData<InstrumentKey>
where
    InstrumentKey: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}_{}_{}",
            self.key,
            self.name_exchange.as_str(),
            self.kind
        )
    }
}

// Implementação From removida temporariamente - incompatível com nova arquitetura híbrida
// TODO: Reimplementar usando traits markets ao invés de campos específicos
