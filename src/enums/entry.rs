use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum EntryDirection {
    Buy,
    Sell,
}

impl fmt::Display for EntryDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl EntryDirection {
    pub fn as_str(&self) -> &str {
        match self {
            EntryDirection::Buy => "Buy",
            EntryDirection::Sell => "Sell",
        }
    }
}
