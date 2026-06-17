use crate::enums::data_format::DataFormatSymbol;
use crate::enums::data_origen::DataOrigen;
use crate::enums::timeframe::Timeframe;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DataFormat {
    pub time: u32,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub volume: f64,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DataFormatTicks {
    pub time: u32,
    pub askPrice: f64,
    pub bidPrice: f64,
    pub askVolume: f64,
    pub bidVolume: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DataSymbol {
    pub id: u32,
    pub name: String,
    pub timeframe: Option<Timeframe>,
    pub ruta: String,
    pub formato: Option<DataFormatSymbol>,
    pub fecha_inicio: String,
    pub fecha_fin: String,
    pub actualizado: bool,
    pub n_data: u32,
    pub origen: Option<DataOrigen>,
}
