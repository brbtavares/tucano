// Exemplo de configuração e uso da ProfitDLL em Windows
//
// Este exemplo demonstra como configurar e usar a ProfitDLL real
// em um ambiente Windows com a DLL da Nelógica instalada.

use tucano_markets::{
    b3::{B3AssetFactory, B3Stock},
    profit_dll::ProfitConnector,
    Asset,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 ProfitDLL Windows Configuration Example");
    println!("==========================================");

    // Verificar se estamos em Windows
    if !cfg!(target_os = "windows") {
        println!("⚠️  Este exemplo requer Windows para usar a DLL real");
        println!("   Executando versão mock...");
    } else {
        println!("✅ Executando em Windows - DLL real disponível");
    }

    // Configurar caminho da DLL (opcional - auto-detecção via build.rs)
    let dll_path = std::env::var("PROFITDLL_PATH").ok();

    if let Some(ref path) = dll_path {
        println!("📁 Caminho da DLL configurado: {}", path);
    } else {
        println!("🔍 Auto-detectando localização da DLL...");
    }

    // Criar assets B3
    let petr4 = B3Stock::new("PETR4".to_string(), "Petrobras PN".to_string());
    let vale3 = B3AssetFactory::from_symbol("VALE3")?;

    println!("\n📊 Assets criados:");
    println!(
        "  • {}: {} ({})",
        petr4.symbol(),
        "Petrobras PN",
        petr4.asset_type()
    );
    println!(
        "  • {}: {} ({})",
        vale3.symbol(),
        "Vale ON",
        vale3.asset_type()
    );

    // Inicializar ProfitConnector
    println!("\n🔌 Inicializando ProfitConnector...");
    let connector = ProfitConnector::new(dll_path.as_deref())?;

    // NOTA: Para usar credenciais reais, descomente e configure:
    // let events = connector.initialize_login(
    //     "sua_chave_ativacao",
    //     "seu_usuario",
    //     "sua_senha"
    // ).await?;

    // Versão de demonstração (mock)
    let mut events = connector
        .initialize_login("demo_key", "demo_user", "demo_pass")
        .await?;

    println!("✅ ProfitConnector inicializado");

    // Subscrever a dados de mercado
    println!("\n📈 Configurando subscrições...");
    connector.subscribe_ticker(&petr4.symbol(), "BOVESPA")?;
    connector.subscribe_ticker(&vale3.symbol(), "BOVESPA")?;

    println!("✅ Subscrições configuradas");

    // Processar eventos por um período limitado
    println!("\n🔄 Processando eventos (5 segundos)...");

    let timeout = tokio::time::sleep(tokio::time::Duration::from_secs(5));
    tokio::pin!(timeout);

    loop {
        tokio::select! {
            Some(event) = events.recv() => {
                match event {
                    tucano_markets::profit_dll::CallbackEvent::StateChanged { connection_type, result } => {
                        println!("🔌 Estado da conexão: {:?} - Resultado: {}", connection_type, result);
                    }
                    tucano_markets::profit_dll::CallbackEvent::NewTrade { ticker, exchange, price, volume, .. } => {
                        println!("💹 Novo negócio: {} @ {} - Preço: {} Volume: {}",
                                ticker, exchange, price, volume);
                    }
                    tucano_markets::profit_dll::CallbackEvent::DailySummary { ticker, open, high, low, close, .. } => {
                        println!("📊 Resumo diário {}: O:{} H:{} L:{} C:{}",
                                ticker, open, high, low, close);
                    }
                    tucano_markets::profit_dll::CallbackEvent::ProgressChanged { ticker, progress, .. } => {
                        println!("⏳ Progresso subscrição {}: {}%", ticker, progress);
                    }
                    _ => {
                        println!("📨 Evento recebido: {:?}", event);
                    }
                }
            }
            _ = &mut timeout => {
                println!("⏰ Timeout atingido");
                break;
            }
        }
    }

    println!("\n✅ Exemplo concluído com sucesso!");

    // Configurações para produção
    println!("\n🔧 Para usar em produção:");
    println!("  1. Instale a ProfitDLL da Nelógica");
    println!("  2. Configure PROFITDLL_PATH se necessário");
    println!("  3. Compile com: cargo build --features real_dll");
    println!("  4. Use credenciais reais no initialize_login()");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_profit_connector_creation() {
        let connector = ProfitConnector::new(None);
        assert!(connector.is_ok());
    }

    #[tokio::test]
    async fn test_mock_initialization() {
        let mut connector = ProfitConnector::new(None).unwrap();
        let result = connector.initialize_login("test", "test", "test").await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_windows_configuration() {
        if cfg!(target_os = "windows") {
            println!("✅ Configuração Windows detectada");

            // Verificar se a feature real_dll está disponível
            #[cfg(feature = "real_dll")]
            println!("✅ Feature real_dll ativada");

            #[cfg(not(feature = "real_dll"))]
            println!("⚠️  Feature real_dll não ativada - usando mock");
        } else {
            println!("ℹ️  Não é Windows - sempre usa mock");
        }
    }
}
