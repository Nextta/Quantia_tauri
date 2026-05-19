use crate::traits::ttipos::TTipos;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum BeTipo {
    Tick,
    Pip,
    Punto,
    Porcentaje,
    PrecioEntrada,
    Indicador,
}

impl TTipos for BeTipo {
    fn to_string(&self) -> String {
        match self {
            BeTipo::Tick => "Tick".to_string(),
            BeTipo::Pip => "Pip".to_string(),
            BeTipo::Punto => "Punto".to_string(),
            BeTipo::Porcentaje => "Porcentaje".to_string(),
            BeTipo::PrecioEntrada => "PrecioEntrada".to_string(),
            BeTipo::Indicador => "Indicador".to_string(),
        }
    }

    fn as_str(&self) -> &str {
        match self {
            BeTipo::Tick => "Tick",
            BeTipo::Pip => "Pip",
            BeTipo::Punto => "Punto",
            BeTipo::Porcentaje => "Porcentaje",
            BeTipo::PrecioEntrada => "PrecioEntrada",
            BeTipo::Indicador => "Indicador",
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum TlTipo {
    Tick,
    Pip,
    Punto,
    Porcentaje,
    Indicador,
    // Velas, Implementar quizas en el futuro si tiene sentido.
}

impl TTipos for TlTipo {
    fn to_string(&self) -> String {
        match self {
            TlTipo::Tick => "Tick".to_string(),
            TlTipo::Pip => "Pip".to_string(),
            TlTipo::Punto => "Punto".to_string(),
            TlTipo::Porcentaje => "Porcentaje".to_string(),
            TlTipo::Indicador => "Indicador".to_string(),
            // TlTipo::Velas => "Velas".to_string(),
        }
    }

    fn as_str(&self) -> &str {
        match self {
            TlTipo::Tick => "Tick",
            TlTipo::Pip => "Pip",
            TlTipo::Punto => "Punto",
            TlTipo::Porcentaje => "Porcentaje",
            TlTipo::Indicador => "Indicador",
            // TlTipo::Velas => "Velas",
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ItTipo {
    Open,
    Close,
    High,
    Low,
}

impl TTipos for ItTipo {
    fn to_string(&self) -> String {
        match self {
            ItTipo::Open => "Open".to_string(),
            ItTipo::Close => "Close".to_string(),
            ItTipo::High => "High".to_string(),
            ItTipo::Low => "Low".to_string(),
        }
    }

    fn as_str(&self) -> &str {
        match self {
            ItTipo::Open => "Open",
            ItTipo::Close => "Close",
            ItTipo::High => "High",
            ItTipo::Low => "Low",
        }
    }
}
