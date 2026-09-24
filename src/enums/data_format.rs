use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum DataFormatSymbol {
    Parquet,
    Csv,
    Json,
}

impl fmt::Display for DataFormatSymbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl DataFormatSymbol {
    pub fn as_str(&self) -> &str {
        match self {
            DataFormatSymbol::Parquet => "parquet",
            DataFormatSymbol::Csv => "csv",
            DataFormatSymbol::Json => "json",
        }
    }

    pub fn as_tf(data_format: &str) -> Option<DataFormatSymbol> {
        match data_format {
            "parquet" => Some(DataFormatSymbol::Parquet),
            "csv" => Some(DataFormatSymbol::Csv),
            "json" => Some(DataFormatSymbol::Json),
            _ => None,
        }
    }
}
