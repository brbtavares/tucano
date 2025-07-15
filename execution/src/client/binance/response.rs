use integration::{
    protocol::http::HttpParser,
    error::SocketError,
};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Binance API error response
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct BinanceApiError {
    pub code: i32,
    pub msg: String,
}

/// Binance HTTP response parser
#[derive(Debug, Clone)]
pub struct BinanceHttpParser;

impl BinanceHttpParser {
    /// Create a new Binance HTTP parser
    pub fn new() -> Self {
        Self
    }
}

impl Default for BinanceHttpParser {
    fn default() -> Self {
        Self::new()
    }
}

impl HttpParser for BinanceHttpParser {
    type ApiError = BinanceApiError;
    type OutputError = SocketError;

    fn parse_api_error(&self, status: StatusCode, api_error: Self::ApiError) -> Self::OutputError {
        let error_message = match api_error.code {
            -1000 => "Unknown error occurred".to_string(),
            -1001 => "Internal error; unable to process your request".to_string(),
            -1002 => "You are not authorized to execute this request".to_string(),
            -1003 => "Too many requests; current limit is %s requests per minute".to_string(),
            -1006 => "Unexpected response received from message bus".to_string(),
            -1007 => "Timeout waiting for response from backend server".to_string(),
            -1013 => "Invalid quantity or price precision".to_string(),
            -1014 => "Unsupported order combination".to_string(),
            -1015 => "Too many orders".to_string(),
            -1016 => "Service is shutting down".to_string(),
            -1020 => "Unsupported operation".to_string(),
            -1021 => "Invalid timestamp".to_string(),
            -1022 => "Invalid signature".to_string(),
            -1100 => "Illegal characters found in a parameter".to_string(),
            -1101 => "Too many parameters; maximum %s parameters are allowed".to_string(),
            -1102 => "Mandatory parameter missing".to_string(),
            -1103 => "Unknown parameter".to_string(),
            -1104 => "Not all sent parameters were read".to_string(),
            -1105 => "Parameter empty".to_string(),
            -1106 => "Parameter not required".to_string(),
            -1111 => "Precision over maximum defined for this asset".to_string(),
            -1112 => "No orders on book for symbol".to_string(),
            -1114 => "TimeInForce parameter sent when not required".to_string(),
            -1115 => "Invalid timeInForce".to_string(),
            -1116 => "Invalid orderType".to_string(),
            -1117 => "Invalid side".to_string(),
            -1118 => "New client order ID was empty".to_string(),
            -1119 => "Original client order ID was empty".to_string(),
            -1120 => "Invalid interval".to_string(),
            -1121 => "Invalid symbol".to_string(),
            -1125 => "Invalid listenKey".to_string(),
            -1127 => "More than %s hours between startTime and endTime".to_string(),
            -1128 => "Combination of optional parameters invalid".to_string(),
            -1130 => "Invalid data sent for a parameter".to_string(),
            -2010 => "Account has insufficient balance for requested action".to_string(),
            -2011 => "Margin account are not allowed to trade this symbol".to_string(),
            -2013 => "Order does not exist".to_string(),
            -2014 => "API key format invalid".to_string(),
            -2015 => "Invalid API key, IP, or permissions for action".to_string(),
            -2016 => "No trading window could be found for the symbol".to_string(),
            _ => format!("Binance API error ({}): {}", api_error.code, api_error.msg),
        };

        match status {
            StatusCode::BAD_REQUEST => SocketError::HttpResponse(status, error_message),
            StatusCode::UNAUTHORIZED => SocketError::Authentication(error_message),
            StatusCode::FORBIDDEN => SocketError::Authentication(error_message),
            StatusCode::TOO_MANY_REQUESTS => SocketError::RateLimit(error_message),
            StatusCode::INTERNAL_SERVER_ERROR => SocketError::HttpResponse(status, error_message),
            StatusCode::BAD_GATEWAY => SocketError::HttpResponse(status, error_message),
            StatusCode::SERVICE_UNAVAILABLE => SocketError::HttpResponse(status, error_message),
            StatusCode::GATEWAY_TIMEOUT => SocketError::Timeout(error_message),
            _ => SocketError::HttpResponse(status, error_message),
        }
    }
}
