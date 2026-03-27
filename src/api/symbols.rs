use crate::backtest::dias::Dias;
use crate::backtest::symbol::SymbolInfoCFD;
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
pub async fn table_symbols_cfd() -> Result<String> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = Builder::new_remote_replica(db_path, sync_url, auth_token)
        .build()
        .await?;

    let conn = db.connect()?;

    conn.query(
        "CREATE TABLE IF NOT EXISTS symbol_cfd
                    (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    broker_id INTEGER NOT NULL,
                    name TEXT NOT NULL,
                    valor_contrato REAL NOT NULL,
                    comision_lote REAL NOT NULL,
                    swap_long REAL NOT NULL,
                    swap_short REAL NOT NULL,
                    dia_triple_swap TEXT NOT NULL,
                    lotaje_minimo REAL NOT NULL,
                    lotaje_maximo REAL NOT NULL,
                    digitos INTEGER NOT NULL,
                    open_weekend BOOLEAN NOT NULL,
                    spread REAL NOT NULL
                    )",
        (),
    )
    .await?;

    Ok("Tabla symbols is ok.".to_string())
}

#[tauri::command]
pub async fn get_symbol_cfd_by_id(id: i64) -> Result<SymbolInfoCFD> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = Builder::new_remote_replica(db_path, sync_url, auth_token)
        .build()
        .await?;

    let conn = db.connect()?;

    let mut rows = conn
        .query("SELECT * FROM symbol_cfd WHERE id = ?", params![id])
        .await?;

    let row = rows.next().await?.unwrap();

    let dia = match row.get::<String>(7)?.as_str() {
        "Lu" => Dias::Lu,
        "Ma" => Dias::Ma,
        "Mi" => Dias::Mi,
        "Ju" => Dias::Ju,
        "Vi" => Dias::Vi,
        "Sa" => Dias::Sa,
        "Do" => Dias::Do,
        _ => Dias::Vi,
    };
    let symbol: SymbolInfoCFD = SymbolInfoCFD::new(
        row.get::<i32>(0)?,
        row.get::<i32>(1)?,
        row.get::<String>(2)?,
        row.get::<f64>(3)?,
        row.get::<f64>(4)?,
        row.get::<f64>(5)?,
        row.get::<f64>(6)?,
        dia,
        row.get::<f64>(8)?,
        row.get::<f64>(9)?,
        row.get::<u32>(10)?,
        row.get::<f64>(11)?,
        row.get::<bool>(12)?,
    )
    .await;
    Ok(symbol)
}

#[tauri::command]
pub async fn insert_symbol_cfd(symbol: SymbolInfoCFD) -> Result<i32> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = Builder::new_remote_replica(db_path, sync_url, auth_token)
        .build()
        .await?;

    let conn = db.connect()?;

    let params = params![
        symbol.broker_id,
        symbol.name,
        symbol.valor_contrato,
        symbol.comision_lote,
        symbol.swap_long,
        symbol.swap_short,
        symbol.dia_triple_swap.to_string(),
        symbol.lotaje_minimo,
        symbol.lotaje_maximo,
        symbol.digitos,
        symbol.open_weekend,
        symbol.spread,
    ];

    conn.query("INSERT INTO symbol_cfd (broker_id, name, valor_contrato, comision_lote, swap_long, swap_short, dia_triple_swap, lotaje_minimo, lotaje_maximo, digitos, open_weekend, spread) VALUES (?, '?', ?, ?, ?, ?, '?', ?, ?, ?, ?, ?) RETURNING id", params).await?;

    let id = conn.last_insert_rowid() as i32;
    Ok(id)
}

