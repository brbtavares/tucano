//! # 📊 Métricas Financeiras
//!
//! Módulo contendo implementações de métricas essenciais para análise quantitativa
//! de estratégias de trading e avaliação de performance de portfólio.
//!
//! ## 🎯 Métricas Disponíveis
//!
//! ### Métricas de Risco-Retorno
//! - **Sharpe Ratio**: Retorno ajustado ao risco total
//! - **Sortino Ratio**: Retorno ajustado ao downside risk
//! - **Calmar Ratio**: Retorno anualizado dividido pelo máximo drawdown
//!
//! ### Métricas de Performance
//! - **Rate of Return**: Taxa de retorno em diferentes períodos
//! - **Win Rate**: Percentual de trades vencedores
//! - **Profit Factor**: Relação entre lucros e prejuízos
//!
//! ### Métricas de Risco
//! - **Drawdown**: Análise de perdas temporárias máximas e médias
//!
//! ## 💡 Exemplo de Uso
//!
//! Cálculo do Sharpe Ratio (estatísticas fictícias) e geração de drawdown usando `DrawdownGenerator`:
//!
//! ```rust
//! use analytics::metric::sharpe::SharpeRatio;
//! use analytics::metric::drawdown::DrawdownGenerator;
//! use rust_decimal_macros::dec;
//! use chrono::{DateTime, Utc};
//!
//! // Estatísticas (exemplo)
//! let risk_free = dec!(0.0015);
//! let mean_ret  = dec!(0.0025);
//! let std_dev   = dec!(0.02);
//! let interval = chrono::TimeDelta::hours(2);
//! let sharpe = SharpeRatio::calculate(risk_free, mean_ret, std_dev, interval);
//! assert!(sharpe.value != rust_decimal::Decimal::ZERO);
//!
//! // Exemplo mínimo de uso do DrawdownGenerator
//! let t0 = DateTime::<Utc>::MIN_UTC;
//! let mut gen = DrawdownGenerator::init(dec!(100), t0);
//! // valor cai (gera drawdown interno, mas não emite ainda)
//! gen.update(dec!(90), t0 + chrono::TimeDelta::days(1));
//! // valor volta acima do pico → emite drawdown
//! let dd = gen.update(dec!(120), t0 + chrono::TimeDelta::days(2));
//! assert!(dd.is_some());
//! Cálculo simples do Sharpe Ratio com estatísticas agregadas do período (valores hipotéticos).
//!
//! ```rust
//! use analytics::metric::sharpe::SharpeRatio;
//! use analytics::time::Daily;
//! use rust_decimal_macros::dec;
//!
//! let risk_free = dec!(0.0015); // 0.15%
//! let mean_ret  = dec!(0.0025); // 0.25%
//! let std_dev   = dec!(0.0200); // 2.00%
//!
//! let sharpe = SharpeRatio::calculate(risk_free, mean_ret, std_dev, Daily);
//! assert_eq!(sharpe.value, dec!(0.05));
//! ```

/// Lógica de cálculo do Calmar Ratio.
///
/// O Calmar Ratio é uma métrica que mede o retorno anualizado dividido
/// pelo máximo drawdown, fornecendo uma medida de retorno ajustado ao
/// risco de cauda (tail risk).
pub mod calmar;

/// Lógica de cálculo de Drawdown.
///
/// Drawdown representa a perda temporária de valor de um portfólio,
/// medindo a maior queda de um pico até um vale. Inclui cálculos de
/// máximo drawdown e drawdown médio.
pub mod drawdown;

/// Lógica de cálculo do Profit Factor.
///
/// Profit Factor é a relação entre o lucro bruto total e a perda bruta
/// total, indicando quantos reais de lucro são gerados para cada real
/// de perda na estratégia.
pub mod profit_factor;

/// Lógica de cálculo da Taxa de Retorno.
///
/// Calcula retornos em diferentes bases temporais (diário, mensal, anual)
/// e permite análise de performance em vários horizontes de tempo.
pub mod rate_of_return;

/// Lógica de cálculo do Sharpe Ratio.
///
/// O Sharpe Ratio mede o excesso de retorno por unidade de risco,
/// sendo uma das métricas mais utilizadas para avaliar estratégias
/// de investimento ajustadas ao risco.
pub mod sharpe;

/// Lógica de cálculo do Sortino Ratio.
///
/// Similar ao Sharpe Ratio, mas considera apenas o desvio padrão
/// dos retornos negativos (downside deviation), focando no risco
/// de perdas em vez de volatilidade total.
pub mod sortino;

/// Lógica de cálculo da Win Rate.
///
/// Win Rate é o percentual de trades que resultaram em lucro,
/// uma métrica fundamental para avaliar a precisão de uma
/// estratégia de trading.
pub mod win_rate;
