/// Tipos de dados específicos para integração com DLL da corretora brasileira
/// 
/// Este módulo define as estruturas de dados que fazem a ponte entre
/// a DLL C/C++ da corretora e o sistema Toucan em Rust.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Side de uma ordem na DLL
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(C)]
pub enum DllSide {
    Buy = 0,
    Sell = 1,
}

/// Tipo de ordem na DLL
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(C)]
pub enum DllOrderType {
    Market = 0,
    Limit = 1,
    Stop = 2,
    StopLimit = 3,
}

/// Status de ordem na DLL
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DllOrderStatus {
    Submitted,
    Open,
    PartiallyFilled,
    Filled,
    Cancelled,
    Rejected,
    Expired,
}

/// Request para criar ordem via DLL
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DllOrderRequest {
    /// Symbol do instrumento (ex: "PETR4")
    pub symbol: String,
    /// Side da ordem
    pub side: DllSide,
    /// Quantidade da ordem
    pub quantity: Decimal,
    /// Preço da ordem (None para market orders)
    pub price: Option<Decimal>,
    /// Tipo da ordem
    pub order_type: DllOrderType,
}

/// Response de criação de ordem via DLL
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DllOrderResponse {
    /// ID da ordem gerado pela corretora
    pub order_id: String,
    /// Timestamp de quando a ordem foi aceita
    pub timestamp: DateTime<Utc>,
    /// Status inicial da ordem
    pub status: DllOrderStatus,
}

/// Ordem retornada pela DLL
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DllOrder {
    /// ID único da ordem
    pub order_id: String,
    /// Symbol do instrumento
    pub symbol: String,
    /// Side da ordem
    pub side: DllSide,
    /// Quantidade total da ordem
    pub quantity: Decimal,
    /// Preço da ordem (None para market orders)
    pub price: Option<Decimal>,
    /// Status atual da ordem
    pub status: DllOrderStatus,
    /// Timestamp da ordem
    pub timestamp: DateTime<Utc>,
}

/// Trade executado retornado pela DLL
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DllTrade {
    /// ID único do trade
    pub trade_id: String,
    /// Symbol do instrumento
    pub symbol: String,
    /// Side do trade
    pub side: DllSide,
    /// Quantidade executada
    pub quantity: Decimal,
    /// Preço de execução
    pub price: Decimal,
    /// Timestamp da execução
    pub timestamp: DateTime<Utc>,
}

/// Balance de um asset retornado pela DLL
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DllBalance {
    /// Nome do asset (ex: "BRL", "PETR4")
    pub asset: String,
    /// Saldo disponível
    pub balance: Decimal,
    /// Timestamp da última atualização
    pub timestamp: DateTime<Utc>,
}

/// Update de ordem em tempo real via callback
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DllOrderUpdate {
    /// ID da ordem atualizada
    pub order_id: String,
    /// Novo status
    pub status: DllOrderStatus,
    /// Quantidade preenchida (acumulada)
    pub filled_quantity: Option<Decimal>,
    /// Preço médio de preenchimento
    pub average_price: Option<Decimal>,
    /// Timestamp do update
    pub timestamp: DateTime<Utc>,
}

/// Update de balance em tempo real via callback
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DllBalanceUpdate {
    /// Asset atualizado
    pub asset: String,
    /// Novo saldo
    pub balance: Decimal,
    /// Timestamp da atualização
    pub timestamp: DateTime<Utc>,
}

/// Informações de conexão da DLL
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DllConnectionInfo {
    /// Servidor conectado
    pub server: String,
    /// Login utilizado
    pub login: String,
    /// Status da conexão
    pub connected: bool,
    /// Timestamp da última conexão
    pub last_connected: Option<DateTime<Utc>>,
    /// Versão da API da corretora
    pub api_version: String,
}

/// Estatísticas de performance da DLL
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DllPerformanceStats {
    /// Latência média das chamadas em microsegundos
    pub avg_call_latency_us: u64,
    /// Latência máxima observada em microsegundos
    pub max_call_latency_us: u64,
    /// Latência mínima observada em microsegundos
    pub min_call_latency_us: u64,
    /// Número total de chamadas realizadas
    pub total_calls: u64,
    /// Número de chamadas com erro
    pub error_calls: u64,
    /// Taxa de sucesso (0.0 a 1.0)
    pub success_rate: f64,
    /// Timestamp da última medição
    pub last_measurement: DateTime<Utc>,
}

impl DllPerformanceStats {
    /// Cria estatísticas iniciais
    pub fn new() -> Self {
        Self {
            avg_call_latency_us: 0,
            max_call_latency_us: 0,
            min_call_latency_us: u64::MAX,
            total_calls: 0,
            error_calls: 0,
            success_rate: 0.0,
            last_measurement: Utc::now(),
        }
    }
    
