use crate::api::trades::get_trades_by_backtest;
use crate::backtest::backtest::Backtest;
use dotenvy::dotenv;
use libsql::{params, Builder};
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
pub async fn table_backtests_cfd() -> Result<String> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = Builder::new_remote_replica(db_path, sync_url, auth_token)
        .build()
        .await?;

    let conn = db.connect()?;

    conn.query(
        "CREATE TABLE IF NOT EXISTS backtest
                    (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    titulo TEXT NOT NULL,
                    balance REAL NOT NULL,
                    tipo TEXT NOT NULL
                    )",
        (),
    )
    .await?;

    Ok("Tabla backtests is ok.".to_string())
}

#[tauri::command]
pub async fn insert_backtest_cfd(backtest: Backtest) -> Result<i32> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = Builder::new_remote_replica(db_path, sync_url, auth_token)
        .build()
        .await?;

    let conn = db.connect()?;

    conn.query(
        "INSERT INTO backtest (titulo, balance, tipo) VALUES (?, ?, ?) RETURNING id",
        params![backtest.titulo, backtest.balance, backtest.tipo],
    )
    .await?;

    let id = conn.last_insert_rowid() as i32;
    Ok(id)
}

#[tauri::command]
pub async fn get_backtests() -> Result<Vec<Backtest>> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = Builder::new_remote_replica(db_path, sync_url, auth_token)
        .build()
        .await?;

    let conn = db.connect()?;

    let mut rows = conn.query("SELECT * FROM backtest", ()).await?;

    let mut backtests = Vec::new();
    while let Some(row) = rows.next().await? {
        let trades = get_trades_by_backtest(row.get::<i32>(0)?).await.unwrap();

        let backtest = Backtest {
            id: row.get::<i32>(0)?,
            titulo: row.get::<String>(1)?,
            balance: row.get::<f64>(2)?,
            tipo: row.get::<String>(3)?,
            trades: trades,
            datos: Vec::new(),
        };
        backtests.push(backtest);
    }

    Ok(backtests)
}

#[tauri::command]
pub async fn get_backtest_by_id(id: i32) -> Result<Vec<Backtest>> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = Builder::new_remote_replica(db_path, sync_url, auth_token)
        .build()
        .await?;

    let conn = db.connect()?;

    let mut rows = conn
        .query("SELECT * FROM backtest WHERE id = ?", [id])
        .await?;

    let mut backtests = Vec::new();
    while let Some(row) = rows.next().await? {
        let trades = get_trades_by_backtest(row.get::<i32>(0)?).await.unwrap();

        let backtest = Backtest {
            id: row.get::<i32>(0)?,
            titulo: row.get::<String>(1)?,
            balance: row.get::<f64>(2)?,
            tipo: row.get::<String>(3)?,
            trades: trades,
            datos: Vec::new(),
        };
        backtests.push(backtest);
    }

    Ok(backtests)
}

#[tauri::command]
pub async fn get_backtests_by_titulo(titulo: String) -> Result<Vec<Backtest>> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = Builder::new_remote_replica(db_path, sync_url, auth_token)
        .build()
        .await?;

    let conn = db.connect()?;

    let mut rows = conn
        .query("SELECT * FROM backtest WHERE titulo = ?", params![titulo])
        .await?;

    let mut backtests = Vec::new();
    while let Some(row) = rows.next().await? {
        let trades = get_trades_by_backtest(row.get::<i32>(0)?).await.unwrap();

        let backtest = Backtest {
            id: row.get::<i32>(0)?,
            titulo: row.get::<String>(1)?,
            balance: row.get::<f64>(2)?,
            tipo: row.get::<String>(3)?,
            trades: trades,
            datos: Vec::new(),
        };
        backtests.push(backtest);
    }

    Ok(backtests)
}

#[tauri::command]
pub async fn get_backtests_by_tipo(tipo: String) -> Result<Vec<Backtest>> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = Builder::new_remote_replica(db_path, sync_url, auth_token)
        .build()
        .await?;

    let conn = db.connect()?;

    let mut rows = conn
        .query("SELECT * FROM backtest WHERE tipo = ?", params![tipo])
        .await?;

    let mut backtests = Vec::new();
    while let Some(row) = rows.next().await? {
        let trades = get_trades_by_backtest(row.get::<i32>(0)?).await.unwrap();

        let backtest = Backtest {
            id: row.get::<i32>(0)?,
            titulo: row.get::<String>(1)?,
            balance: row.get::<f64>(2)?,
            tipo: row.get::<String>(3)?,
            trades: trades,
            datos: Vec::new(),
        };
        backtests.push(backtest);
    }

    Ok(backtests)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_get_backtests() -> Result<()> {
        let _ = get_backtests().await?;
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_get_backtests_by_tipo() -> Result<()> {
        let _ = get_backtests_by_tipo("CFD".to_string()).await?;
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_get_backtest_by_id() -> Result<()> {
        let _ = get_backtest_by_id(1).await?;
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_get_backtests_by_titulo() -> Result<()> {
        let _ = get_backtests_by_titulo("Tests".to_string()).await?;
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_table_backtests_cfd() -> Result<()> {
        let _ = table_backtests_cfd().await?;
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_insert_backtest_cfd() -> Result<()> {
        let _ = insert_backtest_cfd(Backtest {
            id: 0,
            titulo: "Test".to_string(),
            balance: 100.0,
            tipo: "CFD".to_string(),
            trades: Vec::new(),
            datos: Vec::new(),
        })
        .await?;
        Ok(())
    }
}
