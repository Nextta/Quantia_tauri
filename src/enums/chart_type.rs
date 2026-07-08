use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum ChartType {
    Inchart,
    Subchart,
}

impl ChartType {
    pub fn to_string(&self) -> String {
        match self {
            ChartType::Inchart => "Inchart".to_string(),
            ChartType::Subchart => "Subchart".to_string(),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            ChartType::Inchart => "Inchart",
            ChartType::Subchart => "Subchart",
        }
    }

    pub fn as_ct(data_format: &str) -> ChartType {
        match data_format {
            "Inchart" => ChartType::Inchart,
            "Subchart" => ChartType::Subchart,
            _ => ChartType::Subchart,
        }
    }
}
