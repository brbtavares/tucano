# Toucan-Integration

High-performance, low-level framework for composing flexible web integrations. 

Utilised by other [`Toucan`] trading ecosystem crates to build robust financial exchange integrations,
primarily for public data collection & trade execution. It is:
* **Low-Level**: Translates raw data streams communicated over the web into any desired data model using arbitrary data transformations.
* **Flexible**: Compatible with any protocol (WebSocket, FIX, Http, etc.), any input/output model, and any user defined transformations.

Core abstractions include:

- **RestClient** providing configurable signed Http communication between client & server.
- **ExchangeStream** providing configurable communication over any asynchronous stream protocols (WebSocket, FIX, etc.).

Both core abstractions provide the robust glue you need to conveniently translate between server & client data models.

**See: [`Toucan`], [`Toucan-Data`] & [`Toucan-Execution`]**

[API Documentation]

[`Toucan`]: https://github.com/brbtavares/toucan
[`Toucan-Data`]: https://github.com/brbtavares/toucan/tree/main/toucan-data
[`Toucan-Execution`]: https://github.com/brbtavares/toucan/tree/main/toucan-execution
[API Documentation]: https://docs.rs/toucan-data/latest/toucan_integration

## Overview

Toucan-Integration is a high-performance, low-level, configurable framework for composing flexible web 
integrations. 

### RestClient

**(sync private & public Http communication)**

At a high level, a `RestClient` is has a few major components that allow it to execute `RestRequests`:
* `RequestSigner` with configurable signing logic on the target API.
* `HttpParser` that translates API specific responses into the desired output types.

### ExchangeStream

**(async communication using streaming protocols such as WebSocket and FIX)**

At a high level, an `ExchangeStream` is made up of a few major components:
* Inner Stream/Sink socket (eg/ WebSocket, FIX, etc).
* StreamParser that is capable of parsing input protocol messages (eg/ WebSocket, FIX, etc.) as exchange
  specific messages.
* Transformer that transforms from exchange specific message into an iterator of the desired outputs type.

## Examples

#### Fetch Ftx Account Balances Using Signed GET request

```rust,no_run
use std::borrow::Cow;

use toucan_integration::{
    error::SocketError,
    metric::Tag,
    model::Symbol,
    protocol::http::{
        private::{encoder::HexEncoder, RequestSigner, Signer},
        rest::{client::RestClient, RestRequest},
        HttpParser,
    },
};
use bytes::Bytes;
use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac};
use reqwest::{RequestBuilder, StatusCode};
use serde::Deserialize;
use thiserror::Error;
use tokio::sync::mpsc;

struct FtxSigner {
    api_key: String,
}

// Configuration required to sign every Ftx `RestRequest`
struct FtxSignConfig<'a> {
    api_key: &'a str,
    time: DateTime<Utc>,
    method: reqwest::Method,
    path: Cow<'static, str>,
}

impl Signer for FtxSigner {
    type Config<'a> = FtxSignConfig<'a> where Self: 'a;

    fn config<'a, Request>(
        &'a self,
        request: Request,
        _: &RequestBuilder,
    ) -> Result<Self::Config<'a>, SocketError>
    where
        Request: RestRequest,
    {
        Ok(FtxSignConfig {
            api_key: self.api_key.as_str(),
            time: Utc::now(),
            method: Request::method(),
            path: request.path(),
        })
    }

    fn add_bytes_to_sign<M>(mac: &mut M, config: &Self::Config<'a>) -> Bytes
    where
        M: Mac
    {
        mac.update(config.time.to_string().as_bytes());
        mac.update(config.method.as_str().as_bytes());
        mac.update(config.path.as_bytes());
    }

    fn build_signed_request<'a>(
        config: Self::Config<'a>,
        builder: RequestBuilder,
        signature: String,
    ) -> Result<reqwest::Request, SocketError> {
        // Add Ftx required Headers & build reqwest::Request
        builder
            .header("FTX-KEY", config.api_key)
            .header("FTX-TS", &config.time.timestamp_millis().to_string())
            .header("FTX-SIGN", &signature)
            .build()
            .map_err(SocketError::from)
    }
}

struct FtxParser;

impl HttpParser for FtxParser {
    type ApiError = serde_json::Value;
    type OutputError = ExecutionError;

    fn parse_api_error(&self, status: StatusCode, api_error: Self::ApiError) -> Self::OutputError {
        // For simplicity, use serde_json::Value as Error and extract raw String for parsing
        let error = api_error.to_string();

        // Parse Ftx error message to determine custom ExecutionError variant
        match error.as_str() {
            message if message.contains("Invalid login credentials") => {
                ExecutionError::Unauthorised(error)
            }
            _ => ExecutionError::Socket(SocketError::HttpResponse(status, error)),
        }
    }
}

#[derive(Debug, Error)]
enum ExecutionError {
    #[error("request authorisation invalid: {0}")]
    Unauthorised(String),

    #[error("SocketError: {0}")]
    Socket(#[from] SocketError),
}

struct FetchBalancesRequest;

impl RestRequest for FetchBalancesRequest {
    type Response = FetchBalancesResponse; // Define Response type
    type QueryParams = (); // FetchBalances does not require any QueryParams
    type Body = (); // FetchBalances does not require any Body

    fn path(&self) -> Cow<'static, str> {
        Cow::Borrowed("/api/wallet/balances")
    }

    fn method() -> reqwest::Method {
        reqwest::Method::GET
    }
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct FetchBalancesResponse {
    success: bool,
    result: Vec<FtxBalance>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct FtxBalance {
    #[serde(rename = "coin")]
    symbol: Symbol,
    total: f64,
}

/// See Toucan-Execution for a comprehensive real-life example, as well as code you can use out of the
/// box to execute trades on many exchanges.
#[tokio::main]
async fn main() {
    // HMAC-SHA256 encoded account API secret used for signing private http requests
    let mac: Hmac<sha2::Sha256> = Hmac::new_from_slice("api_secret".as_bytes()).unwrap();

    // Build Ftx configured RequestSigner for signing http requests with hex encoding
    let request_signer = RequestSigner::new(
        FtxSigner {
            api_key: "api_key".to_string(),
        },
        mac,
        HexEncoder,
    );

    // Build RestClient with Ftx configuration
    let rest_client = RestClient::new("https://ftx.com", request_signer, FtxParser);

    // Fetch Result<FetchBalancesResponse, ExecutionError>
    let _response = rest_client.execute(FetchBalancesRequest).await;
}
```

