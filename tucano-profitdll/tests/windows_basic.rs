#![cfg(all(target_os = "windows", feature = "real_dll"))]
use tucano_profitdll::*;

// Teste básico: tenta carregar DLL (pode falhar se ausente). Se ausente, garante que erro Load é retornado.
#[test]
fn load_or_missing_symbol() {
    // Tentativa com nome padrão; se falhar assume ausência da DLL no ambiente de CI.
    match ProfitConnector::new(None) {
        Ok(conn) => {
            // Não chama initialize_login aqui porque exigiria credenciais reais.
            // Apenas verifica que objeto foi criado.
            let _ = conn; // sucesso
        }
        Err(e) => {
            // Aceitamos apenas variantes de carregamento/símbolo como falha aqui.
            match e {
                ProfitError::Load(_) | ProfitError::MissingSymbol(_) => {},
                other => panic!("Erro inesperado ao carregar ProfitDLL: {other:?}"),
            }
        }
    }
}
