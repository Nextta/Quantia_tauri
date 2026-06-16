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
            DataFormatSymbol::Parquet => "Parquet".to_string(),
            DataFormatSymbol::Csv => "Csv".to_string(),
            DataFormatSymbol::Json => "Json".to_string(),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            DataFormatSymbol::Parquet => "Parquet",
            DataFormatSymbol::Csv => "Csv",
            DataFormatSymbol::Json => "Json",
        }
    }

    pub fn as_tf(data_format: &str) -> Option<DataFormatSymbol> {
        match data_format {
            "Parquet" => Some(DataFormatSymbol::Parquet),
            "Csv" => Some(DataFormatSymbol::Csv),
            "Json" => Some(DataFormatSymbol::Json),
            _ => None,
        }
    }
}
