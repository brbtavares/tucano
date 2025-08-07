//! Core asset abstractions

use serde::{Deserialize, Serialize};

/// Core trait for financial assets
pub trait Asset {
    fn symbol(&self) -> &str;
    fn asset_type(&self) -> AssetType;
}

/// Basic asset types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AssetType {
    Currency,
    Stock,
    Future,
    Option,
    Fund,
    Bond,
    Other,
}

/// Exchange-specific asset name for compatibility
pub mod name {
    use serde::{Deserialize, Serialize};
    use smol_str::SmolStr;
    use std::{fmt::Display, str::FromStr};

    #[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize, Serialize)]
    pub struct AssetNameExchange(SmolStr);

    impl AssetNameExchange {
        pub fn new(asset: impl Into<SmolStr>) -> Self {
            Self(asset.into())
        }

        pub fn as_str(&self) -> &str {
            self.0.as_str()
        }
    }

    impl std::fmt::Debug for AssetNameExchange {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, r#"AssetNameExchange("{}")"#, self.0)
        }
    }

    impl Display for AssetNameExchange {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    impl From<&str> for AssetNameExchange {
        fn from(input: &str) -> Self {
            Self(SmolStr::new(input))
        }
    }

    impl From<String> for AssetNameExchange {
        fn from(input: String) -> Self {
            Self(SmolStr::new(input))
        }
    }

    impl FromStr for AssetNameExchange {
        type Err = &'static str;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            Ok(AssetNameExchange::new(s))
        }
    }

    /// QuoteAsset for compatibility
    #[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize, Serialize)]
    pub struct QuoteAsset<AssetKey>(AssetKey);

    impl<AssetKey> QuoteAsset<AssetKey> {
        pub fn new(asset: AssetKey) -> Self {
            Self(asset)
        }

        pub fn value(&self) -> &AssetKey {
            &self.0
        }

        pub fn into_inner(self) -> AssetKey {
            self.0
        }
    }

    impl<AssetKey> std::fmt::Debug for QuoteAsset<AssetKey>
    where
        AssetKey: std::fmt::Debug,
    {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "QuoteAsset({:?})", self.0)
        }
    }

    impl<AssetKey> Display for QuoteAsset<AssetKey>
    where
        AssetKey: Display,
    {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    impl<AssetKey> From<AssetKey> for QuoteAsset<AssetKey> {
        fn from(input: AssetKey) -> Self {
            Self(input)
        }
    }
}
