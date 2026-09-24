use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum Timeframe {
    Ticks,
    M1,
    M5,
    M10,
    M15,
    M30,
    H1,
    H4,
    D1,
    W1,
    MM1,
}

impl fmt::Display for Timeframe {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Timeframe {
    pub fn as_str(&self) -> &str {
        match self {
            Timeframe::M1 => "M1",
            Timeframe::M5 => "M5",
            Timeframe::M10 => "M10",
            Timeframe::M15 => "M15",
            Timeframe::M30 => "M30",
            Timeframe::H1 => "H1",
            Timeframe::H4 => "H4",
            Timeframe::D1 => "D1",
            Timeframe::W1 => "W1",
            Timeframe::MM1 => "MM1",
            Timeframe::Ticks => "Ticks",
        }
    }

    pub fn as_tf(tf: &str) -> Option<Timeframe> {
        match tf {
            "M1" => Some(Timeframe::M1),
            "M5" => Some(Timeframe::M5),
            "M10" => Some(Timeframe::M10),
            "M15" => Some(Timeframe::M15),
            "M30" => Some(Timeframe::M30),
            "H1" => Some(Timeframe::H1),
            "H4" => Some(Timeframe::H4),
            "D1" => Some(Timeframe::D1),
            "W1" => Some(Timeframe::W1),
            "MM1" => Some(Timeframe::MM1),
            "Ticks" => Some(Timeframe::Ticks),
            _ => None,
        }
    }
}
