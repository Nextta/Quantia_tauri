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

impl BeTipo {
    pub fn to_string(&self) -> &str {
        match self {
            BeTipo::Tick => "Tick",
            BeTipo::Pip => "Pip",
            BeTipo::Punto => "Punto",
            BeTipo::Porcentaje => "Porcentaje",
            BeTipo::PrecioEntrada => "PrecioEntrada",
            BeTipo::Indicador => "Indicador",
        }
    }

    pub fn as_str(&self) -> &str {
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
    Velas,
}

impl TlTipo {
    pub fn to_string(&self) -> &str {
        match self {
            TlTipo::Tick => "Tick",
            TlTipo::Pip => "Pip",
            TlTipo::Punto => "Punto",
            TlTipo::Porcentaje => "Porcentaje",
            TlTipo::Indicador => "Indicador",
            TlTipo::Velas => "Velas",
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            TlTipo::Tick => "Tick",
            TlTipo::Pip => "Pip",
            TlTipo::Punto => "Punto",
            TlTipo::Porcentaje => "Porcentaje",
            TlTipo::Indicador => "Indicador",
            TlTipo::Velas => "Velas",
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

impl ItTipo {
    pub fn to_string(&self) -> &str {
        match self {
            ItTipo::Open => "Open",
            ItTipo::Close => "Close",
            ItTipo::High => "High",
            ItTipo::Low => "Low",
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            ItTipo::Open => "Open",
            ItTipo::Close => "Close",
            ItTipo::High => "High",
            ItTipo::Low => "Low",
        }
    }
}
