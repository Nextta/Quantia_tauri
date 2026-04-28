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

struct TestsActive {
    pub valor: bool,
}

fn get_db_config() -> Result<(String, String, String, TestsActive)> {
    dotenv().expect(".env file not found");
    let db_path = env::var("DB_PATH").unwrap();
    let sync_url = env::var("TURSO_SYNC_URL").unwrap();
    let auth_token = env::var("TURSO_AUTH_TOKEN").unwrap();
    let tests_active = TestsActive { valor: true };
    Ok((db_path, sync_url, auth_token, tests_active))
}

/// Crea la tabla `trades` en la base de datos si no existe.
///
/// # Returns
/// Returns `Ok(String)` con el mensaje "Tabla Trades is ok." si la tabla se crea correctamente.
///
/// # Errores
/// Retorna un error si no se puede conectar a la base de datos o si la creación de la tabla falla.
#[tauri::command]
pub async fn table_trades() -> Result<String> {
    let (db_path, sync_url, auth_token, test_active) = get_db_config()?;

    let db = if !test_active.valor {
        Builder::new_remote_replica(db_path, sync_url, auth_token)
            .build()
            .await?
    } else {
        Builder::new_local(db_path).build().await?
    };

    let conn = db.connect()?;

    conn.query(
        "CREATE TABLE IF NOT EXISTS trades
                (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                id_backtest INTEGER NOT NULL REFERENCES backtests(id),
                id_symbol INTEGER NOT NULL REFERENCES symbol_cfd(id),
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

/// Inserta un nuevo trade en la base de datos asociado a un backtest.
///
/// # Parámetros
/// - `id_backtest`: ID del backtest al que pertenece el trade.
/// - `trade`: Objeto `Trade` contiene los datos del trade a insertar.
///
/// # Returns
/// Returns `Ok(i32)` con el ID del trade insertado.
///
/// # Errores
/// Retorna un error si no se puede conectar a la base de datos o si la inserción falla.
#[tauri::command]
pub async fn insert_trades(id_backtest: i32, trade: &Trade) -> Result<i32> {
    let (db_path, sync_url, auth_token, test_active) = get_db_config()?;

    let db = if !test_active.valor {
        Builder::new_remote_replica(db_path, sync_url, auth_token)
            .build()
            .await?
    } else {
        Builder::new_local(db_path).build().await?
    };

    let conn = db.connect()?;

    let pl: Decimal = trade.pl.to_string().parse().unwrap();
    let plsc: Decimal = trade.plsc.to_string().parse().unwrap();
    let pips_pl: Decimal = trade.pips_pl.to_string().parse().unwrap();

    let parametros = params![
        id_backtest,
        trade.symbol.id,
        trade.symbol.name.clone(),
        trade.tipo.clone(),
        trade.lotaje,
        trade.multiplicador,
        trade.t0.clone(),
        trade.precio_entrada,
        trade.tp,
        trade.sl,
        trade.t1.clone(),
        trade.precio_cierre,
        trade.precio_maximo,
        trade.precio_minimo,
        trade.duracion_segundos.clone(),
        trade.duracion_minutos.clone(),
        trade.duracion_horas.clone(),
        trade.duracion_dias.clone(),
        trade.label,
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

    conn.query("INSERT INTO trades (id_backtest, id_symbol, symbol, tipo, lotaje, multiplicador, t0, precio_entrada, tp, sl, t1, precio_cierre, precio_maximo, precio_minimo, duracion_segundos, duracion_minutos, duracion_horas, duracion_dias, label, pl, plsc, pips_pl) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) RETURNING id",
        parametros).await?;

    let id = conn.last_insert_rowid() as i32;
    Ok(id)
}

/// Obtiene todos los trades asociados a un backtest específico.
///
/// # Parámetros
/// - `id_backtest`: ID del backtest del cual obtener los trades.
///
/// # Returns
/// Returns `Ok(Vec<Trade>)` con la lista de trades del backtest.
///
/// # Errores
/// Retorna un error si no se puede conectar a la base de datos o si la consulta falla.
#[tauri::command]
pub async fn get_trades_by_backtest(id_backtest: i32) -> Result<Vec<Trade>> {
    let (db_path, sync_url, auth_token, test_active) = get_db_config()?;

    let db = if !test_active.valor {
        Builder::new_remote_replica(db_path, sync_url, auth_token)
            .build()
            .await?
    } else {
        Builder::new_local(db_path).build().await?
    };

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

/// Elimina todos los trades por id.
///
/// # Parámetros
/// - `id`: ID del trade.
///
/// # Returns
/// Returns `Ok(())` si la eliminación fue exitosa.
///
/// # Errores
/// Retorna un error si no se puede conectar a la base de datos o si la consulta falla.
#[tauri::command]
pub async fn delete_trades(id: i32) -> Result<()> {
    let (db_path, sync_url, auth_token, test_active) = get_db_config()?;

    let db = if !test_active.valor {
        Builder::new_remote_replica(db_path, sync_url, auth_token)
            .build()
            .await?
    } else {
        Builder::new_local(db_path).build().await?
    };

    let conn = db.connect()?;

    conn.query("DELETE FROM trades WHERE id = ?", [id]).await?;

    Ok(())
}

/// Elimina todos los trades por backtest.
///
/// # Parámetros
/// - `id_backtest`: ID del backtest del cual eliminar los trades.
///
/// # Returns
/// Returns `Ok(())` si la eliminación fue exitosa.
///
/// # Errores
/// Retorna un error si no se puede conectar a la base de datos o si la consulta falla.
#[tauri::command]
pub async fn delete_trades_by_backtest(id_backtest: i32) -> Result<()> {
    let (db_path, sync_url, auth_token, test_active) = get_db_config()?;

    let db = if !test_active.valor {
        Builder::new_remote_replica(db_path, sync_url, auth_token)
            .build()
            .await?
    } else {
        Builder::new_local(db_path).build().await?
    };

    let conn = db.connect()?;

    conn.query("DELETE FROM trades WHERE id_backtest = ?", [id_backtest])
        .await?;

    Ok(())
}

// ============== TESTS ==============

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_crud_trade() -> Result<()> {
        let mut trade: Trade = Trade::new(
            1,
            SymbolInfoCFD {
                id: 1,
                broker_id: 1,
                name: "EURUSD".to_string(),
                valor_contrato: 100000.0,
                comision_lote: 6.0,
                swap_long: -6.0,
                swap_short: 6.0,
                dia_triple_swap: crate::backtest::dias::Dias::Vi,
                lotaje_minimo: 0.01,
                lotaje_maximo: 100.0,
                digitos: 5,
                open_weekend: false,
                spread: 0.2,
            },
        )
        .await;

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

        let _ = table_trades().await?;

        let id: i32 = insert_trades(trade.id_backtest.clone(), &trade).await?;

        let _ = get_trades_by_backtest(trade.id_backtest.clone()).await?;

        let _ = delete_trades(id).await?;

        Ok(())
    }
}
