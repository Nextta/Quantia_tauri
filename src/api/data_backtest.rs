use crate::api::data::get_data;
use crate::structs::data::{DataBacktest, DataSymbol};
use crate::utils::configuracion::DB_LOCAL;
use crate::utils::configuracion::{get_db_config, Error};
use libsql::{params, Builder};

/// Crea la tabla de data_backtest en la base de datos.
///
/// # Returns
/// * `Result<String>` - Mensaje de éxito si la tabla se crea correctamente.
///
/// # Errores
/// Retorna error si falla la conexión a la base de datos o la ejecución de la query.
pub async fn table_data_backtest() -> Result<String, Error> {
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
pub async fn insert_data_backtest(data: &DataBacktest) -> Result<i32, Error> {
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

            let mut rows = conn.query(
                "INSERT INTO data_backtest (id_backtest, id_data_symbol) VALUES (?, ?) RETURNING id",
                params![data.id_backtest, data.id_data_symbol],
            )
            .await?;

            let row = rows.next().await?.ok_or_else(|| Error {
                msg: format!("No se ha podido insertar los datos."),
            })?;
            let id = row.get::<i32>(0)?;

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
pub async fn get_all_data() -> Result<Vec<DataBacktest>, Error> {
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

/// Obtiene todos los datos de data_backtest de la base de datos de un Backtest en especifico.
///
/// # Parámetros
/// * `id_backtest`: identificador del backtest.
///
/// # Returns
/// * `Result<Vec<DataSymbol>>` - Vector con todos los datos.
///
/// # Errores
/// Retorna error si falla la conexión a la base de datos o la inserción.
pub async fn get_data_by_backtest(id_backtest: i32) -> Result<DataSymbol, Error> {
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
        .query(
            "SELECT * FROM data_backtest WHERE id_backtest = ?",
            params![id_backtest],
        )
        .await?;

    let row = rows.next().await?.ok_or_else(|| Error {
        msg: format!("No data found for backtest {}", id_backtest),
    })?;

    let data_symbol = get_data(row.get::<u32>(2)?).await?;

    Ok(data_symbol)
}

/// Elimina una data_backtest por su ID.
///
/// # Parámetros
/// * `id`: ID de la data_backtest a eliminar.
///
/// # Returns
/// * `Result<()>` - Ok si la eliminación es correcta.
///
/// # Errores
/// Retorna error si falla la conexión a la base de datos o la eliminación.
pub async fn delete_data(id: u32) -> Result<(), Error> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = if !DB_LOCAL {
        Builder::new_remote_replica(db_path, sync_url, auth_token)
            .build()
            .await?
    } else {
        Builder::new_local(db_path).build().await?
    };

    let conn = db.connect()?;

    conn.execute("DELETE FROM data_backtest WHERE id = ?", params![id])
        .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::backtests::{delete_backtest, insert_backtest_cfd};
    use crate::backtest::backtest::Backtest;
    use crate::enums::activos::Activo;
    use crate::enums::data_format::DataFormatSymbol;
    use crate::enums::data_origen::DataOrigen;
    use crate::enums::gestion::GestionStrategy;
    use crate::enums::timeframe::Timeframe;
    use crate::strategy::strategy::Strategy;
    use crate::strategy::strategy_options::{StrategyOptions, TradingDirection};
    use crate::structs::data::DataSymbol;
    use crate::structs::parametros::GestionParams;
    use chrono::Utc;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_crud_data() -> Result<(), Error> {
        let backtest: Backtest = Backtest {
            id: 0,
            titulo: "Test_resultados".to_string(),
            balance: 1000.0,
            tipo: Activo::CDF,
            gestion_strategy: GestionStrategy::Fijo,
            parametros_gestion: GestionParams {
                multiplicador: 1.0,
                lotaje_fijo: 0.01,
            },
            trades: Vec::new(),
            datos: DataSymbol {
                id: 0,
                name: "Test".to_string(),
                timeframe: Some(Timeframe::D1),
                ruta: "data".to_string(),
                formato: Some(DataFormatSymbol::Csv),
                fecha_inicio: "00/00/0000".to_string(),
                fecha_fin: "00/00/0000".to_string(),
                actualizado: false,
                n_data: 1252,
                origen: Some(DataOrigen::DukasCopy),
            },
            estrategia: Strategy {
                id: 0,
                id_user: 0,
                nombre: "Test_Resultados".to_string(),
                descripcion: Some("None".to_string()),
                activa: false,
                creada_en: "00/00/0000".to_string(),
                indicadores: Vec::new(),
                acciones: Vec::new(),
                opciones: StrategyOptions {
                    id: 0,
                    strategy_id: 0,
                    multiples_trades: false,
                    trading_direccion: TradingDirection::Both,
                    operar_finde: false,
                    cerrar_fin_de_dia: false,
                    hora_fin_de_dia: Utc::now(),
                    cerrar_viernes: false,
                    hora_cierre_viernes: Utc::now(),
                    rango_operativo: false,
                    rango_operativo_inicio: Utc::now(),
                    rango_operativo_fin: Utc::now(),
                    cerrar_fin_rango_operativo: false,
                    activar_cierre_numero_velas: false,
                    numero_velas_cierre: 32,
                    cierre_limite_hora: false,
                    hora_cierre_limite: Utc::now(),
                    parametros_stoploss: None,
                    parametros_takeprofit: None,
                },
            },
        };

        let id_back = insert_backtest_cfd(&backtest).await?;

        let data: DataSymbol = DataSymbol {
            id: 0,
            name: "EURUSD".to_string(),
            timeframe: Some(Timeframe::H1),
            ruta: "data".to_string(),
            formato: Some(DataFormatSymbol::Parquet),
            fecha_inicio: "01/01/2020".to_string(),
            fecha_fin: "01/01/2026".to_string(),
            actualizado: false,
            n_data: 2542156,
            origen: Some(DataOrigen::DukasCopy),
        };

        let id_data = crate::api::data::insert_data(&data).await?;

        let data: DataBacktest = DataBacktest {
            id: 0,
            id_backtest: id_back,
            id_data_symbol: id_data,
        };

        let id = insert_data_backtest(&data).await?;

        let _ = get_data_by_backtest(data.id_backtest).await?;

        let _ = get_all_data().await?;

        let _ = delete_data(id as u32).await?;

        let _ = delete_backtest(id_back).await?;

        let _ = crate::api::data::delete_data(id_data).await?;

        Ok(())
    }
}
