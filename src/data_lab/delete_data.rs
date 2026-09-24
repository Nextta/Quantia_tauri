use crate::structs::data::DataSymbol;
use polars::prelude::*;
use std::fs::remove_file;

pub fn delete_data_local(data_info: &DataSymbol) -> PolarsResult<String> {
    let parquet_path = format!(
        "{}/{}.{}",
        data_info.ruta.clone(),
        data_info.name.clone(),
        data_info.formato.unwrap()
    );

    remove_file(parquet_path)?;

    Ok("CSV eliminado exitosamente".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::data_format::DataFormatSymbol;
    use crate::enums::data_origen::DataOrigen;
    use crate::enums::timeframe::Timeframe;
    use crate::utils::configuracion::Error;
    use std::fs::File;
    use std::path::Path;

    #[test]
    fn delete_data_local_test() -> Result<(), Error> {
        let _guardia = crate::utils::data_test::BLOQUEO_RECURSOS.blocking_lock();
        let path = "data/test_delete.csv";
        if !Path::new(path).exists() {
            File::create(path)?;
        }

        let data_info: DataSymbol = DataSymbol {
            id: 0,
            name: "test_delete".to_string(),
            timeframe: Some(Timeframe::H1),
            ruta: "data".to_string(),
            formato: Some(DataFormatSymbol::Csv),
            fecha_inicio: "00/00/0000".to_string(),
            fecha_fin: "00/00/0000".to_string(),
            actualizado: false,
            n_data: 100,
            origen: Some(DataOrigen::DukasCopy),
        };

        match delete_data_local(&data_info) {
            Ok(_) => return Ok(()),
            Err(_) => {
                return Err(Error {
                    msg: "No se ha podido eliminar el archivo.".to_string(),
                })
            }
        }
    }
}
