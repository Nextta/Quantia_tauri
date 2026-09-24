use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum GestionStrategy {
    Formula,
    Fijo,
    Kelly,
    PocertajeEquity,
    PorcentajeBalance,
}

impl fmt::Display for GestionStrategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            GestionStrategy::Formula => "Formula",
            GestionStrategy::Fijo => "Fijo",
            GestionStrategy::Kelly => "Kelly",
            GestionStrategy::PocertajeEquity => "PocertajeEquity",
            GestionStrategy::PorcentajeBalance => "PorcentajeBalance",
        };
        write!(f, "{}", s)
    }
}
