use crate::backtest::trade::Trade;
use crate::utils::truncate_decimal;
use dotenvy::dotenv;
use libsql::{params, Builder, Database};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::env;

use tracing::info;

#[derive(Serialize, Debug)]
pub struct Error {
    msg: String,
}

type Result<T> = std::result::Result<T, Error>;

impl<T> From<T> for Error
where
    T: std::error::Error,
{
    fn from(value: T) -> Self {
        Self {
            msg: value.to_string(),
        }
    }
}

fn get_db_config() -> Result<(String, String, String)> {
    dotenv().expect(".env file not found");
    let db_path = env::var("DB_PATH").unwrap();
    let sync_url = env::var("TURSO_SYNC_URL").unwrap();
    let auth_token = env::var("TURSO_AUTH_TOKEN").unwrap();
    Ok((db_path, sync_url, auth_token))
}

pub async fn table_trades() -> Result<String> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = Builder::new_remote_replica(db_path, sync_url, auth_token)
        .build()
        .await?;

    let conn = db.connect()?;

    conn.query(
        "CREATE TABLE IF NOT EXISTS trades
                (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                id_backtest INTEGER NOT NULL,
                id_symbol INTEGER NOT NULL,
                symbol TEXT NOT NULL,
                tipo TEXT NOT NULL,
                lotaje REAL NOT NULL,
                multiplicador REAL NOT NULL,
                t0 TEXT NOT NULL,
                tp REAL NOT NULL,
                sl REAL NOT NULL,
                precio_entrada REAL NOT NULL,
                t1 TEXT NOT NULL,
                precio_cierre REAL NOT NULL,
                precio_maximo REAL NOT NULL,
                precio_minimo REAL NOT NULL,
                duracion_segundos TEXT NOT NULL,
                duracion_minutos TEXT NOT NULL,
                duracion_horas TEXT NOT NULL,
                duracion_dias TEXT NOT NULL,
                label INTEGER NOT NULL,
                pl REAL NOT NULL,
                plsc REAL NOT NULL,
                pips_pl REAL NOT NULL
                )",
        (),
    )
    .await?;

    Ok("Tabla Trades is ok.".to_string())
}

pub async fn insert_trades(id_backtest: i32, trade: &Trade) -> Result<i32> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = Builder::new_remote_replica(db_path, sync_url, auth_token)
        .build()
        .await?;

    let conn = db.connect()?;

    let pl: Decimal = trade.get_pl().to_string().parse().unwrap();
    let plsc: Decimal = trade.get_plsc().to_string().parse().unwrap();
    let pips_pl: Decimal = trade.get_pip_pl().to_string().parse().unwrap();

    let id = conn.query("INSERT INTO trades (id_backtest, id_symbol, symbol, tipo, lotaje, multiplicador, t0, precio_entrada, tp, sl, t1, precio_cierre, precio_maximo, precio_minimo, duracion_segundos, duracion_minutos, duracion_horas, duracion_dias, label, pl, plsc, pips_pl) VALUES (?, ?, '?', '?', ?, ?, '?', ?, ?, ?, '?', ?, ?, ?, '?', '?', '?', '?', ?, ?, ?, ?) RETURNING id",
        [
            id_backtest,
            trade.get_symbol().get_id(),
            trade.get_symbol().get_name(),
            trade.get_tipo(),
            trade.get_lotaje(),
            trade.get_multiplier(),
            trade.get_t0(),
            trade.get_precio_entrada(),
            trade.get_tp(),
            trade.get_sl(),
            trade.get_t1(),
            trade.get_precio_cierre(),
            trade.get_precio_maximo(),
            trade.get_precio_minimo(),
            trade.get_duracion_segundos(),
            trade.get_duracion_minutos(),
            trade.get_duracion_horas(),
            trade.get_duracion_dias(),
            trade.get_label(),
            truncate_decimal(pl, 2),
            truncate_decimal(plsc, 2),
            truncate_decimal(pips_pl, 5)
        ]).await?;

    Ok(id.first().and_then(|row| row.get(0)).unwrap_or(0) as i32)
}

pub async fn get_trades_by_backtest(id_backtest: i32) -> Result<Vec<Trade>> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = Builder::new_remote_replica(db_path, sync_url, auth_token)
        .build()
        .await?;

    let conn = db.connect()?;

    let mut result = conn
        .query("SELECT * FROM trades WHERE id_backtest = ?", (id_backtest))
        .await?;

    let trades: Vec<Trade> = result
        .iter()
        .map(|row| Trade {
            id: row.get(0)?,
            id_backtest: row.get(1)?,
            id_symbol: row.get(2)?,
            symbol: row.get(3)?,
            tipo: row.get(4)?,
            lotaje: row.get(5)?,
            multiplicador: row.get(6)?,
            t0: row.get(7)?,
            precio_entrada: row.get(8)?,
            tp: row.get(9)?,
            sl: row.get(10)?,
            t1: row.get(11)?,
            precio_cierre: row.get(12)?,
            precio_maximo: row.get(13)?,
            precio_minimo: row.get(14)?,
            duracion_segundos: row.get(15)?,
            duracion_minutos: row.get(16)?,
            duracion_horas: row.get(17)?,
            duracion_dias: row.get(18)?,
            label: row.get(19)?,
            pl: row.get(20)?,
            plsc: row.get(21)?,
            pips_pl: row.get(22)?,
        })
        .collect();

    Ok(trades)
}

// ============== COMANDOS TAURI ==============

// #[tauri::command]
// pub async fn table_trades() -> Result<String> {
//     create_trades_table().await
// }

// #[tauri::command]
// pub async fn insert_trades(id_backtest: i32, trade: &Trade) -> Result<i32> {
//     insert_trade_logic(id_backtest, trade).await
// }

// #[tauri::command]
// pub async fn get_trades_by_backtest(id_backtest: i32) -> Result<Vec<Trade>> {
//     get_trades_by_backtest_logic(id_backtest).await
// }

// ============== TESTS ==============

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_table_trades() -> Result<()> {
        let trades = table_trades().await?;
        assert!(trades == "Tabla Trades is ok.".to_string());
        Ok(())
    }

    #[tokio::test]
    async fn test_insert_trades() -> Result<()> {
        let trades = Trade {
            id: 1,
            id_backtest: 1,
            id_symbol: 1,
            symbol: "BTC".to_string(),
            tipo: "Buy".to_string(),
            lotaje: 1.0,
            multiplicador: 1.0,
            t0: "0".to_string(),
            precio_entrada: 0.0,
            tp: 0.0,
            sl: 0.0,
            t1: "0".to_string(),
            precio_cierre: 0.0,
            precio_maximo: 0.0,
            precio_minimo: 0.0,
            duracion_segundos: 0,
            duracion_minutos: 0,
            duracion_horas: 0,
            duracion_dias: 0,
            label: 1,
            pl: 0.0,
            plsc: 0.0,
            pips_pl: 0.0,
        };
        let id = insert_trades(1, &trades).await?;
        assert!(id > 0);
        Ok(())
    }

    #[tokio::test]
    async fn test_get_trades_by_backtest() -> Result<()> {
        let trades = get_trades_by_backtest(1).await?;
        assert!(!trades.is_empty());
        Ok(())
    }
}
