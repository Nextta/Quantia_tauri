use polars::prelude::*;
use std::fs::File;

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
