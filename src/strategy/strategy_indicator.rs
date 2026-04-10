use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyIndicator {
    pub id: i32,
    pub strategy_id: i32,
    pub nombre: String,
    pub tipo: String,
    pub parametros: Value,
}

impl StrategyIndicator {
    pub fn new(id: i32, strategy_id: i32, nombre: String, tipo: String, parametros: Value) -> Self {
        Self {
            id,
            strategy_id,
            nombre,
            tipo,
            parametros,
        }
    }

    pub fn new_empty() -> Self {
        Self {
            id: 0,
            strategy_id: 0,
            nombre: String::new(),
            tipo: String::new(),
            parametros: Value::Null,
        }
    }
}
