use crate::enums::chart_type::ChartType;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyIndicator {
    pub id: i32,
    pub strategy_id: i32,
    pub nombre: String, // Nombre del columna del indicador en los datos
    pub tipo: String,   // Tipo de indicador EJ: EMA, MA, MACD...etc
    pub chart_type: ChartType,
    pub parametros: Value,
}

impl StrategyIndicator {
    pub fn new(
        id: i32,
        strategy_id: i32,
        nombre: String,
        tipo: String,
        chart_type: ChartType,
        parametros: Value,
    ) -> Self {
        Self {
            id,
            strategy_id,
            nombre,
            tipo,
            chart_type,
            parametros,
        }
    }

    pub fn new_empty() -> Self {
        Self {
            id: 0,
            strategy_id: 0,
            nombre: String::new(),
            tipo: String::new(),
            chart_type: ChartType::Inchart,
            parametros: Value::Null,
        }
    }
}
