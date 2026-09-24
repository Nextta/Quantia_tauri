use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum ChartType {
    Inchart,
    Subchart,
}

impl fmt::Display for ChartType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl ChartType {
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
