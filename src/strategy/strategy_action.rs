use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::enums::actions::Action;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyAction {
    pub id: i32,
    pub strategy_id: i32,
    pub tipo_signal: String,
    pub tipo: Action,
    pub parametros: Value,
}

impl StrategyAction {
    pub fn new(
        id: i32,
        strategy_id: i32,
        tipo_signal: String,
        tipo: Action,
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

    pub fn new_empty() -> Self {
        Self {
            id: 0,
            strategy_id: 0,
            tipo_signal: String::new(),
            tipo: Action::Buy,
            parametros: Value::Null,
        }
    }
}
