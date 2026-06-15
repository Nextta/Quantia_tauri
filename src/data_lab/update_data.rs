use crate::data_lab::utils_data::{join_datasets, join_datasets_ticks};
use crate::structs::data::{DataFormat, DataFormatTicks};
use polars::prelude::*;
use std::fs::File;

/// Actualiza los datos descargados de los archivos parquet
pub fn update_data(
    data: Vec<DataFormat>,
    name: &str,
    ruta_data: Option<&str>,
) -> PolarsResult<String> {
    let ruta: String = format!("{}/{}.parquet", ruta_data.unwrap_or("download"), name);

    let mut df_result = join_datasets(&ruta, &data)?;

    let mut file = File::create(ruta)?;
    let _ = ParquetWriter::new(&mut file).finish(&mut df_result)?;

    Ok("Datos actualizados con éxito".to_string())
}

/// Actualiza los datos descargados de los archivos parquet
pub fn update_data_ticks(
    data: Vec<DataFormatTicks>,
    name: &str,
    ruta_data: Option<&str>,
) -> PolarsResult<String> {
    let ruta: String = format!("{}/{}.parquet", ruta_data.unwrap_or("download"), name);

    let mut df_result = join_datasets_ticks(&ruta, &data)?;

    let mut file = File::create(ruta)?;
    let _ = ParquetWriter::new(&mut file).finish(&mut df_result)?;

    Ok("Datos actualizados con éxito".to_string())
}
