use crate::enums::data_format::DataFormatSymbol;
use crate::structs::data::DataSymbol;

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
pub fn add_data(mut data: DataFrame, data_info: &DataSymbol) -> PolarsResult<String> {
    let ruta = data_info.ruta.clone();
    let formato = data_info.formato.unwrap();
    let parquet_path = format!(
        "{}/{}.{}",
        ruta,
        data_info.name.clone(),
        formato.to_string()
    );

    match formato {
        DataFormatSymbol::Parquet => {
            let mut file = std::fs::File::create(parquet_path).unwrap();
            ParquetWriter::new(&mut file).finish(&mut data).unwrap();
        }
        DataFormatSymbol::Csv => {
            let mut file = File::create(parquet_path)?;
            CsvWriter::new(&mut file).finish(&mut data)?;
        }
        DataFormatSymbol::Json => {
            let mut file = File::create(parquet_path)?;
            let _ = ParquetWriter::new(&mut file).finish(&mut data)?;
        }
    }

    Ok(format!("{} creado exitosamente", formato.to_string()))
}
