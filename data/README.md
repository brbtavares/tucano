
# Toucan-Data

A high-performance WebSocket integration library for streaming public market data from leading cryptocurrency
exchanges - batteries included. It is:

* **Easy**: Toucan-Data's simple StreamBuilder interface allows for easy & quick setup (see example below!).
* **Normalised**: Toucan-Data's unified interface for consuming public WebSocket data means every Exchange returns a normalised data model.
* **Real-Time**: Toucan-Data utilises real-time WebSocket integrations enabling the consumption of normalised tick-by-tick data.
* **Extensible**: Toucan-Data is highly extensible, and therefore easy to contribute to with coding new integrations!

**See: [`Toucan`], [`Toucan-Instrument`], [`Toucan-Execution`] & [`Toucan-Integration`] for
comprehensive documentation of other Toucan libraries.**

[`Toucan`]: https://github.com/brbtavares/toucan
[`Toucan-Instrument`]: https://github.com/brbtavares/toucan/tree/main/toucan-instrument
[`Toucan-Execution`]: https://github.com/brbtavares/toucan/tree/main/toucan-execution
[`Toucan-Integration`]: https://github.com/brbtavares/toucan/tree/main/toucan-integration

## Overview

Toucan-Data is a high-performance WebSocket integration library for streaming public market data from leading cryptocurrency
exchanges. It presents an easy-to-use and extensible set of interfaces that can deliver normalised exchange data in real-time.

From a user perspective, the major component is the `StreamBuilder` structures that assists in initialising an
arbitrary number of exchange `MarketStream`s using input `Subscription`s. Simply build your dream set of
`MarketStreams` and `Toucan-Data` will do the rest!

### Supported Exchange Subscriptions

|        Exchange         |         Constructor Code         |               InstrumentKinds               |                SubscriptionKinds                 |
|:-----------------------:|:--------------------------------:|:-------------------------------------------:|:------------------------------------------------:|
|     **BinanceSpot**     |     `BinanceSpot::default()`     |                    Spot                     | PublicTrades, OrderBooksL1, OrderBooksL2 |
|  **BinanceFuturesUsd**  |  `BinanceFuturesUsd::default()`  |                  Perpetual                  | PublicTrades, OrderBooksL1, OrderBooksL2 |
|      **Bitfinex**       |            `Bitfinex`            |                    Spot                     |                   PublicTrades                   |
|       **Bitmex**        |             `Bitmex`             |                  Perpetual                  |                   PublicTrades                   |
|      **BybitSpot**      |      `BybitSpot::default()`      |                    Spot                     |                   PublicTrades                   |
| **BybitPerpetualsUsd**  | `BybitPerpetualsUsd::default()`  |                  Perpetual                  |                   PublicTrades                   |
|      **Coinbase**       |            `Coinbase`            |                    Spot                     |                   PublicTrades                   |
|     **GateioSpot**      |     `GateioSpot::default()`      |                    Spot                     |                   PublicTrades                   |
|  **GateioFuturesUsd**   |  `GateioFuturesUsd::default()`   |                   Future                    |                   PublicTrades                   |
|  **GateioFuturesBtc**   |  `GateioFuturesBtc::default()`   |                   Future                    |                   PublicTrades                   |
| **GateioPerpetualsUsd** | `GateioPerpetualsUsd::default()` |                  Perpetual                  |                   PublicTrades                   |
| **GateioPerpetualsBtc** | `GateioPerpetualsBtc::default()` |                  Perpetual                  |                   PublicTrades                   |
|  **GateioOptionsBtc**   |    `GateioOptions::default()`    |                   Option                    |                   PublicTrades                   |
|       **Kraken**        |             `Kraken`             |                    Spot                     |          PublicTrades, OrderBooksL1          |
|         **Okx**         |              `Okx`               | Spot, Future, Perpetual, Option |                   PublicTrades                   |

## Examples

See the `/examples` directory for a comprehensive selection of usage examples!

### Multi Exchange Public Trades

