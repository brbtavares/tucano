//! # 📊 Analytics - Módulo de Análise Financeira
//!
//! Este módulo fornece ferramentas abrangentes para análise quantitativa de dados financeiros,
//! incluindo métricas de performance, algoritmos estatísticos e geração de relatórios.
//!
//! ## 🎯 Funcionalidades Principais
//!
//! - **Métricas Financeiras**: Sharpe, Sortino, Calmar, Win Rate, Profit Factor
//! - **Análise de Drawdown**: Cálculo de máximo e médio drawdown
//! - **Algoritmos Estatísticos**: Processamento de datasets financeiros
//! - **Relatórios Automatizados**: Geração de sumários e tear sheets
//! - **Intervalos Temporais**: Suporte a diferentes períodos de análise
//!
//! ## 🏗️ Estrutura do Módulo
//!
//! ```
//! analytics/
//! ├── algorithm.rs     # Algoritmos estatísticos para análise de datasets
//! ├── metric/          # Métricas financeiras (Sharpe, Sortino, etc.)
//! ├── summary/         # Relatórios e sumários financeiros
//! └── time.rs          # Definições de intervalos temporais
//! ```
//!
//! ## 📈 Exemplo de Uso
//!
//! ```rust,no_run
//! use analytics::{metric::SharpeRatio, summary::TradingSummary, time::Annual252};
//! use chrono::{DateTime, Utc};
//!
//! // Calcular Sharpe Ratio
//! let sharpe = SharpeRatio::calculate(&returns, &Annual252, risk_free_rate);
//!
//! // Gerar sumário de trading
//! let summary = TradingSummary::generate(&trades, start_time, end_time);
//! ```
//!
//! ## 🔍 Métricas Disponíveis
//!
//! - **Sharpe Ratio**: Retorno ajustado ao risco
//! - **Sortino Ratio**: Sharpe considerando apenas downside risk  
//! - **Calmar Ratio**: Retorno anualizado / máximo drawdown
//! - **Win Rate**: Percentual de trades vencedores
//! - **Profit Factor**: Lucro bruto / prejuízo bruto
//! - **Drawdown**: Análise de perdas máximas e médias

/// Algoritmos estatísticos para análise de datasets financeiros.
///
/// Contém implementações de algoritmos para processamento e análise
/// de dados financeiros, incluindo cálculos de volatilidade, correlação
/// e outras métricas estatísticas fundamentais.
pub mod algorithm;

/// Métricas financeiras e métodos para calculá-las em diferentes
/// [`TimeIntervals`](time::TimeInterval).
///
/// Inclui todas as métricas essenciais para análise quantitativa:
/// Sharpe, Sortino, Calmar ratios, Win Rate, Profit Factor, e análises
/// de drawdown para avaliação de performance de estratégias.
pub mod metric;

/// Sumários estatísticos para datasets financeiros.
///
/// Fornece estruturas para geração de relatórios abrangentes como
/// `TradingSummary`, `TearSheet`, `TearSheetAsset`, `PnLReturns`, etc.
/// Essenciais para análise de performance e relatórios automatizados.
pub mod summary;

/// Definições de intervalos temporais usados em cálculos financeiros.
///
/// Suporta diferentes convenções de tempo financeiro como `Annual365`,
/// `Annual252` (dias úteis), `Daily`, etc. para cálculos precisos
/// de métricas anualizadas e periódicas.
pub mod time;

use chrono::{DateTime, Utc};

/// Trait para tipos que possuem timestamp.
///
/// Define a interface padrão para objetos que carregam informação temporal,
/// essencial para análises baseadas em tempo e ordenação cronológica.
pub trait Timed {
    /// Retorna o timestamp deste item.
    fn timestamp(&self) -> DateTime<Utc>;
}

/// Estrutura wrapper que combina um valor com timestamp.
///
/// Útil para associar dados financeiros com seus timestamps específicos,
/// permitindo análises temporais precisas e ordenação cronológica.
///
/// # Exemplo
/// ```rust
/// use analytics::{TimedValue, Timed};
/// use chrono::Utc;
///
/// let price = TimedValue::new(100.50, Utc::now());
/// println!("Preço: {} em {}", price.value, price.timestamp());
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct TimedValue<T> {
    /// O valor associado ao timestamp
    pub value: T,
    /// Timestamp UTC do valor
    pub timestamp: DateTime<Utc>,
}

impl<T> TimedValue<T> {
    /// Cria um novo `TimedValue` com o valor e timestamp fornecidos.
    pub fn new(value: T, timestamp: DateTime<Utc>) -> Self {
        Self { value, timestamp }
    }
}

impl<T> Timed for TimedValue<T> {
    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }
}

#[cfg(test)]
pub mod test_utils {
    //! Utilitários para testes do módulo analytics.
    //!
    //! Fornece funções auxiliares para criação de dados de teste
    //! e manipulação temporal em cenários de teste.
    
    use chrono::{DateTime, Utc};

    /// Adiciona dias a uma data base para criação de dados de teste.
    ///
    /// Útil para gerar séries temporais de teste com intervalos
    /// específicos entre observações.
    pub fn time_plus_days(base: DateTime<Utc>, plus: u64) -> DateTime<Utc> {
        base + chrono::Duration::days(plus as i64)
    }
}
