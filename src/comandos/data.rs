use crate::api::backtests::get_backtest_by_id;
use crate::api::data::insert_data;
use crate::data_lab::add_data::add_data;
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
    let ruta_data = ruta.unwrap_or("download".to_string());
    let formato_data = format.unwrap_or(DataFormatSymbol::Parquet);
    let actualizado = actualized.unwrap_or(false);

    let timestamp: Vec<u64> = data.iter().map(|d| d.timestamp).collect();
    let open: Vec<f64> = data.iter().map(|d| d.open).collect();
    let high: Vec<f64> = data.iter().map(|d| d.high).collect();
    let low: Vec<f64> = data.iter().map(|d| d.low).collect();
    let close: Vec<f64> = data.iter().map(|d| d.close).collect();
    let volume: Vec<f64> = data.iter().map(|d| d.volume).collect();

    let columns: Vec<Column> = vec![
        Series::new("timestamp".into(), timestamp).into(),
        Series::new("open".into(), open).into(),
        Series::new("high".into(), high).into(),
        Series::new("low".into(), low).into(),
        Series::new("close".into(), close).into(),
        Series::new("volume".into(), volume).into(),
    ];

    let mut df = DataFrame::new_infer_height(columns)?;

    df = df
        .lazy()
        .select([
            (col("timestamp") / lit(1000i64))
                .cast(DataType::UInt32)
                .alias("time"),
            col("open"),
            col("high"),
            col("low"),
            col("close"),
            col("volume"),
        ])
        .collect()?;

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

    match add_data(
        df,
        &data_symbol.name,
        Some(&data_symbol.ruta),
        Some(formato_data),
    ) {
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
/// Delvuelve un string indicando que elos datos se han guardado con exito.
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
    let ruta_data = ruta.unwrap_or("download".to_string());
    let formato_data = format.unwrap_or(DataFormatSymbol::Parquet);
    let actualizado = actualized.unwrap_or(false);

    let timestamp: Vec<u64> = data.iter().map(|d| d.timestamp).collect();
    let ask_price: Vec<f64> = data.iter().map(|d| d.askPrice).collect();
    let bid_price: Vec<f64> = data.iter().map(|d| d.bidPrice).collect();
    let ask_volume: Vec<f64> = data.iter().map(|d| d.askVolume).collect();
    let bid_volume: Vec<f64> = data.iter().map(|d| d.bidVolume).collect();

    let columns: Vec<Column> = vec![
        Series::new("timestamp".into(), timestamp).into(),
        Series::new("askPrice".into(), ask_price).into(),
        Series::new("bidPrice".into(), bid_price).into(),
        Series::new("askVolume".into(), ask_volume).into(),
        Series::new("bidVolume".into(), bid_volume).into(),
    ];

    let mut df = DataFrame::new_infer_height(columns)?;

    df = df
        .lazy()
        .select([
            (col("timestamp") / lit(1000i64))
                .cast(DataType::UInt32)
                .alias("time"),
            col("askPrice"),
            col("bidPrice"),
            col("askVolume"),
            col("bidVolume"),
        ])
        .collect()?;

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

    match add_data(
        df,
        &data_symbol.name,
        Some(&data_symbol.ruta),
        Some(formato_data),
    ) {
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
pub struct DataTv {
    pub time: u32,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

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

            let velas = prev_bars.unwrap_or(2000);
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

    fn save_data(df_result: Vec<DataTv>, path: &str) -> PolarsResult<()> {
        let times: Vec<u32> = df_result.iter().map(|d| d.time).collect();
        let open: Vec<f64> = df_result.iter().map(|d| d.open).collect();
        let high: Vec<f64> = df_result.iter().map(|d| d.high).collect();
        let low: Vec<f64> = df_result.iter().map(|d| d.low).collect();
        let close: Vec<f64> = df_result.iter().map(|d| d.close).collect();
        let volume: Vec<f64> = df_result.iter().map(|d| d.volume).collect();

        let columns: Vec<Column> = vec![
            Series::new("time".into(), times).into(),
            Series::new("open".into(), open).into(),
            Series::new("high".into(), high).into(),
            Series::new("low".into(), low).into(),
            Series::new("close".into(), close).into(),
            Series::new("volume".into(), volume).into(),
        ];
        let mut df = DataFrame::new_infer_height(columns)?;

        let mut file = std::fs::File::create(path).unwrap();
        CsvWriter::new(&mut file).finish(&mut df).unwrap();
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_get_data_for_tv() -> Result<(), Error> {
        let datos =
            get_data_for_tv(35, "2024-08-19 04:00:00", "2024-08-19 11:00:00", Some(40)).await?;
        save_data(datos, "download/get_data_tv.csv").unwrap();
        Ok(())
    }

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

        Ok(())
    }
}
