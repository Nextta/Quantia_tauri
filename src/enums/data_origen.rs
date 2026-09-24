use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum DataOrigen {
    DukasCopy,
    MT5,
    Import,
}

impl fmt::Display for DataOrigen {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl DataOrigen {
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
