use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum DataFormatSymbol {
    Parquet,
    Csv,
    Json,
}

impl DataFormatSymbol {
    pub fn to_string(&self) -> String {
        match self {
            DataFormatSymbol::Parquet => "parquet".to_string(),
            DataFormatSymbol::Csv => "csv".to_string(),
            DataFormatSymbol::Json => "json".to_string(),
        }
    }

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
