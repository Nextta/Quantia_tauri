use crate::enums::logics::Logic;
use crate::strategy::strategy_condition::StrategyCondition;
use libsql::Row;
use rust_decimal::Decimal;

pub fn truncate_decimal(valor: Decimal, decimales: u32) -> Decimal {
    let potencia = Decimal::from(10_i64.pow(decimales));
    (valor * potencia).trunc() / potencia
}

pub fn add_condition(condition: &mut StrategyCondition, row: &Row) {
    if let Some(next) = &mut condition.next_condition {
        add_condition(next, row);
    } else {
        condition.next_condition = Some(Box::new(StrategyCondition {
            id: row.get::<i32>(0).unwrap(),
            strategy_id: row.get::<i32>(1).unwrap(),
            action_id: row.get::<i32>(2).unwrap(),
            campo_a: row.get::<String>(3).unwrap(),
            shift_a: row.get::<i32>(4).unwrap(),
            operador: row.get::<String>(5).unwrap(),
            campo_b: row.get::<String>(6).unwrap(),
            shift_b: row.get::<i32>(7).unwrap(),
            logica: match row.get::<String>(8).unwrap().as_str() {
                "AND" => Some(Logic::AND),
                "OR" => Some(Logic::OR),
                _ => None,
            },
            orden: row.get::<i32>(9).unwrap(),
            next_condition: None,
        }));
    }
}