#### Consume Binance Futures tick-by-tick Trades and calculate a rolling sum of volume

```rust,no_run
use toucan_integration::{
    error::SocketError,
    protocol::websocket::{WebSocket, WebSocketParser, WsMessage},
    ExchangeStream, Transformer,
};
use futures::{SinkExt, StreamExt};
use serde::{de, Deserialize};
use serde_json::json;
use std::str::FromStr;
use tokio_tungstenite::connect_async;
use tracing::debug;

// Convenient type alias for an `ExchangeStream` utilising a tungstenite `WebSocket`
type ExchangeWsStream<Exchange> = ExchangeStream<WebSocketParser, WebSocket, Exchange, VolumeSum>;

// Communicative type alias for what the VolumeSum the Transformer is generating
type VolumeSum = f64;

#[derive(Deserialize)]
#[serde(untagged, rename_all = "camelCase")]
enum BinanceMessage {
    SubResponse {
        result: Option<Vec<String>>,
        id: u32,
    },
    Trade {
        #[serde(rename = "q", deserialize_with = "de_str")]
        quantity: f64,
    },
}

struct StatefulTransformer {
    sum_of_volume: VolumeSum,
}

impl Transformer<VolumeSum> for StatefulTransformer {
    type Input = BinanceMessage;
    type OutputIter = Vec<Result<VolumeSum, SocketError>>;

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

/// See Toucan-Data for a comprehensive real-life example, as well as code you can use out of the
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
        .send(WsMessage::Text(
            json!({"method": "SUBSCRIBE","params": ["btcusdt@aggTrade"],"id": 1}).to_string(),
        ))
        .await
        .expect("failed to send WsMessage over socket");

    // Instantiate some arbitrary Transformer to apply to data parsed from the WebSocket protocol
    let transformer = StatefulTransformer { sum_of_volume: 0.0 };

    // ExchangeWsStream includes pre-defined WebSocket Sink/Stream & WebSocket StreamParser
    let mut ws_stream = ExchangeWsStream::new(binance_conn, transformer);

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
```

**For a larger, "real world" example, see the [`Toucan-Data`] repository.**

## Related Projects

In addition to the Toucan-Integration crate, the Toucan project also maintains:
* [`Toucan`]: High-performance, extensible & modular trading components with batteries-included. Contains a
  pre-built trading Engine that can serve as a live-trading or backtesting system.
* [`Toucan-Data`]: A high-performance WebSocket integration library for streaming public data from leading 
  cryptocurrency exchanges.
* [`Toucan-Execution`]: Financial exchange integrations for trade execution.

## Roadmap

* Add new default StreamParser implementations to enable integration with other popular systems such as Kafka. 
