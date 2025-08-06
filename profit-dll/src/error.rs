//! Definições de erro para ProfitDLL wrapper

use thiserror::Error;

/// Erros específicos do wrapper ProfitDLL
#[derive(Error, Debug)]
pub enum ProfitError {
    #[error("DLL not initialized")]
    NotInitialized,
    
    #[error("Invalid arguments: {0}")]
    InvalidArguments(String),
    
    #[error("Waiting for server response")]
    WaitingServer,
    
    #[error("No login found")]
    NoLogin,
    
    #[error("No license found")]
    NoLicense,
    
    #[error("Value out of range")]
    OutOfRange,
    
    #[error("Market data only license - trading not allowed")]
    MarketOnly,
    
    #[error("No position found")]
    NoPosition,
    
    #[error("Resource not found")]
    NotFound,
    
    #[error("Version not supported")]
    VersionNotSupported,
    
    #[error("OCO has no rules")]
    OcoNoRules,
    
    #[error("Exchange is unknown")]
    ExchangeUnknown,
    
    #[error("No OCO defined")]
    NoOcoDefined,
    
    #[error("Invalid serie")]
    InvalidSerie,
    
    #[error("License not allowed")]
    LicenseNotAllowed,
    
    #[error("Not in hard logout state")]
    NotHardLogout,
    
    #[error("Connection failure: {0}")]
    ConnectionFailure(String),
    
    #[error("Serie has no history")]
    SerieNoHistory,
    
    #[error("Asset has no data")]
    AssetNoData,
    
    #[error("Serie has no data")]
    SerieNoData,
    
    #[error("Serie has no more history")]
    SerieNoMoreHistory,
    
    #[error("Serie at max count limit")]
    SerieMaxCount,
    
    #[error("Duplicate resource")]
    DuplicateResource,
    
    #[error("Unsigned contract")]
    UnsignedContract,
    
    #[error("No password supplied")]
    NoPassword,
    
    #[error("No user supplied")]
    NoUser,
    
    #[error("File already exists")]
    FileAlreadyExists,
    
    #[error("Invalid ticker")]
    InvalidTicker,
    
    #[error("Account is not a master account")]
    NotMasterAccount,
    
    #[error("Internal DLL error")]
    InternalError,
    
    #[cfg(windows)]
    #[error("Windows API error: {0}")]
    WindowsError(#[from] windows::core::Error),
    
    #[error("String conversion error")]
    StringConversion,
    
    #[error("DLL function not found: {0}")]
    FunctionNotFound(String),
    
    #[error("DLL library not found at path: {0}")]
    LibraryNotFound(String),
    
    #[error("Invalid date format: {0}")]
    InvalidDateFormat(String),
    
    #[error("Unknown error code: {0}")]
    Unknown(i32),
}

impl From<crate::types::NResult> for ProfitError {
    fn from(result: crate::types::NResult) -> Self {
        use crate::types::*;
        
        match result {
            NL_OK => unreachable!("NL_OK should not be converted to error"),
            NL_INTERNAL_ERROR => ProfitError::InternalError,
            NL_NOT_INITIALIZED => ProfitError::NotInitialized,
            NL_INVALID_ARGS => ProfitError::InvalidArguments("Invalid arguments from DLL".to_string()),
            NL_WAITING_SERVER => ProfitError::WaitingServer,
            NL_NO_LOGIN => ProfitError::NoLogin,
            NL_NO_LICENSE => ProfitError::NoLicense,
            NL_OUT_OF_RANGE => ProfitError::OutOfRange,
            NL_MARKET_ONLY => ProfitError::MarketOnly,
            NL_NO_POSITION => ProfitError::NoPosition,
            NL_NOT_FOUND => ProfitError::NotFound,
            NL_VERSION_NOT_SUPPORTED => ProfitError::VersionNotSupported,
            NL_OCO_NO_RULES => ProfitError::OcoNoRules,
            NL_EXCHANGE_UNKNOWN => ProfitError::ExchangeUnknown,
            NL_NO_OCO_DEFINED => ProfitError::NoOcoDefined,
            NL_INVALID_SERIE => ProfitError::InvalidSerie,
            NL_LICENSE_NOT_ALLOWED => ProfitError::LicenseNotAllowed,
            NL_NOT_HARD_LOGOUT => ProfitError::NotHardLogout,
            NL_SERIE_NO_HISTORY => ProfitError::SerieNoHistory,
            NL_ASSET_NO_DATA => ProfitError::AssetNoData,
            NL_SERIE_NO_DATA => ProfitError::SerieNoData,
            NL_SERIE_NO_MORE_HISTORY => ProfitError::SerieNoMoreHistory,
            NL_SERIE_MAX_COUNT => ProfitError::SerieMaxCount,
            NL_DUPLICATE_RESOURCE => ProfitError::DuplicateResource,
            NL_UNSIGNED_CONTRACT => ProfitError::UnsignedContract,
            NL_NO_PASSWORD => ProfitError::NoPassword,
            NL_NO_USER => ProfitError::NoUser,
            NL_FILE_ALREADY_EXISTS => ProfitError::FileAlreadyExists,
            NL_INVALID_TICKER => ProfitError::InvalidTicker,
            NL_NOT_MASTER_ACCOUNT => ProfitError::NotMasterAccount,
            code => ProfitError::Unknown(code),
        }
    }
}

/// Alias para Result com ProfitError
pub type Result<T> = std::result::Result<T, ProfitError>;
