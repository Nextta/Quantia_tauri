use crate::traits::tparametro::TParametro;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NBarsOptions {
    pub valor: f64,
}

impl TParametro for NBarsOptions {
    fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}
