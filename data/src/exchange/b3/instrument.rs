// Mini-Disclaimer: Uso educacional/experimental; sem recomendação de investimento ou afiliação; sem remuneração de terceiros; Profit/ProfitDLL © Nelógica; veja README & DISCLAIMER.
//! B3 instrument definitions and utilities

use crate::instrument::InstrumentData;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use tucano_markets::{
    Instrument,
    InstrumentKind, // Import trait from simplified markets
};

/// B3 instrument data structure
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct B3Instrument {
    pub ticker: String,
    pub exchange: String,
    pub security_type: B3SecurityType,
    pub isin: Option<String>,
    pub description: Option<String>,
    pub lot_size: Option<i32>,
    pub tick_size: Option<Decimal>,
}

/// Types of securities available on B3
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum B3SecurityType {
    /// Common stocks (Ações)
    Stock,
    /// Exchange Traded Funds
    Etf,
    /// Real Estate Investment Trusts (FIIs)
    Reit,
    /// Brazilian Depositary Receipts
    Bdr,
    /// Options on stocks
    Option,
    /// Futures contracts
    Future,
    /// Forward contracts
    Forward,
    /// Government bonds
    Bond,
    /// Corporate debentures
    Debenture,
    /// Other instrument types
    Other,
}

impl From<&str> for B3SecurityType {
    fn from(security_type: &str) -> Self {
        match security_type.to_uppercase().as_str() {
            "STOCK" | "AÇÃO" => B3SecurityType::Stock,
            "ETF" => B3SecurityType::Etf,
            "REIT" | "FII" => B3SecurityType::Reit,
            "BDR" => B3SecurityType::Bdr,
            "OPTION" | "OPÇÃO" => B3SecurityType::Option,
            "FUTURE" | "FUTURO" => B3SecurityType::Future,
            "FORWARD" | "TERMO" => B3SecurityType::Forward,
            "BOND" | "TÍTULO" => B3SecurityType::Bond,
            "DEBENTURE" | "DEBÊNTURE" => B3SecurityType::Debenture,
            _ => B3SecurityType::Other,
        }
    }
}

impl From<B3SecurityType> for InstrumentKind {
    fn from(b3_type: B3SecurityType) -> Self {
        match b3_type {
            B3SecurityType::Stock
            | B3SecurityType::Etf
            | B3SecurityType::Reit
            | B3SecurityType::Bdr => InstrumentKind::Spot,
            B3SecurityType::Option => {
                // For now, return Spot since we don't have option contract details
                InstrumentKind::Spot
            }
            B3SecurityType::Future | B3SecurityType::Forward => {
                // For now, return Perpetual since we don't have expiry details
                InstrumentKind::Perpetual
            }
            B3SecurityType::Bond | B3SecurityType::Debenture => InstrumentKind::Spot,
            B3SecurityType::Other => InstrumentKind::Spot,
        }
    }
}

impl B3Instrument {
    /// Create a new B3 instrument
    pub fn new(ticker: String, exchange: String, security_type: B3SecurityType) -> Self {
        Self {
            ticker,
            exchange,
            security_type,
            isin: None,
            description: None,
            lot_size: None,
            tick_size: None,
        }
    }

    /// Create a BOVESPA stock instrument
    pub fn bovespa(ticker: impl Into<String>) -> Self {
        Self::new(ticker.into(), "BOVESPA".to_string(), B3SecurityType::Stock)
    }

    /// Create a BMF futures instrument
    pub fn bmf(ticker: impl Into<String>) -> Self {
        Self::new(ticker.into(), "BMF".to_string(), B3SecurityType::Future)
    }

    /// Get the full symbol including exchange
    pub fn symbol(&self) -> String {
        format!("{}@{}", self.ticker, self.exchange)
    }

    /// Check if this is a stock instrument
    pub fn is_stock(&self) -> bool {
        matches!(self.security_type, B3SecurityType::Stock)
    }

    /// Check if this is a derivative instrument
    pub fn is_derivative(&self) -> bool {
        matches!(
            self.security_type,
            B3SecurityType::Option | B3SecurityType::Future | B3SecurityType::Forward
        )
    }

    /// Check if this is a fixed income instrument
    pub fn is_fixed_income(&self) -> bool {
        matches!(
            self.security_type,
            B3SecurityType::Bond | B3SecurityType::Debenture
        )
    }
}

/// Implement tucano_markets::Instrument trait for B3Instrument
impl Instrument for B3Instrument {
    type Symbol = String;

    fn symbol(&self) -> &Self::Symbol {
        &self.ticker
    }

    fn market(&self) -> &str {
        &self.exchange
    }
}

impl InstrumentData for B3Instrument {
    type Key = String;

    fn key(&self) -> &Self::Key {
        &self.ticker
    }

    fn kind(&self) -> &InstrumentKind {
        // For now, we'll store the kind in the instrument
        // This is a simplified implementation
        match self.security_type {
            B3SecurityType::Stock
            | B3SecurityType::Etf
            | B3SecurityType::Reit
            | B3SecurityType::Bdr => &InstrumentKind::Spot,
            _ => &InstrumentKind::Spot,
        }
    }
}

/// Utility functions for B3 instrument handling
pub mod utils {
    use super::*;

