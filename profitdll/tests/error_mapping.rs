// Mini-Disclaimer: Educational/experimental use; not investment advice or affiliation; see README & DISCLAIMER.
//! Testes unitários para mapeamento de erros NL_* -> ProfitError.

use tucano_profitdll::{
    ProfitError, NL_ASSET_NO_DATA, NL_DUPLICATE_RESOURCE, NL_EXCHANGE_UNKNOWN,
    NL_FILE_ALREADY_EXISTS, NL_HAS_STRATEGY_RUNNING, NL_INTERNAL_ERROR, NL_INVALID_ARGS,
    NL_INVALID_SERIE, NL_INVALID_TICKER, NL_LICENSE_NOT_ALLOWED, NL_MARKET_ONLY, NL_NOT_FOUND,
    NL_NOT_HARD_LOGOUT, NL_NOT_INITIALIZED, NL_NOT_MASTER_ACCOUNT, NL_NO_LICENSE, NL_NO_LOGIN,
    NL_NO_OCO_DEFINED, NL_NO_PASSWORD, NL_NO_POSITION, NL_NO_USER, NL_OCO_NO_RULES, NL_OK,
    NL_OUT_OF_RANGE, NL_SERIE_MAX_COUNT, NL_SERIE_NO_DATA, NL_SERIE_NO_HISTORY,
    NL_SERIE_NO_MORE_HISTORY, NL_UNSIGNED_CONTRACT, NL_VERSION_NOT_SUPPORTED, NL_WAITING_SERVER,
};

#[test]
fn all_known_error_codes_map() {
    let known = [
        NL_OK,
        NL_INTERNAL_ERROR,
        NL_NOT_INITIALIZED,
        NL_INVALID_ARGS,
        NL_WAITING_SERVER,
        NL_NO_LOGIN,
        NL_NO_LICENSE,
        NL_OUT_OF_RANGE,
        NL_MARKET_ONLY,
        NL_NO_POSITION,
        NL_NOT_FOUND,
        NL_VERSION_NOT_SUPPORTED,
        NL_OCO_NO_RULES,
        NL_EXCHANGE_UNKNOWN,
        NL_NO_OCO_DEFINED,
        NL_INVALID_SERIE,
        NL_LICENSE_NOT_ALLOWED,
        NL_NOT_HARD_LOGOUT,
        NL_SERIE_NO_HISTORY,
        NL_ASSET_NO_DATA,
        NL_SERIE_NO_DATA,
        NL_HAS_STRATEGY_RUNNING,
        NL_SERIE_NO_MORE_HISTORY,
        NL_SERIE_MAX_COUNT,
        NL_DUPLICATE_RESOURCE,
        NL_UNSIGNED_CONTRACT,
        NL_NO_PASSWORD,
        NL_NO_USER,
        NL_FILE_ALREADY_EXISTS,
        NL_INVALID_TICKER,
        NL_NOT_MASTER_ACCOUNT,
    ];
    for code in known {
        let _ = ProfitError::from_nresult(code);
    }
}

#[test]
fn unknown_code_maps_to_unknown() {
    let err = ProfitError::from_nresult(-1234567).unwrap_err();
    match err {
        ProfitError::Unknown(v) => assert_eq!(v, -1234567),
        _ => panic!("esperado Unknown"),
    }
}
