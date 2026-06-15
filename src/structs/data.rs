use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DataFormat {
    time: u32,
    open: f64,
    high: f64,
    low: f64,
    volume: f64,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DataFormatTicks {
    time: u32,
    askPrice: f64,
    bidPrice: f64,
    askVolume: f64,
    bidVolume: f64,
}
