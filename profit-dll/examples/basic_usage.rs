//! Exemplo básico de uso da ProfitDLL
//! 
//! Este exemplo demonstra como:
//! - Conectar à ProfitDLL
//! - Subscrever para market data
//! - Enviar ordens simples
//! - Processar eventos assíncronos

use profit_dll::{
    ProfitConnector, CallbackEvent, AssetIdentifier, AccountIdentifier, 
    OrderSide, SendOrder, exchanges
};
use tokio::time::{sleep, Duration};
use tracing::{info, warn, error};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Inicializar logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("Iniciando exemplo ProfitDLL");

    // Criar connector
    let connector = ProfitConnector::new(None)?;
    
    // === CONFIGURAÇÃO ===
    // ATENÇÃO: Substitua pelas suas credenciais reais
    let activation_key = "SUA_CHAVE_ATIVACAO_AQUI";
    let user = "seu_usuario";
    let password = "sua_senha";
    
    // === INICIALIZAÇÃO ===
    info!("Conectando à ProfitDLL...");
    
    // Para market data apenas, use:
    // let mut events = connector.initialize_market_data(activation_key, user, password).await?;
    
    // Para trading completo:
    let mut events = connector.initialize_login(activation_key, user, password).await?;
    
    info!("Conectado com sucesso!");

    // === MARKET DATA ===
    info!("Subscrevendo para market data...");
    
    // Subscrever ações da Bovespa
    connector.subscribe_ticker("PETR4", exchanges::BOVESPA)?;
    connector.subscribe_ticker("VALE3", exchanges::BOVESPA)?;
    connector.subscribe_ticker("ITUB4", exchanges::BOVESPA)?;
    
    // Subscrever futuros do BMF
    connector.subscribe_ticker("WINFUT", exchanges::BMF)?;
    
    // Subscrever book de preços
    connector.subscribe_price_book("PETR4", exchanges::BOVESPA)?;
    
    info!("Subscrições realizadas");

    // === CONFIGURAÇÕES DE TRADING ===
    if connector.is_trading_enabled() {
        info!("Trading habilitado - configurando...");
        
        // Ativar day trade
        connector.set_day_trade(true)?;
        
        // Obter horário do servidor
        match connector.get_server_clock() {
            Ok(server_time) => info!("Horário do servidor: {}", server_time),
            Err(e) => warn!("Erro ao obter horário do servidor: {}", e),
        }
        
        // Obter número de contas
        match connector.get_account_count() {
            Ok(count) => info!("Número de contas disponíveis: {}", count),
            Err(e) => warn!("Erro ao obter contas: {}", e),
        }
    }

    // === EXEMPLO DE ORDEM (apenas demonstrativo) ===
    // ATENÇÃO: Descomente apenas se quiser REALMENTE enviar uma ordem!
    /*
    if connector.is_trading_enabled() {
        info!("Enviando ordem de exemplo...");
        
        let account = AccountIdentifier::new(
            12345, // Seu broker ID
            "123456".to_string(), // Sua conta
            "".to_string() // Subconta (se aplicável)
        );
        
        let asset = AssetIdentifier::bovespa("PETR4");
        
        // Ordem limitada de compra
        let order = SendOrder::new_limit_order(
            account.clone(),
            asset.clone(),
            "sua_senha_trading".to_string(),
            OrderSide::Buy,
            25.50, // preço
            100    // quantidade
        );
        
        match connector.send_order(&order) {
            Ok(order_id) => info!("Ordem enviada com sucesso! ID: {}", order_id),
            Err(e) => error!("Erro ao enviar ordem: {}", e),
        }
        
        // Consultar posição
        match connector.get_position(&account, &asset) {
            Ok(position) => {
                info!("Posição atual: {} @ {}", 
                      position.open_quantity, 
                      position.open_average_price);
            }
            Err(e) => warn!("Erro ao consultar posição: {}", e),
        }
    }
    */

    // === PROCESSAMENTO DE EVENTOS ===
    info!("Processando eventos... (Ctrl+C para parar)");
    
    let mut trade_count = 0;
    let mut max_trades = 50; // Limitar para exemplo
    
    // Timeout para exemplo
    let timeout = sleep(Duration::from_secs(60));
    tokio::pin!(timeout);
    
    loop {
        tokio::select! {
            // Processar eventos da DLL
            Some(event) = events.recv() => {
                match event {
                    CallbackEvent::StateChanged { connection_type, result } => {
                        info!("Estado da conexão mudou: {:?} -> resultado: {}", 
                              connection_type, result);
                    }
                    
                    CallbackEvent::NewTrade(trade) => {
                        info!("Novo trade: {} @ {} - vol: {} ({})", 
                              trade.asset_id.ticker(),
                              trade.price,
                              trade.volume,
                              trade.trade_date.format("%H:%M:%S"));
                        
                        trade_count += 1;
                        if trade_count >= max_trades {
                            info!("Limite de trades para exemplo atingido");
                            break;
                        }
                    }
                    
                    CallbackEvent::DailyData(daily) => {
                        info!("Dados diários {}: O:{} H:{} L:{} C:{} V:{}", 
                              daily.asset_id.ticker(),
                              daily.open, daily.high, daily.low, daily.close, daily.volume);
                    }
                    
                    CallbackEvent::PriceBookUpdate { asset_id, action, position, side, entry } => {
                        if let Some(entry) = entry {
                            info!("Book atualizado {}: {:?} pos:{} side:{} -> {} x {}", 
                                  asset_id.ticker(), action, position, side, 
                                  entry.price, entry.quantity);
                        }
                    }
                    
                    CallbackEvent::OrderChanged(order) => {
                        info!("Ordem modificada: {} - status: {} - qty: {}/{}", 
                              order.order_id.cl_order_id,
                              order.order_status,
                              order.traded_quantity,
                              order.quantity);
                    }
                    
                    CallbackEvent::AccountInfo { broker_id, broker_name, account_id, account_holder } => {
                        info!("Conta: {} ({}) - {} [{}]", 
                              account_id, broker_id, account_holder, broker_name);
                    }
                    
                    CallbackEvent::Progress { asset_id, progress } => {
                        info!("Progresso {}: {}%", asset_id.ticker(), progress);
                    }
                    
                    CallbackEvent::InvalidTicker(asset_id) => {
                        warn!("Ticker inválido: {}@{}", asset_id.ticker(), asset_id.exchange());
                    }
                    
                    _ => {
                        // Outros eventos...
                    }
                }
            }
            
            // Timeout do exemplo
            _ = &mut timeout => {
                info!("Tempo limite do exemplo atingido");
                break;
            }
        }
    }

    // === LIMPEZA ===
    info!("Finalizando exemplo...");
    
    // Desinscrever dos tickers
    let _ = connector.unsubscribe_ticker("PETR4", exchanges::BOVESPA);
    let _ = connector.unsubscribe_ticker("VALE3", exchanges::BOVESPA);
    let _ = connector.unsubscribe_ticker("ITUB4", exchanges::BOVESPA);
    let _ = connector.unsubscribe_ticker("WINFUT", exchanges::BMF);
    let _ = connector.unsubscribe_price_book("PETR4", exchanges::BOVESPA);
    
    // Finalizar (acontece automaticamente no Drop, mas pode ser explícito)
    connector.finalize()?;
    
    info!("Exemplo finalizado com sucesso!");
    Ok(())
}
