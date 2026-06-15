use crate::structs::data::{DataFormat, DataFormatTicks};

use polars::prelude::*;
use std::fs::File;

/// Guarda los datos de los activos descargados en determinado timeframe en un archivo parquet.
///
/// # Argments:
/// data: El array de datos del activo descargado.
/// name: Nombre con el que se guarda el archivo.
/// ruta_dist: Ruta donde se guardará el archivo. Por defecto en la carpeta download.
///
/// # Return
/// Delvuelve un string indicando que el archivo parquet se ha creado con exito.
pub fn add_data(
    data: &Vec<DataFormat>,
    name: &str,
    ruta_dist: Option<&str>,
) -> PolarsResult<String> {
    let ruta = ruta_dist.unwrap_or("download");
    let parquet_path = format!("{}/{}.parquet", ruta, name);

    let json_str = serde_json::to_string(data).unwrap();
    let mut df = JsonReader::new(std::io::Cursor::new(json_str))
        .with_json_format(JsonFormat::JsonLines)
        .finish()?;

    let mut file = File::create(parquet_path)?;
    let _ = ParquetWriter::new(&mut file).finish(&mut df)?;

    Ok("Parquet creado exitosamente".to_string())
}

/// Guarda los datos de los activos descargados en ticks en un archivo parquet.
///
/// # Argments:
/// data: El array de datos del activo descargado.
/// name: Nombre con el que se guarda el archivo.
/// ruta_dist: Ruta donde se guardará el archivo. Por defecto en la carpeta download.
///
/// # Return
/// Delvuelve un string indicando que el archivo parquet se ha creado con exito.
pub fn add_data_ticks(
    data: &Vec<DataFormatTicks>,
    name: &str,
    ruta_dist: Option<&str>,
) -> PolarsResult<String> {
    let ruta = ruta_dist.unwrap_or("download");
    let parquet_path = format!("{}/{}_Ticks.parquet", ruta, name);

    let json_str = serde_json::to_string(data).unwrap();
    let mut df = JsonReader::new(std::io::Cursor::new(json_str))
        .with_json_format(JsonFormat::JsonLines)
        .finish()?;

    let mut file = File::create(parquet_path)?;
    let _ = ParquetWriter::new(&mut file).finish(&mut df)?;

    Ok("Parquet creado exitosamente".to_string())
}
