//! Erros e códigos (NL_*) unificados entre mock e FFI.

pub type NResult = i32;

// Códigos NL_* (extendidos conforme manual). Valores negativos (HRESULT signed style).
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

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ProfitError {
    #[error("Erro interno DLL")] Internal,
    #[error("DLL não inicializada")] NotInitialized,
    #[error("Argumentos inválidos")] InvalidArgs,
    #[error("Aguardando servidor")] WaitingServer,
    #[error("Sem login")] NoLogin,
    #[error("Sem licença")] NoLicense,
    #[error("Fora de faixa")] OutOfRange,
    #[error("Função requer roteamento")] MarketOnly,
    #[error("Posição inexistente")] NoPosition,
    #[error("Recurso não encontrado")] NotFound,
    #[error("Versão não suportada")] VersionNotSupported,
    #[error("OCO sem regras")] OcoNoRules,
    #[error("Bolsa desconhecida")] ExchangeUnknown,
    #[error("OCO inexistente")] NoOcoDefined,
    #[error("Série inválida")] InvalidSerie,
    #[error("Recurso não liberado pela licença")] LicenseNotAllowed,
    #[error("Não está em HardLogout")] NotHardLogout,
    #[error("Série sem histórico")] SerieNoHistory,
    #[error("Ativo sem dados")] AssetNoData,
    #[error("Série sem dados")] SerieNoData,
    #[error("Estratégia em execução")] HasStrategyRunning,
    #[error("Sem mais histórico")] SerieNoMoreHistory,
    #[error("Série atingiu limite")] SerieMaxCount,
    #[error("Recurso duplicado")] DuplicateResource,
    #[error("Contrato não assinado")] UnsignedContract,
    #[error("Senha ausente")] NoPassword,
    #[error("Usuário ausente")] NoUser,
    #[error("Arquivo já existe")] FileAlreadyExists,
    #[error("Ticker inválido")] InvalidTicker,
    #[error("Conta não é master")] NotMasterAccount,
    #[error("Resultado desconhecido: {0}")] Unknown(NResult),
    #[cfg(all(target_os = "windows", feature = "real_dll"))]
    #[error("Falha carregando DLL: {0}")] Load(String),
    #[cfg(all(target_os = "windows", feature = "real_dll"))]
    #[error("Função não encontrada: {0}")] MissingSymbol(&'static str),
    #[error("Conexão falhou: {0}")] ConnectionFailed(String),
}

impl ProfitError {
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
