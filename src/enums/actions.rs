use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum Action {
    Buy,
    Sell,
    BuyLimit,
    SellLimit,
    BuyStop,
    SellStop,
    Close,
}

impl Action {
    pub fn to_string(&self) -> String {
        match self {
            Action::Buy => "Buy".to_string(),
            Action::Sell => "Sell".to_string(),
            Action::BuyLimit => "Buy Limit".to_string(),
            Action::SellLimit => "Sell Limit".to_string(),
            Action::BuyStop => "Buy Stop".to_string(),
            Action::SellStop => "Sell Stop".to_string(),
            Action::Close => "Close".to_string(),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Action::Buy => "Buy",
            Action::Sell => "Sell",
            Action::BuyLimit => "Buy Limit",
            Action::SellLimit => "Sell Limit",
            Action::BuyStop => "Buy Stop",
            Action::SellStop => "Sell Stop",
            Action::Close => "Close",
        }
    }
}