```rust,no_run
use toucan_data::{
    exchange::{
        binance::{futures::BinanceFuturesUsd, spot::BinanceSpot},
        bitmex::Bitmex,
        bybit::{futures::BybitPerpetualsUsd, spot::BybitSpot},
        coinbase::Coinbase,
        gateio::{
            option::GateioOptions,
            perpetual::{GateioPerpetualsBtc, GateioPerpetualsUsd},
            spot::GateioSpot,
        },
        okx::Okx,
    },
    streams::{Streams, reconnect::stream::ReconnectingStream},
    subscription::trade::PublicTrades,
};
use toucan_integration::model::instrument::kind::{
    FutureContract, InstrumentKind, OptionContract, OptionExercise, OptionKind,
};
use chrono::{TimeZone, Utc};
use futures::StreamExt;

#[tokio::main]
async fn main() {
    // Initialise PublicTrades Streams for various exchanges
    // '--> each call to StreamBuilder::subscribe() creates a separate WebSocket connection
    let streams = Streams::<PublicTrades>::builder()
        .subscribe([
            (BinanceSpot::default(), "btc", "usdt", InstrumentKind::Spot, PublicTrades),
            (BinanceSpot::default(), "eth", "usdt", InstrumentKind::Spot, PublicTrades),
        ])
        .subscribe([
            (BinanceFuturesUsd::default(), "btc", "usdt", InstrumentKind::Perpetual, PublicTrades),
            (BinanceFuturesUsd::default(), "eth", "usdt", InstrumentKind::Perpetual, PublicTrades),
        ])
        .subscribe([
            (Coinbase, "btc", "usd", InstrumentKind::Spot, PublicTrades),
            (Coinbase, "eth", "usd", InstrumentKind::Spot, PublicTrades),
        ])
        .subscribe([
            (GateioSpot::default(), "btc", "usdt", InstrumentKind::Spot, PublicTrades),
        ])
        .subscribe([
            (GateioPerpetualsUsd::default(), "btc", "usdt", InstrumentKind::Perpetual, PublicTrades),
        ])
        .subscribe([
            (GateioPerpetualsBtc::default(), "btc", "usd", InstrumentKind::Perpetual, PublicTrades),
        ])
        .subscribe([
            (GateioOptions::default(), "btc", "usdt", InstrumentKind::Option(put_contract()), PublicTrades),
        ])
        .subscribe([
            (Okx, "btc", "usdt", InstrumentKind::Spot, PublicTrades),
            (Okx, "btc", "usdt", InstrumentKind::Perpetual, PublicTrades),
            (Okx, "btc", "usd", InstrumentKind::Future(future_contract()), PublicTrades),
            (Okx, "btc", "usd", InstrumentKind::Option(call_contract()), PublicTrades),
        ])
        .subscribe([
            (BybitSpot::default(), "btc", "usdt", InstrumentKind::Spot, PublicTrades),
            (BybitSpot::default(), "eth", "usdt", InstrumentKind::Spot, PublicTrades),
        ])
        .subscribe([
            (BybitPerpetualsUsd::default(), "btc", "usdt", InstrumentKind::Perpetual, PublicTrades),
        ])
        .subscribe([
            (Bitmex, "xbt", "usd", InstrumentKind::Perpetual, PublicTrades)
        ])
        .init()
        .await
        .unwrap();

    // Select and merge every exchange Stream using futures_util::stream::select_all
    // Note: use `Streams.select(ExchangeId)` to interact with individual exchange streams!
    let mut joined_stream = streams
        .select_all()
        .with_error_handler(|error| println!(format!("MarketStream generated error: {error:?}")));

    while let Some(event) = joined_stream.next().await {
        println!("{event:?}");
    }
}
```

### Adding A New Exchange Connector

1. Add a new `Connector` trait implementation in src/exchange/<exchange_name>.mod.rs (eg/ see exchange::okx::Okx).
2. Follow on from "Adding A New Subscription Kind For An Existing Exchange Connector" below!

### Adding A New SubscriptionKind For An Existing Exchange Connector

1. Add a new `SubscriptionKind` trait implementation in src/subscription/<sub_kind_name>.rs (eg/ see subscription::trade::PublicTrades).
2. Define the `SubscriptionKind::Event` data model (eg/ see subscription::trade::PublicTrade).
3. Define the `MarketStream` type the exchange `Connector` will initialise for the new `SubscriptionKind`:
   ie/ `impl StreamSelector<SubscriptionKind> for <ExistingExchangeConnector> { ... }`
4. Try to compile and follow the remaining steps!
5. Add a toucan-data-rs/examples/<sub_kind_name>_streams.rs example in the standard format :)
