use crate::data_lab::utils_data::get_timeframe_data;
use crate::enums::data_format::DataFormatSymbol;
use crate::enums::timeframe::Timeframe;
use crate::structs::data::DataSymbol;
use polars::prelude::*;
use std::fs::File;

fn timeframe_to_duration(tf: &Timeframe) -> &'static str {
    match tf {
        Timeframe::M1 => "1m",
        Timeframe::M5 => "5m",
        Timeframe::M10 => "10m",
        Timeframe::M15 => "15m",
        Timeframe::M30 => "30m",
        Timeframe::H1 => "1h",
        Timeframe::H4 => "4h",
        Timeframe::D1 => "1d",
        Timeframe::W1 => "1w",
        Timeframe::MM1 => "1mo",
        Timeframe::Ticks => "1ms",
    }
}

pub fn export_data(data_info: &DataSymbol, ruta_export: &str) {
    match data_info.formato.unwrap() {
        DataFormatSymbol::Csv => {
            export_csv(
                &data_info.name,
                &data_info.ruta,
                ruta_export,
                timeframe_to_duration(&data_info.timeframe.unwrap()),
            )
            .unwrap();
        }
        DataFormatSymbol::Json => {
            export_json(
                &data_info.name,
                &data_info.ruta,
                ruta_export,
                timeframe_to_duration(&data_info.timeframe.unwrap()),
            )
            .unwrap();
        }
        DataFormatSymbol::Parquet => {
            export_parquet(
                &data_info.name,
                &data_info.ruta,
                ruta_export,
                timeframe_to_duration(&data_info.timeframe.unwrap()),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::data_format::DataFormatSymbol;
    use crate::enums::data_origen::DataOrigen;
    use crate::enums::timeframe::Timeframe;
    use crate::utils::configuracion::Error;
    use crate::utils::data_test::{create_test_data, create_test_data_ticks};
    use std::fs::remove_file;

    #[test]
    fn export_data_test() -> Result<(), Error> {
        create_test_data(&DataFormatSymbol::Parquet);
        create_test_data_ticks(&DataFormatSymbol::Parquet);

        let data_info_csv: DataSymbol = DataSymbol {
            id: 0,
            name: "test".to_string(),
            timeframe: Some(Timeframe::M1),
            ruta: "data".to_string(),
            formato: Some(DataFormatSymbol::Csv),
            fecha_inicio: "00/00/0000".to_string(),
            fecha_fin: "00/00/0000".to_string(),
            actualizado: false,
            n_data: 100,
            origen: Some(DataOrigen::DukasCopy),
        };

        export_data(&data_info_csv, "data");

        let data_info_csv_tick: DataSymbol = DataSymbol {
            id: 0,
            name: "test_ticks".to_string(),
            timeframe: Some(Timeframe::Ticks),
            ruta: "data".to_string(),
            formato: Some(DataFormatSymbol::Csv),
            fecha_inicio: "00/00/0000".to_string(),
            fecha_fin: "00/00/0000".to_string(),
            actualizado: false,
            n_data: 100,
            origen: Some(DataOrigen::DukasCopy),
        };

        export_data(&data_info_csv_tick, "data");

        let data_info_csv_tf_change: DataSymbol = DataSymbol {
            id: 0,
            name: "test".to_string(),
            timeframe: Some(Timeframe::M15),
            ruta: "data".to_string(),
            formato: Some(DataFormatSymbol::Csv),
            fecha_inicio: "00/00/0000".to_string(),
            fecha_fin: "00/00/0000".to_string(),
            actualizado: false,
            n_data: 100,
            origen: Some(DataOrigen::DukasCopy),
        };

        export_data(&data_info_csv_tf_change, "data");

        remove_file("data/test.csv")?;
        remove_file("data/test_ticks.csv")?;
        remove_file("data/test.parquet")?;
        remove_file("data/test_ticks.parquet")?;
        Ok(())
    }
}
