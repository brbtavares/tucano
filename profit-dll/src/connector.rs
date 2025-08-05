//! Conector principal para ProfitDLL

use crate::{
    error::ProfitError,
    types::*,
    callbacks::{add_event_handler, CallbackEvent, ChannelEventHandler},
    utils::DllLoader,
};
use parking_lot::RwLock;
use std::sync::Arc;
use tokio::sync::mpsc;
use chrono::{DateTime, Utc};

// Alias para Result com ProfitError como erro padrão
type Result<T> = std::result::Result<T, ProfitError>;

/// Conector principal para interface com ProfitDLL
/// 
/// Este é o ponto de entrada principal para todas as operações
/// da ProfitDLL, incluindo login, market data e trading.
pub struct ProfitConnector {
    dll_loader: DllLoader,
    is_initialized: Arc<RwLock<bool>>,
    is_trading_enabled: Arc<RwLock<bool>>,
}

impl ProfitConnector {
    /// Cria uma nova instância do conector
    /// 
    /// # Argumentos
    /// * `dll_path` - Caminho opcional para a DLL (usa "ProfitDLL.dll" se None)
    /// 
    /// # Exemplo
    /// ```rust,no_run
    /// use profit_dll::ProfitConnector;
    /// 
    /// // Usa DLL no PATH do sistema
    /// let connector = ProfitConnector::new(None)?;
    /// 
    /// // Usa DLL em caminho específico
    /// let connector = ProfitConnector::new(Some("C:\\MyApp\\ProfitDLL.dll"))?;
    /// # Ok::<(), profit_dll::ProfitError>(())
    /// ```
    pub fn new(dll_path: Option<&str>) -> Result<Self> {
        let dll_loader = DllLoader::new(dll_path)?;
        
        Ok(Self {
            dll_loader,
            is_initialized: Arc::new(RwLock::new(false)),
            is_trading_enabled: Arc::new(RwLock::new(false)),
        })
    }

    /// Inicializa a DLL com login completo (routing + market data)
    /// 
    /// Esta função habilita tanto market data quanto trading.
    /// 
    /// # Argumentos
    /// * `activation_key` - Chave de ativação fornecida
    /// * `user` - Nome do usuário
    /// * `password` - Senha do usuário
    /// 
    /// # Retorno
    /// Retorna um receiver para eventos assíncronos da DLL
    /// 
    /// # Exemplo
    /// ```rust,no_run
    /// use profit_dll::{ProfitConnector, CallbackEvent};
    /// 
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let connector = ProfitConnector::new(None)?;
    /// 
    /// let mut events = connector.initialize_login(
    ///     "SUA_CHAVE_ATIVACAO",
    ///     "seu_usuario",
    ///     "sua_senha"
    /// ).await?;
    /// 
    /// // Processar eventos
    /// while let Some(event) = events.recv().await {
    ///     match event {
    ///         CallbackEvent::StateChanged { connection_type, result } => {
    ///             println!("Estado mudou: {:?} -> {}", connection_type, result);
    ///         }
    ///         _ => {}
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn initialize_login(
        &self,
        activation_key: &str,
        user: &str,
        password: &str,
    ) -> Result<mpsc::UnboundedReceiver<CallbackEvent>> {
        // Setup event handler
        let (handler, receiver) = ChannelEventHandler::new();
        add_event_handler(Box::new(handler));

        // Call DLL initialization
        let result = self.dll_loader.dll_initialize_login(
            activation_key,
            user,
            password,
        )?;

        if result != NL_OK {
            return Err(ProfitError::from(result));
        }

        *self.is_initialized.write() = true;
        *self.is_trading_enabled.write() = true;
        
        tracing::info!("ProfitDLL initialized successfully with full login");
        
        Ok(receiver)
    }

