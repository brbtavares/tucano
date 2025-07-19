/*!
 * Binance WebSocket Basic Integration Example
 * 
 * This example demonstrates the fundamental concepts of using Toucan's low-level
 * WebSocket integration components to connect to Binance futures WebSocket API.
 * 
 * Key Learning Objectives:
 * - Manual WebSocket connection establishment
 * - Custom message parsing with Serde
 * - Implementing the Transformer trait for data processing
 * - Basic stateful data aggregation (volume accumulation)
 * - Error handling in real-time data streams
 * 
 * Technical Components Used:
 * - tokio-tungstenite for WebSocket connectivity
 * - Toucan's ExchangeStream for data flow management
 * - Custom BinanceMessage enum for message parsing
 * - StatefulTransformer for volume accumulation
 * 
 * Use Case:
 * This is an educational example showing how to build custom integrations
 * when you need fine-grained control over WebSocket handling. For production
 * use cases, consider using the higher-level Data module instead.
 * 
 * Setup:
 * No API keys required - uses public market data only.
 * Run: cargo run --bin binance_websocket_basic_integration
 */

use integration::{
    Transformer,
    error::SocketError,
    protocol::websocket::{WebSocket, WebSocketParser, WsMessage},
    stream::ExchangeStream,
};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, de};
use serde_json::json;
use std::{collections::VecDeque, str::FromStr};
use tokio_tungstenite::connect_async;
use tracing::debug;

// Type aliases for better code readability
type ExchangeWsStream<Exchange> = ExchangeStream<WebSocketParser, WebSocket, Exchange>;
type VolumeSum = f64;

/// Binance WebSocket message types
/// 
/// This enum handles two types of messages from Binance:
/// 1. SubResponse: Confirmation of subscription requests
/// 2. Trade: Individual trade data containing quantity and other trade information
#[derive(Deserialize)]
#[serde(untagged, rename_all = "camelCase")]
enum BinanceMessage {
    /// Subscription confirmation response
    SubResponse {
        result: Option<Vec<String>>,
        id: u32,
    },
    /// Individual trade event
    Trade {
        #[serde(rename = "q", deserialize_with = "de_str")]
        quantity: f64,
    },
}

/// Stateful transformer that accumulates trading volume
/// 
/// This demonstrates how to implement the Transformer trait to maintain
/// internal state across multiple incoming messages. In this case, we're
/// accumulating the total volume of all trades received.
struct StatefulTransformer {
    sum_of_volume: VolumeSum,
}

impl Transformer for StatefulTransformer {
    type Error = SocketError;
    type Input = BinanceMessage;
    type Output = VolumeSum;
    type OutputIter = Vec<Result<Self::Output, Self::Error>>;

    fn transform(&mut self, input: Self::Input) -> Self::OutputIter {
        // Add new input Trade quantity to sum
        match input {
            BinanceMessage::SubResponse { result, id } => {
                debug!("Received SubResponse for {}: {:?}", id, result);
                // Don't care about this for the example
            }
            BinanceMessage::Trade { quantity, .. } => {
                // Add new Trade volume to internal state VolumeSum
                self.sum_of_volume += quantity;
            }
        };

        // Return IntoIterator of length 1 containing the running sum of volume
        vec![Ok(self.sum_of_volume)]
    }
}

/// See Data for a comprehensive real-life example, as well as code you can use out of the
/// box to collect real-time public market data from many exchanges.
#[tokio::main]
async fn main() {
    // Establish Sink/Stream communication with desired WebSocket server
    let mut binance_conn = connect_async("wss://fstream.binance.com/ws/")
        .await
        .map(|(ws_conn, _)| ws_conn)
        .expect("failed to connect");

    // Send something over the socket (eg/ Binance trades subscription)
    binance_conn
        .send(WsMessage::text(
            json!({"method": "SUBSCRIBE","params": ["btcusdt@aggTrade"],"id": 1}).to_string(),
        ))
        .await
        .expect("failed to send WsMessage over socket");

    // Instantiate some arbitrary Transformer to apply to data parsed from the WebSocket protocol
    let transformer = StatefulTransformer { sum_of_volume: 0.0 };

    // ExchangeWsStream includes pre-defined WebSocket Sink/Stream & WebSocket StreamParser
    let mut ws_stream = ExchangeWsStream::new(binance_conn, transformer, VecDeque::new());

    // Receive a stream of your desired Output data model from the ExchangeStream
    while let Some(volume_result) = ws_stream.next().await {
        match volume_result {
            Ok(cumulative_volume) => {
                // Do something with your data
                println!("{cumulative_volume:?}");
            }
            Err(error) => {
                // React to any errors produced by the internal transformation
                eprintln!("{error}")
            }
        }
    }
}

/// Deserialize a `String` as the desired type.
fn de_str<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: de::Deserializer<'de>,
    T: FromStr,
    T::Err: std::fmt::Display,
{
    let data: String = Deserialize::deserialize(deserializer)?;
    data.parse::<T>().map_err(de::Error::custom)
}