    /// Parse a ticker symbol to extract underlying asset for derivatives
    pub fn parse_underlying(ticker: &str) -> Option<String> {
        // Padrões suportados (heurísticos simples):
        //  - Opções:  <UNDERLYING><LETRA><DIGITOS{2,}>   ex: PETR4P250 -> PETR4
        //  - Futuros: <UNDERLYING><LETRA><DIGITOS{2}>     ex: WINV24   -> WIN
        // Heurística: contar dígitos finais (>=2), pegar letra imediatamente anterior
        // e retornar prefixo antes dessa letra.
        let chars: Vec<char> = ticker.chars().collect();
        if chars.len() < 6 {
            // mínimo razoável para derivativo (ex: WINV24 tem 6)
            return None;
        }
        // Heurísticas simples para B3:
        // Opções: subjacente + letra(s) + dígitos (ex: PETR4P250 => PETR4)
        // Futuros: código base + letra mês + dois dígitos ano (ex: WINV24 => WIN)

        if ticker.len() < 3 {
            return None;
        }

        // 1. Detectar um sufixo de mês/ano de futuro (Letra + 2 dígitos finais)
        if ticker.len() >= 4 {
            let chars: Vec<char> = ticker.chars().collect();
            if chars.len() >= 3
                && chars[chars.len() - 3].is_alphabetic()
                && chars[chars.len() - 2].is_ascii_digit()
                && chars[chars.len() - 1].is_ascii_digit()
            {
                // Exemplo: WINV24 -> base até antes da letra do mês (remover 3 últimos e depois remover eventual letra final do base?)
                // Simplificação: cortar os últimos 3 chars e se o resultante terminar com letra do mês anterior deixamos como está.
                let base = &ticker[..ticker.len() - 3];
                if base.len() >= 3 {
                    // heurística mínima
                    return Some(base.to_string());
                }
            }
        }

        // 2. Opções: encontrar fronteira onde começam blocos alfanuméricos após o subjacente.
        // Estratégia: contar dígitos finais consecutivos. Se >= 2 e antes houver uma letra distinta do padrão numérico de ação, extrair prefixo antes da sequência letra+digitos.
        let mut digits_at_end = 0;
        for c in ticker.chars().rev() {
            if c.is_ascii_digit() {
                digits_at_end += 1;
            } else {
                break;
            }
        }
        if digits_at_end >= 2 && digits_at_end < ticker.len() {
            // Remover dígitos finais para analisar possível letra(s) de série
            let without_digits = &ticker[..ticker.len() - digits_at_end];
            // Ex: PETR4P -> remover último bloco de letras de série (P) se existir
            let mut series_len = 0;
            for c in without_digits.chars().rev() {
                if c.is_ascii_alphabetic() {
                    series_len += 1;
                } else {
                    break;
                }
            }
            if series_len > 0 && series_len < without_digits.len() {
                let underlying_candidate = &without_digits[..without_digits.len() - series_len];
                // Validar que termina com dígito típico de ação (ex 3/4/11 etc.)
                if underlying_candidate
                    .chars()
                    .last()
                    .map(|c| c.is_ascii_digit())
                    .unwrap_or(false)
                {
                    return Some(underlying_candidate.to_string());
                }
            }
        }

        // Contar dígitos consecutivos no fim
        let mut idx = chars.len();
        while idx > 0 && chars[idx - 1].is_ascii_digit() {
            idx -= 1;
        }
        let digits_start = idx;
        let digits_len = chars.len() - digits_start;
        if digits_len < 2 {
            // precisa de pelo menos 2 dígitos para ano / strike
            return None;
        }
        if digits_start == 0 {
            return None;
        }
        let letter_pos = digits_start - 1;
        let letter = chars[letter_pos];
        if !letter.is_ascii_alphabetic() {
            return None;
        }
        if letter_pos < 3 {
            // subjacente muito curto, provavelmente não derivativo
            return None;
        }
        let underlying = &ticker[..letter_pos];
        Some(underlying.to_string())
    }

    /// Determine security type from ticker pattern
    pub fn infer_security_type(ticker: &str) -> B3SecurityType {
        if ticker.ends_with("11") {
            B3SecurityType::Reit
        } else if ticker.len() > 5 && ticker.chars().nth_back(0).is_some_and(|c| c.is_numeric()) {
            B3SecurityType::Option
        } else if ticker.len() <= 4 || (ticker.len() == 5 && ticker.ends_with('4')) {
            B3SecurityType::Stock
        } else {
            B3SecurityType::Other
        }
    }

    /// Validate B3 ticker format
    pub fn is_valid_ticker(ticker: &str) -> bool {
        if ticker.is_empty() || ticker.len() > 12 {
            return false;
        }

        // Must start with alphabetic characters
        let first_char = ticker.chars().next().unwrap_or(' ');
        if !first_char.is_alphabetic() {
            return false;
        }

        // Can contain only alphanumeric characters
        ticker.chars().all(|c| c.is_alphanumeric())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_type_inference() {
        assert_eq!(utils::infer_security_type("PETR4"), B3SecurityType::Stock);
        assert_eq!(utils::infer_security_type("BBDC11"), B3SecurityType::Reit);
        assert_eq!(
            utils::infer_security_type("PETR4P250"),
            B3SecurityType::Option
        );
    }

    #[test]
    fn test_underlying_parsing() {
        assert_eq!(
            utils::parse_underlying("PETR4P250"),
            Some("PETR4".to_string())
        );
        assert_eq!(utils::parse_underlying("WINV24"), Some("WIN".to_string()));
        assert_eq!(utils::parse_underlying("PETR4"), None);
    }

    #[test]
    fn test_ticker_validation() {
        assert!(utils::is_valid_ticker("PETR4"));
        assert!(utils::is_valid_ticker("BBDC11"));
        assert!(!utils::is_valid_ticker(""));
        assert!(!utils::is_valid_ticker("123PETR"));
        assert!(!utils::is_valid_ticker("PETR-4"));
    }
}
