use crate::api::backtests::get_backtests;
use crate::backtest::backtest::Backtest;

#[tauri::command]
pub async fn get_alls_backtests() -> Vec<Backtest> {
    let backtests: Vec<Backtest> = get_backtests().await.unwrap_or(Vec::new());
    backtests
}
