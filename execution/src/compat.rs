// Tipos de compatibilidade para migração da arquitetura markets
// Mantemos aliases simples para não quebrar o build; adicionamos módulo opcional
// `typed` com newtypes para migração progressiva (opt-in).

pub type AssetIndex = String;
pub type InstrumentIndex = String;
pub type AssetNameExchange = String;
pub type InstrumentNameExchange = String;
pub type QuoteAsset = String;
pub type ExchangeIndex = String;
pub type ExchangeKey = String;
pub type AssetKey = String;
pub type InstrumentKey = String;

/// Newtypes experimentais para migração futura (ainda não usados). Fornecem maior segurança
/// sem quebrar o código atual. Quando adotados, bastará substituir imports:
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
    string_newtype!(pub struct AssetIndex;);
    string_newtype!(pub struct InstrumentIndex;);
    string_newtype!(pub struct AssetNameExchange;);
    string_newtype!(pub struct InstrumentNameExchange;);
    string_newtype!(pub struct ExchangeIndex;);
    string_newtype!(pub struct QuoteAsset;);
}

// Re-export do markets - mantendo ExchangeId como enum original
pub use markets::{ExchangeId, Side};

// Import dos tipos de order necessários
use crate::order::OrderKey;

// Tipos de response compatíveis
pub type UnindexedOrderKey = OrderKey<String>;

// Para compatibilidade com código antigo que esperava IndexError
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

// Conversions de &str/String -> IndexError helpers (ergonomia futura)
impl IndexError {
    pub fn asset<S: Into<String>>(s: S) -> Self { Self::AssetIndex(s.into()) }
    pub fn instrument<S: Into<String>>(s: S) -> Self { Self::InstrumentIndex(s.into()) }
    pub fn exchange<S: Into<String>>(s: S) -> Self { Self::ExchangeIndex(s.into()) }
}

// -----------------------------------------------------------------------------
// Notas de design / migração de tipagem (resumo)
// -----------------------------------------------------------------------------
// Alias = String vs Newtype:
//   pub type AssetIndex = String;  // alias: nenhum tipo novo, só outro nome.
//   pub struct AssetIndex(String); // newtype: tipo distinto, segurança extra.
//
// Situação atual:
//   Mantemos aliases para velocidade de desenvolvimento e rollback simples.
//   ExchangeId permanece enum canônico vindo de `markets`.
//   AssetNameExchange / InstrumentNameExchange representam símbolos brutos de entrada.
//   AssetIndex / InstrumentIndex representam identificadores internos (ainda Strings).
//   AssetKey / InstrumentKey são aliases genéricos para parametrizar estruturas.
//   QuoteAsset identifica o ativo de cotação/fees (ainda alias).
//
// Motivações para migrar futuramente para newtypes:
//   * Segurança: impedir trocar InstrumentIndex por AssetIndex inadvertidamente.
//   * Evolução: mudar representação interna (ex: String -> u32) sem quebrar API externa.
//   * Validação: normalizar símbolos (B3 vs crypto) no construtor.
//   * Performance: hashing mais barato / interning / armazenamento compacto.
//
// Eixos de classificação usados para cada tipo:
//   Dimensão  : Exchange | Asset | Instrument | Quote
//   Natureza  : Nome externo (NameExchange) | Índice interno (Index) | Id enum | Chave genérica (Key)
//   Fluxo     : Entrada (APIs/streams) | Núcleo (engine/state) | Saída (analytics)
//
// Fases de migração sugeridas:
//   F0: (atual) aliases = String.
//   F1: Introduzir *Symbol newtypes (AssetSymbol, InstrumentSymbol) para NameExchange.
//   F2: Introduzir AssetIndex/InstrumentIndex newtypes leves (String inside) com From<&str>.
//   F3: Trocar mapas internos para usar newtypes como chave.
//   F4: Otimizar representação (u32 / NonZeroU32) mantendo API pública.
//   F5: (Opcional) Escopo por exchange: struct AssetId { exchange: ExchangeId, index: AssetIndex }.
//
// Critérios antes de migrar cada alias:
//   1. Definir unicidade (global ou por exchange).
//   2. Garantir round-trip (name -> index -> name) testado.
//   3. Centralizar conversões (map.rs / indexer).
//   4. Escrever invariantes em doc comments.
//
// Próximos passos (quando decidido migrar):
//   * Criar módulo definitivo (ex: execution::types) com newtypes.
//   * Substituir usos nos mapas e indexer primeiro (ponto de menor superfície pública).
//   * Ajustar traits de cliente (ExecutionClient) adicionando bounds genéricas se necessário.
//   * Remover gradualmente aliases do compat.rs quando cobertura >90%.
//
// Este bloco serve como referência rápida para futuras refactors; manter atualizado conforme decisões.
