use crate::structs::data::DataSymbol;
use polars::prelude::*;
use std::fs::remove_file;

pub fn delete_data_local(data_info: &DataSymbol) -> PolarsResult<String> {
    let parquet_path = format!(
        "{}/{}.{}",
        data_info.ruta.clone(),
        data_info.name.clone(),
        data_info.formato.unwrap().to_string()
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

    //Para los test crear una carpeta llamada download en la raiz de este proyecto
    // y llamar a los datos test_delete.csv para hacer la prueba.

    #[test]
    fn delete_data_local_test() -> Result<(), Error> {
        let data_info: DataSymbol = DataSymbol {
            id: 0,
            name: "test_delete".to_string(),
            timeframe: Some(Timeframe::H1),
            ruta: "download".to_string(),
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
