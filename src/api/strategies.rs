use crate::backtest::strategy::Strategy;
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
//================================Strategies================================
#[tauri::command]
pub async fn table_strategies() -> Result<String> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = Builder::new_remote_replica(db_path, sync_url, auth_token)
        .build()
        .await?;

    let conn = db.connect()?;

    conn.query(
        "CREATE TABLE IF NOT EXISTS strategies (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            id_user     INTEGER NOT NULL,
            nombre      TEXT NOT NULL,
            descripcion TEXT,
            activa      INTEGER DEFAULT 1,
            creada_en   TEXT DEFAULT (datetime('now'))
        )",
        (),
    )
    .await?;

    Ok("Tabla strategies is ok.".to_string())
}

#[tauri::command]
pub async fn get_strategies() -> Result<Vec<Strategy>> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = Builder::new_remote_replica(db_path, sync_url, auth_token)
        .build()
        .await?;

    let conn = db.connect()?;

    let sql = "SELECT * FROM strategies";

    let mut result = conn.query(sql, ()).await?;

    let mut strategies = Vec::new();
    while let Some(row) = result.next().await? {
        let mut activa = false;
        if row.get::<i32>(4)? != 0 {
            activa = true;
        }
        let strategy = Strategy {
            id: row.get::<i32>(0)?,
            id_user: row.get::<i32>(1)?,
            nombre: row.get::<String>(2)?,
            descripcion: row.get::<Option<String>>(3)?,
            activa: activa,
            creada_en: row.get::<String>(5)?,
        };
        strategies.push(strategy);
    }

    Ok(strategies)
}

#[tauri::command]
pub async fn get_strategies_by_id(id: i32) -> Result<Strategy> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = Builder::new_remote_replica(db_path, sync_url, auth_token)
        .build()
        .await?;

    let conn = db.connect()?;

    let sql = "SELECT * FROM strategies WHERE id = ?";

    let parametros = params![id];

    let mut result = conn.query(sql, parametros).await?;

    let row = result.next().await?.unwrap();

    let mut activa = false;
    if row.get::<i32>(4)? != 0 {
        activa = true;
    }

    let strategy = Strategy {
        id: row.get::<i32>(0)?,
        id_user: row.get::<i32>(1)?,
        nombre: row.get::<String>(2)?,
        descripcion: row.get::<Option<String>>(3)?,
        activa: activa,
        creada_en: row.get::<String>(5)?,
    };

    Ok(strategy)
}

#[tauri::command]
pub async fn get_strategies_by_id_user(id_user: i32) -> Result<Vec<Strategy>> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = Builder::new_remote_replica(db_path, sync_url, auth_token)
        .build()
        .await?;

    let conn = db.connect()?;

    let parametros = params![id_user];

    let sql = "SELECT * FROM strategies WHERE id_user = ?  AND activa = 1";

    let mut result = conn.query(sql, parametros).await?;

    let mut strategies = Vec::new();
    while let Some(row) = result.next().await? {
        let mut activa = false;
        if row.get::<i32>(4)? != 0 {
            activa = true;
        }
        let strategy = Strategy {
            id: row.get::<i32>(0)?,
            id_user: row.get::<i32>(1)?,
            nombre: row.get::<String>(2)?,
            descripcion: row.get::<Option<String>>(3)?,
            activa: activa,
            creada_en: row.get::<String>(5)?,
        };
        strategies.push(strategy);
    }

    Ok(strategies)
}

#[tauri::command]
pub async fn get_strategies_by_nombre(nombre: &str) -> Result<Vec<Strategy>> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = Builder::new_remote_replica(db_path, sync_url, auth_token)
        .build()
        .await?;

    let conn = db.connect()?;

    let parametros = params![nombre];

    let sql = "SELECT * FROM strategies WHERE nombre = ?  AND activa = 1";

    let mut result = conn.query(sql, parametros).await?;

    let mut strategies = Vec::new();
    while let Some(row) = result.next().await? {
        let mut activa = false;
        if row.get::<i32>(4)? != 0 {
            activa = true;
        }
        let strategy = Strategy {
            id: row.get::<i32>(0)?,
            id_user: row.get::<i32>(1)?,
            nombre: row.get::<String>(2)?,
            descripcion: row.get::<Option<String>>(3)?,
            activa: activa,
            creada_en: row.get::<String>(5)?,
        };
        strategies.push(strategy);
    }

    Ok(strategies)
}

