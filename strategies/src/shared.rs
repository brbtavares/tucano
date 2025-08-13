//! Módulos compartilhados (indicadores, utilidades) entre estratégias.

/// Estado vazio que pode ser reutilizado por estratégias que ainda não
/// necessitam de dados específicos do motor. Serve como placeholder.
#[derive(Debug, Default, Clone, Copy)]
pub struct NoOpState;
