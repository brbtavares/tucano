use crate::model::*;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

/// Helper to build a PQO certification record valid from `now` (open-ended)
fn pqo(now: DateTime<Utc>) -> CertificationRecord {
    CertificationRecord {
        certification: BrokerCertification::PqoB3,
        valid_from: now,
        valid_to: None,
    }
}

/// Canonical slug generator (very naive: lowercase alnum)
fn slug(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .map(|c| c.to_ascii_lowercase())
        .collect()
}

/// Conjunto inicial de corretoras certificadas (lista parcial; pode ser expandida).
pub fn certified_b3_brokers(now: DateTime<Utc>) -> BrokerRegistry {
    let names = [
        "AGORA CTVM S/A",
        "ATIVA INVESTIMENTOS S.A. CTCV",
        "BANRISUL S/A CVMC",
        "BB BANCO DE INVESTIMENTO S/A",
        "BGC LIQUIDEZ DTVM",
        "BRADESCO S/A CTVM",
        "BTG PACTUAL CTVM S.A.",
        "C6 CTVM LTDA",
        "CITIGROUP GMB CCTVM S.A.",
        "CLEAR CORRETORA - GRUPO XP",
        "CM CAPITAL MARKETS CCTVM LTDA",
        "CREDIT SUISSE BRASIL S.A. CTVM",
        "GENIAL INSTITUCIONAL CCTVM S.A.",
        "GENIAL INVESTIMENTOS CVM S.A.",
        "GOLDMAN SACHS DO BRASIL CTVM",
        "GUIDE INVESTIMENTOS S.A. CV",
        "ICAP DO BRASIL CTVM LTDA",
        "IDEAL CTVM SA",
        "INTER DTVM LTDA",
        "ITAU CV S/A",
        "J. P. MORGAN CCVM S.A.",
        "LEV DTVM",
        "MERRILL LYNCH S/A CTVM",
        "MIRAE ASSET WEALTH MANAGEMENT (BRASIL) CCTVM LTDA",
        "MORGAN STANLEY CTVM S/A",
        "NECTON INVESTIMENTOS S.A. CVMC",
        "NOVA FUTURA CTVM LTDA",
        "NU INVEST CORRETORA DE VALORES S.A",
        "PLANNER CV S.A",
        "RB CAPITAL DTVM LTDA",
        "RENASCENÇA DTVM LTDA",
        "SAFRA DTVM LTDA",
        "SANTANDER CCVM S/A",
        "SCOTIABANK BRASIL S.A. CTVM",
        "STONEX DISTRIBUIDORA DE TITULOS E VALORES MOBILIARIOS LTDA",
        "TERRA INVESTIMENTOS DTVM LTDA",
        "TORO CVTM LTDA",
        "TULLETT PREBON BRASIL CVC LTDA",
        "UBS BRASIL CCTVM S/A",
        "XP INVESTIMENTOS CCTVM S/A",
    ];

    let mut reg = BrokerRegistry::new();
    for name in names {
        let id = slug(name);
        let code_hint = match name {
            "XP INVESTIMENTOS CCTVM S/A" => Some("XP"),
            "BTG PACTUAL CTVM S.A." => Some("BTG"),
            "ITAU CV S/A" => Some("ITAU"),
            "BRADESCO S/A CTVM" => Some("BBDC"),
            "SANTANDER CCVM S/A" => Some("SAN"),
            "SAFRA DTVM LTDA" => Some("SAFRA"),
            _ => None,
        }
        .map(|c| c.to_string());
        let meta = BrokerMetadata::new(id, code_hint, BrokerName(name.to_string()))
            .add_certification(pqo(now))
            .with_cost_model(CostModel {
                default: CostFormula {
                    fixed: Decimal::ZERO,
                    rate_gross: Decimal::ZERO,
                    per_contract: Decimal::ZERO,
                },
                instrument_overrides: Default::default(),
            });
        reg.insert(meta);
    }
    reg
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn certified_b3_brokers_populates_registry() {
        let now = Utc::now();
        let reg = certified_b3_brokers(now);
        assert!(reg.iter().count() > 10, "expected many certified brokers");
        assert!(
            reg.get("xpinvestimentoscctvmsa").is_some(),
            "XP slug missing"
        );
        for (_id, meta) in reg.iter() {
            assert!(!meta.name.as_ref().is_empty());
            assert!(
                !meta.certifications.is_empty(),
                "broker missing certification record"
            );
        }
    }
}