#[tauri::command]
pub async fn get_active_strategies_by_user(id_user: i32) -> Result<Vec<Strategy>> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = Builder::new_remote_replica(db_path, sync_url, auth_token)
        .build()
        .await?;

    let conn = db.connect()?;

    let parametros = params![id_user];

    let sql = "SELECT * FROM strategies WHERE id_user = ? AND activa = 1";

    let mut result = conn.query(sql, parametros).await?;

    let mut strategies = Vec::new();
    while let Some(row) = result.next().await? {
        let mut activa = false;
        if row.get::<i32>(4)? != 0 {
            activa = true;
        }
        let strategy = Strategy {
            id: row.get::<i32>(0)?,
            id_user: row.get::<i32>(1)?,
            nombre: row.get::<String>(2)?,
            descripcion: row.get::<Option<String>>(3)?,
            activa: activa,
            creada_en: row.get::<String>(5)?,
        };
        strategies.push(strategy);
    }

    Ok(strategies)
}

#[tauri::command]
pub async fn get_all_active_strategies() -> Result<Vec<Strategy>> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = Builder::new_remote_replica(db_path, sync_url, auth_token)
        .build()
        .await?;

    let conn = db.connect()?;

    // let parametros = params![id_user];

    let sql = "SELECT * FROM strategies WHERE activa = 1";

    let mut result = conn.query(sql, ()).await?;

    let mut strategies = Vec::new();
    while let Some(row) = result.next().await? {
        let mut activa = false;
        if row.get::<i32>(4)? != 0 {
            activa = true;
        }
        let strategy = Strategy {
            id: row.get::<i32>(0)?,
            id_user: row.get::<i32>(1)?,
            nombre: row.get::<String>(2)?,
            descripcion: row.get::<Option<String>>(3)?,
            activa: activa,
            creada_en: row.get::<String>(5)?,
        };
        strategies.push(strategy);
    }

    Ok(strategies)
}

#[tauri::command]
pub async fn get_desactive_strategies_by_user(id_user: i32) -> Result<Vec<Strategy>> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = Builder::new_remote_replica(db_path, sync_url, auth_token)
        .build()
        .await?;

    let conn = db.connect()?;

    let parametros = params![id_user];

    let sql = "SELECT * FROM strategies WHERE id_user = ? AND activa = 0";

    let mut result = conn.query(sql, parametros).await?;

    let mut strategies = Vec::new();
    while let Some(row) = result.next().await? {
        let mut activa = false;
        if row.get::<i32>(4)? != 0 {
            activa = true;
        }
        let strategy = Strategy {
            id: row.get::<i32>(0)?,
            id_user: row.get::<i32>(1)?,
            nombre: row.get::<String>(2)?,
            descripcion: row.get::<Option<String>>(3)?,
            activa: activa,
            creada_en: row.get::<String>(5)?,
        };
        strategies.push(strategy);
    }

    Ok(strategies)
}

#[tauri::command]
pub async fn get_all_desactive_strategies() -> Result<Vec<Strategy>> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = Builder::new_remote_replica(db_path, sync_url, auth_token)
        .build()
        .await?;

    let conn = db.connect()?;

    // let parametros = params![id_user];

    let sql = "SELECT * FROM strategies WHERE activa = 0";

    let mut result = conn.query(sql, ()).await?;

    let mut strategies = Vec::new();
    while let Some(row) = result.next().await? {
        let mut activa = false;
        if row.get::<i32>(4)? != 0 {
            activa = true;
        }
        let strategy = Strategy {
            id: row.get::<i32>(0)?,
            id_user: row.get::<i32>(1)?,
            nombre: row.get::<String>(2)?,
            descripcion: row.get::<Option<String>>(3)?,
            activa: activa,
            creada_en: row.get::<String>(5)?,
        };
        strategies.push(strategy);
    }

    Ok(strategies)
}