    /// Inicializa apenas market data (sem trading)
    /// 
    /// Use esta função quando precisar apenas de cotações
    /// e dados de mercado, sem enviar ordens.
    /// 
    /// # Argumentos
    /// * `activation_key` - Chave de ativação fornecida
    /// * `user` - Nome do usuário
    /// * `password` - Senha do usuário
    /// 
    /// # Exemplo
    /// ```rust,no_run
    /// use profit_dll::{ProfitConnector, CallbackEvent};
    /// 
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let connector = ProfitConnector::new(None)?;
    /// 
    /// let mut events = connector.initialize_market_data(
    ///     "SUA_CHAVE_ATIVACAO",
    ///     "seu_usuario",
    ///     "sua_senha"
    /// ).await?;
    /// 
    /// // Apenas market data estará disponível
    /// connector.subscribe_ticker("PETR4", "B")?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn initialize_market_data(
        &self,
        activation_key: &str,
        user: &str,
        password: &str,
    ) -> Result<mpsc::UnboundedReceiver<CallbackEvent>> {
        let (handler, receiver) = ChannelEventHandler::new();
        add_event_handler(Box::new(handler));

        let result = self.dll_loader.dll_initialize_market_login(
            activation_key,
            user,
            password,
        )?;

        if result != NL_OK {
            return Err(ProfitError::from(result));
        }

        *self.is_initialized.write() = true;
        *self.is_trading_enabled.write() = false;
        
        tracing::info!("ProfitDLL initialized successfully for market data only");
        
        Ok(receiver)
    }

    /// Finaliza a DLL e encerra conexões
    pub fn finalize(&self) -> Result<()> {
        if *self.is_initialized.read() {
            let result = self.dll_loader.dll_finalize()?;
            if result != NL_OK {
                return Err(ProfitError::from(result));
            }
            *self.is_initialized.write() = false;
            *self.is_trading_enabled.write() = false;
            
            tracing::info!("ProfitDLL finalized successfully");
        }
        Ok(())
    }

    /// Verifica se está inicializado
    pub fn is_initialized(&self) -> bool {
        *self.is_initialized.read()
    }

    /// Verifica se trading está habilitado
    pub fn is_trading_enabled(&self) -> bool {
        *self.is_trading_enabled.read()
    }

    // === MARKET DATA FUNCTIONS ===

    /// Subscreve-se para receber cotações de um ticker
    /// 
    /// # Argumentos
    /// * `ticker` - Código do ativo (ex: "PETR4")
    /// * `exchange` - Bolsa (ex: "B" para Bovespa, "F" para BMF)
    /// 
    /// # Exemplo
    /// ```rust,no_run
    /// # use profit_dll::ProfitConnector;
    /// # fn example(connector: &ProfitConnector) -> Result<(), profit_dll::ProfitError> {
    /// // Subscrever ações da Bovespa
    /// connector.subscribe_ticker("PETR4", "B")?;
    /// connector.subscribe_ticker("VALE3", "B")?;
    /// 
    /// // Subscrever futuros do BMF
    /// connector.subscribe_ticker("WINFUT", "F")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn subscribe_ticker(&self, ticker: &str, exchange: &str) -> Result<()> {
        self.check_initialized()?;
        let result = self.dll_loader.subscribe_ticker(ticker, exchange)?;
        if result != NL_OK {
            return Err(ProfitError::from(result));
        }
        
        tracing::debug!("Subscribed to ticker: {}@{}", ticker, exchange);
        Ok(())
    }

    /// Cancela inscrição de ticker
    pub fn unsubscribe_ticker(&self, ticker: &str, exchange: &str) -> Result<()> {
        self.check_initialized()?;
        let result = self.dll_loader.unsubscribe_ticker(ticker, exchange)?;
        if result != NL_OK {
            return Err(ProfitError::from(result));
        }
        
        tracing::debug!("Unsubscribed from ticker: {}@{}", ticker, exchange);
        Ok(())
    }

    /// Inscreve-se para receber book de preços
    /// 
    /// O book de preços mostra os níveis de preço e quantidades
    /// para compra e venda.
    pub fn subscribe_price_book(&self, ticker: &str, exchange: &str) -> Result<()> {
        self.check_initialized()?;
        let result = self.dll_loader.subscribe_price_book(ticker, exchange)?;
        if result != NL_OK {
            return Err(ProfitError::from(result));
        }
        
        tracing::debug!("Subscribed to price book: {}@{}", ticker, exchange);
        Ok(())
    }

    /// Cancela inscrição do book de preços
    pub fn unsubscribe_price_book(&self, ticker: &str, exchange: &str) -> Result<()> {
        self.check_initialized()?;
        let result = self.dll_loader.unsubscribe_price_book(ticker, exchange)?;
        if result != NL_OK {
            return Err(ProfitError::from(result));
        }
        
        tracing::debug!("Unsubscribed from price book: {}@{}", ticker, exchange);
        Ok(())
    }

    // === TRADING FUNCTIONS ===

    /// Envia uma ordem usando a estrutura SendOrder
    /// 
    /// Esta é a forma recomendada de enviar ordens.
    /// 
    /// # Exemplo
    /// ```rust,no_run
    /// # use profit_dll::*;
    /// # fn example(connector: &ProfitConnector) -> Result<(), ProfitError> {
    /// let account = AccountIdentifier::new(123, "12345".to_string(), "".to_string());
    /// let asset = AssetIdentifier::bovespa("PETR4");
    /// 
    /// // Ordem limitada de compra
    /// let order = SendOrder::new_limit_order(
    ///     account,
    ///     asset,
    ///     "senha123".to_string(),
    ///     OrderSide::Buy,
    ///     25.50,  // preço
    ///     100     // quantidade
    /// );
    /// 
    /// let order_id = connector.send_order(&order)?;
    /// println!("Ordem enviada com ID: {}", order_id);
    /// # Ok(())
    /// # }
    /// ```
    pub fn send_order(&self, order: &SendOrder) -> Result<i64> {
        self.check_trading_enabled()?;
        let order_id = self.dll_loader.send_order(order)?;
        
        tracing::info!(
            "Order sent: {} {} {} @ {} - ID: {}", 
            order.asset_id.ticker(),
            order.order_side as u8,
            order.quantity,
            order.price,
            order_id
        );
        
        Ok(order_id)
    }

    /// Envia ordem de compra limitada (função de conveniência)
    /// 
    /// # Argumentos
    /// * `account_id` - Identificador da conta
    /// * `asset_id` - Identificador do ativo
    /// * `password` - Senha de trading
    /// * `price` - Preço limite
    /// * `quantity` - Quantidade
    pub fn send_buy_order(
        &self,
        account_id: &AccountIdentifier,
        asset_id: &AssetIdentifier,
        password: &str,
        price: f64,
        quantity: i32,
    ) -> Result<i64> {
        self.check_trading_enabled()?;
        let order_id = self.dll_loader.send_buy_order(
            &account_id.account_id,
            &account_id.broker_id.to_string(),
            password,
            asset_id.ticker(),
            asset_id.exchange(),
            price,
            quantity,
        )?;
        
        tracing::info!(
            "Buy order sent: {} {} @ {} - ID: {}", 
            asset_id.ticker(),
            quantity,
            price,
            order_id
        );
        
        Ok(order_id)
    }

    /// Envia ordem de venda limitada (função de conveniência)
    pub fn send_sell_order(
        &self,
        account_id: &AccountIdentifier,
        asset_id: &AssetIdentifier,
        password: &str,
        price: f64,
        quantity: i32,
    ) -> Result<i64> {
        self.check_trading_enabled()?;
        let order_id = self.dll_loader.send_sell_order(
            &account_id.account_id,
            &account_id.broker_id.to_string(),
            password,
            asset_id.ticker(),
            asset_id.exchange(),
            price,
            quantity,
        )?;
        
        tracing::info!(
            "Sell order sent: {} {} @ {} - ID: {}", 
            asset_id.ticker(),
            quantity,
            price,
            order_id
        );
        
        Ok(order_id)
    }

    /// Cancela uma ordem
    /// 
    /// # Argumentos
    /// * `account_id` - Identificador da conta
    /// * `cl_order_id` - ClOrderID da ordem (fornecido nos callbacks)
    /// * `password` - Senha de trading
    pub fn cancel_order(
        &self,
        account_id: &AccountIdentifier,
        cl_order_id: &str,
        password: &str,
    ) -> Result<()> {
        self.check_trading_enabled()?;
        let result = self.dll_loader.send_cancel_order(
            &account_id.account_id,
            &account_id.broker_id.to_string(),
            cl_order_id,
            password,
        )?;
        
        if result != NL_OK {
            return Err(ProfitError::from(result));
        }
        
        tracing::info!("Cancel order sent for ClOrderID: {}", cl_order_id);
        Ok(())
    }

    // === POSITION & ACCOUNT FUNCTIONS ===

    /// Obtém informações de posição para uma conta e ativo
    /// 
    /// # Exemplo
    /// ```rust,no_run
    /// # use profit_dll::*;
    /// # fn example(connector: &ProfitConnector) -> Result<(), ProfitError> {
    /// let account = AccountIdentifier::new(123, "12345".to_string(), "".to_string());
    /// let asset = AssetIdentifier::bovespa("PETR4");
    /// 
    /// let position = connector.get_position(&account, &asset)?;
    /// println!("Posição aberta: {} @ {}", position.open_quantity, position.open_average_price);
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_position(
        &self,
        account_id: &AccountIdentifier,
        asset_id: &AssetIdentifier,
    ) -> Result<Position> {
        self.check_initialized()?;
        let position = self.dll_loader.get_position(
            &account_id.account_id,
            &account_id.broker_id.to_string(),
            asset_id.ticker(),
            asset_id.exchange(),
        )?;
        Ok(position)
    }

    /// Obtém número de contas disponíveis
    pub fn get_account_count(&self) -> Result<i32> {
        self.check_initialized()?;
        let count = self.dll_loader.get_account_count()?;
        Ok(count)
    }

    /// Obtém horário do servidor
    /// 
    /// Útil para sincronização de tempo com o servidor de trading.
    pub fn get_server_clock(&self) -> Result<DateTime<Utc>> {
        self.check_initialized()?;
        let server_time = self.dll_loader.get_server_clock()?;
        Ok(server_time)
    }

    /// Define se deve usar day trade
    /// 
    /// # Argumentos
    /// * `use_day_trade` - true para ativar day trade, false para desativar
    /// 
    /// Esta configuração afeta como as ordens são enviadas para
    /// corretoras que têm controle de risco específico para day trade.
    pub fn set_day_trade(&self, use_day_trade: bool) -> Result<()> {
        self.check_initialized()?;
        let result = self.dll_loader.set_day_trade(if use_day_trade { 1 } else { 0 })?;
        if result != NL_OK {
            return Err(ProfitError::from(result));
        }
        
        tracing::info!("Day trade mode set to: {}", use_day_trade);
        Ok(())
    }

    // === HELPER FUNCTIONS ===

    /// Conveniência para subscrever múltiplos tickers
    /// 
    /// # Exemplo
    /// ```rust,no_run
    /// # use profit_dll::*;
    /// # fn example(connector: &ProfitConnector) -> Result<(), ProfitError> {
    /// let tickers = vec![
    ///     ("PETR4", "B"),
    ///     ("VALE3", "B"),
    ///     ("WINFUT", "F"),
    /// ];
    /// 
    /// connector.subscribe_multiple_tickers(&tickers)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn subscribe_multiple_tickers(&self, tickers: &[(&str, &str)]) -> Result<()> {
        for (ticker, exchange) in tickers {
            self.subscribe_ticker(ticker, exchange)?;
        }
        Ok(())
    }

    /// Conveniência para criar ordem de mercado
    /// 
    /// # Exemplo
    /// ```rust,no_run
    /// # use profit_dll::*;
    /// # fn example(connector: &ProfitConnector) -> Result<(), ProfitError> {
    /// let account = AccountIdentifier::new(123, "12345".to_string(), "".to_string());
    /// 
    /// // Compra a mercado
    /// let order_id = connector.send_market_order(
    ///     &account,
    ///     "PETR4",
    ///     "B",
    ///     "senha123",
    ///     OrderSide::Buy,
    ///     100
    /// )?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn send_market_order(
        &self,
        account_id: &AccountIdentifier,
        ticker: &str,
        exchange: &str,
        password: &str,
        side: OrderSide,
        quantity: i64,
    ) -> Result<i64> {
        let asset_id = AssetIdentifier::new(ticker, exchange, 0);
        let order = SendOrder::new_market_order(
            account_id.clone(),
            asset_id,
            password.to_string(),
            side,
            quantity,
        );
        
        self.send_order(&order)
    }

    // === INTERNAL FUNCTIONS ===

    fn check_initialized(&self) -> Result<()> {
        if !*self.is_initialized.read() {
            return Err(ProfitError::NotInitialized);
        }
        Ok(())
    }

    fn check_trading_enabled(&self) -> Result<()> {
        self.check_initialized()?;
        if !*self.is_trading_enabled.read() {
            return Err(ProfitError::MarketOnly);
        }
        Ok(())
    }
}

impl Drop for ProfitConnector {
    /// Automaticamente finaliza a DLL quando o connector é dropado
    fn drop(&mut self) {
        if let Err(e) = self.finalize() {
            tracing::error!("Error finalizing ProfitDLL in drop: {}", e);
        }
    }
}

// Thread safety
unsafe impl Send for ProfitConnector {}
unsafe impl Sync for ProfitConnector {}
