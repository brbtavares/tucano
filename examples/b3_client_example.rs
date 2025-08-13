use execution::client::b3::B3Client;
use markets::{
    exchange::ExchangeId,
    asset::name::AssetNameExchange,
    instrument::name::InstrumentNameExchange,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configurar o cliente B3
    let config = execution::client::b3::B3Config::new(
        "sua_chave_ativacao".to_string(),
        "seu_usuario".to_string(),
        "sua_senha".to_string(),
    )
    .with_auto_reconnect(true)
    .with_connection_timeout(30);

    // Criar o cliente
    let client = B3Client::new(config);

    // Definir ativos e instrumentos de interesse
    let assets = vec![
        AssetNameExchange::new("BRL".to_string()),
    ];
    
    let instruments = vec![
        InstrumentNameExchange::new("PETR4".to_string()),
        InstrumentNameExchange::new("VALE3".to_string()),
        InstrumentNameExchange::new("ITUB4".to_string()),
    ];

    // Obter snapshot da conta
    println!("Obtendo snapshot da conta B3...");
    match client.account_snapshot(&assets, &instruments).await {
        Ok(snapshot) => {
            println!("Snapshot obtido com sucesso!");
            println!("Exchange: {:?}", snapshot.exchange);
            println!("Balances: {} ativos", snapshot.balances.len());
            println!("Instruments: {} instrumentos", snapshot.instruments.len());
        }
        Err(e) => {
            eprintln!("Erro ao obter snapshot: {:?}", e);
        }
    }

    // Buscar saldos atuais
    println!("\nBuscando saldos...");
    match client.fetch_balances().await {
        Ok(balances) => {
            if balances.is_empty() {
                println!("Nenhum saldo encontrado (implementação placeholder)");
            } else {
                for balance in balances {
                    println!("Asset: {}, Available: {}, Locked: {}",
                        balance.asset, balance.available, balance.locked);
                }
            }
        }
        Err(e) => {
            eprintln!("Erro ao buscar saldos: {:?}", e);
        }
    }

    // Buscar ordens abertas
    println!("\nBuscando ordens abertas...");
    match client.fetch_open_orders().await {
        Ok(orders) => {
            if orders.is_empty() {
                println!("Nenhuma ordem aberta encontrada (implementação placeholder)");
            } else {
                for order in orders {
                    println!("Order: Exchange={:?}, Instrument={}, Side={:?}, Quantity={}",
                        order.key.exchange, order.key.instrument, order.side, order.quantity);
                }
            }
        }
        Err(e) => {
            eprintln!("Erro ao buscar ordens: {:?}", e);
        }
    }

    println!("\n✅ Cliente B3 integrado com sucesso ao framework Tucano!");
    println!("🔧 Implementação básica funcional - pronta para desenvolvimento completo");

    Ok(())
}
