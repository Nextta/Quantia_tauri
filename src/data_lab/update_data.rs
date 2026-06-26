use crate::data_lab::utils_data::{join_datasets, join_datasets_ticks};
use crate::structs::data::DataSymbol;
use polars::prelude::*;
use std::fs::File;

/// Actualiza los datos descargados de los archivos parquet
pub fn update_data(data: &DataFrame, data_info: &DataSymbol) -> PolarsResult<String> {
    let ruta: String = format!("{}/{}.parquet", &data_info.ruta, &data_info.name);

    let mut df_result = join_datasets(&data_info.ruta, data.clone())?;

    let mut file = File::create(ruta)?;
    let _ = ParquetWriter::new(&mut file).finish(&mut df_result)?;

    Ok("Datos actualizados con éxito".to_string())
}

/// Actualiza los datos descargados de los archivos parquet
pub fn update_data_ticks(data: &DataFrame, data_info: &DataSymbol) -> PolarsResult<String> {
    let ruta: String = format!("{}/{}.parquet", &data_info.ruta, &data_info.name);

    let mut df_result = join_datasets_ticks(&data_info.ruta, data.clone())?;

    let mut file = File::create(ruta)?;
    let _ = ParquetWriter::new(&mut file).finish(&mut df_result)?;

    Ok("Datos actualizados con éxito".to_string())
}
