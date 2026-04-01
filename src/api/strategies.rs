use crate::strategy::strategy::Strategy;
use crate::strategy::strategy_action::StrategyAction;
use crate::strategy::strategy_condition::StrategyCondition;
use crate::strategy::strategy_indicator::StrategyIndicator;
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
pub async fn insert_strategies(strategy: Strategy) -> Result<i32> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = Builder::new_remote_replica(db_path, sync_url, auth_token)
        .build()
        .await?;

    let conn = db.connect()?;

    let parametros = params![
        strategy.id_user,
        strategy.nombre,
        strategy.descripcion,
        strategy.activa,
        strategy.creada_en
    ];
    conn.query(
        "INSERT INTO strategies (id_user, nombre, descripcion, activa, creada_en) VALUES (?, ?, ?, ?, ?) RETURNING id",
        parametros,
    )
    .await?;

    let id = conn.last_insert_rowid() as i32;
    Ok(id)
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
            parametros   TEXT DEFAULT '{}'   -- JSON: '{\"period\": 20}'
        )",
        (),
    )
    .await?;

    Ok("Tabla strategy_indicators is ok.".to_string())
}

#[tauri::command]
pub async fn insert_strategies_indicator(indicator: StrategyIndicator) -> Result<i32> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = Builder::new_remote_replica(db_path, sync_url, auth_token)
        .build()
        .await?;

    let conn = db.connect()?;

    let parametros = params![
        indicator.strategy_id,
        indicator.nombre,
        indicator.tipo,
        indicator.parametros.to_string()
    ];
    conn.query(
        "INSERT INTO strategy_indicators (strategy_id, nombre, tipo, parametros) VALUES (?, ?, ?, ?) RETURNING id",
        parametros,
    )
    .await?;

    let id = conn.last_insert_rowid() as i32;
    Ok(id)
}

#[tauri::command]
pub async fn get_strategy_indicator_by_id(id: i32) -> Result<StrategyIndicator> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = Builder::new_remote_replica(db_path, sync_url, auth_token)
        .build()
        .await?;

    let conn = db.connect()?;

    let sql = "SELECT * FROM strategy_indicators WHERE id = ?";

    let parametros = params![id];

    let mut result = conn.query(sql, parametros).await?;

    let row = result.next().await?.unwrap();

    let indicator = StrategyIndicator {
        id: row.get::<i32>(0)?,
        strategy_id: row.get::<i32>(1)?,
        nombre: row.get::<String>(2)?,
        tipo: row.get::<String>(3)?,
        parametros: serde_json::from_str(&row.get::<String>(4)?).unwrap(),
    };

    Ok(indicator)
}

#[tauri::command]
pub async fn get_strategies_indicatros_by_strategy_id(
    startegy_id: i32,
) -> Result<Vec<StrategyIndicator>> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = Builder::new_remote_replica(db_path, sync_url, auth_token)
        .build()
        .await?;

    let conn = db.connect()?;

    let parametros = params![startegy_id];

    let sql = "SELECT * FROM strategy_indicators WHERE strategy_id = ?";

    let mut result = conn.query(sql, parametros).await?;

    let mut indicators = Vec::new();
    while let Some(row) = result.next().await? {
        let indicator = StrategyIndicator {
            id: row.get::<i32>(0)?,
            strategy_id: row.get::<i32>(1)?,
            nombre: row.get::<String>(2)?,
            tipo: row.get::<String>(3)?,
            parametros: serde_json::from_str(&row.get::<String>(4)?).unwrap(),
        };
        indicators.push(indicator);
    }

    Ok(indicators)
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
            parametros    TEXT DEFAULT '{}'  -- JSON con parámetros extra
        )",
        (),
    )
    .await?;

    Ok("Tabla strategy_actions is ok.".to_string())
}

#[tauri::command]
pub async fn insert_strategies_action(action: StrategyAction) -> Result<i32> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = Builder::new_remote_replica(db_path, sync_url, auth_token)
        .build()
        .await?;

    let conn = db.connect()?;

    let parametros = params![
        action.strategy_id,
        action.tipo_signal,
        action.tipo,
        action.parametros.to_string()
    ];
    conn.query(
        "INSERT INTO strategy_actions (strategy_id, tipo_signal, tipo, parametros) VALUES (?, ?, ?, ?) RETURNING id",
        parametros,
    )
    .await?;

    let id = conn.last_insert_rowid() as i32;
    Ok(id)
}

