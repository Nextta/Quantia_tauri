use crate::enums::data_format::DataFormatSymbol;
use crate::enums::timeframe::Timeframe;
use crate::structs::data::DataSymbol;
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

/// Crea la tabla de datos en la base de datos.
///
/// # Returns
/// * `Result<String>` - Mensaje de éxito si la tabla se crea correctamente.
///
/// # Errores
/// Retorna error si falla la conexión a la base de datos o la ejecución de la query.
pub async fn table_data() -> Result<String> {
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
        "CREATE TABLE IF NOT EXISTS data (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            timeframe TEXT,
            ruta TEXT NOT NULL,
            formato TEXT,
            fecha_inicio TEXT NOT NULL,
            fecha_fin TEXT NOT NULL,
            actualizado INTEGER DEFAULT 0,
            n_data INTEGER DEFAULT 0,
            origen TEXT NOT NULL
        )",
        (),
    )
    .await?;

    Ok("Tabla de datos OK".to_string())
}

/// Inserta nueva info de data en la base de datos.
///
/// # Parámetros
/// * `&DataSymbol`: Objeto DataSymbol con los datos de la data a insertar.
///
/// # Returns
/// * `Result<i32>` - ID de la data insertada.
///
/// # Errores
/// Retorna error si falla la conexión a la base de datos o la inserción.
pub async fn insert_data(data_symbol: &DataSymbol) -> Result<u32> {
    match table_data().await {
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
                "INSERT INTO data (name, timeframe, ruta, formato, fecha_inicio, fecha_fin, actualizado, n_data, origen) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?) RETURNING id",
                params![
                    data_symbol.name.as_str(),
                    data_symbol.timeframe.unwrap().as_str(),
                    data_symbol.ruta.as_str(),
                    data_symbol.formato.unwrap().as_str(),
                    data_symbol.fecha_inicio.as_str(),
                    data_symbol.fecha_fin.as_str(),
                    data_symbol.actualizado,
                    data_symbol.n_data,
                    data_symbol.origen.as_str()
                ],
            )
            .await?;

            let id = conn.last_insert_rowid() as u32;

            Ok(id)
        }
        Err(e) => Err(e),
    }
}

/// Obtiene toda lista de data de la base de datos.
///
/// # Returns
/// * `Result<Vec<DataSymbol>>` - Vector de DataSymbol.
///
/// # Errores
/// Retorna error si falla la conexión a la base de datos o la consulta.
pub async fn get_all_data() -> Result<Vec<DataSymbol>> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = if !DB_LOCAL {
        Builder::new_remote_replica(db_path, sync_url, auth_token)
            .build()
            .await?
    } else {
        Builder::new_local(db_path).build().await?
    };

    let conn = db.connect()?;

    let mut rows = conn.query("SELECT * FROM data", ()).await?;

    let mut data_list: Vec<DataSymbol> = Vec::new();

    while let Some(row) = rows.next().await? {
        let data = DataSymbol {
            id: row.get::<u32>(0)?,
            name: row.get::<String>(1)?,
            timeframe: Timeframe::as_tf(row.get::<String>(2)?.as_str()),
            ruta: row.get::<String>(3)?,
            formato: DataFormatSymbol::as_tf(row.get::<String>(4)?.as_str()),
            fecha_inicio: row.get::<String>(5)?,
            fecha_fin: row.get::<String>(6)?,
            actualizado: if row.get::<i32>(7)? == 0 { false } else { true },
            n_data: row.get::<u32>(8)?,
            origen: row.get::<String>(9)?,
        };
        data_list.push(data);
    }

    Ok(data_list)
}

/// Obtiene una info de data específica por su ID.
///
/// # Parámetros
/// * `id`: ID de la data a buscar.
///
/// # Returns
/// * `Result<DataSymbol>` - DataSymbol encontrado con todos sus campos.
///
/// # Errores
/// Retorna error si no se encuentra la data o falla la conexión.
pub async fn get_data(id: u32) -> Result<DataSymbol> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = if !DB_LOCAL {
        Builder::new_remote_replica(db_path, sync_url, auth_token)
            .build()
            .await?
    } else {
        Builder::new_local(db_path).build().await?
    };

    let conn = db.connect()?;

    let mut rows = conn
        .query("SELECT * FROM data WHERE id = ?", params![id])
        .await?;

    let row = rows.next().await?.unwrap();

    let data: DataSymbol = DataSymbol {
        id: row.get::<u32>(0)?,
        name: row.get::<String>(1)?,
        timeframe: Timeframe::as_tf(row.get::<String>(2)?.as_str()),
        ruta: row.get::<String>(3)?,
        formato: DataFormatSymbol::as_tf(row.get::<String>(4)?.as_str()),
        fecha_inicio: row.get::<String>(5)?,
        fecha_fin: row.get::<String>(6)?,
        actualizado: if row.get::<i32>(7)? == 0 { false } else { true },
        n_data: row.get::<u32>(8)?,
        origen: row.get::<String>(9)?,
    };

    Ok(data)
}

/// Elimina una data por su ID.
///
/// # Parámetros
/// * `id`: ID de la data a eliminar.
///
/// # Returns
/// * `Result<()>` - Ok si la eliminación es correcta.
///
/// # Errores
/// Retorna error si falla la conexión a la base de datos o la eliminación.
pub async fn delete_data(id: u32) -> Result<()> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = if !DB_LOCAL {
        Builder::new_remote_replica(db_path, sync_url, auth_token)
            .build()
            .await?
    } else {
        Builder::new_local(db_path).build().await?
    };

    let conn = db.connect()?;

    conn.execute("DELETE FROM data WHERE id = ?", params![id])
        .await?;

    Ok(())
}

/// Actualiza el valor actualizado de data por su ID.
///
/// # Parámetros
/// * `id`: ID de la data a eliminar.
/// * `actualizado`: Bool que indica si esta actualizado o no.
///
/// # Returns
/// * `Result<()>` - Ok si la actualización es correcta.
///
/// # Errores
/// Retorna error si falla la conexión a la base de datos o la eliminación.
pub async fn update_data_actualizado(id: u32, actualizado: bool) -> Result<()> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = if !DB_LOCAL {
        Builder::new_remote_replica(db_path, sync_url, auth_token)
            .build()
            .await?
    } else {
        Builder::new_local(db_path).build().await?
    };

    let conn = db.connect()?;

    conn.execute(
        "UPDATE data SET actualizado = ? WHERE id = ?",
        params![if actualizado { 1 } else { 0 }, id],
    )
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_crud_data() -> Result<()> {
        let data: DataSymbol = DataSymbol {
            id: 0,
            name: "EURUSD".to_string(),
            timeframe: Some(Timeframe::H1),
            ruta: "download".to_string(),
            formato: Some(DataFormatSymbol::Parquet),
            fecha_inicio: "01/01/2020".to_string(),
            fecha_fin: "01/01/2026".to_string(),
            actualizado: false,
            n_data: 2542156,
            origen: "DukasCopy".to_string(),
        };

        let id = insert_data(&data).await?;

        let _ = get_data(id).await?;

        let _ = get_all_data().await?;

        let _ = update_data_actualizado(id, true).await?;

        let _ = delete_data(id).await?;

        Ok(())
    }
}
