use crate::api::backtests::{get_backtest_by_id, get_backtests};
use crate::backtest::backtest::Backtest;

/// Obtiene todos los backtest realizados.
///
/// # Return
/// Delvuelve un vector los todos los backtest.
#[tauri::command]
pub async fn get_alls_backtests() -> Vec<Backtest> {
    let backtests: Vec<Backtest> = get_backtests().await.unwrap();
    backtests
}

/// Optiene un backtest por id.
///
/// # Argments:
/// id: Identificador del backtest.
///
/// # Return
/// Delvuelve el backtest.
#[tauri::command]
pub async fn get_backtest(id: i32) -> Backtest {
    let backtest: Backtest = get_backtest_by_id(id).await.unwrap();

    backtest
}
