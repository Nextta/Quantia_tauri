// Estructura válida: el módulo motor se llama igual que su directorio contenedor.
#[allow(clippy::module_inception)]
pub mod backtest;
pub mod broker;
pub mod datos;
pub mod dias;
pub mod resultados;
pub mod symbol;
pub mod trade;
