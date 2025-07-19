/*!
 * Binance REST API Signed Request Example
 * 
 * This example demonstrates how to:
 * 1. Create a signed REST request to Binance exchange
 * 2. Handle authentication with API keys loaded from environment variables
 * 3. Parse and display the response
 * 
 * Setup:
 * 1. Copy .env.example to .env
 * 2. Add your Binance API credentials to .env
 * 3. Run: cargo run --bin signed_get_request
 */

use examples::credentials::ExchangeCredentials;
use serde::Deserialize;
use serde_json::Value;
use std::error::Error;
use std::fmt;

/// Custom error type for this example
#[derive(Debug)]
pub enum ExampleError {
    Http(Box<dyn Error + Send + Sync>),
    Json(serde_json::Error),
    ApiError(String),
    CredentialsError(String),
}

impl fmt::Display for ExampleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExampleError::Http(err) => write!(f, "HTTP error: {}", err),
            ExampleError::Json(err) => write!(f, "JSON error: {}", err),
            ExampleError::ApiError(msg) => write!(f, "API error: {}", msg),
            ExampleError::CredentialsError(msg) => write!(f, "Credentials error: {}", msg),
        }
    }
}

impl Error for ExampleError {}

impl From<serde_json::Error> for ExampleError {
    fn from(err: serde_json::Error) -> Self {
        ExampleError::Json(err)
    }
}

impl From<reqwest::Error> for ExampleError {
    fn from(err: reqwest::Error) -> Self {
        ExampleError::Http(Box::new(err))
    }
}

/// Binance API signature implementation
pub struct BinanceSigner {
    api_key: String,
    secret_key: String,
}

impl BinanceSigner {
    pub fn new(api_key: String, secret_key: String) -> Self {
        Self { api_key, secret_key }
    }

    fn create_signature(&self, query_string: &str) -> String {
        use hmac::{Hmac, Mac};
        use sha2::Sha256;
        
        type HmacSha256 = Hmac<Sha256>;
        
        let mut mac = HmacSha256::new_from_slice(self.secret_key.as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(query_string.as_bytes());
        hex::encode(mac.finalize().into_bytes())
    }

    pub fn sign_request(&self, query_params: &[(&str, &str)]) -> (String, String) {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();
        
        // Build query string with timestamp
        let mut params = query_params.to_vec();
        let timestamp_str = timestamp.to_string();
        params.push(("timestamp", &timestamp_str));
        
        let query_string = params
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("&");
        
        let signature = self.create_signature(&query_string);
        
        (format!("{}&signature={}", query_string, signature), self.api_key.clone())
    }
}

#[derive(Deserialize, Debug)]
pub struct BinanceAccountInfo {
    #[serde(rename = "makerCommission")]
    pub maker_commission: u64,
    #[serde(rename = "takerCommission")]
    pub taker_commission: u64,
    #[serde(rename = "buyerCommission")]
    pub buyer_commission: u64,
    #[serde(rename = "sellerCommission")]
    pub seller_commission: u64,
    #[serde(rename = "canTrade")]
    pub can_trade: bool,
    #[serde(rename = "canWithdraw")]
    pub can_withdraw: bool,
    #[serde(rename = "canDeposit")]
    pub can_deposit: bool,
    pub balances: Vec<BinanceBalance>,
}

#[derive(Deserialize, Debug)]
pub struct BinanceBalance {
    pub asset: String,
    pub free: String,
    pub locked: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("🚀 Binance Signed Request Example");
    println!("==================================");
    
    // Load API credentials securely from environment
    println!("📋 Loading API credentials...");
    let credentials = ExchangeCredentials::from_env()
        .map_err(|e| ExampleError::CredentialsError(format!("Failed to load credentials: {}", e)))?;
    
    let binance_creds = credentials.binance.ok_or_else(|| {
        ExampleError::CredentialsError(
            "No Binance credentials found. Please set BINANCE_API_KEY and BINANCE_SECRET_KEY in your .env file".to_string()
        )
    })?;
    
    println!("✅ Binance credentials loaded (API Key: {}...)", &binance_creds.api_key[..8]);
    
    // Create the signer with loaded credentials
    let signer = BinanceSigner::new(binance_creds.api_key.clone(), binance_creds.secret_key);
    
    // Create HTTP client
    let client = reqwest::Client::new();
    
    // Define the API endpoint and parameters
    let base_url = "https://api.binance.com";
    let endpoint = "/api/v3/account";
    let url = format!("{}{}", base_url, endpoint);
    
    println!("🌐 Target URL: {}", url);
    
    // Sign the request
    let query_params = &[]; // No additional query params for account info
    let (signed_query, api_key) = signer.sign_request(query_params);
    
    println!("🔐 Request signed successfully");
    
    // Build the complete URL with signed parameters
    let full_url = format!("{}?{}", url, signed_query);
    
    // Execute the request
    println!("📡 Executing signed request...");
    
    let response = client
        .get(&full_url)
        .header("X-MBX-APIKEY", &api_key)
        .send()
        .await
        .map_err(|e| ExampleError::Http(Box::new(e)))?;
    
    let status = response.status();
    println!("📊 Response Status: {}", status);
    
    if status.is_success() {
        // Parse successful response
        let response_text = response.text().await
            .map_err(|e| ExampleError::Http(Box::new(e)))?;
        
        match serde_json::from_str::<BinanceAccountInfo>(&response_text) {
            Ok(account_info) => {
                println!("✅ Request successful!");
                println!("📊 Account Info:");
                println!("   - Can Trade: {}", account_info.can_trade);
                println!("   - Can Withdraw: {}", account_info.can_withdraw);
                println!("   - Can Deposit: {}", account_info.can_deposit);
                println!("   - Maker Commission: {}", account_info.maker_commission);
                println!("   - Taker Commission: {}", account_info.taker_commission);
                println!("   - Total Balances: {} assets", account_info.balances.len());
                
                // Show first few non-zero balances
                let non_zero_balances: Vec<_> = account_info.balances
                    .iter()
                    .filter(|b| b.free != "0.00000000" || b.locked != "0.00000000")
                    .take(5)
                    .collect();
                
                if !non_zero_balances.is_empty() {
                    println!("   - Non-zero balances:");
                    for balance in non_zero_balances {
                        println!("     {} - Free: {}, Locked: {}", 
                            balance.asset, balance.free, balance.locked);
                    }
                } else {
                    println!("   - No non-zero balances found");
                }
            }
            Err(parse_err) => {
                println!("⚠️  Response received but failed to parse as account info:");
                println!("{}", response_text);
                return Err(Box::new(ExampleError::Json(parse_err)) as Box<dyn Error>);
            }
        }
    } else {
        // Handle error response
        let error_text = response.text().await
            .map_err(|e| ExampleError::Http(Box::new(e)))?;
        
        // Try to parse as Binance error format
        match serde_json::from_str::<Value>(&error_text) {
            Ok(error_json) => {
                if let Some(code) = error_json.get("code") {
                    if let Some(msg) = error_json.get("msg") {
                        let error_msg = format!("Binance API Error {}: {}", code, msg);
                        println!("❌ {}", error_msg);
                        return Err(Box::new(ExampleError::ApiError(error_msg)) as Box<dyn Error>);
                    }
                }
                println!("❌ API Error: {}", error_json);
            }
            Err(_) => {
                println!("❌ HTTP Error {}: {}", status, error_text);
            }
        }
        
        return Err(Box::new(ExampleError::ApiError(format!("HTTP {}: {}", status, error_text))) as Box<dyn Error>);
    }
    
    println!("🎉 Example completed successfully!");
    Ok(())
}
