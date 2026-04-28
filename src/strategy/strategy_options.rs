use chrono::DateTime;
use chrono::Utc;

#[derive(Debug, Clone)]
pub struct StrategyOptions {
    pub id: i32,
    pub strategy_id: i32,
    pub multiples_tardes: bool,
    pub trading_direccion: TradingDirection,
    pub operar_finde: bool,
    pub cerrar_fin_de_dia: bool,
    pub hora_fin_de_dia: DateTime<Utc>,
    pub cerrar_viernes: bool,
    pub hora_cierre_viernes: DateTime<Utc>,
    pub rango_operativo: bool,
    pub rango_operativo_inicio: DateTime<Utc>,
    pub rango_operativo_fin: DateTime<Utc>,
    pub cerrar_fin_rango_operativo: bool,
    pub activar_cierre_numero_velas: bool,
    pub numero_velas_cierre: i32,
    pub cierre_limite_hora: bool,
    pub hora_cierre_limite: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TradingDirection {
    Long,
    Short,
    Both,
}

impl TradingDirection {
    pub fn to_string(&self) -> &str {
        match self {
            TradingDirection::Long => "Long",
            TradingDirection::Short => "Short",
            TradingDirection::Both => "Both",
        }
    }
}

impl StrategyOptions {
    pub fn new(
        id: i32,
        strategy_id: i32,
        multiples_tardes: bool,
        trading_direccion: TradingDirection,
        operar_finde: bool,
        cerrar_fin_de_dia: bool,
        hora_fin_de_dia: DateTime<Utc>,
        cerrar_viernes: bool,
        hora_cierre_viernes: DateTime<Utc>,
        rango_operativo: bool,
        rango_operativo_inicio: DateTime<Utc>,
        rango_operativo_fin: DateTime<Utc>,
        cerrar_fin_rango_operativo: bool,
        activar_cierre_numero_velas: bool,
        numero_velas_cierre: i32,
        cierre_limite_hora: bool,
        hora_cierre_limite: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            strategy_id,
            multiples_tardes,
            trading_direccion,
            operar_finde,
            cerrar_fin_de_dia,
            hora_fin_de_dia,
            cerrar_viernes,
            hora_cierre_viernes,
            rango_operativo,
            rango_operativo_inicio,
            rango_operativo_fin,
            cerrar_fin_rango_operativo,
            activar_cierre_numero_velas,
            numero_velas_cierre,
            cierre_limite_hora,
            hora_cierre_limite,
        }
    }

    pub fn new_empty() -> Self {
        Self {
            id: 0,
            strategy_id: 0,
            multiples_tardes: false,
            trading_direccion: TradingDirection::Both,
            operar_finde: false,
            cerrar_fin_de_dia: false,
            hora_fin_de_dia: Utc::now(),
            cerrar_viernes: false,
            hora_cierre_viernes: Utc::now(),
            rango_operativo: false,
            rango_operativo_inicio: Utc::now(),
            rango_operativo_fin: Utc::now(),
            cerrar_fin_rango_operativo: false,
            activar_cierre_numero_velas: false,
            numero_velas_cierre: 0,
            cierre_limite_hora: false,
            hora_cierre_limite: Utc::now(),
        }
    }
}
