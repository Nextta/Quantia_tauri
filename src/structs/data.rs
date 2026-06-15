use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DataFormat {
    time: u32,
    open: f64,
    high: f64,
    low: f64,
    volume: f64,
}
