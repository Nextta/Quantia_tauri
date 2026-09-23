use crate::api::backtests::get_backtest_by_id;
use crate::api::data::{delete_data, insert_data, update_data_actualizado};
use crate::data_lab::add_data::add_data;
use crate::data_lab::delete_data::delete_data_local;
use crate::data_lab::export_data::export_data;
use crate::data_lab::update_data::{update_data, update_data_ticks};
use crate::data_lab::utils_data::{to_dataframe_data, to_dataframe_data_ticks};
use crate::enums::data_format::DataFormatSymbol;
use crate::enums::data_origen::DataOrigen;
use crate::enums::timeframe::Timeframe;
use crate::structs::data::DataSymbol;
use crate::utils::configuracion::Error;
use chrono::NaiveDateTime;
use polars::prelude::*;
use serde::{Deserialize, Serialize};

fn parse_date_to_millis(s: &str) -> Result<i64, Error> {
    let naive = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")?;
    Ok(naive.and_utc().timestamp_millis())
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DataDukas {
    pub timestamp: u64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

/// Guarda los datos de los activos descargados desde el frontend en determinado timeframe en un determinado formato. No Timeframe en ticks.
///
/// # Argments:
/// data: El array de datos del activo descargado.
/// name: Nombre con el que se guarda el archivo.
/// timeframe: El timeframe en que llegan los datos. No datos de Ticks.
/// from_date: Fecha donde inician los datos.
/// to_date: Fecha donde finalizan los datos.
/// broker_data: Origen de donde provienen los datos.
/// ruta: Carpeta donde se almacenarán los datos.
/// format: Formato del archivo de los datos.
/// actualized: Si los datos estan actualizados a la fecha de hoy (true) o no (false).
///
/// # Return
/// Delvuelve un string indicando que elos datos se han guardado con exito.
#[tauri::command]
pub async fn save_data_dukas(
    data: Vec<DataDukas>,
    name: String,
    timeframe: Timeframe,
    from_date: String,
    to_date: String,
    broker_data: DataOrigen,
    ruta: Option<String>,
    format: Option<DataFormatSymbol>,
    actualized: Option<bool>,
) -> Result<String, Error> {
    let ruta_data = ruta.unwrap_or("data".to_string());
    let formato_data = format.unwrap_or(DataFormatSymbol::Parquet);
    let actualizado = actualized.unwrap_or(false);

    let df = to_dataframe_data(data);

    let n_data: u32 = df.height() as u32;

    let data_symbol: DataSymbol = DataSymbol {
        id: 0,
        name: name,
        timeframe: Some(timeframe),
        ruta: ruta_data.clone(),
        formato: Some(formato_data),
        fecha_inicio: from_date,
        fecha_fin: to_date,
        actualizado: actualizado,
        n_data: n_data,
        origen: Some(broker_data),
    };

    match add_data(df, &data_symbol) {
        Ok(_) => {
            insert_data(&data_symbol).await?;
        }
        Err(_) => (),
    }

    Ok(format!(
        "Datos guardados correctamente en {}/{} - Data: {:?}",
        ruta_data,
        formato_data.to_string(),
        data_symbol
    ))
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[allow(non_snake_case)]
pub struct DataDukasTicks {
    pub timestamp: u64,
    pub askPrice: f64,
    pub bidPrice: f64,
    pub askVolume: f64,
    pub bidVolume: f64,
}

/// Guarda los datos de los activos descargados desde el frontend en determinado timeframe en un determinado formato. Solo Timeframe en ticks.
///
/// # Argments:
/// data: El array de datos del activo descargado.
/// name: Nombre con el que se guarda el archivo.
/// timeframe: El timeframe en que llegan los datos. No datos de Ticks.
/// from_date: Fecha donde inician los datos.
/// to_date: Fecha donde finalizan los datos.
/// broker_data: Origen de donde provienen los datos.
/// ruta: Carpeta donde se almacenarán los datos.
/// format: Formato del archivo de los datos.
/// actualized: Si los datos estan actualizados a la fecha de hoy (true) o no (false).
///
/// # Return
/// Delvuelve un string indicando que los datos se han guardado con exito.
#[tauri::command]
pub async fn save_data_dukas_ticks(
    data: Vec<DataDukasTicks>,
    name: String,
    timeframe: Timeframe,
    from_date: String,
    to_date: String,
    broker_data: DataOrigen,
    ruta: Option<String>,
    format: Option<DataFormatSymbol>,
    actualized: Option<bool>,
) -> Result<String, Error> {
    let ruta_data = ruta.unwrap_or("data".to_string());
    let formato_data = format.unwrap_or(DataFormatSymbol::Parquet);
    let actualizado = actualized.unwrap_or(false);

    let df = to_dataframe_data_ticks(data);

    let n_data: u32 = df.height() as u32;

    let data_symbol: DataSymbol = DataSymbol {
        id: 0,
        name: name,
        timeframe: Some(timeframe),
        ruta: ruta_data.clone(),
        formato: Some(formato_data),
        fecha_inicio: from_date,
        fecha_fin: to_date,
        actualizado: actualizado,
        n_data: n_data,
        origen: Some(broker_data),
    };

    match add_data(df, &data_symbol) {
        Ok(_) => {
            insert_data(&data_symbol).await?;
        }
        Err(_) => (),
    }

    Ok(format!(
        "Datos guardados correctamente en {}/{} - Data: {:?}",
        ruta_data,
        formato_data.to_string(),
        data_symbol
    ))
}

/// Elimina los datos de un symbolo de forma fisica y la información de la base de datos.
///
/// # Argments:
/// data_info: La información del symbolo a eliminar.
///
/// # Return
/// Delvuelve un string indicando que los datos se han eliminado con exito.
#[tauri::command]
pub async fn delete_data_symbol(data_info: DataSymbol) -> Result<String, Error> {
    match delete_data(data_info.id).await {
        Ok(_) => match delete_data_local(&data_info) {
            Ok(_) => {
                return Ok(format!(
                    "Symbol {} eliminado correctamente.",
                    data_info.name
                ));
            }
            Err(_) => {
                return Err(Error {
                    msg: "Error al eliminar los datos locales del symbolo".to_string(),
                })
            }
        },
        Err(_) => {
            return Err(Error {
                msg: "Error al eliminar los datos de la base de datos".to_string(),
            })
        }
    }
}

/// Exporta un symbolo a la ruta expecificada.
///
/// # Argments:
/// data_info: La información del symbolo a exportar.
/// ruta_export: Ruta donde se exportará el símbolo.
///
/// # Return
/// Delvuelve un string indicando que los datos se han exportado con exito.
#[tauri::command]
pub fn export_data_symbol(data_info: DataSymbol, ruta_export: &str) -> Result<String, Error> {
    export_data(&data_info, &ruta_export);

    Ok(format!(
        "Symbolo {} exportado con exito en la ruta {}",
        data_info.name,
        ruta_export.to_string()
    ))
}

/// Importa un symbolo a la ruta expecificada.
///
/// # Argments:
/// data_info: La información del symbolo a importar.
/// ruta_import: Ruta desde donde se importa el símbolo.
///
/// # Return
/// Delvuelve un string indicando que los datos se han importado con exito.
#[tauri::command]
pub fn import_data_symbol(
    data_info: DataSymbol,
    ruta_import: Option<&str>,
) -> Result<String, Error> {
    let ruta = ruta_import.unwrap_or("data");

    export_data(&data_info, &ruta);

    Ok(format!("Symbolo {} importado con exito.", data_info.name))
}

/// Actualiza un symbolo expecifico.
///
/// # Argments:
/// data: Vector con todos los datos OHLCV.
/// data_info: La información del symbolo a actualizar.
///
/// # Return
/// Delvuelve un string indicando que los datos se han actualizado con exito.
#[tauri::command]
pub async fn update_data_symbol(
    data: Vec<DataDukas>,
    data_info: DataSymbol,
) -> Result<String, Error> {
    let df = to_dataframe_data(data);

    match update_data(&df, &data_info) {
        Ok(_) => match update_data_actualizado(data_info.id, true).await {
            Ok(_) => {
                return Ok(format!(
                    "Datos del Symbol {} actializado correctamnete en la base de datos.",
                    data_info.name
                ));
            }
            Err(_) => {
                return Err(Error {
                    msg: "No se han podido actualizar los datos del Symbol en la base de datos."
                        .to_string(),
                });
            }
        },
        Err(_) => {
            return Err(Error {
                msg: "No se han podido actualizar los datos del Symbol en local.".to_string(),
            });
        }
    }
}

/// Actualiza los ticks del symbolo expecifico.
///
/// # Argments:
/// data: Vector con todos los datos en ticks.
/// data_info: La información del symbolo a actualizar.
///
/// # Return
/// Delvuelve un string indicando que los datos se han actualizado con exito.
#[tauri::command]
pub async fn update_data_symbol_ticks(
    data: Vec<DataDukasTicks>,
    data_info: DataSymbol,
) -> Result<String, Error> {
    let df = to_dataframe_data_ticks(data);

    match update_data_ticks(&df, &data_info) {
        Ok(_) => match update_data_actualizado(data_info.id, true).await {
            Ok(_) => {
                return Ok(format!(
                    "Datos del Symbol {} actializado correctamnete en la base de datos.",
                    data_info.name
                ));
            }
            Err(_) => {
                return Err(Error {
                    msg: "No se han podido actualizar los datos del Symbol en la base de datos."
                        .to_string(),
                });
            }
        },
        Err(_) => {
            return Err(Error {
                msg: "No se han podido actualizar los datos del Symbol en local.".to_string(),
            });
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DataTv {
    pub time: u32,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

/// Devuelve todos los datos de velas OHLCV dentro de un rango de fecha.
///
/// # Argments:
/// id_backtest: El id correspondiente al backtest.
/// from_date: Fecha desde donde inicia el rango de datos.
/// to_date: Fecha desde donde inicia el rango de datos.
/// prev_bars: Velas que se van a cargar previas al rango de fecha y despues del rango de fecha. Por defecto 20.
///
/// # Return
/// Delvuelve un vector con los datos de velas OHLCV.
#[tauri::command]
pub async fn get_data_for_tv(
    id_backtest: i32,
    from_date: &str,
    to_date: &str,
    prev_bars: Option<u32>,
) -> Result<Vec<DataTv>, Error> {
    match get_backtest_by_id(id_backtest).await {
        Ok(mut backtest) => {
            let mut df = backtest.get_datos().unwrap();

            let velas = prev_bars.unwrap_or(20);
            let from_ts = parse_date_to_millis(from_date)?;
            let to_ts = parse_date_to_millis(to_date)?;

            df = df.with_row_index("__idx".into(), None)?;
            // Encontrar índice mínimo/máximo dentro del rango de fechas
            let range_idx = df
                .clone()
                .lazy()
                .filter(
                    col("time")
                        .gt_eq(lit(from_ts))
                        .and(col("time").lt_eq(lit(to_ts))),
                )
                .select([col("__idx")])
                .collect()?;

            let min_idx = range_idx.column("__idx")?.idx()?.min();
            let max_idx = range_idx.column("__idx")?.idx()?.max();

            let (start, end) = match (min_idx, max_idx) {
                (Some(min), Some(max)) => {
                    let s = (min as i64 - velas as i64).max(0) as usize;
                    let e = (max as usize + velas as usize + 1).min(df.height());
                    (s, e)
                }
                _ => return Ok(vec![]),
            };

            // Slice, seleccionar columnas, convertir time a segundos
            let result = df
                .slice(start as i64, end - start)
                .lazy()
                .select([
                    (col("time") / lit(1000i64))
                        .cast(DataType::UInt32)
                        .alias("time"),
                    col("open"),
                    col("high"),
                    col("low"),
                    col("close"),
                    col("volume"),
                ])
                .collect()?;

            // Convertir a Vec<DataIndicator>
            let times: Vec<u32> = result
                .column("time")?
                .u32()?
                .into_iter()
                .map(|t| t.unwrap())
                .collect();
            let open: Vec<f64> = result
                .column("open")?
                .f64()?
                .into_iter()
                .map(|o| o.unwrap())
                .collect();
            let high: Vec<f64> = result
                .column("high")?
                .f64()?
                .into_iter()
                .map(|h| h.unwrap())
                .collect();
            let low: Vec<f64> = result
                .column("low")?
                .f64()?
                .into_iter()
                .map(|l| l.unwrap())
                .collect();
            let close: Vec<f64> = result
                .column("close")?
                .f64()?
                .into_iter()
                .map(|c| c.unwrap())
                .collect();
            let volume: Vec<f64> = result
                .column("volume")?
                .f64()?
                .into_iter()
                .map(|v| v.unwrap())
                .collect();

            let indicators: Vec<DataTv> = (0..times.len())
                .map(|i| DataTv {
                    time: times[i],
                    open: open[i],
                    high: high[i],
                    low: low[i],
                    close: close[i],
                    volume: volume[i],
                })
                .collect();

            return Ok(indicators);
        }
        Err(e) => return Err(e),
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::remove_file;

    // fn save_data(df_result: Vec<DataTv>, path: &str) -> PolarsResult<()> {
    //     let times: Vec<u32> = df_result.iter().map(|d| d.time).collect();
    //     let open: Vec<f64> = df_result.iter().map(|d| d.open).collect();
    //     let high: Vec<f64> = df_result.iter().map(|d| d.high).collect();
    //     let low: Vec<f64> = df_result.iter().map(|d| d.low).collect();
    //     let close: Vec<f64> = df_result.iter().map(|d| d.close).collect();
    //     let volume: Vec<f64> = df_result.iter().map(|d| d.volume).collect();

    //     let columns: Vec<Column> = vec![
    //         Series::new("time".into(), times).into(),
    //         Series::new("open".into(), open).into(),
    //         Series::new("high".into(), high).into(),
    //         Series::new("low".into(), low).into(),
    //         Series::new("close".into(), close).into(),
    //         Series::new("volume".into(), volume).into(),
    //     ];
    //     let mut df = DataFrame::new_infer_height(columns)?;

    //     let mut file = std::fs::File::create(path).unwrap();
    //     CsvWriter::new(&mut file).finish(&mut df).unwrap();
    //     Ok(())
    // }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_save_data_dukas() -> Result<(), Error> {
        let mut datos: Vec<DataDukas> = vec![];
        let dato: DataDukas = DataDukas {
            timestamp: 1547416809672,
            open: 1.2525,
            high: 1.2336,
            low: 1.2222,
            close: 1.2563,
            volume: 12524.0,
        };

        datos.push(dato.clone());
        datos.push(dato.clone());
        datos.push(dato.clone());
        datos.push(dato.clone());

        save_data_dukas(
            datos,
            "name".to_string(),
            Timeframe::M1,
            "00/00/0000".to_string(),
            "00/00/0000".to_string(),
            DataOrigen::DukasCopy,
            None,
            Some(DataFormatSymbol::Csv),
            Some(true),
        )
        .await?;

        remove_file("data/name.csv").unwrap();

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_save_data_dukas_ticks() -> Result<(), Error> {
        let mut datos: Vec<DataDukasTicks> = vec![];
        let dato: DataDukasTicks = DataDukasTicks {
            timestamp: 1547416809672,
            askPrice: 1.2221,
            bidPrice: 1.2221,
            askVolume: 1.2221,
            bidVolume: 1.2221,
        };

        datos.push(dato.clone());
        datos.push(dato.clone());
        datos.push(dato.clone());
        datos.push(dato.clone());

        save_data_dukas_ticks(
            datos,
            "name_ticks".to_string(),
            Timeframe::M1,
            "00/00/0000".to_string(),
            "00/00/0000".to_string(),
            DataOrigen::DukasCopy,
            None,
            Some(DataFormatSymbol::Csv),
            Some(true),
        )
        .await?;

        remove_file("data/name_ticks.csv").unwrap();

        Ok(())
    }
}
