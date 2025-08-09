//! # 🧠 Estratégias Algorítmicas
//!
//! Define a interface principal para estratégias algorítmicas no framework Toucan.
//! Estratégias implementam lógica de trading baseada no estado atual do sistema,
//! gerando ordens de abertura e cancelamento de forma automatizada.

use execution::{
    order::request::{OrderRequestCancel, OrderRequestOpen},
    ExchangeIndex,
    InstrumentIndex,
};

/// Interface de estratégia para geração de ordens algorítmicas baseadas no
/// `EngineState` atual.
///
/// Esta trait define o contrato principal para todas as estratégias algorítmicas
/// no ecossistema Toucan. Implementações desta trait analisam o estado atual
/// do sistema e geram requests de ordens (abertura e cancelamento) baseados
/// em sua lógica específica.
///
/// # Filosofia de Design
///
/// - **Stateless**: A estratégia não mantém estado interno, recebe estado completo
/// - **Pure Functions**: Mesma entrada sempre produz mesma saída
/// - **Composable**: Estratégias podem ser combinadas e reutilizadas
/// - **Testable**: Fácil de testar por ser determinística
///
/// # Parâmetros de Tipo
/// * `ExchangeKey` - Tipo usado para identificar um exchange (padrão: [`ExchangeIndex`])
/// * `InstrumentKey` - Tipo usado para identificar um instrumento (padrão: [`InstrumentIndex`])
///
/// # Exemplo de Implementação
///
/// ```rust,no_run
/// use strategy::algo::AlgoStrategy;
/// use core::engine::EngineState;
/// 
/// struct MovingAverageStrategy {
///     short_period: usize,
///     long_period: usize,
/// }
/// 
/// impl AlgoStrategy for MovingAverageStrategy {
///     type State = EngineState<MyGlobalData, MyInstrumentData>;
///     
///     fn generate_algo_orders(&self, state: &Self::State) -> (Vec<OrderRequestCancel>, Vec<OrderRequestOpen>) {
///         let mut cancel_orders = Vec::new();
///         let mut open_orders = Vec::new();
///         
///         // Implementar lógica de médias móveis
///         for (instrument, data) in state.market_data.iter() {
///             if should_buy(data) {
///                 open_orders.push(OrderRequestOpen::market_buy(instrument, 100.0));
///             } else if should_sell(data) {
///                 open_orders.push(OrderRequestOpen::market_sell(instrument, 100.0));
///             }
///         }
///         
///         (cancel_orders, open_orders)
///     }
/// }
/// ```
///
/// # Tipos de Estratégia Comum
///
/// ## Estratégias Técnicas
/// - **Moving Average Crossover**: Cruzamento de médias móveis
/// - **RSI Overbought/Oversold**: Baseado no índice de força relativa
/// - **Bollinger Bands**: Bandas de volatilidade
/// - **MACD Signal**: Convergência/divergência de médias
///
/// ## Estratégias Fundamentalistas
/// - **Earnings Momentum**: Baseado em resultados trimestrais
/// - **Value Investing**: Métricas de valor (P/E, P/B, etc.)
/// - **Economic Indicators**: Indicadores macroeconômicos
///
/// ## Estratégias Quantitativas
/// - **Statistical Arbitrage**: Arbitragem estatística
/// - **Mean Reversion**: Reversão à média
/// - **Momentum**: Seguimento de tendência
/// - **Market Making**: Provisão de liquidez
pub trait AlgoStrategy<ExchangeKey = ExchangeIndex, InstrumentKey = InstrumentIndex> {
    /// Estado usado pela `AlgoStrategy` para determinar quais requests de abertura
    /// e cancelamento gerar.
    ///
    /// Para estratégias do ecossistema Toucan, este é o `EngineState` completo
    /// do sistema de trading, contendo todos os dados de mercado, posições,
    /// ordens abertas, e outros dados necessários para tomada de decisão.
    ///
    /// ## Exemplo
    /// ```rust,ignore
    /// type State = EngineState<DefaultGlobalData, DefaultInstrumentMarketData>;
    /// ```
    ///
    /// O estado tipicamente inclui:
    /// - **Market Data**: Preços, volumes, orderbook
    /// - **Portfolio**: Posições atuais e P&L
    /// - **Orders**: Ordens abertas e histórico
    /// - **Account**: Saldos e margem disponível
    /// - **Connectivity**: Status de conexão com exchanges
    type State;

    /// Gera ordens algorítmicas baseadas no `State` atual do sistema.
    ///
    /// Este método é o coração da estratégia, sendo chamado pelo engine
    /// sempre que há atualizações no estado do sistema. Deve analisar o
    /// estado atual e retornar duas listas:
    ///
    /// 1. **Cancel Orders**: Ordens a serem canceladas
    /// 2. **Open Orders**: Novas ordens a serem abertas
    ///
    /// # Parâmetros
    /// - `state`: Estado atual completo do sistema de trading
    ///
    /// # Retorno
    /// Tupla contendo:
    /// - Iterator de requests de cancelamento
    /// - Iterator de requests de abertura
    ///
    /// # Performance
    /// Este método é chamado frequentemente e deve ser otimizado:
    /// - Evitar alocações desnecessárias
    /// - Usar lazy evaluation quando possível
    /// - Cachear cálculos pesados quando apropriado
    ///
    /// # Exemplo
    /// ```rust,no_run
    /// fn generate_algo_orders(&self, state: &Self::State) -> (Vec<CancelRequest>, Vec<OpenRequest>) {
    ///     // Analisar dados de mercado
    ///     let signals = self.analyze_market_data(&state.market_data);
    ///     
    ///     // Gerar orders baseadas nos sinais
    ///     let open_orders = signals.into_iter()
    ///         .filter_map(|signal| self.signal_to_order(signal))
    ///         .collect();
    ///     
    ///     // Cancelar ordens obsoletas
    ///     let cancel_orders = state.open_orders.iter()
    ///         .filter(|order| self.should_cancel_order(order))
    ///         .map(|order| OrderRequestCancel::new(order.id))
    ///         .collect();
    ///         
    ///     (cancel_orders, open_orders)
    /// }
    /// ```
    fn generate_algo_orders(
        &self,
        state: &Self::State,
    ) -> (
        impl IntoIterator<Item = OrderRequestCancel<ExchangeKey, InstrumentKey>>,
        impl IntoIterator<Item = OrderRequestOpen<ExchangeKey, InstrumentKey>>,
    );
}
