use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyAction {
    pub id: i32,
    pub strategy_id: i32,
    pub tipo_signal: String,
    pub tipo: String,
    pub parametros: Value,
}

impl StrategyAction {
    pub fn new(
        id: i32,
        strategy_id: i32,
        tipo_signal: String,
        tipo: String,
        parametros: Value,
    ) -> Self {
        Self {
            id,
            strategy_id,
            tipo_signal,
            tipo,
            parametros,
        }
    }
}
