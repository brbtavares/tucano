// Mini-Disclaimer: Educational/experimental use; not investment advice or affiliation; see README & DISCLAIMER.
use chrono::{DateTime, Utc};
use indexmap::IndexMap;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display, EnumIter, EnumString};

pub type BrokerId = String; // slug interno (ex: "xp", "atva")
pub type BrokerCode = String; // código operacional B3 (quando existir)
pub type InstrumentKey = String; // reutiliza chave de instrumento global

/// Nome oficial conforme publicação B3 / regulador
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BrokerName(pub String);
impl From<&str> for BrokerName {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}
impl AsRef<str> for BrokerName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
impl std::fmt::Display for BrokerName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// Certificações / selos relevantes
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    EnumIter,
    EnumString,
    AsRefStr,
    Display,
)]
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
    pub fixed: Decimal,        // custo fixo por ordem
    pub rate_gross: Decimal,   // % sobre valor bruto negociado (ex: 0.0015 => 0.15%)
    pub per_contract: Decimal, // custo por contrato / unidade (se aplicável)
}
impl CostFormula {
    pub fn cost(&self, gross_value: Decimal, contracts: Decimal) -> Decimal {
        self.fixed + (self.rate_gross * gross_value) + (self.per_contract * contracts)
    }
    /// Aplica custo a um valor de ordem (sinônimo de cost para semântica de domínio).
    pub fn apply(&self, order_value: Decimal, qty_contracts: Decimal) -> Decimal {
        self.cost(order_value, qty_contracts)
    }
}

/// Modelo de custos diferenciados por instrumento (ou família).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct CostModel {
    pub default: CostFormula,
    pub instrument_overrides: IndexMap<InstrumentKey, CostFormula>,
}
impl CostModel {
    pub fn cost(
        &self,
        instrument: &InstrumentKey,
        gross_value: Decimal,
        contracts: Decimal,
    ) -> Decimal {
        self.instrument_overrides
            .get(instrument)
            .unwrap_or(&self.default)
            .cost(gross_value, contracts)
    }

    /// Aplica custo diretamente (alias de cost) para semântica.
    pub fn apply(
        &self,
        instrument: &InstrumentKey,
        order_value: Decimal,
        qty_contracts: Decimal,
    ) -> Decimal {
        self.cost(instrument, order_value, qty_contracts)
    }

    /// Insere/atualiza override para um instrumento.
    pub fn with_override(
        mut self,
        instrument: impl Into<InstrumentKey>,
        formula: CostFormula,
    ) -> Self {
        self.instrument_overrides.insert(instrument.into(), formula);
        self
    }

    /// Verdadeiro se não há qualquer custo configurado.
    pub fn is_zero(&self) -> bool {
        let z = Decimal::ZERO;
        self.default.fixed == z
            && self.default.rate_gross == z
            && self.default.per_contract == z
            && self
                .instrument_overrides
                .values()
                .all(|f| f.fixed == z && f.rate_gross == z && f.per_contract == z)
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
        Self {
            id,
            code,
            name: name.into(),
            certifications: Vec::new(),
            cost_model: CostModel::default(),
        }
    }
    pub fn with_cost_model(mut self, cost_model: CostModel) -> Self {
        self.cost_model = cost_model;
        self
    }
    pub fn add_certification(mut self, certification: CertificationRecord) -> Self {
        self.certifications.push(certification);
        self
    }
}

/// Registro de corretoras
#[derive(Debug, Default)]
pub struct BrokerRegistry {
    by_id: IndexMap<BrokerId, BrokerMetadata>,
}
impl BrokerRegistry {
    pub fn new() -> Self {
        Self {
            by_id: IndexMap::new(),
        }
    }
    pub fn insert(&mut self, meta: BrokerMetadata) {
        self.by_id.insert(meta.id.clone(), meta);
    }
    pub fn get(&self, id: &str) -> Option<&BrokerMetadata> {
        self.by_id.get(id)
    }
    pub fn iter(&self) -> impl Iterator<Item = (&BrokerId, &BrokerMetadata)> {
        self.by_id.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeDelta;
    use rust_decimal_macros::dec;

    #[test]
    fn cost_formula_basic() {
        let f = CostFormula {
            fixed: dec!(2),
            rate_gross: dec!(0.001),
            per_contract: dec!(0.5),
        };
        let gross = dec!(10_000); // 10k * 0.1% = 10
        let contracts = dec!(3); // 3 * 0.5 = 1.5
        assert_eq!(f.cost(gross, contracts), dec!(2) + dec!(10) + dec!(1.5));
    }

    #[test]
    fn cost_model_override() {
        let mut cm = CostModel {
            default: CostFormula {
                fixed: dec!(1),
                rate_gross: dec!(0),
                per_contract: dec!(0),
            },
            instrument_overrides: IndexMap::new(),
        };
        cm.instrument_overrides.insert(
            "instA".into(),
            CostFormula {
                fixed: dec!(0),
                rate_gross: dec!(0.002),
                per_contract: dec!(0),
            },
        );
        // instA override
        assert_eq!(cm.cost(&"instA".into(), dec!(1000), dec!(1)), dec!(2));
        // instB fallback default
        assert_eq!(cm.cost(&"instB".into(), dec!(1000), dec!(1)), dec!(1));
    }

    #[test]
    fn broker_metadata_builders() {
        let meta = BrokerMetadata::new("xp".into(), Some("123".into()), "XP INVESTIMENTOS")
            .add_certification(CertificationRecord {
                certification: BrokerCertification::PqoB3,
                valid_from: Utc::now(),
                valid_to: None,
            })
            .with_cost_model(CostModel {
                default: CostFormula::default(),
                instrument_overrides: IndexMap::new(),
            });
        assert_eq!(meta.id, "xp");
        assert_eq!(meta.code.as_deref(), Some("123"));
        assert!(meta
            .certifications
            .iter()
            .any(|c| c.certification == BrokerCertification::PqoB3));
    }

    #[test]
    fn certification_record_validity() {
        let now = Utc::now();
        let rec = CertificationRecord {
            certification: BrokerCertification::PqoB3,
            valid_from: now - TimeDelta::days(10),
            valid_to: None,
        };
        assert!(rec.valid_from < now);
        assert!(rec.valid_to.is_none());
    }
}
