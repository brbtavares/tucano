#![forbid(unsafe_code)]
#![warn(
    unused,
    clippy::cognitive_complexity,
    unused_crate_dependencies,
    unused_extern_crates,
    clippy::unused_self,
    clippy::useless_let_if_seq,
    missing_debug_implementations,
    rust_2018_idioms
)]
#![allow(clippy::type_complexity, clippy::too_many_arguments, type_alias_bounds)]

//! # 🧩 Toucan Strategy - Framework de Estratégias de Trading
//!
//! Interfaces e implementações de estratégias para o ecossistema de trading Toucan.
//! Fornece as estratégias fundamentais e implementações padrão que podem ser
//! utilizadas com o engine de trading Toucan.
//!
//! ## 🎯 Estratégias Principais
//!
//! - [`AlgoStrategy`] - Geração de ordens algorítmicas baseadas em sinais
//! - [`ClosePositionsStrategy`] - Fechamento automático de posições abertas  
//! - [`OnDisconnectStrategy`] - Tratamento de desconexões de exchanges
//! - [`OnTradingDisabledStrategy`] - Ações quando trading é desabilitado
//!
//! ## 🏗️ Arquitetura de Estratégias
//!
//! ### Filosofia Modular
//! Estratégias no ecossistema Toucan são componentes plugáveis que podem ser
//! combinados com diferentes engines, gestores de risco e sistemas de execução.
//! Cada tipo de estratégia lida com um aspecto específico da lógica de trading.
//!
//! ### Composição de Estratégias
//! ```rust,no_run
//! use strategy::{
//!     algo::AlgoStrategy,
//!     close_positions::ClosePositionsStrategy,
//!     on_disconnect::OnDisconnectStrategy
//! };
//! 
//! struct MyTradingStrategy {
//!     algo: MyAlgoStrategy,
//!     risk_management: RiskStrategy,
//!     emergency: EmergencyStrategy,
//! }
//! ```
//!
//! ## 🚀 Tipos de Estratégia
//!
//! ### Estratégias Algorítmicas
//! Geram sinais de compra/venda baseados em análise técnica ou fundamentalista:
//! - **Mean Reversion**: Reversão à média
//! - **Momentum**: Seguimento de tendência  
//! - **Arbitrage**: Arbitragem entre mercados
//! - **Market Making**: Provisão de liquidez
//!
//! ### Estratégias de Gestão de Risco
//! Controlam exposição e limitam perdas:
//! - **Stop Loss**: Corte automático de perdas
//! - **Position Sizing**: Dimensionamento de posição
//! - **Exposure Limits**: Limites de exposição
//! - **Correlation Checks**: Verificação de correlação
//!
//! ### Estratégias de Contingência
//! Respondem a eventos excepcionais:
//! - **Disconnect Handling**: Desconexão de exchanges
//! - **Trading Halt**: Paralisação do trading
//! - **Emergency Exit**: Saída de emergência
//! - **Risk Breach**: Violação de limites de risco
//!
//! ## 💡 Exemplo de Implementação
//!
//! ```rust,no_run
//! use strategy::algo::AlgoStrategy;
//! use core::engine::EngineState;
//! 
//! struct MovingAverageStrategy {
//!     short_period: usize,
//!     long_period: usize,
//! }
//! 
//! impl AlgoStrategy for MovingAverageStrategy {
//!     fn generate_orders(&mut self, state: &EngineState) -> Vec<Order> {
//!         // Implementar lógica de médias móveis
//!         vec![]
//!     }
//! }
//! ```
//!
//! ## 🔄 Ciclo de Vida da Estratégia
//!
//! 1. **Inicialização**: Setup de parâmetros e estado inicial
//! 2. **Processamento**: Análise de dados de mercado
//! 3. **Geração de Sinais**: Criação de ordens baseadas em lógica
//! 4. **Execução**: Envio de ordens para o exchange
//! 5. **Monitoramento**: Acompanhamento de posições e performance

/// Define interface de estratégia para geração de ordens algorítmicas baseadas
/// no `EngineState` atual.
///
/// Estratégias algorítmicas são o coração do sistema de trading, gerando sinais
/// de compra e venda baseados em análise de dados de mercado em tempo real.
pub mod algo;

/// Define interface de estratégia para geração de ordens que fecham posições abertas.
///
/// Essencial para gestão de risco e saída controlada de posições, especialmente
/// em situações de emergência ou fim de sessão de trading.
pub mod close_positions;

/// Define interface de estratégia para ações customizadas em caso de desconexão
/// de exchange.
///
/// Permite definir comportamentos específicos quando a conectividade com o
/// exchange é perdida, como fechamento de posições ou cancelamento de ordens.
pub mod on_disconnect;

/// Define interface de estratégia para ações customizadas quando o estado de
/// trading é definido como `TradingState::Disabled`.
///
/// Importante para conformidade regulatória e gestão de sessões de trading,
/// permitindo ações específicas quando o trading é suspenso.
pub mod on_trading_disabled;

/// Implementações padrão de estratégias.
///
/// Fornece implementações básicas e reutilizáveis das interfaces de estratégia,
/// servindo como ponto de partida para estratégias customizadas.
pub mod default;

// Re-export the main traits for convenience
pub use algo::AlgoStrategy;
pub use close_positions::ClosePositionsStrategy;
pub use on_disconnect::OnDisconnectStrategy;
pub use on_trading_disabled::OnTradingDisabled;
pub use default::DefaultStrategy;
