use crate::enums::data_format::DataFormatSymbol;
use crate::structs::data::DataSymbol;
use polars::prelude::*;
use std::fs::File;

pub fn import_data(data_info: &DataSymbol, ruta_import: &str) {
    match data_info.formato.unwrap() {
        DataFormatSymbol::Csv => {
            import_csv(&data_info.name, &data_info.ruta, ruta_import).unwrap();
        }
        DataFormatSymbol::Json => {
            import_json(&data_info.name, &data_info.ruta, ruta_import).unwrap();
        }
        DataFormatSymbol::Parquet => {
            import_parquet(&data_info.name, &data_info.ruta, ruta_import).unwrap();
        }
    }
}

/// Importar en formato csv
pub fn import_csv(symbol_name: &str, ruta_data: &str, ruta_dist: &str) -> PolarsResult<String> {
    let mut df: DataFrame = CsvReadOptions::default()
        .try_into_reader_with_file_path(Some(ruta_data.into()))?
        .finish()?;

    let ruta: String = format!("{}/{}.parquet", ruta_dist, symbol_name);
    let mut file: File = File::create(ruta)?;
    let _ = ParquetWriter::new(&mut file).finish(&mut df)?;

    Ok("CSV importado exitosamente".to_string())
}

/// Importar formato Json
pub fn import_json(symbol_name: &str, ruta_data: &str, ruta_dist: &str) -> PolarsResult<String> {
    let mut file_read: File = File::open(ruta_data)?;
    let mut df = JsonReader::new(&mut file_read).finish()?;

    let ruta: String = format!("{}/{}.parquet", ruta_dist, symbol_name);
    let mut file: File = File::create(ruta)?;
    let _ = ParquetWriter::new(&mut file).finish(&mut df)?;

    Ok("JSON importado exitosamente".to_string())
}

/// Importar formato parquet
pub fn import_parquet(symbol_name: &str, ruta_data: &str, ruta_dist: &str) -> PolarsResult<String> {
    let mut file_read: File = File::open(ruta_data)?;
    let mut df: DataFrame = ParquetReader::new(&mut file_read).finish()?;

    let ruta: String = format!("{}/{}.parquet", ruta_dist, symbol_name);
    let mut file: File = File::create(ruta)?;
    let _ = ParquetWriter::new(&mut file).finish(&mut df)?;

    Ok("Parquet importado exitosamente".to_string())
}
