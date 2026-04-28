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

/// Crea la tabla de brokers CFD en la base de datos.
///
/// # Returns
/// * `Result<String>` - Mensaje de éxito si la tabla se crea correctamente.
///
/// # Errores
/// Retorna error si falla la conexión a la base de datos o la ejecución de la query.
#[tauri::command]
pub async fn table_brokers_cfd() -> Result<String> {
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

/// Inserta un nuevo broker CFD en la base de datos.
///
/// # Parámetros
/// * `broker`: Objeto BrokerCFD con los datos del broker a insertar.
///
/// # Returns
/// * `Result<i32>` - ID del broker insertado.
///
/// # Errores
/// Retorna error si falla la conexión a la base de datos o la inserción.
#[tauri::command]
pub async fn insert_broker_cfd(broker: BrokerCFD) -> Result<i32> {
    let (db_path, sync_url, auth_token, test_active) = get_db_config()?;

    let db = if !test_active.valor {
        Builder::new_remote_replica(db_path, sync_url, auth_token)
            .build()
            .await?
    } else {
        Builder::new_local(db_path).build().await?
    };

    let conn = db.connect()?;

    let parametros = params![broker.name];

    conn.query(
        "INSERT INTO broker_cfd (name) VALUES (?) RETURNING id",
        parametros,
    )
    .await?;

    let id = conn.last_insert_rowid() as i32;
    Ok(id)
}

/// Obtiene todos los brokers CFD de la base de datos, incluyendo sus símbolos asociados.
///
/// # Returns
/// * `Result<Vec<BrokerCFD>>` - Vector de brokers con todos sus componentes.
///
/// # Errores
/// Retorna error si falla la conexión a la base de datos o la consulta.
#[tauri::command]
pub async fn get_brokers_cfd() -> Result<Vec<BrokerCFD>> {
    let (db_path, sync_url, auth_token, test_active) = get_db_config()?;

    let db = if !test_active.valor {
        Builder::new_remote_replica(db_path, sync_url, auth_token)
            .build()
            .await?
    } else {
        Builder::new_local(db_path).build().await?
    };

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

/// Obtiene brokers CFD por nombre.
///
/// # Parámetros
/// * `name`: Nombre del broker a buscar.
///
/// # Returns
/// * `Result<Vec<BrokerCFD>>` - Vector de brokers que coinciden con el nombre.
///
/// # Errores
/// Retorna error si falla la conexión a la base de datos o la consulta.
#[tauri::command]
pub async fn get_brokers_cfd_by_name(name: &str) -> Result<Vec<BrokerCFD>> {
    let (db_path, sync_url, auth_token, test_active) = get_db_config()?;

    let db = if !test_active.valor {
        Builder::new_remote_replica(db_path, sync_url, auth_token)
            .build()
            .await?
    } else {
        Builder::new_local(db_path).build().await?
    };

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

/// Obtiene un broker CFD específico por su ID.
///
/// # Parámetros
/// * `id`: ID del broker a buscar.
///
/// # Returns
/// * `Result<BrokerCFD>` - Broker encontrado con todos sus componentes.
///
/// # Errores
/// Retorna error si no se encuentra el broker o falla la conexión.
#[tauri::command]
pub async fn get_broker_cfd_by_id(id: i32) -> Result<BrokerCFD> {
    let (db_path, sync_url, auth_token, test_active) = get_db_config()?;

    let db = if !test_active.valor {
        Builder::new_remote_replica(db_path, sync_url, auth_token)
            .build()
            .await?
    } else {
        Builder::new_local(db_path).build().await?
    };

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

/// Elimina un broker CFD por su ID.
///
/// # Parámetros
/// * `id`: ID del broker a eliminar.
///
/// # Returns
/// * `Result<()>` - Ok si la eliminación es correcta.
///
/// # Errores
/// Retorna error si falla la conexión a la base de datos o la eliminación.
#[tauri::command]
pub async fn delete_broker_cfd(id: i32) -> Result<()> {
    let (db_path, sync_url, auth_token, test_active) = get_db_config()?;

    let db = if !test_active.valor {
        Builder::new_remote_replica(db_path, sync_url, auth_token)
            .build()
            .await?
    } else {
        Builder::new_local(db_path).build().await?
    };

    let conn = db.connect()?;

    conn.execute("DELETE FROM broker_cfd WHERE id = ?", params![id])
        .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_crud_broker_cfd() -> Result<()> {
        let broker: BrokerCFD = BrokerCFD {
            id: 2,
            name: "Darwinex".to_string(),
            symbol_info: Vec::new(),
        };

        let _ = table_brokers_cfd().await?;

        let id: i32 = insert_broker_cfd(broker.clone()).await?;

        let _ = get_brokers_cfd().await?;

        let _ = get_broker_cfd_by_id(id).await?;

        let _ = get_brokers_cfd_by_name(&broker.name.clone()).await?;

        let _ = delete_broker_cfd(id).await?;
        Ok(())
    }
}
