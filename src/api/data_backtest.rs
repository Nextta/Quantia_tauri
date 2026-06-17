use crate::structs::data::DataBacktest;
use crate::utils::configuracion::DB_LOCAL;
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

/// Crea la tabla de data_backtest en la base de datos.
///
/// # Returns
/// * `Result<String>` - Mensaje de éxito si la tabla se crea correctamente.
///
/// # Errores
/// Retorna error si falla la conexión a la base de datos o la ejecución de la query.
pub async fn table_data_backtest() -> Result<String> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = if !DB_LOCAL {
        Builder::new_remote_replica(db_path, sync_url, auth_token)
            .build()
            .await?
    } else {
        Builder::new_local(db_path).build().await?
    };

    let conn = db.connect()?;

    conn.query(
        "CREATE TABLE IF NOT EXISTS data_backtest
                    (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    id_backtest INTEGER NOT NULL REFERENCES backtest(id),
                    id_data_symbol INTEGER NOT NULL REFERENCES data(id)
                    )",
        (),
    )
    .await?;

    Ok("Tabla data_backtest is ok.".to_string())
}

/// Inserta un nuevo en data_backtest en la base de datos.
///
/// # Parámetros
/// * `id_backtest`: identificador del backtest.
/// * `id_data`: identificador de la condiguración de los datos del symbolo.
///
/// # Returns
/// * `Result<i32>` - ID de la data_backtest insertado.
///
/// # Errores
/// Retorna error si falla la conexión a la base de datos o la inserción.
pub async fn insert_data_backtest(id_backtest: i32, id_data: u32) -> Result<i32> {
    match table_data_backtest().await {
        Ok(_) => {
            let (db_path, sync_url, auth_token) = get_db_config()?;

            let db = if !DB_LOCAL {
                Builder::new_remote_replica(db_path, sync_url, auth_token)
                    .build()
                    .await?
            } else {
                Builder::new_local(db_path).build().await?
            };

            let conn = db.connect()?;

            conn.query(
                "INSERT INTO data_backtest (id_backtest, id_data_symbol) VALUES (?, ?) RETURNING id",
                params![id_backtest, id_data],
            )
            .await?;

            let id = conn.last_insert_rowid() as i32;
            Ok(id)
        }
        Err(e) => Err(e),
    }
}

/// Obtiene todos los datos de data_backtest de la base de datos.
///
/// # Returns
/// * `Result<Vec<DataBacktest>>` - Vector con todos los datos.
///
/// # Errores
/// Retorna error si falla la conexión a la base de datos o la inserción.
pub async fn get_all_data() -> Result<Vec<DataBacktest>> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = if !DB_LOCAL {
        Builder::new_remote_replica(db_path, sync_url, auth_token)
            .build()
            .await?
    } else {
        Builder::new_local(db_path).build().await?
    };

    let conn = db.connect()?;

    let mut rows = conn.query("SELECT * FROM data_backtest", ()).await?;

    let mut data_list: Vec<DataBacktest> = Vec::new();

    while let Some(row) = rows.next().await? {
        let data = DataBacktest {
            id: row.get::<u32>(0)?,
            id_backtest: row.get::<i32>(1)?,
            id_data_symbol: row.get::<u32>(2)?,
        };
        data_list.push(data);
    }

    Ok(data_list)
}
