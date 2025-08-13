use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Representa a taxa de acerto (win rate) entre 0 e 1, calculada como `wins/total`.
///
/// Calculada como a razão absoluta de trades vencedores sobre o total.
///
/// Retorna None se não há trades (total = 0) ou se a divisão overflow.
///
/// Referência: <https://www.investopedia.com/terms/w/win-loss-ratio.asp>
#[derive(Debug, Clone, PartialEq, PartialOrd, Default, Deserialize, Serialize)]
pub struct WinRate {
    pub value: Decimal,
}

impl WinRate {
    /// Calcula o [`WinRate`] a partir do número de vitórias e total de posições.
    pub fn calculate(wins: Decimal, total: Decimal) -> Option<Self> {
        if total == Decimal::ZERO {
            None
        } else {
            let value = wins.abs().checked_div(total.abs())?;
            Some(Self { value })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_win_rate_calculate() {
        // no trades
        assert_eq!(WinRate::calculate(Decimal::ZERO, Decimal::ZERO), None);

        // all winning trades
        assert_eq!(
            WinRate::calculate(Decimal::TEN, Decimal::TEN)
                .unwrap()
                .value,
            Decimal::ONE
        );

        // no winning trades
        assert_eq!(
            WinRate::calculate(Decimal::ZERO, Decimal::TEN)
                .unwrap()
                .value,
            Decimal::ZERO
        );

        // mixed winning and losing trades
        assert_eq!(
            WinRate::calculate(dec!(6), Decimal::TEN).unwrap().value,
            dec!(0.6)
        );
    }
}
