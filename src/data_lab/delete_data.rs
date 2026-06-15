use polars::prelude::*;
use std::fs::remove_file;

pub fn delete_parquet(parquet_name: &str, ruta_dist: Option<&str>) -> PolarsResult<String> {
    let parquet_path = format!(
        "{}/{}.parquet",
        ruta_dist.unwrap_or("download"),
        parquet_name
    );

    remove_file(parquet_path)?;

    Ok("CSV eliminado exitosamente".to_string())
}
