// Mini-Disclaimer: Uso educacional/experimental; sem recomendação de investimento ou afiliação; sem remuneração de terceiros; Profit/ProfitDLL © Nelógica; veja README & DISCLAIMER.
use std::time::Duration;
use tucano_profitdll::*;

#[tokio::test]
async fn mock_emits_extended_placeholders() {
    // Acelera geração para testes (intervalo curto)
    std::env::set_var("MOCK_INTERVAL_MS", "5");
    let connector = ProfitConnector::new(None).unwrap();
    let mut rx = connector
        .initialize_login("act", "user", "pass")
        .await
        .unwrap();
    connector.subscribe_ticker("TESTE", "BVMF").unwrap();

    let mut got_history = false;
    let mut got_adjust = false;
    let mut got_theoretical = false;
    let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
    while tokio::time::Instant::now() < deadline && !(got_history && got_adjust && got_theoretical)
    {
        if let Some(evt) = tokio::time::timeout(Duration::from_millis(200), rx.recv())
            .await
            .unwrap()
        {
            match evt {
                CallbackEvent::HistoryTrade { .. } => got_history = true,
                CallbackEvent::AdjustHistory { .. } => got_adjust = true,
                CallbackEvent::TheoreticalPrice { .. } => got_theoretical = true,
                _ => {}
            }
        }
    }
    assert!(got_history, "HistoryTrade placeholder not received");
    assert!(got_adjust, "AdjustHistory placeholder not received");
    assert!(got_theoretical, "TheoreticalPrice placeholder not received");
}
