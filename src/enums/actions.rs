use std::fmt;

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
    ExitBuy,
    ExitSell,
    Nbars,
    CloseAllRules,
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Action {
    pub fn as_str(&self) -> &str {
        match self {
            Action::Buy => "Buy",
            Action::Sell => "Sell",
            Action::BuyLimit => "Buy Limit",
            Action::SellLimit => "Sell Limit",
            Action::BuyStop => "Buy Stop",
            Action::SellStop => "Sell Stop",
            Action::Close => "Close",
            Action::ExitBuy => "Exit Buy",
            Action::ExitSell => "Exit Sell",
            Action::Nbars => "Nbars",
            Action::CloseAllRules => "Close All Rules",
        }
    }
}
