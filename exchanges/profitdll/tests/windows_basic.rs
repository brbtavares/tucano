

#![cfg(all(target_os = "windows", feature = "real_dll"))]
use toucan_profitdll::*;

#[test]
fn load_or_missing_symbol() {
    match ProfitConnector::new(None) {
        Ok(conn) => {
            let _ = conn;
        }
        Err(e) => match e {
            ProfitError::Load(_) | ProfitError::MissingSymbol(_) => {}
            other => panic!("Erro inesperado ao carregar ProfitDLL: {other:?}"),
        },
    }
}
