use crate::enums::tipos::{BeTipo, ItTipo, TlTipo};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BeParams {
    pub tipo: BeTipo,
    pub valor: f64,
    pub be_plus: f64,
    pub col_name: Option<String>,
}

impl BeParams {
    pub fn to_json(&self) -> String {
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
    pub indicador_nombre: String,
    pub columna_nombre: String,
    pub indicador_tipo: ItTipo,
}

impl TlParams {
    pub fn to_json(&self) -> String {
        let json = serde_json::to_string(self).unwrap();
        json
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GestionParams {
    pub multiplicador: f64,
    pub lotaje_fijo: f64,
}

impl GestionParams {
    pub fn to_json(&self) -> String {
        let json = serde_json::to_string(self).unwrap();
        json
    }
}