#[tauri::command]
pub async fn get_desactive_strategies_by_date(fecha: &str) -> Result<Vec<Strategy>> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = Builder::new_remote_replica(db_path, sync_url, auth_token)
        .build()
        .await?;

    let conn = db.connect()?;

    let parametros = params![fecha];

    let sql = "SELECT * FROM strategies WHERE creada_en = ? AND activa = 1";

    let mut result = conn.query(sql, parametros).await?;

    let mut strategies = Vec::new();
    while let Some(row) = result.next().await? {
        let mut activa = false;
        if row.get::<i32>(4)? != 0 {
            activa = true;
        }
        let strategy = Strategy {
            id: row.get::<i32>(0)?,
            id_user: row.get::<i32>(1)?,
            nombre: row.get::<String>(2)?,
            descripcion: row.get::<Option<String>>(3)?,
            activa: activa,
            creada_en: row.get::<String>(5)?,
        };
        strategies.push(strategy);
    }

    Ok(strategies)
}

//================================Indicators================================
#[tauri::command]
pub async fn table_strategy_indicators() -> Result<String> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = Builder::new_remote_replica(db_path, sync_url, auth_token)
        .build()
        .await?;

    let conn = db.connect()?;

    conn.query(
        "CREATE TABLE IF NOT EXISTS strategy_indicators (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            strategy_id  INTEGER NOT NULL REFERENCES strategies(id),
            nombre       TEXT NOT NULL,  -- nombre del campo: 'sma_20'
            tipo         TEXT NOT NULL,  -- 'SMA', 'EMA', 'RSI', 'MACD', 'BB'
            parametros   TEXT NOT NULL   -- JSON: '{\"period\": 20}'
        )",
        (),
    )
    .await?;

    Ok("Tabla strategy_indicators is ok.".to_string())
}

//================================Actions================================
#[tauri::command]
pub async fn table_strategy_actions() -> Result<String> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = Builder::new_remote_replica(db_path, sync_url, auth_token)
        .build()
        .await?;

    let conn = db.connect()?;

    conn.query(
        "CREATE TABLE IF NOT EXISTS strategy_actions (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            strategy_id  INTEGER NOT NULL REFERENCES strategies(id),
            tipo_signal  TEXT NOT NULL,  -- 'entry' | 'exit'
            tipo         TEXT NOT NULL,  -- 'buy', 'sell', 'close', 'set_sl', 'set_tp'
            parametro    TEXT DEFAULT '{}'  -- JSON con parámetros extra
        )",
        (),
    )
    .await?;

    Ok("Tabla strategy_actions is ok.".to_string())
}

//================================Conditions================================
#[tauri::command]
pub async fn table_strategy_conditions() -> Result<String> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = Builder::new_remote_replica(db_path, sync_url, auth_token)
        .build()
        .await?;

    let conn = db.connect()?;

    conn.query(
        "CREATE TABLE IF NOT EXISTS strategy_conditions (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            strategy_id  INTEGER NOT NULL REFERENCES strategies(id),
            action_id    INTEGER NOT NULL REFERENCES strategy_actions(id),
            campo_a      TEXT NOT NULL,  -- 'close', 'sma_20', 'rsi'
            operador     TEXT NOT NULL,  -- '>', '<', '>=', '<=', '==', 'cross_above', 'cross_below'
            campo_b      TEXT NOT NULL,  -- 'sma_50' o valor literal '30.0'
            logica       TEXT DEFAULT 'NULL',
            orden        INTEGER DEFAULT 0
        )",
        (),
    )
    .await?;

    Ok("Tabla strategy_conditions is ok.".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_table_strategies() -> Result<()> {
        let _ = table_strategies().await?;
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_table_strategy_indicators() -> Result<()> {
        let _ = table_strategy_indicators().await?;
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_table_strategy_actions() -> Result<()> {
        let _ = table_strategy_actions().await?;
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_table_strategy_conditions() -> Result<()> {
        let _ = table_strategy_conditions().await?;
        Ok(())
    }
}
