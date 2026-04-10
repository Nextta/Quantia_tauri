use crate::api::resultados::delete_resultados_by_backtest;
use crate::api::trades::{delete_trades_by_backtest, get_trades_by_backtest};
use crate::backtest::backtest::Backtest;
use crate::strategy::strategy::Strategy;
use crate::strategy::strategy_options::StrategyOptions;
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

/// Crea la tabla de backtests en la base de datos.
///
/// # Returns
/// * `Result<String>` - Mensaje de éxito si la tabla se crea correctamente.
///
/// # Errores
/// Retorna error si falla la conexión a la base de datos o la ejecución de la query.
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

/// Inserta un nuevo backtest en la base de datos.
///
/// # Parámetros
/// * `backtest`: Objeto Backtest con los datos del backtest a insertar.
///
/// # Returns
/// * `Result<i32>` - ID del backtest insertado.
///
/// # Errores
/// Retorna error si falla la conexión a la base de datos o la inserción.
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

/// Obtiene todos los backtests de la base de datos, incluyendo sus trades asociados.
///
/// # Returns
/// * `Result<Vec<Backtest>>` - Vector de backtests con todos sus componentes.
///
/// # Errores
/// Retorna error si falla la conexión a la base de datos o la consulta.
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
            estrategia: Strategy {
                id: 0,
                id_user: 0,
                nombre: String::new(),
                descripcion: None,
                activa: false,
                creada_en: String::new(),
                indicadores: Vec::new(),
                condiciones: Vec::new(),
                acciones: Vec::new(),
                opciones: StrategyOptions::new_empty(),
            },
        };
        backtests.push(backtest);
    }

    Ok(backtests)
}

/// Obtiene un backtest específico por su ID.
///
/// # Parámetros
/// * `id`: ID del backtest a buscar.
///
/// # Returns
/// * `Result<Vec<Backtest>>` - Vector con el backtest encontrado (vacío si no existe).
///
/// # Errores
/// Retorna error si falla la conexión a la base de datos o la consulta.
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
            estrategia: Strategy {
                id: 0,
                id_user: 0,
                nombre: String::new(),
                descripcion: None,
                activa: false,
                creada_en: String::new(),
                indicadores: Vec::new(),
                condiciones: Vec::new(),
                acciones: Vec::new(),
                opciones: StrategyOptions::new_empty(),
            },
        };
        backtests.push(backtest);
    }

    Ok(backtests)
}

/// Obtiene backtests por título.
///
/// # Parámetros
/// * `titulo`: Título del backtest a buscar.
///
/// # Returns
/// * `Result<Vec<Backtest>>` - Vector de backtests que coinciden con el título.
///
/// # Errores
/// Retorna error si falla la conexión a la base de datos o la consulta.
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
            estrategia: Strategy {
                id: 0,
                id_user: 0,
                nombre: String::new(),
                descripcion: None,
                activa: false,
                creada_en: String::new(),
                indicadores: Vec::new(),
                condiciones: Vec::new(),
                acciones: Vec::new(),
                opciones: StrategyOptions::new_empty(),
            },
        };
        backtests.push(backtest);
    }

    Ok(backtests)
}

/// Obtiene backtests por tipo.
///
/// # Parámetros
/// * `tipo`: Tipo de backtest a buscar (ej: "CFD", "Forex", etc.).
///
/// # Returns
/// * `Result<Vec<Backtest>>` - Vector de backtests que coinciden con el tipo.
///
/// # Errores
/// Retorna error si falla la conexión a la base de datos o la consulta.
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
            estrategia: Strategy {
                id: 0,
                id_user: 0,
                nombre: String::new(),
                descripcion: None,
                activa: false,
                creada_en: String::new(),
                indicadores: Vec::new(),
                condiciones: Vec::new(),
                acciones: Vec::new(),
                opciones: StrategyOptions::new_empty(),
            },
        };
        backtests.push(backtest);
    }

    Ok(backtests)
}

/// Elimina un backtest por su ID.
///
/// # Parámetros
/// * `id`: ID del backtest a eliminar.
///
/// # Returns
/// * `Result<()>` - Ok si la eliminación es correcta.
///
/// # Errores
/// Retorna error si falla la conexión a la base de datos o la eliminación.
#[tauri::command]
pub async fn delete_backtest(id: i32) -> Result<()> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = Builder::new_remote_replica(db_path, sync_url, auth_token)
        .build()
        .await?;

    let conn = db.connect()?;

    match delete_resultados_by_backtest(id).await {
        Ok(_) => {
            match delete_trades_by_backtest(id).await {
                Ok(_) => {
                    conn.execute("DELETE FROM backtests WHERE id = ?", [id])
                        .await?;
                }
                Err(e) => println!("{:?}", e),
            };
        }
        Err(e) => println!("{:?}", e),
    };

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_crud_backtest_cfd() -> Result<()> {
        let backtest: Backtest = Backtest {
            id: 0,
            titulo: "Test".to_string(),
            balance: 100.0,
            tipo: "CFD".to_string(),
            trades: Vec::new(),
            datos: Vec::new(),
            estrategia: Strategy {
                id: 0,
                id_user: 0,
                nombre: String::new(),
                descripcion: None,
                activa: false,
                creada_en: String::new(),
                indicadores: Vec::new(),
                condiciones: Vec::new(),
                acciones: Vec::new(),
                opciones: StrategyOptions::new_empty(),
            },
        };

        let _ = table_backtests_cfd().await?;

        let id: i32 = insert_backtest_cfd(backtest.clone()).await?;

        let _ = get_backtests().await?;

        let _ = get_backtests_by_tipo(backtest.tipo.clone()).await?;

        let _ = get_backtest_by_id(id).await?;

        let _ = get_backtests_by_titulo(backtest.titulo.clone()).await?;

        let _ = delete_backtest(id).await?;

        Ok(())
    }
}