#[tauri::command]
pub async fn get_strategy_action_by_id(id: i32) -> Result<StrategyAction> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = Builder::new_remote_replica(db_path, sync_url, auth_token)
        .build()
        .await?;

    let conn = db.connect()?;

    let sql = "SELECT * FROM strategy_actions WHERE id = ?";

    let parametros = params![id];

    let mut result = conn.query(sql, parametros).await?;

    let row = result.next().await?.unwrap();

    let action = StrategyAction {
        id: row.get::<i32>(0)?,
        strategy_id: row.get::<i32>(1)?,
        tipo_signal: row.get::<String>(2)?,
        tipo: row.get::<String>(3)?,
        parametros: serde_json::from_str(&row.get::<String>(4)?).unwrap(),
    };

    Ok(action)
}

#[tauri::command]
pub async fn get_strategies_actions_by_strategy_id(
    startegy_id: i32,
) -> Result<Vec<StrategyAction>> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = Builder::new_remote_replica(db_path, sync_url, auth_token)
        .build()
        .await?;

    let conn = db.connect()?;

    let parametros = params![startegy_id];

    let sql = "SELECT * FROM strategy_actions WHERE strategy_id = ?";

    let mut result = conn.query(sql, parametros).await?;

    let mut actions = Vec::new();
    while let Some(row) = result.next().await? {
        let action = StrategyAction {
            id: row.get::<i32>(0)?,
            strategy_id: row.get::<i32>(1)?,
            tipo_signal: row.get::<String>(2)?,
            tipo: row.get::<String>(3)?,
            parametros: serde_json::from_str(&row.get::<String>(4)?).unwrap(),
        };
        actions.push(action);
    }

    Ok(actions)
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

#[tauri::command]
pub async fn insert_strategy_condition(condition: StrategyCondition) -> Result<i32> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = Builder::new_remote_replica(db_path, sync_url, auth_token)
        .build()
        .await?;

    let conn = db.connect()?;

    let parametros = params![
        condition.strategy_id,
        condition.action_id,
        condition.campo_a,
        condition.operador,
        condition.campo_b,
        condition.logica,
        condition.orden
    ];
    conn.query(
        "INSERT INTO strategy_conditions (strategy_id, action_id, campo_a, operador, campo_b, logica, orden) VALUES (?, ?, ?, ?, ?, ?, ?) RETURNING id",
        parametros,
    )
    .await?;

    let id = conn.last_insert_rowid() as i32;
    Ok(id)
}

#[tauri::command]
pub async fn get_strategy_condition_by_id(id: i32) -> Result<StrategyCondition> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = Builder::new_remote_replica(db_path, sync_url, auth_token)
        .build()
        .await?;

    let conn = db.connect()?;

    let sql = "SELECT * FROM strategy_conditions WHERE id = ?";

    let parametros = params![id];

    let mut result = conn.query(sql, parametros).await?;

    let row = result.next().await?.unwrap();

    let condition = StrategyCondition {
        id: row.get::<i32>(0)?,
        strategy_id: row.get::<i32>(1)?,
        action_id: row.get::<i32>(2)?,
        campo_a: row.get::<String>(3)?,
        operador: row.get::<String>(4)?,
        campo_b: row.get::<String>(5)?,
        logica: row.get::<String>(6)?,
        orden: row.get::<i32>(7)?,
    };

    Ok(condition)
}

#[tauri::command]
pub async fn get_strategies_conditions_by_strategy_id(
    startegy_id: i32,
) -> Result<Vec<StrategyCondition>> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = Builder::new_remote_replica(db_path, sync_url, auth_token)
        .build()
        .await?;

    let conn = db.connect()?;

    let parametros = params![startegy_id];

    let sql = "SELECT * FROM strategy_conditions WHERE strategy_id = ?";

    let mut result = conn.query(sql, parametros).await?;

    let mut conditions = Vec::new();
    while let Some(row) = result.next().await? {
        let condition = StrategyCondition {
            id: row.get::<i32>(0)?,
            strategy_id: row.get::<i32>(1)?,
            action_id: row.get::<i32>(2)?,
            campo_a: row.get::<String>(3)?,
            operador: row.get::<String>(4)?,
            campo_b: row.get::<String>(5)?,
            logica: row.get::<String>(6)?,
            orden: row.get::<i32>(7)?,
        };
        conditions.push(condition);
    }

    Ok(conditions)
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
