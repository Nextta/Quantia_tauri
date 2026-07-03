use crate::api::trades::{get_trade_by_id, get_trades_by_backtest, get_trades_by_backtest_limit};
use crate::backtest::trade::Trade;

#[tauri::command]
pub async fn get_trades(id_backtest: i32) -> Vec<Trade> {
    let trades: Vec<Trade> = get_trades_by_backtest(id_backtest)
        .await
        .unwrap_or(Vec::new());

    trades
}

#[tauri::command]
pub async fn get_trades_page(id_backtest: i32, limite: i32, pagina: i32) -> Vec<Trade> {
    let trades: Vec<Trade> = get_trades_by_backtest_limit(id_backtest, limite, pagina)
        .await
        .unwrap_or(Vec::new());

    trades
}

#[tauri::command]
pub async fn get_trade(id: i32) -> Trade {
    let trade: Trade = get_trade_by_id(id).await.unwrap();

    trade
}
