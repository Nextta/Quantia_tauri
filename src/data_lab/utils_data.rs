use crate::comandos::data::{DataDukas, DataDukasTicks};
use chrono::DateTime;
use polars::prelude::*;

fn timeframe_to_seconds(tf: &str) -> i64 {
    match tf {
        "M1" => 60,
        "M5" => 300,
        "M10" => 600,
        "M15" => 900,
        "M30" => 1800,
        "H1" => 3600,
        "H4" => 14400,
        "D1" => 86400,
        "W1" => 604800,
        _ => 3600,
    }
}

/// Devuelve un Dataframe con los datos formateados en un timeframe especifico.
pub fn get_timeframe_data(
    data_name: &str,
    timeframe: &str,
    ruta_dist: &str,
) -> PolarsResult<DataFrame> {
    let ruta_data: String = format!("{}/{}.parquet", ruta_dist, data_name);

    let args = ScanArgsParquet::default();
    let lz: LazyFrame = LazyFrame::scan_parquet(PlRefPath::new(ruta_data.as_str()), args)?;

    let seconds = timeframe_to_seconds(timeframe);
    let duration_str = format!("{}i", seconds);

    let schema = lz
        .clone()
        .limit(0)
        .collect()?
        .schema()
        .get("open")
        .is_some();

    let df: DataFrame = if schema {
        lz.group_by_dynamic(
            col("time"),
            [],
            DynamicGroupOptions {
                every: Duration::parse(&duration_str),
                period: Duration::parse(&duration_str),
                offset: Duration::parse("0ms"),
                ..Default::default()
            },
        )
        .agg([
            col("open").first(),
            col("high").max(),
            col("low").min(),
            col("close").last(),
            col("volume").sum(),
        ])
        .collect()?
    } else {
        lz.select([
            col("time"),
            col("bidPrice").alias("open"),
            col("bidPrice").alias("high"),
            col("bidPrice").alias("low"),
            col("bidPrice").alias("close"),
            (col("bidVolume") + col("askVolume")).alias("volume"),
        ])
        .group_by_dynamic(
            col("time"),
            [],
            DynamicGroupOptions {
                every: Duration::parse(&duration_str),
                period: Duration::parse(&duration_str),
                offset: Duration::parse("0ms"),
                ..Default::default()
            },
        )
        .agg([
            col("open").first(),
            col("high").max(),
            col("low").min(),
            col("close").last(),
            col("volume").sum(),
        ])
        .collect()?
    };

    Ok(df)
}

/// Obtiene la ultima fecha de un archivo parquet
pub fn get_last_date(symbol: &str, ruta_dist: &str) -> PolarsResult<String> {
    let ruta: String = format!("{}/{}.parquet", ruta_dist, symbol);
    let args = ScanArgsParquet::default();

    let result: DataFrame = LazyFrame::scan_parquet(PlRefPath::new(&ruta), args)?
        .select([col("time")])
        .tail(1)
        .collect()?;

    let timestamp = result
        .column("time")?
        .i64()?
        .get(0)
        .ok_or_else(|| PolarsError::ComputeError("No timestamp value found".into()))?;

    let datetime = DateTime::from_timestamp_millis(timestamp)
        .ok_or_else(|| PolarsError::ComputeError("Invalid timestamp".into()))?;

    let start_date = datetime.format("%Y-%m-%d").to_string();

    Ok(start_date)
}

/// Une los nuevos datos descargados al dataset ya exsitente para actualizarlos.
pub fn join_datasets(parquet_ruta: &str, data_to_join: DataFrame) -> PolarsResult<DataFrame> {
    let ruta: String = parquet_ruta.to_string();
    let args = ScanArgsParquet::default();
    let lz: LazyFrame = LazyFrame::scan_parquet(PlRefPath::new(&ruta), args)?;

    let new_data = data_to_join.lazy().join(
        lz.clone(),
        [col("time")],
        [col("time")],
        JoinArgs {
            how: JoinType::Anti,
            ..Default::default()
        },
    );

    // Combinar solo los datos nuevos
    let final_result = concat([lz, new_data], UnionArgs::default())?;

    final_result.collect()
}

/// Une los nuevos datos descargados al dataset ya exsitente para actualizarlos.
pub fn join_datasets_ticks(parquet_ruta: &str, data_to_join: DataFrame) -> PolarsResult<DataFrame> {
    let ruta: String = parquet_ruta.to_string();
    let args = ScanArgsParquet::default();
    let lz: LazyFrame = LazyFrame::scan_parquet(PlRefPath::new(&ruta), args)?;

    let new_data = data_to_join.lazy().join(
        lz.clone(),
        [col("time")],
        [col("time")],
        JoinArgs {
            how: JoinType::Anti,
            ..Default::default()
        },
    );

    // Combinar solo los datos nuevos
    let final_result = concat([lz, new_data], UnionArgs::default())?;

    final_result.collect()
}

pub fn to_dataframe_data(data: Vec<DataDukas>) -> DataFrame {
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

    let mut df = DataFrame::new_infer_height(columns).unwrap();

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
        .collect()
        .unwrap();

    df
}

pub fn to_dataframe_data_ticks(data: Vec<DataDukasTicks>) -> DataFrame {
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

    let mut df = DataFrame::new_infer_height(columns).unwrap();

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
        .collect()
        .unwrap();

    df
}
