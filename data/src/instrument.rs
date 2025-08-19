// Mini-Disclaimer: Educational/experimental use; not investment advice or affiliation; see README & DISCLAIMER.
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use tucano_instrument::{InstrumentKind, Keyed, MarketDataInstrument};

/// Instrument related data that defines an associated unique `Id`.
///
/// Verbose `InstrumentData` is often used to subscribe to market data feeds, but its unique `Id`
/// can then be used to key consumed [MarketEvents](crate::event::MarketEvent), significantly reducing
/// duplication in the case of complex instruments (e.g., options).
pub trait InstrumentData: Debug + Clone + Eq + Send + Sync {
    type Key: Debug + Clone + Eq + Send + Sync;
    type Kind;
    fn key(&self) -> &Self::Key;
    fn kind(&self) -> &Self::Kind;
}

impl<InstrumentKey> InstrumentData for Keyed<InstrumentKey, MarketDataInstrument>
where
    InstrumentKey: Debug + Clone + Eq + Send + Sync,
{
    type Key = InstrumentKey;
    type Kind = tucano_instrument::MarketDataInstrumentKind;

    fn key(&self) -> &Self::Key {
        &self.key
    }

    fn kind(&self) -> &Self::Kind {
        &self.value.kind
    }
}

impl InstrumentData for MarketDataInstrument {
    type Key = Self;
    type Kind = tucano_instrument::MarketDataInstrumentKind;

    fn key(&self) -> &Self::Key {
        self
    }

    fn kind(&self) -> &Self::Kind {
        &self.kind
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize, Serialize)]
pub struct MarketInstrumentData<InstrumentKey> {
    pub key: InstrumentKey,
    pub name_exchange: String,
    pub kind: InstrumentKind<String>,
}

impl<InstrumentKey> InstrumentData for MarketInstrumentData<InstrumentKey>
where
    InstrumentKey: Debug + Clone + Eq + Send + Sync,
{
    type Key = InstrumentKey;
    type Kind = InstrumentKind<String>;

    fn key(&self) -> &Self::Key {
        &self.key
    }

    fn kind(&self) -> &Self::Kind {
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
            "{}_{}_{:?}",
            self.key,
            self.name_exchange.as_str(),
            self.kind
        )
    }
}

// Implementação From removida temporariamente - incompatível com nova arquitetura híbrida
// TODO: Reimplementar usando traits markets ao invés de campos específicos
