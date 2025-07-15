use integration::{
    protocol::http::{
        private::RequestSigner,
        rest::RestRequest,
    },
    error::SocketError,
};
use chrono::Utc;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::borrow::Cow;
use reqwest::Method;
use serde::Serialize;

use super::model::*;

/// Binance request signer for API authentication
#[derive(Debug, Clone)]
pub struct BinanceRequestSigner {
    api_key: String,
    secret_key: String,
}

impl BinanceRequestSigner {
    /// Create a new Binance request signer
    pub fn new(api_key: String, secret_key: String) -> Self {
        Self {
            api_key,
            secret_key,
        }
    }

    /// Generate HMAC-SHA256 signature for Binance API
    fn generate_signature(&self, query_string: &str) -> String {
        let mut mac = Hmac::<Sha256>::new_from_slice(self.secret_key.as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(query_string.as_bytes());
        hex::encode(mac.finalize().into_bytes())
    }

    /// Create query string from parameters
    fn create_query_string(&self, params: &[(&str, &str)]) -> String {
        let timestamp = Utc::now().timestamp_millis();
        let mut query_params = params.to_vec();
        query_params.push(("timestamp", &timestamp.to_string()));
        
        query_params
            .iter()
            .map(|(key, value)| format!("{}={}", key, value))
            .collect::<Vec<_>>()
            .join("&")
    }
}

impl RequestSigner for BinanceRequestSigner {
    type Config<'a> = (&'a str, &'a str);

    fn new(config: Self::Config<'_>) -> Self {
        Self::new(config.0.to_string(), config.1.to_string())
    }

    fn sign<Request>(
        &self,
        request: &Request,
        builder: reqwest::RequestBuilder,
    ) -> Result<reqwest::Request, SocketError>
    where
        Request: RestRequest,
    {
        let mut builder = builder
            .header("X-MBX-APIKEY", &self.api_key)
            .header("Content-Type", "application/x-www-form-urlencoded");

        // For authenticated endpoints, we need to add signature
        let query_string = match request.query_params() {
            Some(params) => {
                // Convert params to query string format
                let serialized = serde_urlencoded::to_string(params)
                    .map_err(|e| SocketError::Serialization(format!("Failed to serialize params: {}", e)))?;
                format!("{}&timestamp={}", serialized, Utc::now().timestamp_millis())
            }
            None => format!("timestamp={}", Utc::now().timestamp_millis()),
        };

        let signature = self.generate_signature(&query_string);
        let signed_query = format!("{}&signature={}", query_string, signature);

        match Request::method() {
            Method::GET | Method::DELETE => {
                // For GET/DELETE, add signature to query parameters
                builder = builder.query(&[("signature", signature)]);
            }
            Method::POST | Method::PUT => {
                // For POST/PUT, add signature to body
                builder = builder.body(signed_query);
            }
            _ => {}
        }

        builder.build().map_err(|e| SocketError::BuildRequest(e.to_string()))
    }
}

// REST Request implementations for Binance API endpoints

/// Get server time
#[derive(Debug, Clone)]
pub struct GetServerTime;

impl RestRequest for GetServerTime {
    type Response = BinanceServerTime;
    type QueryParams = ();
    type Body = ();

    fn path(&self) -> Cow<'static, str> {
        Cow::Borrowed("/api/v3/time")
    }

    fn method() -> Method {
        Method::GET
    }
}

/// Get account information
#[derive(Debug, Clone)]
pub struct GetAccountInfo;

impl RestRequest for GetAccountInfo {
    type Response = BinanceAccountInfo;
    type QueryParams = ();
    type Body = ();

    fn path(&self) -> Cow<'static, str> {
        Cow::Borrowed("/api/v3/account")
    }

    fn method() -> Method {
        Method::GET
    }
}

/// Get open orders
#[derive(Debug, Clone)]
pub struct GetOpenOrders {
    pub symbol: Option<String>,
}

impl RestRequest for GetOpenOrders {
    type Response = Vec<BinanceOrder>;
    type QueryParams = GetOpenOrdersParams;
    type Body = ();

    fn path(&self) -> Cow<'static, str> {
        Cow::Borrowed("/api/v3/openOrders")
    }

    fn method() -> Method {
        Method::GET
    }

    fn query_params(&self) -> Option<&Self::QueryParams> {
        None // We'll handle this differently for Binance
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct GetOpenOrdersParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
}

/// Create new order
#[derive(Debug, Clone)]
pub struct CreateOrder {
    pub symbol: String,
    pub side: String,
    pub order_type: String,
    pub quantity: String,
    pub price: Option<String>,
    pub time_in_force: Option<String>,
    pub new_client_order_id: String,
}

impl RestRequest for CreateOrder {
    type Response = BinanceOrderResponse;
    type QueryParams = ();
    type Body = CreateOrderParams;

    fn path(&self) -> Cow<'static, str> {
        Cow::Borrowed("/api/v3/order")
    }

    fn method() -> Method {
        Method::POST
    }

    fn body(&self) -> Option<&Self::Body> {
        None // We'll handle this differently for Binance
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateOrderParams {
    pub symbol: String,
    pub side: String,
    #[serde(rename = "type")]
    pub order_type: String,
    pub quantity: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<String>,
    #[serde(rename = "timeInForce", skip_serializing_if = "Option::is_none")]
    pub time_in_force: Option<String>,
    #[serde(rename = "newClientOrderId")]
    pub new_client_order_id: String,
}

/// Cancel order
#[derive(Debug, Clone)]
pub struct CancelOrder {
    pub symbol: String,
    pub order_id: Option<u64>,
    pub orig_client_order_id: Option<String>,
}

impl RestRequest for CancelOrder {
    type Response = BinanceOrderCancel;
    type QueryParams = ();
    type Body = ();

    fn path(&self) -> Cow<'static, str> {
        Cow::Borrowed("/api/v3/order")
    }

    fn method() -> Method {
        Method::DELETE
    }
}

/// Get account trades
#[derive(Debug, Clone)]
pub struct GetAccountTrades {
    pub symbol: String,
    pub start_time: Option<u64>,
    pub end_time: Option<u64>,
    pub from_id: Option<u64>,
    pub limit: Option<u16>,
}

impl RestRequest for GetAccountTrades {
    type Response = Vec<BinanceTrade>;
    type QueryParams = ();
    type Body = ();

    fn path(&self) -> Cow<'static, str> {
        Cow::Borrowed("/api/v3/myTrades")
    }

    fn method() -> Method {
        Method::GET
    }
}
