// Mini-Disclaimer: Educational/experimental use; not investment advice or affiliation; see README & DISCLAIMER.
use rust_decimal::Decimal;
use tucano_profitdll::*;

#[tokio::test]
async fn mock_emits_synthetic_events() {
    let connector = ProfitConnector::new(None).unwrap();
    let mut rx = connector
        .initialize_login("act", "user", "pass")
        .await
        .unwrap();
    connector.subscribe_ticker("PETR4", "BVMF").unwrap();
    let order = SendOrder::new_market_order(
        AssetIdentifier::new("PETR4".into(), "BVMF".into()),
        AccountIdentifier::new("ACC".into(), "BRK".into()),
        OrderSide::Buy,
        Decimal::from(1),
    );
    connector.send_order(&order).unwrap();

    // Collect a few events
    let mut got_progress = false;
    let mut got_book = false;
    let mut got_order = false;
    for _ in 0..10 {
        if let Some(evt) = rx.recv().await {
            match evt {
                CallbackEvent::ProgressChanged { .. } => got_progress = true,
                CallbackEvent::PriceBookOffer { .. } | CallbackEvent::OfferBookBid { .. } => {
                    got_book = true
                }
                CallbackEvent::OrderUpdated { .. } | CallbackEvent::OrderSnapshot { .. } => {
                    got_order = true
                }
                _ => {}
            }
        }
        if got_progress && got_book && got_order {
            break;
        }
    }
    assert!(
        got_progress && got_book && got_order,
        "missing synthetic events: progress={got_progress} book={got_book} order={got_order}"
    );
}
