use crate::traits::ttipos::TTipos;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub enum Activo {
    Forex,
    Futuros,
    CDF,
    Acciones,
    ETF,
    Opciones,
}

impl TTipos for Activo {
    fn to_string(&self) -> String {
        match self {
            Activo::Forex => "Forex".to_string(),
            Activo::Futuros => "Futuros".to_string(),
            Activo::CDF => "CDF".to_string(),
            Activo::Acciones => "Acciones".to_string(),
            Activo::ETF => "ETF".to_string(),
            Activo::Opciones => "Opciones".to_string(),
        }
    }
    fn as_str(&self) -> &str {
        match self {
            Activo::Forex => "Forex",
            Activo::Futuros => "Futuros",
            Activo::CDF => "CDF",
            Activo::Acciones => "Acciones",
            Activo::ETF => "ETF",
            Activo::Opciones => "Opciones",
        }
    }
}
