// Mini-Disclaimer: Uso educacional/experimental; sem recomendação de investimento ou afiliação.
use tucano_profitdll::*;

#[tokio::test]
async fn mock_history_trades_generation() {
    let connector = ProfitConnector::new(None).unwrap();
    let mut rx = connector
        .initialize_login("act", "user", "pass")
        .await
        .unwrap();
    connector
        .get_history_trades("HIST", "BVMF", 0, 10_000, 2_000)
        .unwrap();
    let mut count = 0;
    while let Ok(Some(evt)) =
        tokio::time::timeout(std::time::Duration::from_millis(50), rx.recv()).await
    {
        if let CallbackEvent::HistoryTrade { .. } = evt {
            count += 1;
        }
        if count >= 5 {
            break;
        }
    }
    assert!(
        count >= 5,
        "expected at least 5 history trade events, got {count}"
    );
}
