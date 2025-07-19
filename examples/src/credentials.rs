/*!
 * Secure API Key Configuration Module
 * 
 * This module provides secure methods to load and manage API keys for trading.
 * It supports multiple approaches for different deployment scenarios.
 */

use std::env;

#[derive(Debug, Clone)]
pub struct ApiCredentials {
    pub api_key: String,
    pub secret_key: String,
}

#[derive(Debug, Clone)]
pub struct ExchangeCredentials {
    pub binance: Option<ApiCredentials>,
    pub binance_testnet: Option<ApiCredentials>,
    pub coinbase: Option<ApiCredentials>,
}

impl ExchangeCredentials {
    /// Load credentials from environment variables
    /// 
    /// This is the recommended approach for production and development.
    /// Create a .env file in your project root with your API keys.
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        // Try to load .env file (will silently fail if not found)
        let _ = dotenv::dotenv();

        Ok(ExchangeCredentials {
            binance: Self::load_binance_credentials()?,
            binance_testnet: Self::load_binance_testnet_credentials()?,
            coinbase: Self::load_coinbase_credentials()?,
        })
    }

    /// Load test credentials for unit testing
    /// 
    /// Uses mock credentials that are safe for automated testing.
    pub fn mock_credentials() -> Self {
        ExchangeCredentials {
            binance: Some(ApiCredentials {
                api_key: "mock_binance_api_key".to_string(),
                secret_key: "mock_binance_secret_key".to_string(),
            }),
            binance_testnet: Some(ApiCredentials {
                api_key: "mock_binance_testnet_api_key".to_string(),
                secret_key: "mock_binance_testnet_secret_key".to_string(),
            }),
            coinbase: Some(ApiCredentials {
                api_key: "mock_coinbase_api_key".to_string(),
                secret_key: "mock_coinbase_secret_key".to_string(),
            }),
        }
    }

    fn load_binance_credentials() -> Result<Option<ApiCredentials>, Box<dyn std::error::Error>> {
        match (env::var("BINANCE_API_KEY"), env::var("BINANCE_SECRET_KEY")) {
            (Ok(api_key), Ok(secret_key)) => {
                if !api_key.is_empty() && !secret_key.is_empty() {
                    Ok(Some(ApiCredentials { api_key, secret_key }))
                } else {
                    Ok(None)
                }
            }
            _ => Ok(None),
        }
    }

    fn load_binance_testnet_credentials() -> Result<Option<ApiCredentials>, Box<dyn std::error::Error>> {
        match (env::var("BINANCE_TESTNET_API_KEY"), env::var("BINANCE_TESTNET_SECRET_KEY")) {
            (Ok(api_key), Ok(secret_key)) => {
                if !api_key.is_empty() && !secret_key.is_empty() {
                    Ok(Some(ApiCredentials { api_key, secret_key }))
                } else {
                    Ok(None)
                }
            }
            _ => Ok(None),
        }
    }

    fn load_coinbase_credentials() -> Result<Option<ApiCredentials>, Box<dyn std::error::Error>> {
        match (env::var("COINBASE_API_KEY"), env::var("COINBASE_SECRET_KEY")) {
            (Ok(api_key), Ok(secret_key)) => {
                if !api_key.is_empty() && !secret_key.is_empty() {
                    Ok(Some(ApiCredentials { api_key, secret_key }))
                } else {
                    Ok(None)
                }
            }
            _ => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_credentials() {
        let creds = ExchangeCredentials::mock_credentials();
        assert!(creds.binance.is_some());
        assert!(creds.binance_testnet.is_some());
        assert!(creds.coinbase.is_some());
    }

    #[test]
    fn test_env_credentials_safe() {
        // This test should pass even without API keys set
        let result = ExchangeCredentials::from_env();
        assert!(result.is_ok());
    }
}
