use crate::api::symbols::get_symbol_cfd_by_id;
use crate::backtest::symbol::SymbolInfoCFD;
use crate::backtest::trade::Trade;
use crate::utils::tools::truncate_decimal;
use dotenvy::dotenv;
use libsql::{params, Builder};
use rust_decimal::Decimal;
use serde::Serialize;
use std::env;

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

#[tauri::command]
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

#[tauri::command]
pub async fn insert_trades(id_backtest: i32, trade: &Trade) -> Result<i32> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = Builder::new_remote_replica(db_path, sync_url, auth_token)
        .build()
        .await?;

    let conn = db.connect()?;

    let pl: Decimal = trade.get_pl().to_string().parse().unwrap();
    let plsc: Decimal = trade.get_plsc().to_string().parse().unwrap();
    let pips_pl: Decimal = trade.get_pip_pl().to_string().parse().unwrap();

    let parametros = params![
        id_backtest,
        trade.get_symbol().get_id(),
        trade.get_symbol().get_name().clone(),
        trade.get_tipo().clone(),
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
        trade.get_duracion_segundos().clone(),
        trade.get_duracion_minutos().clone(),
        trade.get_duracion_horas().clone(),
        trade.get_duracion_dias().clone(),
        trade.get_label(),
        truncate_decimal(pl, 2).to_string().parse::<f64>().unwrap(),
        truncate_decimal(plsc, 2)
            .to_string()
            .parse::<f64>()
            .unwrap(),
        truncate_decimal(pips_pl, 5)
            .to_string()
            .parse::<f64>()
            .unwrap()
    ];

    conn.query("INSERT INTO trades (id_backtest, id_symbol, symbol, tipo, lotaje, multiplicador, t0, precio_entrada, tp, sl, t1, precio_cierre, precio_maximo, precio_minimo, duracion_segundos, duracion_minutos, duracion_horas, duracion_dias, label, pl, plsc, pips_pl) VALUES (?, ?, '?', '?', ?, ?, '?', ?, ?, ?, '?', ?, ?, ?, '?', '?', '?', '?', ?, ?, ?, ?) RETURNING id",
        parametros).await?;

    let id = conn.last_insert_rowid() as i32;
    Ok(id)
}

#[tauri::command]
pub async fn get_trades_by_backtest(id_backtest: i32) -> Result<Vec<Trade>> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = Builder::new_remote_replica(db_path, sync_url, auth_token)
        .build()
        .await?;

    let conn = db.connect()?;

    let mut result = conn
        .query("SELECT * FROM trades WHERE id_backtest = ?", [id_backtest])
        .await?;

    let mut trades: Vec<Trade> = Vec::new();

    while let Some(row) = result.next().await? {
        let symbol: SymbolInfoCFD = get_symbol_cfd_by_id(row.get(2)?).await.unwrap(); // Necesito implementar la api de symbol.
        let mut trade: Trade = Trade::new(row.get(1)?, symbol).await;

        trade.id = row.get::<i32>(0)?;
        trade.id_symbol = row.get::<i32>(2)?;
        trade.tipo = row.get::<String>(4)?;
        trade.lotaje = row.get::<f64>(5)?;
        trade.multiplicador = row.get::<f64>(6)?;
        trade.t0 = row.get::<String>(7)?;
        trade.precio_entrada = row.get::<f64>(8)?;
        trade.tp = row.get::<f64>(9)?;
        trade.sl = row.get::<f64>(10)?;
        trade.t1 = row.get::<String>(11)?;
        trade.precio_cierre = row.get::<f64>(12)?;
        trade.precio_maximo = row.get::<f64>(13)?;
        trade.precio_minimo = row.get::<f64>(14)?;
        trade.duracion_segundos = row.get::<String>(15)?;
        trade.duracion_minutos = row.get::<String>(16)?;
        trade.duracion_horas = row.get::<String>(17)?;
        trade.duracion_dias = row.get::<String>(18)?;
        trade.label = row.get::<u32>(19)?;
        trade.pl = row.get::<f64>(20)?;
        trade.plsc = row.get::<f64>(21)?;
        trade.pips_pl = row.get::<f64>(22)?;

        trades.push(trade);
    }

    Ok(trades)
}

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
        let symbol: SymbolInfoCFD = get_symbol_cfd_by_id(1).await.unwrap(); // Necesito implementar la api de symbol.
        let mut trade: Trade = Trade::new(1, symbol).await;

        trade.id = 1;
        trade.id_symbol = 1;
        trade.tipo = "Sell".to_string();
        trade.lotaje = 1.0;
        trade.multiplicador = 1.0;
        trade.t0 = "12-12-2000".to_string();
        trade.precio_entrada = 1.25244;
        trade.tp = 1.25244;
        trade.sl = 1.25244;
        trade.t1 = "14-12-2000".to_string();
        trade.precio_cierre = 1.25244;
        trade.precio_maximo = 1.25244;
        trade.precio_minimo = 1.25244;
        trade.duracion_segundos = "100".to_string();
        trade.duracion_minutos = "100".to_string();
        trade.duracion_horas = "100".to_string();
        trade.duracion_dias = "100".to_string();
        trade.label = 1;
        trade.pl = 500.23;
        trade.plsc = 500.23;
        trade.pips_pl = 500.23;

        let id = insert_trades(1, &trade).await?;
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
