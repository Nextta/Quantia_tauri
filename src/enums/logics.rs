use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum Logic {
    AND,
    OR,
}

impl Logic {
    pub fn to_string(&self) -> String {
        match self {
            Logic::AND => "AND".to_string(),
            Logic::OR => "OR".to_string(),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Logic::AND => "AND",
            Logic::OR => "OR",
        }
    }
}
