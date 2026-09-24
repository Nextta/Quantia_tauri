use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(PartialEq, Clone, Copy, Debug, Deserialize, Serialize)]
pub enum Dias {
    Lu,
    Ma,
    Mi,
    Ju,
    Vi,
    Sa,
    Do,
}

impl fmt::Display for Dias {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Dias::Lu => "Lu",
            Dias::Ma => "Ma",
            Dias::Mi => "Mi",
            Dias::Ju => "Ju",
            Dias::Vi => "Vi",
            Dias::Sa => "Sa",
            Dias::Do => "Do",
        };
        write!(f, "{}", s)
    }
}
