use crate::api::backtests::get_backtest_by_id;
use crate::utils::configuracion::Error;
use chrono::NaiveDateTime;
use polars::prelude::*;

fn parse_date_to_millis(s: &str) -> Result<i64, Error> {
    let naive = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")?;
    Ok(naive.and_utc().timestamp_millis())
}

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
}
