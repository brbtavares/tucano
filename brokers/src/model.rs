use chrono::{DateTime, Utc};
use indexmap::IndexMap;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use strum::{EnumIter, EnumString, AsRefStr, Display};

pub type BrokerId = String;        // slug interno (ex: "xp", "atva")
pub type BrokerCode = String;      // código operacional B3 (quando existir)
pub type InstrumentKey = String;   // reutiliza chave de instrumento global

/// Nome oficial conforme publicação B3 / regulador
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BrokerName(pub String);
impl From<&str> for BrokerName { fn from(s: &str) -> Self { Self(s.to_string()) } }
impl AsRef<str> for BrokerName { fn as_ref(&self) -> &str { &self.0 } }
impl std::fmt::Display for BrokerName { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.write_str(&self.0) } }

/// Certificações / selos relevantes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, EnumIter, EnumString, AsRefStr, Display)]
#[strum(serialize_all = "snake_case")]
pub enum BrokerCertification {
    PqoB3, // Selo PQO B3
}

/// Marca que um broker possui determinada certificação em determinada data
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CertificationRecord {
    pub certification: BrokerCertification,
    pub valid_from: DateTime<Utc>,
    pub valid_to: Option<DateTime<Utc>>, // None => vigente
}

/// Estrutura simples para custos de transação.
/// Modelo extensível: custo fixo + percentual sobre valor + por contrato.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct CostFormula {
    pub fixed: Decimal,          // custo fixo por ordem
    pub rate_gross: Decimal,     // % sobre valor bruto negociado (ex: 0.0015 => 0.15%)
    pub per_contract: Decimal,   // custo por contrato / unidade (se aplicável)
}
impl CostFormula {
    pub fn cost(&self, gross_value: Decimal, contracts: Decimal) -> Decimal {
        self.fixed + (self.rate_gross * gross_value) + (self.per_contract * contracts)
    }
}

/// Modelo de custos diferenciados por instrumento (ou família).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct CostModel {
    pub default: CostFormula,
    pub instrument_overrides: IndexMap<InstrumentKey, CostFormula>,
}
impl CostModel {
    pub fn cost(&self, instrument: &InstrumentKey, gross_value: Decimal, contracts: Decimal) -> Decimal {
        self.instrument_overrides
            .get(instrument)
            .unwrap_or(&self.default)
            .cost(gross_value, contracts)
    }
}

/// Metadados completos de uma corretora
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BrokerMetadata {
    pub id: BrokerId,
    pub code: Option<BrokerCode>,
    pub name: BrokerName,
    pub certifications: Vec<CertificationRecord>,
    pub cost_model: CostModel,
}
impl BrokerMetadata {
    pub fn new(id: BrokerId, code: Option<BrokerCode>, name: impl Into<BrokerName>) -> Self {
        Self { id, code, name: name.into(), certifications: Vec::new(), cost_model: CostModel::default() }
    }
    pub fn with_cost_model(mut self, cost_model: CostModel) -> Self { self.cost_model = cost_model; self }
    pub fn add_certification(mut self, certification: CertificationRecord) -> Self { self.certifications.push(certification); self }
}

/// Registro de corretoras
#[derive(Debug, Default)]
pub struct BrokerRegistry {
    by_id: IndexMap<BrokerId, BrokerMetadata>,
}
impl BrokerRegistry {
    pub fn new() -> Self { Self { by_id: IndexMap::new() } }
    pub fn insert(&mut self, meta: BrokerMetadata) { self.by_id.insert(meta.id.clone(), meta); }
    pub fn get(&self, id: &str) -> Option<&BrokerMetadata> { self.by_id.get(id) }
    pub fn iter(&self) -> impl Iterator<Item = (&BrokerId, &BrokerMetadata)> { self.by_id.iter() }
}