    /// Adiciona uma medição de latência
    pub fn add_measurement(&mut self, latency_us: u64, success: bool) {
        self.total_calls += 1;
        
        if !success {
            self.error_calls += 1;
        }
        
        if success {
            // Atualiza estatísticas de latência apenas para chamadas bem-sucedidas
            self.max_call_latency_us = self.max_call_latency_us.max(latency_us);
            self.min_call_latency_us = self.min_call_latency_us.min(latency_us);
            
            // Recalcula média (simples, pode ser otimizado com rolling average)
            let successful_calls = self.total_calls - self.error_calls;
            if successful_calls > 0 {
                self.avg_call_latency_us = 
                    (self.avg_call_latency_us * (successful_calls - 1) + latency_us) / successful_calls;
            }
        }
        
        self.success_rate = if self.total_calls > 0 {
            (self.total_calls - self.error_calls) as f64 / self.total_calls as f64
        } else {
            0.0
        };
        
        self.last_measurement = Utc::now();
    }
    
    /// Reseta as estatísticas
    pub fn reset(&mut self) {
        *self = Self::new();
    }
    
    /// Verifica se a performance está dentro dos limites aceitáveis
    pub fn is_performance_acceptable(&self, max_latency_us: u64, min_success_rate: f64) -> bool {
        self.avg_call_latency_us <= max_latency_us && self.success_rate >= min_success_rate
    }
}

/// Configuração de limites de performance para a DLL
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DllPerformanceLimits {
    /// Latência máxima aceitável em microsegundos (default: 5000 = 5ms)
    pub max_latency_us: u64,
    /// Taxa mínima de sucesso (default: 0.95 = 95%)
    pub min_success_rate: f64,
    /// Timeout para operações em ms (default: 10000 = 10s)
    pub operation_timeout_ms: u64,
    /// Número máximo de tentativas de reconexão
    pub max_reconnect_attempts: u32,
}

impl Default for DllPerformanceLimits {
    fn default() -> Self {
        Self {
            max_latency_us: 5_000,        // 5ms
            min_success_rate: 0.95,       // 95%
            operation_timeout_ms: 10_000, // 10s
            max_reconnect_attempts: 3,
        }
    }
}

// Conversões para tipos do Toucan

impl From<DllSide> for markets::Side {
    fn from(dll_side: DllSide) -> Self {
        match dll_side {
            DllSide::Buy => markets::Side::Buy,
            DllSide::Sell => markets::Side::Sell,
        }
    }
}

impl From<markets::Side> for DllSide {
    fn from(side: markets::Side) -> Self {
        match side {
            markets::Side::Buy => DllSide::Buy,
            markets::Side::Sell => DllSide::Sell,
        }
    }
}

impl From<DllOrderType> for crate::order::OrderKind {
    fn from(dll_type: DllOrderType) -> Self {
        match dll_type {
            DllOrderType::Market => crate::order::OrderKind::Market,
            DllOrderType::Limit => crate::order::OrderKind::Limit,
            DllOrderType::Stop => crate::order::OrderKind::Market, // Aproximação
            DllOrderType::StopLimit => crate::order::OrderKind::Limit, // Aproximação
        }
    }
}

impl From<crate::order::OrderKind> for DllOrderType {
    fn from(kind: crate::order::OrderKind) -> Self {
        match kind {
            crate::order::OrderKind::Market => DllOrderType::Market,
            crate::order::OrderKind::Limit => DllOrderType::Limit,
            _ => DllOrderType::Limit, // Default fallback
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_performance_stats() {
        let mut stats = DllPerformanceStats::new();
        
        // Adiciona algumas medições
        stats.add_measurement(1000, true);  // 1ms success
        stats.add_measurement(2000, true);  // 2ms success
        stats.add_measurement(5000, false); // 5ms error
        
        assert_eq!(stats.total_calls, 3);
        assert_eq!(stats.error_calls, 1);
        assert_eq!(stats.success_rate, 2.0 / 3.0);
        assert_eq!(stats.avg_call_latency_us, 1500); // (1000 + 2000) / 2
        assert_eq!(stats.max_call_latency_us, 2000);
        assert_eq!(stats.min_call_latency_us, 1000);
    }
    
    #[test]
    fn test_performance_limits() {
        let limits = DllPerformanceLimits::default();
        let mut stats = DllPerformanceStats::new();
        
        // Performance boa
        stats.add_measurement(3000, true);
        stats.add_measurement(4000, true);
        assert!(stats.is_performance_acceptable(limits.max_latency_us, limits.min_success_rate));
        
        // Performance ruim (latência alta)
        stats.add_measurement(10000, true);
        assert!(!stats.is_performance_acceptable(limits.max_latency_us, limits.min_success_rate));
    }
    
    #[test]
    fn test_side_conversion() {
        assert_eq!(markets::Side::from(DllSide::Buy), markets::Side::Buy);
        assert_eq!(markets::Side::from(DllSide::Sell), markets::Side::Sell);
        
        assert_eq!(DllSide::from(markets::Side::Buy), DllSide::Buy);
        assert_eq!(DllSide::from(markets::Side::Sell), DllSide::Sell);
    }
    
    #[test]
    fn test_order_type_conversion() {
        assert_eq!(
            crate::order::OrderKind::from(DllOrderType::Market), 
            crate::order::OrderKind::Market
        );
        assert_eq!(
            crate::order::OrderKind::from(DllOrderType::Limit), 
            crate::order::OrderKind::Limit
        );
        
        assert_eq!(
            DllOrderType::from(crate::order::OrderKind::Market), 
            DllOrderType::Market
        );
        assert_eq!(
            DllOrderType::from(crate::order::OrderKind::Limit), 
            DllOrderType::Limit
        );
    }
}
