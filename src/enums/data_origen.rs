use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum DataOrigen {
    DukasCopy,
    MT5,
    Import,
}

impl DataOrigen {
    pub fn to_string(&self) -> String {
        match self {
            DataOrigen::DukasCopy => "DukasCopy".to_string(),
            DataOrigen::MT5 => "MT5".to_string(),
            DataOrigen::Import => "Import".to_string(),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            DataOrigen::DukasCopy => "DukasCopy",
            DataOrigen::MT5 => "MT5",
            DataOrigen::Import => "Import",
        }
    }

    pub fn as_do(data_origen: &str) -> Option<DataOrigen> {
        match data_origen {
            "DukasCopy" => Some(DataOrigen::DukasCopy),
            "MT5" => Some(DataOrigen::MT5),
            _ => Some(DataOrigen::Import),
        }
    }
}
