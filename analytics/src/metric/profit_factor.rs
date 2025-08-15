// Mini-Disclaimer: Uso educacional/experimental; sem recomendação de investimento ou afiliação; sem remuneração de terceiros; Profit/ProfitDLL © Nelógica; veja README & DISCLAIMER.
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// ProfitFactor é uma métrica de performance que divide o valor absoluto dos lucros brutos
/// pelo valor absoluto das perdas brutas. Valor > 1 indica estratégia lucrativa.
///
/// Casos especiais:
/// - Retorna 1.0 quando lucros e perdas são zero (neutro)
/// - Retorna INFINITO quando há lucros e nenhuma perda (perfeito)
/// - Retorna -INFINITO quando há perdas e nenhum lucro (pior caso)
///
/// Referência: <https://www.investopedia.com/articles/fundamental-analysis/10/strategy-performance-reports.asp#toc-profit-factor>
#[derive(Debug, Clone, PartialEq, PartialOrd, Default, Deserialize, Serialize)]
pub struct ProfitFactor {
    pub value: Decimal,
}

impl ProfitFactor {
    /// Calcula o [`ProfitFactor`] a partir dos lucros e perdas brutos (valores absolutos).
    pub fn calculate(profits_gross_abs: Decimal, losses_gross_abs: Decimal) -> Option<Self> {
        if profits_gross_abs.is_zero() && losses_gross_abs.is_zero() {
            return None;
        }

        let value = if losses_gross_abs.is_zero() {
            Decimal::MAX
        } else if profits_gross_abs.is_zero() {
            Decimal::MIN
        } else {
            profits_gross_abs
                .abs()
                .checked_div(losses_gross_abs.abs())?
        };

        Some(Self { value })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;
    use std::str::FromStr;

    #[test]
    fn test_profit_factor_calculate() {
        // both profits & losses are very small
        assert_eq!(
            ProfitFactor::calculate(
                Decimal::from_scientific("1e-20").unwrap(),
                Decimal::from_scientific("1e-20").unwrap()
            )
            .unwrap()
            .value,
            Decimal::ONE
        );

        // both profits & losses are very large
        assert_eq!(
            ProfitFactor::calculate(Decimal::MAX / dec!(2), Decimal::MAX / dec!(2))
                .unwrap()
                .value,
            Decimal::ONE
        );

        // both profits & losses are zero
        assert_eq!(ProfitFactor::calculate(dec!(0.0), dec!(0.0)), None);

        // profits are zero
        assert_eq!(
            ProfitFactor::calculate(dec!(0.0), dec!(1.0)).unwrap().value,
            Decimal::MIN
        );

        // losses are zero
        assert_eq!(
            ProfitFactor::calculate(dec!(1.0), dec!(0.0)).unwrap().value,
            Decimal::MAX
        );

        // both profits & losses are non-zero
        assert_eq!(
            ProfitFactor::calculate(dec!(10.0), dec!(5.0))
                .unwrap()
                .value,
            dec!(2.0)
        );

        // both profits & losses are non-zero, but input losses are not abs
        assert_eq!(
            ProfitFactor::calculate(dec!(10.0), dec!(-5.0))
                .unwrap()
                .value,
            dec!(2.0)
        );

        // test with precise decimal values
        assert_eq!(
            ProfitFactor::calculate(dec!(10.5555), dec!(5.2345))
                .unwrap()
                .value,
            Decimal::from_str("2.016524978507975928933040405").unwrap()
        );
    }
}
