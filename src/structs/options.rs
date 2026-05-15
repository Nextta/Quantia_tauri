use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NBarsOptions {
    pub valor: f64,
}

impl NBarsOptions {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}
