//! Exemplo básico de uso do ProfitDLL wrapper

use profit_dll::{ProfitConnector, ProfitError};

#[tokio::main]
async fn main() -> Result<(), ProfitError> {
    println!("=== Exemplo de uso do ProfitDLL Wrapper ===");

    // Criar instância do connector
    let connector = match ProfitConnector::new(None) {
        Ok(conn) => {
            println!("✓ Connector criado com sucesso");
            conn
        }
        Err(e) => {
            println!("✗ Erro ao criar connector: {}", e);
            return Err(e);
        }
    };

    // Tentar login (vai falhar em ambiente não-Windows/sem DLL)
    println!("Tentando login de teste...");
    match connector.initialize_login("user", "pass", "key").await {
        Ok(_) => {
            println!("✓ Login realizado com sucesso");
        }
        Err(e) => {
            println!("✗ Login falhou (esperado em ambiente não-Windows): {}", e);
        }
    }

    // Tentar finalizar
    match connector.finalize() {
        Ok(_) => {
            println!("✓ Finalização realizada com sucesso");
        }
        Err(e) => {
            println!("✗ Finalização falhou: {}", e);
        }
    }

    println!("=== Exemplo finalizado ===");
    Ok(())
}
