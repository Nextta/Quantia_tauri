use crate::data_lab::utils_data::{join_datasets, join_datasets_ticks};
use crate::structs::data::DataSymbol;
use polars::prelude::*;
use std::fs::File;

/// Actualiza los datos descargados de los archivos parquet
pub fn update_data(data: &DataFrame, data_info: &DataSymbol) -> PolarsResult<String> {
    let ruta: String = format!("{}/{}.parquet", &data_info.ruta, &data_info.name);

    let mut df_result = join_datasets(&ruta, data.clone())?;

    let mut file = File::create(ruta)?;
    let _ = ParquetWriter::new(&mut file).finish(&mut df_result)?;

    Ok("Datos actualizados con éxito".to_string())
}

/// Actualiza los datos descargados de los archivos parquet
pub fn update_data_ticks(data: &DataFrame, data_info: &DataSymbol) -> PolarsResult<String> {
    let ruta: String = format!("{}/{}.parquet", &data_info.ruta, &data_info.name);

    let mut df_result = join_datasets_ticks(&ruta, data.clone())?;

    let mut file = File::create(ruta)?;
    let _ = ParquetWriter::new(&mut file).finish(&mut df_result)?;

    Ok("Datos actualizados con éxito".to_string())
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

    fn load_data() -> PolarsResult<DataFrame> {
        create_test_data(&DataFormatSymbol::Csv);

        let df = CsvReadOptions::default()
            .try_into_reader_with_file_path(Some("data/test.csv".into()))
            .unwrap()
            .finish()
            .unwrap();
        Ok(df)
    }

    fn load_data_ticks() -> PolarsResult<DataFrame> {
        create_test_data_ticks(&DataFormatSymbol::Csv);

        let df = CsvReadOptions::default()
            .try_into_reader_with_file_path(Some("data/test_ticks.csv".into()))
            .unwrap()
            .finish()
            .unwrap();
        Ok(df)
    }

    #[test]
    fn update_data_test() -> Result<(), Error> {
        let _guardia = crate::utils::data_test::BLOQUEO_RECURSOS.blocking_lock();
        match load_data() {
            Ok(df) => {
                let data_info: DataSymbol = DataSymbol {
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

                match update_data(&df, &data_info) {
                    Ok(_) => {
                        remove_file("data/test.csv").unwrap();
                        remove_file("data/test.parquet").unwrap();
                        return Ok(());
                    }
                    Err(_) => {
                        remove_file("data/test.csv").unwrap();
                        remove_file("data/test.parquet").unwrap();
                        return Err(Error {
                            msg: "No se ha podido actualizar los datos".to_string(),
                        });
                    }
                }
            }
            Err(_) => {
                remove_file("data/test.csv").unwrap();
                remove_file("data/test.parquet").unwrap();
                return Err(Error {
                    msg: "No se ha podido cargar los datos".to_string(),
                });
            }
        }
    }

    #[test]
    fn update_data_ticks_test() -> Result<(), Error> {
        let _guardia = crate::utils::data_test::BLOQUEO_RECURSOS.blocking_lock();
        match load_data_ticks() {
            Ok(df) => {
                let data_info: DataSymbol = DataSymbol {
                    id: 0,
                    name: "test_ticks".to_string(),
                    timeframe: Some(Timeframe::M1),
                    ruta: "data".to_string(),
                    formato: Some(DataFormatSymbol::Csv),
                    fecha_inicio: "00/00/0000".to_string(),
                    fecha_fin: "00/00/0000".to_string(),
                    actualizado: false,
                    n_data: 100,
                    origen: Some(DataOrigen::DukasCopy),
                };

                match update_data_ticks(&df, &data_info) {
                    Ok(_) => {
                        remove_file("data/test_ticks.csv").unwrap();
                        remove_file("data/test_ticks.parquet").unwrap();
                        return Ok(());
                    }
                    Err(_) => {
                        remove_file("data/test_ticks.csv").unwrap();
                        remove_file("data/test_ticks.parquet").unwrap();
                        return Err(Error {
                            msg: "No se ha podido actualizar los datos".to_string(),
                        });
                    }
                }
            }
            Err(_) => {
                remove_file("data/test.csv").unwrap();
                remove_file("data/test.parquet").unwrap();
                return Err(Error {
                    msg: "No se ha podido cargar los datos".to_string(),
                });
            }
        }
    }
}
