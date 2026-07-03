use crate::enums::entry::EntryDirection;
use crate::enums::tipos::{BeTipo, TlTipo};
use crate::traits::tparametro::TParametro;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BeParams {
    pub tipo: BeTipo,
    pub valor: f64,
    pub be_plus: f64,
    pub col_name: Option<String>,
}

impl TParametro for BeParams {
    fn to_json(&self) -> String {
        let json = serde_json::to_string(self).unwrap();
        json
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TlParams {
    pub tipo: TlTipo,
    pub valor: f64,
    pub activacion_tipo: TlTipo,
    pub activacion_valor: f64,
    pub columna_nombre: String,
}

impl TParametro for TlParams {
    fn to_json(&self) -> String {
        let json = serde_json::to_string(self).unwrap();
        json
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct GestionParams {
    #[serde(default = "default_multiplicador")]
    pub multiplicador: f64,
    #[serde(default = "default_lotaje")]
    pub lotaje_fijo: f64,
}

fn default_multiplicador() -> f64 {
    1.0
}

fn default_lotaje() -> f64 {
    0.01
}

impl TParametro for GestionParams {
    fn to_json(&self) -> String {
        let json = serde_json::to_string(self).unwrap();
        json
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LimitParams {
    pub tipo: String,              // Tipo de limite: ask, bid, bb, atr... etc
    pub direccion: EntryDirection, // Direccion del limite: buy, sell
    pub nombre_col: String,        // Nombre de la columna a usar como limite
    pub shift: usize,              // Numero de filas a desplazar
    pub valor: f64,                // en caso de ser por pip, ticks o puntos
}

impl TParametro for LimitParams {
    fn to_json(&self) -> String {
        let json = serde_json::to_string(self).unwrap();
        json
    }
}
