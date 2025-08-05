//! Exemplo super simples do ProfitDLL wrapper

fn main() {
    println!("=== Teste básico ProfitDLL ===");
    
    // Apenas testar se conseguimos instanciar
    match profit_dll::ProfitConnector::new(None) {
        Ok(_) => println!("✓ Connector criado com sucesso"),
        Err(e) => println!("✗ Erro ao criar connector: {}", e),
    }
    
    println!("=== Teste finalizado ===");
}
