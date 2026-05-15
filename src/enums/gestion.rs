#[derive(Debug, Clone)]
pub enum GestionStrategy {
    Formula,
    Fijo,
    Kelly,
    PocertajeEquity,
    PorcentajeBalance,
}

impl ToString for GestionStrategy {
    fn to_string(&self) -> String {
        match self {
            GestionStrategy::Formula => "Formula".to_string(),
            GestionStrategy::Fijo => "Fijo".to_string(),
            GestionStrategy::Kelly => "Kelly".to_string(),
            GestionStrategy::PocertajeEquity => "PocertajeEquity".to_string(),
            GestionStrategy::PorcentajeBalance => "PorcentajeBalance".to_string(),
        }
    }
}
