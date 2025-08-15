// Mini-Disclaimer: Uso educacional/experimental; sem recomendação de investimento ou afiliação; sem remuneração de terceiros; Profit/ProfitDLL © Nelógica; veja README & DISCLAIMER.
//! Módulos compartilhados (indicadores, utilidades) entre estratégias.

/// Estado vazio que pode ser reutilizado por estratégias que ainda não
/// necessitam de dados específicos do motor. Serve como placeholder.
#[derive(Debug, Default, Clone, Copy)]
pub struct NoOpState;
