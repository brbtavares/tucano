
//! # 🔧 Macros - Code Generation Utilities
//!
//! Crate containing Rust procedural macros to automate boilerplate code generation
//! common in the Toucan framework. Reduces repetitive code and ensures
//! consistency in implementations.
//!
//! ## 🎯 Main Features
//!
//! ### Exchange Serialization
//! - `#[derive(SerExchange)]`: Automatically generates `Serialize` implementation for exchanges
//! - `#[derive(DeExchange)]`: Automatically generates `Deserialize` implementation for exchanges
//! - `#[derive(SerDe)]`: Combination of both for convenience
//!
//! ### Identifier Generation
//! - Automatic creation of unique IDs for exchanges
//! - Name conversion to different cases (snake_case, CamelCase, etc.)
//! - Format validation during deserialization
//!
//! ## 💡 Usage Example
//!
//! ```rust,ignore
//! use toucan_macros::{DeExchange, SerExchange};
//!
//! #[derive(DeExchange, SerExchange)]
//! struct B3Exchange {
//!     // specific fields
//! }
//!
//! impl B3Exchange {
//!     const ID: &'static str = "b3";
//! }
//! ```
//!
//! ## 🔍 Available Macros
//!
//! ### DeExchange
//! Generates a `Deserialize` implementation that validates the exchange ID:
//! - Checks if the deserialized ID matches the expected one
//! - Returns a descriptive error in case of mismatch
//! - Uses the type's `ID` constant for validation
//!
//! ### SerExchange
//! Generates a `Serialize` implementation that converts to string:
//! - Serializes using the exchange's unique ID
//! - Ensures consistency between serialization and deserialization
//! - Supports different output formats
//!
//! ## 🏗️ Internal Implementation
//!
//! The macros use:
//! - **syn**: Rust AST parsing
//! - **quote**: Rust code generation
//! - **convert_case**: String case conversion
//! - **proc_macro**: Procedural macro interface

extern crate proc_macro;

use convert_case::{Boundary, Case, Casing};
use proc_macro::TokenStream;
use quote::quote;
use syn::DeriveInput;

/// Procedural macro to automatically derive `Deserialize` implementation for exchanges.
///
/// Generates code that:
/// 1. Deserializes a string from input
/// 2. Compares with the expected exchange ID (constant `ID`)
/// 3. Returns the exchange if it matches or an error if it does not
///
/// # Requirements
/// The type must have a constant `ID: &'static str` defined.
///
/// # Example
/// ```rust,ignore
/// #[derive(DeExchange)]
/// struct B3Exchange;
///
/// impl B3Exchange {
///     const ID: &'static str = "b3";
/// }
/// ```
#[proc_macro_derive(DeExchange)]
pub fn de_exchange_derive(input: TokenStream) -> TokenStream {
    // Parse the Rust abstract syntax tree with Syn from TokenStream -> DeriveInput
    let ast: DeriveInput = syn::parse(input)
    .expect("de_exchange_derive() failed to parse input TokenStream");

    // Determina o nome do exchange
    let exchange = &ast.ident;

    let generated = quote! {
        impl<'de> serde::Deserialize<'de> for #exchange {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::de::Deserializer<'de>
            {
                let input = <String as serde::Deserialize>::deserialize(deserializer)?;
                let exchange = #exchange::ID;
                let expected = exchange.as_str();

                if input.as_str() == expected {
                    Ok(Self::default())
                } else {
                    Err(serde::de::Error::invalid_value(
                        serde::de::Unexpected::Str(input.as_str()),
                        &expected
                    ))
                }
            }
        }
    };

    TokenStream::from(generated)
}

/// Procedural macro to automatically derive `Serialize` implementation for exchanges.
///
/// Generates code that serializes the exchange using its unique ID defined
/// in the constant `ID`. Ensures consistency with deserialization.
///
/// # Requirements
/// The type must have a constant `ID: &'static str` defined.
///
/// # Example
/// ```rust,ignore
/// #[derive(SerExchange)]
/// struct B3Exchange;
///
/// impl B3Exchange {
///     const ID: &'static str = "b3";
/// }
/// ```
#[proc_macro_derive(SerExchange)]
pub fn ser_exchange_derive(input: TokenStream) -> TokenStream {
    // Parse the Rust abstract syntax tree with Syn from TokenStream -> DeriveInput
    let ast: DeriveInput = syn::parse(input)
    .expect("ser_exchange_derive() failed to parse input TokenStream");

    // Determina o Exchange
    let exchange = &ast.ident;

    let generated = quote! {
        impl serde::Serialize for #exchange {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::ser::Serializer,
            {
                serializer.serialize_str(#exchange::ID.as_str())
            }
        }
    };

    TokenStream::from(generated)
}

#[proc_macro_derive(DeSubKind)]
pub fn de_sub_kind_derive(input: TokenStream) -> TokenStream {
    // Parse Rust code abstract syntax tree with Syn from TokenStream -> DeriveInput
    let ast: DeriveInput =
        syn::parse(input).expect("de_sub_kind_derive() failed to parse input TokenStream");

    // Determine SubKind name
    let sub_kind = &ast.ident;

    let expected_sub_kind = sub_kind
        .to_string()
        .from_case(Case::Pascal)
        .without_boundaries(&Boundary::letter_digit())
        .to_case(Case::Snake);

    let generated = quote! {
        impl<'de> serde::Deserialize<'de> for #sub_kind {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::de::Deserializer<'de>
            {
                let input = <String as serde::Deserialize>::deserialize(deserializer)?;

                if input == #expected_sub_kind {
                    Ok(Self)
                } else {
                    Err(serde::de::Error::invalid_value(
                        serde::de::Unexpected::Str(input.as_str()),
                        &#expected_sub_kind
                    ))
                }
            }
        }
    };

    TokenStream::from(generated)
}

#[proc_macro_derive(SerSubKind)]
pub fn ser_sub_kind_derive(input: TokenStream) -> TokenStream {
    // Parse Rust code abstract syntax tree with Syn from TokenStream -> DeriveInput
    let ast: DeriveInput =
        syn::parse(input).expect("ser_sub_kind_derive() failed to parse input TokenStream");

    // Determine SubKind name
    let sub_kind = &ast.ident;
    let sub_kind_string = sub_kind.to_string().to_case(Case::Snake);
    let sub_kind_str = sub_kind_string.as_str();

    let generated = quote! {
        impl serde::Serialize for #sub_kind {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::ser::Serializer,
            {
                serializer.serialize_str(#sub_kind_str)
            }
        }
    };

    TokenStream::from(generated)
}
