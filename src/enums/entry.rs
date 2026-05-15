use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum EntryDirection {
    Buy,
    Sell,
}

impl EntryDirection {
    pub fn to_string(&self) -> String {
        match self {
            EntryDirection::Buy => "Buy".to_string(),
            EntryDirection::Sell => "Sell".to_string(),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            EntryDirection::Buy => "Buy",
            EntryDirection::Sell => "Sell",
        }
    }
}
