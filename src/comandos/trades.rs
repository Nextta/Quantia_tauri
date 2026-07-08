use crate::api::trades::{get_trade_by_id, get_trades_by_backtest, get_trades_by_backtest_limit};
use crate::backtest::trade::Trade;

/// Optiene todos los trades de un backtest.
///
/// # Argments:
/// id_backtest: Identificador del backtest al que vamos a optener los datos de los trades.
///
/// # Return
/// Delvuelve un vector con los trades del backtest.
#[tauri::command]
pub async fn get_trades(id_backtest: i32) -> Vec<Trade> {
    let trades: Vec<Trade> = get_trades_by_backtest(id_backtest)
        .await
        .unwrap_or(Vec::new());

    trades
}

/// Optiene un conjunto de trades de un backtest de forma paginada.
///
/// # Argments:
/// id_backtest: Identificador del backtest al que vamos a optener los datos de los trades.
/// limite: Cantidad de trades a mostrar.
/// pagina: Número de la pagina a mostrar.
/// # Return
/// Delvuelve un vector con los trades del backtest.
#[tauri::command]
pub async fn get_trades_page(id_backtest: i32, limite: i32, pagina: i32) -> Vec<Trade> {
    let trades: Vec<Trade> = get_trades_by_backtest_limit(id_backtest, limite, pagina)
        .await
        .unwrap_or(Vec::new());

    trades
}

/// Optiene un trade especifico de un backtest.
///
/// # Argments:
/// id: Identificador del backtest al que vamos a optener el trade.
///
/// # Return
/// Delvuelve la información de un trade del backtest.
#[tauri::command]
pub async fn get_trade(id: i32) -> Trade {
    let trade: Trade = get_trade_by_id(id).await.unwrap();

    trade
}
