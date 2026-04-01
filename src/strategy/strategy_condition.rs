#[derive(Debug, Clone)]
pub struct StrategyCondition {
    pub id: i32,
    pub strategy_id: i32,
    pub action_id: i32,
    pub campo_a: String,
    pub operador: String,
    pub campo_b: String,
    pub logica: String,
    pub orden: i32,
}

impl StrategyCondition {
    pub fn new(
        id: i32,
        strategy_id: i32,
        action_id: i32,
        campo_a: String,
        operador: String,
        campo_b: String,
        logica: String,
        orden: i32,
    ) -> Self {
        Self {
            id,
            strategy_id,
            action_id,
            campo_a,
            operador,
            campo_b,
            logica,
            orden,
        }
    }
}
