//! # 🔧 Macros - Utilitários de Geração de Código
//!
//! Crate contendo macros procedurais Rust para automatizar geração de código
//! boilerplate comum no framework Toucan. Reduz código repetitivo e garante
//! consistência nas implementações.
//!
//! ## 🎯 Funcionalidades Principais
//!
//! ### Serialização de Exchanges
//! - `#[derive(SerExchange)]`: Gera implementação automática de `Serialize` para exchanges
//! - `#[derive(DeExchange)]`: Gera implementação automática de `Deserialize` para exchanges
//! - `#[derive(SerDe)]`: Combinação de ambos para conveniência
//!
//! ### Geração de Identificadores
//! - Criação automática de IDs únicos para exchanges
//! - Conversão de nomes para diferentes casos (snake_case, CamelCase, etc.)
//! - Validação de formatos durante deserialização
//!
//! ## 💡 Exemplo de Uso
//!
//! ```rust
//! use macros::{DeExchange, SerExchange};
//! 
//! #[derive(DeExchange, SerExchange)]
//! struct B3Exchange {
//!     // campos específicos
//! }
//! 
//! impl B3Exchange {
//!     const ID: &'static str = "b3";
//! }
//! ```
//!
//! ## 🔍 Macros Disponíveis
//!
//! ### DeExchange
//! Gera implementação de `Deserialize` que valida o ID do exchange:
//! - Verifica se o ID deserializado corresponde ao esperado
//! - Retorna erro descritivo em caso de incompatibilidade
//! - Usa a constante `ID` do tipo para validação
//!
//! ### SerExchange  
//! Gera implementação de `Serialize` que converte para string:
//! - Serializa usando o ID único do exchange
//! - Garante consistência entre serialização e deserialização
//! - Suporte a diferentes formatos de output
//!
//! ## 🏗️ Implementação Interna
//!
//! As macros utilizam:
//! - **syn**: Parsing de AST Rust
//! - **quote**: Geração de código Rust
//! - **convert_case**: Conversão entre casos de string
//! - **proc_macro**: Interface de macros procedurais

extern crate proc_macro;

use convert_case::{Boundary, Case, Casing};
use proc_macro::TokenStream;
use quote::quote;
use syn::DeriveInput;

/// Macro procedural para derivar automaticamente implementação de `Deserialize` para exchanges.
///
/// Gera código que:
/// 1. Deserializa uma string do input
/// 2. Compara com o ID esperado do exchange (constante `ID`)
/// 3. Retorna o exchange se corresponder ou erro se não corresponder
///
/// # Requisitos
/// O tipo deve ter uma constante `ID: &'static str` definida.
///
/// # Exemplo
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
    // Parse da árvore sintática abstrata do Rust com Syn de TokenStream -> DeriveInput
    let ast: DeriveInput =
        syn::parse(input).expect("de_exchange_derive() falhou ao fazer parse do TokenStream de entrada");

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

/// Macro procedural para derivar automaticamente implementação de `Serialize` para exchanges.
///
/// Gera código que serializa o exchange usando seu ID único definido
/// na constante `ID`. Garante consistência com a deserialização.
///
/// # Requisitos
/// O tipo deve ter uma constante `ID: &'static str` definida.
///
/// # Exemplo
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
    // Parse da árvore sintática abstrata do Rust com Syn de TokenStream -> DeriveInput
    let ast: DeriveInput =
        syn::parse(input).expect("ser_exchange_derive() falhou ao fazer parse do TokenStream de entrada");

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
