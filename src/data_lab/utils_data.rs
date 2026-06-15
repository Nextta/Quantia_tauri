use crate::structs::data::{DataFormat, DataFormatTicks};
use chrono::DateTime;
use polars::prelude::*;

/// Devuelve un Dataframe con los datos formateados en un timeframe especifico.
pub fn get_timeframe_data(
    data_name: &str,
    timeframe: &str,
    ruta_dist: &str,
) -> PolarsResult<DataFrame> {
    let ruta_data: String = format!("{}/{}.parquet", ruta_dist, data_name);

    let args = ScanArgsParquet::default();
    let lz: LazyFrame = LazyFrame::scan_parquet(PlRefPath::new(ruta_data.as_str()), args)?;

    let df: DataFrame;
    let schema = lz
        .clone()
        .limit(0)
        .collect()?
        .schema()
        .get("open")
        .is_some();

    if schema {
        df = lz
            .group_by_dynamic(
                col("timestamp"),
                [],
                DynamicGroupOptions {
                    every: Duration::parse(&timeframe),
                    period: Duration::parse(&timeframe),
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
            .collect()?;
    } else {
        df = lz
            .select([
                col("timestamp"),
                col("bidPrice").alias("open"),
                col("bidPrice").alias("high"),
                col("bidPrice").alias("low"),
                col("bidPrice").alias("close"),
                (col("bidVolume") + col("askVolume")).alias("volume"),
            ])
            .group_by_dynamic(
                col("timestamp"),
                [],
                DynamicGroupOptions {
                    every: Duration::parse(&timeframe),
                    period: Duration::parse(&timeframe),
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
            .collect()?;
    }

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
pub fn join_datasets(
    parquet_ruta: &str,
    data_to_join: &Vec<DataFormat>,
) -> PolarsResult<DataFrame> {
    let json_str = serde_json::to_string(data_to_join).unwrap();

    let df_csv = JsonReader::new(std::io::Cursor::new(json_str))
        .with_json_format(JsonFormat::JsonLines)
        .finish()?;

    let ruta: String = parquet_ruta.to_string();
    let args = ScanArgsParquet::default();
    let lz: LazyFrame = LazyFrame::scan_parquet(PlRefPath::new(&ruta), args)?;

    let new_data = df_csv.lazy().join(
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

    Ok(final_result.collect()?)
}

/// Une los nuevos datos descargados al dataset ya exsitente para actualizarlos.
pub fn join_datasets_ticks(
    parquet_ruta: &str,
    data_to_join: &Vec<DataFormatTicks>,
) -> PolarsResult<DataFrame> {
    let json_str = serde_json::to_string(data_to_join).unwrap();

    let df_csv = JsonReader::new(std::io::Cursor::new(json_str))
        .with_json_format(JsonFormat::JsonLines)
        .finish()?;

    let ruta: String = parquet_ruta.to_string();
    let args = ScanArgsParquet::default();
    let lz: LazyFrame = LazyFrame::scan_parquet(PlRefPath::new(&ruta), args)?;

    let new_data = df_csv.lazy().join(
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

    Ok(final_result.collect()?)
}
