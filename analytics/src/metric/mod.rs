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
//! ```rust,no_run
//! use analytics::metric::{sharpe::SharpeRatio, drawdown::MaxDrawdown};
//! use analytics::time::Annual252;
//!
//! // Calcular Sharpe Ratio
//! let sharpe = SharpeRatio::calculate(&returns, &Annual252, 0.02)?;
//!
//! // Calcular Maximum Drawdown
//! let max_dd = MaxDrawdown::calculate(&portfolio_values)?;
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
