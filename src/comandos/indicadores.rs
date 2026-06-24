use crate::api::backtests::get_backtest_by_id;
use crate::api::strategies::get_strategies_by_id;
use crate::utils::configuracion::Error;
use chrono::NaiveDateTime;
use polars::prelude::*;

fn parse_date_to_millis(s: &str) -> Result<i64, Error> {
    let naive = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")?;
    Ok(naive.and_utc().timestamp_millis())
}

pub struct DataIndicator {
    pub time: u32,
    pub value: f64,
}

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

            let velas = prev_bars.unwrap_or(2000);
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
                .zip(values.into_iter())
                .map(|(t, v)| DataIndicator {
                    time: t.unwrap(),
                    value: v.unwrap(),
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

    fn save_data(df_result: Vec<DataIndicator>, path: &str) -> PolarsResult<()> {
        let times: Vec<u32> = df_result.iter().map(|d| d.time).collect();
        let values: Vec<f64> = df_result.iter().map(|d| d.value).collect();

        let columns: Vec<Column> = vec![
            Series::new("time".into(), times).into(),
            Series::new("value".into(), values).into(),
        ];
        let mut df = DataFrame::new_infer_height(columns)?;

        let mut file = std::fs::File::create(path).unwrap();
        CsvWriter::new(&mut file).finish(&mut df).unwrap();
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_get_indicator() -> Result<(), Error> {
        let datos = get_indicator_for_tv(
            35,
            1,
            "ema_50",
            "2024-08-19 04:00:00",
            "2024-08-19 11:00:00",
            Some(40),
        )
        .await?;
        save_data(datos, "download/get_indicador.csv").unwrap();
        Ok(())
    }
}
