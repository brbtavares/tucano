

//! Unified errors and codes (**NL_***) between mock and FFI ProfitDLL.
//!
//! All codes and enums follow the official DLL specification (see [MANUAL.md](../MANUAL.md#erros)).

/// Default return type of the DLL (**NResult**).
pub type NResult = i32;

// **NL_*** codes (extended as per manual). Negative values (HRESULT signed style).
pub const NL_OK: NResult = 0;
pub const NL_INTERNAL_ERROR: NResult = -2147483647;
pub const NL_NOT_INITIALIZED: NResult = -2147483646;
pub const NL_INVALID_ARGS: NResult = -2147483645;
pub const NL_WAITING_SERVER: NResult = -2147483644;
pub const NL_NO_LOGIN: NResult = -2147483643;
pub const NL_NO_LICENSE: NResult = -2147483642;
pub const NL_OUT_OF_RANGE: NResult = -2147483639;
pub const NL_MARKET_ONLY: NResult = -2147483638;
pub const NL_NO_POSITION: NResult = -2147483637;
pub const NL_NOT_FOUND: NResult = -2147483636;
pub const NL_VERSION_NOT_SUPPORTED: NResult = -2147483635;
pub const NL_OCO_NO_RULES: NResult = -2147483634;
pub const NL_EXCHANGE_UNKNOWN: NResult = -2147483633;
pub const NL_NO_OCO_DEFINED: NResult = -2147483632;
pub const NL_INVALID_SERIE: NResult = -2147483631;
pub const NL_LICENSE_NOT_ALLOWED: NResult = -2147483630;
pub const NL_NOT_HARD_LOGOUT: NResult = -2147483629;
pub const NL_SERIE_NO_HISTORY: NResult = -2147483628;
pub const NL_ASSET_NO_DATA: NResult = -2147483627;
pub const NL_SERIE_NO_DATA: NResult = -2147483626;
pub const NL_HAS_STRATEGY_RUNNING: NResult = -2147483625;
pub const NL_SERIE_NO_MORE_HISTORY: NResult = -2147483624;
pub const NL_SERIE_MAX_COUNT: NResult = -2147483623;
pub const NL_DUPLICATE_RESOURCE: NResult = -2147483622;
pub const NL_UNSIGNED_CONTRACT: NResult = -2147483621;
pub const NL_NO_PASSWORD: NResult = -2147483620;
pub const NL_NO_USER: NResult = -2147483619;
pub const NL_FILE_ALREADY_EXISTS: NResult = -2147483618;
pub const NL_INVALID_TICKER: NResult = -2147483617;
pub const NL_NOT_MASTER_ACCOUNT: NResult = -2147483616;

