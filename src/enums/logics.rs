use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum Logic {
    AND,
    OR,
}

impl fmt::Display for Logic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Logic {
    pub fn as_str(&self) -> &str {
        match self {
            Logic::AND => "AND",
            Logic::OR => "OR",
        }
    }
}
