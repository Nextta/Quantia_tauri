use crate::data_lab::utils_data::get_timeframe_data;
use crate::enums::data_format::DataFormatSymbol;
use crate::structs::data::DataSymbol;
use polars::prelude::*;
use std::fs::File;

pub fn export_data(data_info: &DataSymbol, ruta_export: &str) {
    match data_info.formato.unwrap() {
        DataFormatSymbol::Csv => {
            export_csv(
                &data_info.name,
                &data_info.ruta,
                ruta_export,
                &data_info.timeframe.unwrap().as_str(),
            )
            .unwrap();
        }
        DataFormatSymbol::Json => {
            export_json(
                &data_info.name,
                &data_info.ruta,
                ruta_export,
                &data_info.timeframe.unwrap().as_str(),
            )
            .unwrap();
        }
        DataFormatSymbol::Parquet => {
            export_parquet(
                &data_info.name,
                &data_info.ruta,
                ruta_export,
                &data_info.timeframe.unwrap().as_str(),
            )
            .unwrap();
        }
    }
}

/// Exportar en formato csv
pub fn export_csv(
    data_name: &str,
    ruta_dist: &str,
    ruta_export: &str,
    timeframe: &str,
) -> PolarsResult<String> {
    let mut df: DataFrame;
    if timeframe != "ticks" {
        df = get_timeframe_data(data_name, timeframe, ruta_dist)?;
    } else {
        let ruta_data: String = format!("{}/{}.parquet", ruta_dist, data_name);
        let mut file_parquet: File = File::open(ruta_data)?;
        df = ParquetReader::new(&mut file_parquet).finish()?;
    }

    let ruta_expt = format!("{}/{}.csv", ruta_export, data_name);
    let mut file = File::create(ruta_expt)?;
    CsvWriter::new(&mut file).finish(&mut df)?;

    Ok("CSV exportado exitosamente".to_string())
}

/// Exportar en formato json
pub fn export_json(
    data_name: &str,
    ruta_dist: &str,
    ruta_export: &str,
    timeframe: &str,
) -> PolarsResult<String> {
    let mut df: DataFrame;
    if timeframe != "ticks" {
        df = get_timeframe_data(data_name, timeframe, ruta_dist)?;
    } else {
        let ruta_data: String = format!("{}/{}.parquet", ruta_dist, data_name);
        let mut file_parquet: File = File::open(ruta_data)?;
        df = ParquetReader::new(&mut file_parquet).finish()?;
    }

    let ruta_expt = format!("{}/{}.json", ruta_export, data_name);
    let mut file: File = File::create(ruta_expt)?;
    JsonWriter::new(&mut file)
        .with_json_format(JsonFormat::Json)
        .finish(&mut df)?;

    Ok("JSON exportado exitosamente".to_string())
}

/// Exportar en formato Parquet
pub fn export_parquet(
    data_name: &str,
    ruta_dist: &str,
    ruta_export: &str,
    timeframe: &str,
) -> PolarsResult<String> {
    let mut df: DataFrame;

    if timeframe != "ticks" {
        df = get_timeframe_data(data_name, timeframe, ruta_dist)?;
    } else {
        let ruta_data: String = format!("{}/{}.parquet", ruta_dist, data_name);
        let mut file_parquet: File = File::open(ruta_data)?;
        df = ParquetReader::new(&mut file_parquet).finish()?;
    }

    let ruta_expt = format!("{}/{}.parquet", ruta_export, data_name);
    let mut file: File = File::create(ruta_expt)?;
    let _ = ParquetWriter::new(&mut file).finish(&mut df)?;

    Ok("Parquet exportado exitosamente".to_string())
}
