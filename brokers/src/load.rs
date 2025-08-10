//! Carregamento opcional de brokers via JSON (feature `json`).
//! Formato esperado:
//! {
//!   "brokers": [ { "id": "xp", "code": "XP", "name": "XP INVESTIMENTOS CCTVM S/A", "certifications": [...], "cost_model": { "default": {"fixed":0,"rate_gross":0,"per_contract":0}, "instrument_overrides": {"WINZ24": {"fixed":1,"rate_gross":0.0005,"per_contract":0}} } } ]
//! }

use std::{fs::File, io::Read, path::Path};
use serde::{Deserialize, Serialize};
use crate::{BrokerRegistry, BrokerMetadata};

#[derive(Debug, thiserror::Error)]
pub enum LoadError {
    #[error("io: {0}")] Io(#[from] std::io::Error),
    #[error("json: {0}")] Json(#[from] serde_json::Error),
}

#[derive(Debug, Serialize, Deserialize)]
struct BrokerRegistryJson { brokers: Vec<BrokerMetadata> }

pub fn load_brokers_from_reader<R: Read>(mut r: R) -> Result<BrokerRegistry, LoadError> {
    let mut buf = String::new();
    r.read_to_string(&mut buf)?;
    let parsed: BrokerRegistryJson = serde_json::from_str(&buf)?;
    let mut reg = BrokerRegistry::new();
    for b in parsed.brokers { reg.insert(b); }
    Ok(reg)
}

pub fn load_brokers_from_file<P: AsRef<Path>>(path: P) -> Result<BrokerRegistry, LoadError> {
    let f = File::open(path)?;
    load_brokers_from_reader(f)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;
    use rust_decimal::Decimal;
    use crate::{CostFormula, CostModel, CertificationRecord, BrokerCertification};
    use chrono::{Utc, TimeDelta};

    #[test]
    fn load_round_trip() {
        let now = Utc::now();
        let meta = BrokerMetadata {
            id: "xp".into(),
            code: Some("XP".into()),
            name: crate::BrokerName("XP INVESTIMENTOS".into()),
            certifications: vec![CertificationRecord { certification: BrokerCertification::PqoB3, valid_from: now-TimeDelta::days(1), valid_to: None }],
            cost_model: CostModel { default: CostFormula { fixed: dec!(1), rate_gross: dec!(0.001), per_contract: Decimal::ZERO }, instrument_overrides: Default::default() }
        };
        let json = serde_json::json!({"brokers":[meta]});
        let reg = load_brokers_from_reader(json.to_string().as_bytes()).unwrap();
        let xp = reg.get("xp").unwrap();
        assert_eq!(xp.code.as_deref(), Some("XP"));
        assert_eq!(xp.cost_model.default.fixed, dec!(1));
    }
}
