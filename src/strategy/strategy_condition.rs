#[derive(Debug, Clone)]
pub struct StrategyCondition {
    pub id: i32,
    pub strategy_id: i32,
    pub action_id: i32,
    pub campo_a: String,
    pub shift_a: i32,
    pub operador: String,
    pub campo_b: String,
    pub shift_b: i32,
    pub logica: String,
    pub orden: i32,
}

impl StrategyCondition {
    pub fn new(
        id: i32,
        strategy_id: i32,
        action_id: i32,
        campo_a: String,
        shift_a: i32,
        operador: String,
        campo_b: String,
        shift_b: i32,
        logica: String,
        orden: i32,
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
            logica: String::new(),
            orden: 0,
        }
    }
}
