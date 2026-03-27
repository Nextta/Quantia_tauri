use crate::api::symbols::get_symbols_cfd_by_broker;
use crate::backtest::broker::BrokerCFD;
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
pub async fn table_brokers_cfd() -> Result<String> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = Builder::new_remote_replica(db_path, sync_url, auth_token)
        .build()
        .await?;

    let conn = db.connect()?;

    conn.query(
        "CREATE TABLE IF NOT EXISTS broker_cfd
                    (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    name TEXT NOT NULL
                    )",
        (),
    )
    .await?;

    Ok("Tabla brokers is ok.".to_string())
}

#[tauri::command]
pub async fn insert_broker_cfd(broker: BrokerCFD) -> Result<i32> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = Builder::new_remote_replica(db_path, sync_url, auth_token)
        .build()
        .await?;

    let conn = db.connect()?;

    let parametros = params![broker.name];

    conn.query(
        "INSERT INTO broker_cfd (name) VALUES ('?') RETURNING id",
        parametros,
    )
    .await?;

    let id = conn.last_insert_rowid() as i32;
    Ok(id)
}

#[tauri::command]
pub async fn get_brokers_cfd() -> Result<Vec<BrokerCFD>> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = Builder::new_remote_replica(db_path, sync_url, auth_token)
        .build()
        .await?;

    let conn = db.connect()?;

    let mut rows = conn.query("SELECT * FROM broker_cfd", ()).await?;

    let mut brokers: Vec<BrokerCFD> = Vec::new();
    while let Some(row) = rows.next().await? {
        let symbol_info = get_symbols_cfd_by_broker(row.get::<i32>(0)?).await.unwrap();

        let broker: BrokerCFD = BrokerCFD {
            id: row.get::<i32>(0)?,
            name: row.get::<String>(1)?,
            symbol_info: symbol_info,
        };

        brokers.push(broker);
    }

    Ok(brokers)
}

#[tauri::command]
pub async fn get_brokers_cfd_by_name(name: &str) -> Result<Vec<BrokerCFD>> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = Builder::new_remote_replica(db_path, sync_url, auth_token)
        .build()
        .await?;

    let conn = db.connect()?;

    let mut rows = conn
        .query("SELECT * FROM broker_cfd WHERE name = ?", params![name])
        .await?;

    let mut brokers: Vec<BrokerCFD> = Vec::new();

    while let Some(row) = rows.next().await? {
        let symbol_info = get_symbols_cfd_by_broker(row.get::<i32>(0)?).await.unwrap();

        let broker: BrokerCFD = BrokerCFD {
            id: row.get::<i32>(0)?,
            name: row.get::<String>(1)?,
            symbol_info: symbol_info,
        };
        brokers.push(broker);
    }
    Ok(brokers)
}

#[tauri::command]
pub async fn get_broker_cfd_by_id(id: i32) -> Result<BrokerCFD> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = Builder::new_remote_replica(db_path, sync_url, auth_token)
        .build()
        .await?;

    let conn = db.connect()?;

    let mut rows = conn
        .query("SELECT * FROM broker_cfd WHERE id = ?", params![id])
        .await?;

    let row = rows.next().await?.unwrap();

    let symbol_info = get_symbols_cfd_by_broker(row.get::<i32>(0)?).await.unwrap();

    let broker: BrokerCFD = BrokerCFD {
        id: row.get::<i32>(0)?,
        name: row.get::<String>(1)?,
        symbol_info: symbol_info,
    };

    Ok(broker)
}