#[tauri::command]
pub async fn get_symbols_cfd() -> Result<Vec<SymbolInfoCFD>> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = Builder::new_remote_replica(db_path, sync_url, auth_token)
        .build()
        .await?;

    let conn = db.connect()?;

    let mut rows = conn.query("SELECT * FROM symbol_cfd", ()).await?;

    let mut symbols: Vec<SymbolInfoCFD> = Vec::new();

    while let Some(row) = rows.next().await? {
        let dia = match row.get::<String>(7)?.as_str() {
            "Lu" => Dias::Lu,
            "Ma" => Dias::Ma,
            "Mi" => Dias::Mi,
            "Ju" => Dias::Ju,
            "Vi" => Dias::Vi,
            "Sa" => Dias::Sa,
            "Do" => Dias::Do,
            _ => Dias::Vi,
        };

        let symbol = SymbolInfoCFD {
            id: row.get::<i32>(0)?,
            broker_id: row.get::<i32>(1)?,
            name: row.get::<String>(2)?,
            valor_contrato: row.get::<f64>(3)?,
            comision_lote: row.get::<f64>(4)?,
            swap_long: row.get::<f64>(5)?,
            swap_short: row.get::<f64>(6)?,
            dia_triple_swap: dia,
            lotaje_minimo: row.get::<f64>(8)?,
            lotaje_maximo: row.get::<f64>(9)?,
            digitos: row.get::<u32>(10)?,
            open_weekend: row.get::<bool>(11)?,
            spread: row.get::<f64>(12)?,
        };
        symbols.push(symbol);
    }
    Ok(symbols)
}

#[tauri::command]
pub async fn get_symbols_cfd_by_name(name: &str) -> Result<Vec<SymbolInfoCFD>> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = Builder::new_remote_replica(db_path, sync_url, auth_token)
        .build()
        .await?;

    let conn = db.connect()?;

    let mut rows = conn
        .query("SELECT * FROM symbol_cfd WHERE name = ?", [name])
        .await?;

    let mut symbols: Vec<SymbolInfoCFD> = Vec::new();

    while let Some(row) = rows.next().await? {
        let dia = match row.get::<String>(7)?.as_str() {
            "Lu" => Dias::Lu,
            "Ma" => Dias::Ma,
            "Mi" => Dias::Mi,
            "Ju" => Dias::Ju,
            "Vi" => Dias::Vi,
            "Sa" => Dias::Sa,
            "Do" => Dias::Do,
            _ => Dias::Vi,
        };

        let symbol = SymbolInfoCFD {
            id: row.get::<i32>(0)?,
            broker_id: row.get::<i32>(1)?,
            name: row.get::<String>(2)?,
            valor_contrato: row.get::<f64>(3)?,
            comision_lote: row.get::<f64>(4)?,
            swap_long: row.get::<f64>(5)?,
            swap_short: row.get::<f64>(6)?,
            dia_triple_swap: dia,
            lotaje_minimo: row.get::<f64>(8)?,
            lotaje_maximo: row.get::<f64>(9)?,
            digitos: row.get::<u32>(10)?,
            open_weekend: row.get::<bool>(11)?,
            spread: row.get::<f64>(12)?,
        };
        symbols.push(symbol);
    }
    Ok(symbols)
}

#[tauri::command]
pub async fn get_symbols_cfd_by_broker(broker_id: i32) -> Result<Vec<SymbolInfoCFD>> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = Builder::new_remote_replica(db_path, sync_url, auth_token)
        .build()
        .await?;

    let conn = db.connect()?;

    let mut rows = conn
        .query("SELECT * FROM symbol_cfd WHERE broker_id = ?", [broker_id])
        .await?;

    let mut symbols: Vec<SymbolInfoCFD> = Vec::new();

    while let Some(row) = rows.next().await? {
        let dia = match row.get::<String>(7)?.as_str() {
            "Lu" => Dias::Lu,
            "Ma" => Dias::Ma,
            "Mi" => Dias::Mi,
            "Ju" => Dias::Ju,
            "Vi" => Dias::Vi,
            "Sa" => Dias::Sa,
            "Do" => Dias::Do,
            _ => Dias::Vi,
        };

        let symbol = SymbolInfoCFD {
            id: row.get::<i32>(0)?,
            broker_id: row.get::<i32>(1)?,
            name: row.get::<String>(2)?,
            valor_contrato: row.get::<f64>(3)?,
            comision_lote: row.get::<f64>(4)?,
            swap_long: row.get::<f64>(5)?,
            swap_short: row.get::<f64>(6)?,
            dia_triple_swap: dia,
            lotaje_minimo: row.get::<f64>(8)?,
            lotaje_maximo: row.get::<f64>(9)?,
            digitos: row.get::<u32>(10)?,
            open_weekend: row.get::<bool>(11)?,
            spread: row.get::<f64>(12)?,
        };
        symbols.push(symbol);
    }
    Ok(symbols)
}
