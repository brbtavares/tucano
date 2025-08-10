//! Broker abstraction layer
//!
//! Fornece tipos para identificar corretoras, associar metadados de certificação
//! e modelos de custo aplicáveis a instrumentos negociados. Esta camada é
//! independente de `markets` (instrumentos) e `execution` (ordens) para permitir
//! evolução incremental. Futuramente poderemos ligar Brokers a Accounts e
//! Exchanges.

pub mod model;
pub mod registry;
#[cfg(feature = "json")]
pub mod load;

pub use model::*;
pub use registry::*;

use rust_decimal::Decimal;

/// Aplica custo do broker para (instrument, valor bruto, quantidade) usando o modelo fornecido.
pub fn apply_broker_cost(model: &CostModel, instrument: &str, order_value: Decimal, qty_contracts: Decimal) -> Decimal {
	model.apply(&instrument.to_string(), order_value, qty_contracts)
}

#[cfg(test)]
mod integration_tests {
	use super::*;
	use rust_decimal_macros::dec;
	use chrono::Utc;

	#[test]
	fn cost_model_applied_to_certified_registry() {
		let now = Utc::now();
		let mut reg = certified_b3_brokers(now);
		// Pick one broker and attach a cost override
		let target_id = "xpinvestimentoscctvmsa"; // XP slug
		let meta = reg.get(target_id).expect("xp broker present").clone();
		let mut custom = meta.clone();
		custom.cost_model = CostModel {
			default: CostFormula { fixed: dec!(1), rate_gross: dec!(0.0), per_contract: dec!(0) },
			instrument_overrides: {
				let mut m = indexmap::IndexMap::new();
				m.insert("WINZ25".into(), CostFormula { fixed: dec!(0), rate_gross: dec!(0.0005), per_contract: dec!(0) });
				m
			},
		};
		// Replace
		reg.insert(custom.clone());
		let win_cost = custom.cost_model.cost(&"WINZ25".into(), dec!(100_000), dec!(1)); // 0.0005 * 100k = 50
		assert_eq!(win_cost, dec!(50));
		let other_cost = custom.cost_model.cost(&"PETR4".into(), dec!(10_000), dec!(1)); // default fixed 1
		assert_eq!(other_cost, dec!(1));
	}
}
