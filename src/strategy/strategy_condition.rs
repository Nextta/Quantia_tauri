use crate::enums::logics::Logic;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyCondition {
    pub id: i32,
    pub strategy_id: i32,
    pub action_id: i32,
    pub campo_a: String,
    pub shift_a: i32,
    pub operador: String,
    pub campo_b: String,
    pub shift_b: i32,
    pub logica: Option<Logic>,
    pub orden: i32,
    pub next_condition: Option<Box<StrategyCondition>>,
}

impl StrategyCondition {
    // Constructor con todos los campos de la entidad; agrupar en structs sería un refactor mayor.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: i32,
        strategy_id: i32,
        action_id: i32,
        campo_a: String,
        shift_a: i32,
        operador: String,
        campo_b: String,
        shift_b: i32,
        logica: Option<Logic>,
        orden: i32,
        next_condition: Option<Box<StrategyCondition>>,
    ) -> Self {
        Self {
            id,
            strategy_id,
            action_id,
            campo_a,
            shift_a,
            operador,
            campo_b,
            shift_b,
            logica,
            orden,
            next_condition,
        }
    }

    pub fn new_empty() -> Self {
        Self {
            id: 0,
            strategy_id: 0,
            action_id: 0,
            campo_a: String::new(),
            shift_a: 0,
            operador: String::new(),
            campo_b: String::new(),
            shift_b: 0,
            logica: None,
            orden: 0,
            next_condition: None,
        }
    }
}
