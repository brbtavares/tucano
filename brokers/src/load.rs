// Mini-Disclaimer: Educational/experimental use; not investment advice or affiliation; see README & DISCLAIMER.
//! Carregamento opcional de brokers via JSON (feature `json`).
//! Formato esperado:
//! {
//!   "brokers": [ { "id": "xp", "code": "XP", "name": "XP INVESTIMENTOS CCTVM S/A", "certifications": [...], "cost_model": { "default": {"fixed":0,"rate_gross":0,"per_contract":0}, "instrument_overrides": {"WINZ24": {"fixed":1,"rate_gross":0.0005,"per_contract":0}} } } ]
//! }

use crate::{BrokerMetadata, BrokerRegistry};
use serde::{Deserialize, Serialize};
use std::{fs::File, io::Read, path::Path};

#[derive(Debug, thiserror::Error)]
pub enum LoadError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Serialize, Deserialize)]
struct BrokerRegistryJson {
    brokers: Vec<BrokerMetadata>,
}

pub fn load_brokers_from_reader<R: Read>(mut r: R) -> Result<BrokerRegistry, LoadError> {
    let mut buf = String::new();
    r.read_to_string(&mut buf)?;
    let parsed: BrokerRegistryJson = serde_json::from_str(&buf)?;
    let mut reg = BrokerRegistry::new();
    for b in parsed.brokers {
        reg.insert(b);
    }
    Ok(reg)
}

pub fn load_brokers_from_file<P: AsRef<Path>>(path: P) -> Result<BrokerRegistry, LoadError> {
    let f = File::open(path)?;
    load_brokers_from_reader(f)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BrokerCertification, CertificationRecord, CostFormula, CostModel};
    use chrono::{TimeDelta, Utc};
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;

    #[test]
    fn load_round_trip() {
        let now = Utc::now();
        let meta = BrokerMetadata {
            id: "xp".into(),
            code: Some("XP".into()),
            name: crate::BrokerName("XP INVESTIMENTOS".into()),
            certifications: vec![CertificationRecord {
                certification: BrokerCertification::PqoB3,
                valid_from: now - TimeDelta::days(1),
                valid_to: None,
            }],
            cost_model: CostModel {
                default: CostFormula {
                    fixed: dec!(1),
                    rate_gross: dec!(0.001),
                    per_contract: Decimal::ZERO,
                },
                instrument_overrides: Default::default(),
            },
        };
        let json = serde_json::json!({"brokers":[meta]});
        let reg = load_brokers_from_reader(json.to_string().as_bytes()).unwrap();
        let xp = reg.get("xp").unwrap();
        assert_eq!(xp.code.as_deref(), Some("XP"));
        assert_eq!(xp.cost_model.default.fixed, dec!(1));
    }
}