/// Unified error enum for the ProfitDLL interface (**ProfitError**).
///
/// Each variant corresponds to an **NL_*** code or integration error described in [MANUAL.md](../MANUAL.md#erros).
#[non_exhaustive]
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ProfitError {
    #[error("DLL internal error (**NL_INTERNAL_ERROR**)")]
    Internal,
    #[error("DLL not initialized (**NL_NOT_INITIALIZED**)")]
    NotInitialized,
    #[error("Invalid arguments (**NL_INVALID_ARGS**)")]
    InvalidArgs,
    #[error("Waiting for server (**NL_WAITING_SERVER**)")]
    WaitingServer,
    #[error("No login (**NL_NO_LOGIN**)")]
    NoLogin,
    #[error("No license (**NL_NO_LICENSE**)")]
    NoLicense,
    #[error("Fora de faixa (**NL_OUT_OF_RANGE**)")]
    OutOfRange,
    #[error("Function requires routing (**NL_MARKET_ONLY**)")]
    MarketOnly,
    #[error("Position does not exist (**NL_NO_POSITION**)")]
    NoPosition,
    #[error("Resource not found (**NL_NOT_FOUND**)")]
    NotFound,
    #[error("Version not supported (**NL_VERSION_NOT_SUPPORTED**)")]
    VersionNotSupported,
    #[error("OCO no rules (**NL_OCO_NO_RULES**)")]
    OcoNoRules,
    #[error("Unknown exchange (**NL_EXCHANGE_UNKNOWN**)")]
    ExchangeUnknown,
    #[error("No OCO defined (**NL_NO_OCO_DEFINED**)")]
    NoOcoDefined,
    #[error("Invalid serie (**NL_INVALID_SERIE**)")]
    InvalidSerie,
    #[error("Resource not allowed by license (**NL_LICENSE_NOT_ALLOWED**)")]
    LicenseNotAllowed,
    #[error("Not in HardLogout (**NL_NOT_HARD_LOGOUT**)")]
    NotHardLogout,
    #[error("Serie has no history (**NL_SERIE_NO_HISTORY**)")]
    SerieNoHistory,
    #[error("Asset has no data (**NL_ASSET_NO_DATA**)")]
    AssetNoData,
    #[error("Serie has no data (**NL_SERIE_NO_DATA**)")]
    SerieNoData,
    #[error("Strategy running (**NL_HAS_STRATEGY_RUNNING**)")]
    HasStrategyRunning,
    #[error("No more history (**NL_SERIE_NO_MORE_HISTORY**)")]
    SerieNoMoreHistory,
    #[error("Serie reached limit (**NL_SERIE_MAX_COUNT**)")]
    SerieMaxCount,
    #[error("Duplicate resource (**NL_DUPLICATE_RESOURCE**)")]
    DuplicateResource,
    #[error("Unsigned contract (**NL_UNSIGNED_CONTRACT**)")]
    UnsignedContract,
    #[error("Password missing (**NL_NO_PASSWORD**)")]
    NoPassword,
    #[error("User missing (**NL_NO_USER**)")]
    NoUser,
    #[error("File already exists (**NL_FILE_ALREADY_EXISTS**)")]
    FileAlreadyExists,
    #[error("Invalid ticker (**NL_INVALID_TICKER**)")]
    InvalidTicker,
    #[error("Account is not master (**NL_NOT_MASTER_ACCOUNT**)")]
    NotMasterAccount,
    #[error("Unknown result: {0}")]
    Unknown(NResult),
    #[cfg(all(target_os = "windows", feature = "real_dll"))]
    #[error("DLL load failed: {0}")]
    Load(String),
    #[cfg(all(target_os = "windows", feature = "real_dll"))]
    #[error("Function not found: {0}")]
    MissingSymbol(&'static str),
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
}

impl ProfitError {
    /// Converts a **NResult** code from the DLL into a [`ProfitError`].
    pub fn from_nresult(code: NResult) -> Result<(), ProfitError> {
        use ProfitError::*;
        match code {
            NL_OK => Ok(()),
            NL_INTERNAL_ERROR => Err(Internal),
            NL_NOT_INITIALIZED => Err(NotInitialized),
            NL_INVALID_ARGS => Err(InvalidArgs),
            NL_WAITING_SERVER => Err(WaitingServer),
            NL_NO_LOGIN => Err(NoLogin),
            NL_NO_LICENSE => Err(NoLicense),
            NL_OUT_OF_RANGE => Err(OutOfRange),
            NL_MARKET_ONLY => Err(MarketOnly),
            NL_NO_POSITION => Err(NoPosition),
            NL_NOT_FOUND => Err(NotFound),
            NL_VERSION_NOT_SUPPORTED => Err(VersionNotSupported),
            NL_OCO_NO_RULES => Err(OcoNoRules),
            NL_EXCHANGE_UNKNOWN => Err(ExchangeUnknown),
            NL_NO_OCO_DEFINED => Err(NoOcoDefined),
            NL_INVALID_SERIE => Err(InvalidSerie),
            NL_LICENSE_NOT_ALLOWED => Err(LicenseNotAllowed),
            NL_NOT_HARD_LOGOUT => Err(NotHardLogout),
            NL_SERIE_NO_HISTORY => Err(SerieNoHistory),
            NL_ASSET_NO_DATA => Err(AssetNoData),
            NL_SERIE_NO_DATA => Err(SerieNoData),
            NL_HAS_STRATEGY_RUNNING => Err(HasStrategyRunning),
            NL_SERIE_NO_MORE_HISTORY => Err(SerieNoMoreHistory),
            NL_SERIE_MAX_COUNT => Err(SerieMaxCount),
            NL_DUPLICATE_RESOURCE => Err(DuplicateResource),
            NL_UNSIGNED_CONTRACT => Err(UnsignedContract),
            NL_NO_PASSWORD => Err(NoPassword),
            NL_NO_USER => Err(NoUser),
            NL_FILE_ALREADY_EXISTS => Err(FileAlreadyExists),
            NL_INVALID_TICKER => Err(InvalidTicker),
            NL_NOT_MASTER_ACCOUNT => Err(NotMasterAccount),
            other => Err(Unknown(other)),
        }
    }
}
