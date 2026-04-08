use crate::strategy::strategy_action::StrategyAction;
use crate::strategy::strategy_condition::StrategyCondition;
use crate::strategy::strategy_indicator::StrategyIndicator;
use crate::strategy::strategy_options::StrategyOptions;

#[derive(Debug, Clone)]
pub struct Strategy {
    pub id: i32,
    pub id_user: i32,
    pub nombre: String,
    pub descripcion: Option<String>,
    pub activa: bool,
    pub creada_en: String,
    pub indicadores: Vec<StrategyIndicator>,
    pub condiciones: Vec<StrategyCondition>,
    pub acciones: Vec<StrategyAction>,
    pub opciones: StrategyOptions,
}

impl Strategy {
    pub fn new(
        id: i32,
        id_user: i32,
        nombre: String,
        descripcion: Option<String>,
        activa: bool,
        creada_en: String,
        opciones: StrategyOptions,
    ) -> Self {
        Self {
            id,
            id_user,
            nombre,
            descripcion,
            activa,
            creada_en,
            indicadores: Vec::<StrategyIndicator>::new(),
            condiciones: Vec::<StrategyCondition>::new(),
            acciones: Vec::<StrategyAction>::new(),
            opciones,
        }
    }

    pub fn new_empty() -> Self {
        Self {
            id: 0,
            id_user: 0,
            nombre: String::new(),
            descripcion: None,
            activa: false,
            creada_en: String::new(),
            indicadores: Vec::<StrategyIndicator>::new(),
            condiciones: Vec::<StrategyCondition>::new(),
            acciones: Vec::<StrategyAction>::new(),
            opciones: StrategyOptions::new_empty(),
        }
    }
}
