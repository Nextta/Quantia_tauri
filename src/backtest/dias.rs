use serde::{Deserialize, Serialize};

#[derive(PartialEq, Clone, Debug, Deserialize, Serialize)]
pub enum Dias {
    Lu,
    Ma,
    Mi,
    Ju,
    Vi,
    Sa,
    Do,
}

impl Dias {
    pub fn to_string(&self) -> String {
        match self {
            Dias::Lu => "Lu".to_string(),
            Dias::Ma => "Ma".to_string(),
            Dias::Mi => "Mi".to_string(),
            Dias::Ju => "Ju".to_string(),
            Dias::Vi => "Vi".to_string(),
            Dias::Sa => "Sa".to_string(),
            Dias::Do => "Do".to_string(),
        }
    }
}
