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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::data_format::DataFormatSymbol;
    use crate::enums::data_origen::DataOrigen;
    use crate::enums::timeframe::Timeframe;
    use crate::utils::configuracion::Error;
    use crate::utils::data_test::create_test_data;
    use std::fs::remove_file;

    //Para los test crear una carpeta llamada download en la raiz de este proyecto
    // y llamar a los datos test.csv
    fn load_data() -> PolarsResult<DataFrame> {
        create_test_data();

        let df = CsvReadOptions::default()
            .try_into_reader_with_file_path(Some("download/test.csv".into()))
            .unwrap()
            .finish()
            .unwrap();
        Ok(df)
    }

    #[test]
    fn add_data_test_csv() -> Result<(), Error> {
        match load_data() {
            Ok(df) => {
                let data_info: DataSymbol = DataSymbol {
                    id: 0,
                    name: "Test_csv".to_string(),
                    timeframe: Some(Timeframe::H1),
                    ruta: "download".to_string(),
                    formato: Some(DataFormatSymbol::Csv),
                    fecha_inicio: "00/00/0000".to_string(),
                    fecha_fin: "00/00/0000".to_string(),
                    actualizado: false,
                    n_data: 100,
                    origen: Some(DataOrigen::DukasCopy),
                };

                match add_data(df, &data_info) {
                    Ok(_) => {
                        remove_file("download/test.csv").unwrap();
                        remove_file("download/Test_csv.csv").unwrap();
                        return Ok(());
                    }
                    Err(_) => {
                        remove_file("download/test.csv").unwrap();
                        return Err(Error {
                            msg: "No se ha podido añadir los datos en el test".to_string(),
                        });
                    }
                }
            }
            Err(_) => {
                remove_file("download/test.csv").unwrap();
                return Err(Error {
                    msg: "No se ha podido cargar el dataframe para el test".to_string(),
                });
            }
        }
    }

    #[test]
    fn add_data_test_json() -> Result<(), Error> {
        match load_data() {
            Ok(df) => {
                let data_info: DataSymbol = DataSymbol {
                    id: 0,
                    name: "Test_json".to_string(),
                    timeframe: Some(Timeframe::H1),
                    ruta: "download".to_string(),
                    formato: Some(DataFormatSymbol::Json),
                    fecha_inicio: "00/00/0000".to_string(),
                    fecha_fin: "00/00/0000".to_string(),
                    actualizado: false,
                    n_data: 100,
                    origen: Some(DataOrigen::DukasCopy),
                };

                match add_data(df, &data_info) {
                    Ok(_) => {
                        remove_file("download/test.csv").unwrap();
                        remove_file("download/Test_json.json").unwrap();
                        return Ok(());
                    }
                    Err(_) => {
                        remove_file("download/test.csv").unwrap();
                        return Err(Error {
                            msg: "No se ha podido añadir los datos en el test".to_string(),
                        });
                    }
                }
            }
            Err(_) => {
                remove_file("download/test.csv").unwrap();
                return Err(Error {
                    msg: "No se ha podido cargar el dataframe para el test".to_string(),
                });
            }
        }
    }

    #[test]
    fn add_data_test_parquet() -> Result<(), Error> {
        match load_data() {
            Ok(df) => {
                let data_info: DataSymbol = DataSymbol {
                    id: 0,
                    name: "Test_parquet".to_string(),
                    timeframe: Some(Timeframe::H1),
                    ruta: "download".to_string(),
                    formato: Some(DataFormatSymbol::Parquet),
                    fecha_inicio: "00/00/0000".to_string(),
                    fecha_fin: "00/00/0000".to_string(),
                    actualizado: false,
                    n_data: 100,
                    origen: Some(DataOrigen::DukasCopy),
                };

                match add_data(df, &data_info) {
                    Ok(_) => {
                        remove_file("download/test.csv").unwrap();
                        remove_file("download/Test_parquet.parquet").unwrap();
                        return Ok(());
                    }
                    Err(_) => {
                        remove_file("download/test.csv").unwrap();
                        return Err(Error {
                            msg: "No se ha podido añadir los datos en el test".to_string(),
                        });
                    }
                }
            }
            Err(_) => {
                remove_file("download/test.csv").unwrap();
                return Err(Error {
                    msg: "No se ha podido cargar el dataframe para el test".to_string(),
                });
            }
        }
    }
}
