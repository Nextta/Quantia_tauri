use crate::structs::data::DataSymbol;
use polars::prelude::*;
use std::fs::remove_file;

pub fn delete_data(data_info: &DataSymbol) -> PolarsResult<String> {
    let parquet_path = format!(
        "{}/{}.{}",
        data_info.ruta.clone(),
        data_info.name.clone(),
        data_info.formato.unwrap().to_string()
    );

    remove_file(parquet_path)?;

    Ok("CSV eliminado exitosamente".to_string())
}
