use crate::api::backtests::get_backtest_by_id;
use crate::api::strategies::get_strategies_by_id;
use crate::utils::configuracion::Error;
use chrono::NaiveDateTime;
use polars::prelude::*;
use serde::{Deserialize, Serialize};

fn parse_date_to_millis(s: &str) -> Result<i64, Error> {
    let naive = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")?;
    Ok(naive.and_utc().timestamp_millis())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DataIndicator {
    pub time: u32,
    pub value: f64,
}

/// Optiene los datos de un indicador para mostrar el indicador en tradingview.
///
/// # Argments:
/// id_backtest: Identificador del backtest al que vamos a optener los datos del indicador.
/// id_estrategia: Identificador de la estrategia.
/// column: Nombre de la columna del indicador.
/// from_date: Fecha donde inician los datos.
/// to_date: Fecha donde finalizan los datos.
/// prev_bars: Velas previas a cargar antes de from_date y despues de to_date. Por defecto 20.
///
/// # Return
/// Delvuelve un vector con los datos del indicador.
#[tauri::command]
pub async fn get_indicator_for_tv(
    id_backtest: i32,
    id_estrategia: i32,
    column: &str,
    from_date: &str,
    to_date: &str,
    prev_bars: Option<u32>,
) -> Result<Vec<DataIndicator>, Error> {
    match get_backtest_by_id(id_backtest).await {
        Ok(mut backtest) => {
            backtest.estrategia = get_strategies_by_id(id_estrategia).await?;
            let mut df = backtest.get_datos().unwrap();

            backtest.set_indicators_strategy(&mut df);
            df = df
                .lazy()
                .fill_nan(lit(NULL))
                .drop_nulls(None)
                .collect()
                .unwrap();

            let velas = prev_bars.unwrap_or(20);
            let from_ts = parse_date_to_millis(from_date)?;
            let to_ts = parse_date_to_millis(to_date)?;

            let df = df.with_row_index("__idx".into(), None)?;
            // 3. Encontrar índice mínimo/máximo dentro del rango de fechas
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

            // 4. Slice, seleccionar columnas, convertir time a segundos
            let result = df
                .slice(start as i64, end - start)
                .lazy()
                .select([
                    (col("time") / lit(1000i64))
                        .cast(DataType::UInt32)
                        .alias("time"),
                    col(column).alias("value"),
                ])
                .collect()?;

            // 5. Convertir a Vec<DataIndicator>
            let times = result.column("time")?.u32()?;
            let values = result.column("value")?.f64()?;

            let indicators: Vec<DataIndicator> = times
                .into_iter()
                .zip(values)
                .map(|(t, v)| DataIndicator {
                    time: t.unwrap(),
                    value: v.unwrap(),
                })
                .collect();

            Ok(indicators)
        }
        Err(e) => Err(e),
    }
}
