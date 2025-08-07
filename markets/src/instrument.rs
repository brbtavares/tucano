//! Core instrument abstractions

use std::fmt::Display;

/// Core trait for financial instruments
pub trait Instrument {
    type Symbol: Display + Clone;
    
    fn symbol(&self) -> &Self::Symbol;
    fn market(&self) -> &str;
}
