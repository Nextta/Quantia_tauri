use crate::api::trades::table_trades;
use crate::backtest::backtest::{Backtest, GestionParams, GestionStrategy};
use crate::backtest::dias::Dias;
use crate::backtest::symbol::SymbolInfoCFD;
use crate::utils::tools::truncate_decimal;
use rand::prelude::*;
use serde::{Deserialize, Serialize};

use chrono::{Datelike, NaiveDateTime, Weekday}; // Utc, Month, DateTime,
use rust_decimal::Decimal;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Trade {
    pub id: i32, // id del Backtest
    pub id_backtest: i32,
    pub id_symbol: i32,
    pub symbol: SymbolInfoCFD,
    pub tipo: String, // tipo = Tipo de operación (buy/sell)
    pub lotaje: f64,
    pub multiplicador: f64, // Multiplicador del lotaje por operación.
    pub t0: String,         // t0 = Fecha y hora de entrada
    pub precio_entrada: f64,
    pub tp: f64,
    pub sl: f64,
    pub t1: String, // t1 = Fecha y hora de cierre
    pub precio_cierre: f64,
    pub precio_maximo: f64, // precioMaximo = Precio máximo alcanzado durante la operación
    pub precio_minimo: f64, // precioMinimo = Precio mínimo alcanzado durante la operación
    pub duracion_segundos: String, // duracionSegundos = Duración en segundos de la operación
    pub duracion_minutos: String, // duracionMinutos = Duración en minutos de la operación
    pub duracion_horas: String, // duracionHoras = Duración en horas de la operación
    pub duracion_dias: String, // duracionDias = Duración en días de la operación
    pub label: u32,         // label = Etiqueta de la operación 1 ganada, 0 perdida
    pub pl: f64,            // pl = Ganancia o pérdida de la operación con comisión
    pub plsc: f64,          // plsc = Ganancia o pérdida de la operación sin comisiones
    pub pips_pl: f64,       // pipsPL = Ganancia o pérdida de la operación en pips
}

impl Trade {
    pub async fn new(id_backtest: i32, symbol: SymbolInfoCFD) -> Self {
        let table = table_trades().await;

        match table {
            Ok(_) => {}
            Err(e) => println!("Error al obtener la tabla de trades: {:?}", e),
        }

        Trade {
            id: 0,
            id_backtest,
            id_symbol: symbol.id,
            symbol,
            tipo: "none".to_string(),
            lotaje: 0.0,
            multiplicador: 1.0,
            t0: "0000-00-00 00:00:00".to_string(),
            precio_entrada: 0.0,
            tp: 0.0,
            sl: 0.0,
            t1: "0000-00-00 00:00:00".to_string(),
            precio_cierre: 0.0,
            precio_maximo: 0.0,
            precio_minimo: 0.0,
            duracion_segundos: "0".to_string(),
            duracion_minutos: "0".to_string(),
            duracion_horas: "0".to_string(),
            duracion_dias: "0".to_string(),
            label: 0,
            pl: 0.0,
            plsc: 0.0,
            pips_pl: 0.0,
        }
    }

    fn lotaje_formato(&mut self, lotaje: f64) {
        let mut lote: Decimal = lotaje.to_string().parse().unwrap();

        lote = truncate_decimal(lote, 2);

        let lote_truncado: f64 = lote.to_string().parse().unwrap();

        self.lotaje = lote_truncado;

        if self.symbol.lotaje_maximo < self.lotaje {
            self.lotaje = self.symbol.lotaje_maximo;
        }

        if self.lotaje < self.symbol.lotaje_minimo {
            self.lotaje = self.symbol.lotaje_minimo
        }
    }

    /// Funciones
    fn random_spread(&self) -> f64 {
        let mut rng = rand::rng();
        let spread = self.symbol.spread;
        let mut numero: f64 = rng.random_range((spread / 2.0)..=spread);

        let numero_truncado: Decimal = truncate_decimal(
            numero.to_string().parse().unwrap(),
            self.symbol.digitos as u32,
        );
        numero = numero_truncado.to_string().parse().unwrap();
        numero
    }

    pub fn buy(
        &mut self,
        t0: String,
        precio_entrada: f64,
        gestion: GestionStrategy,
        parametros: GestionParams,
        backtest: &Backtest,
        tp: Option<f64>,
        sl: Option<f64>,
    ) {
        self.tipo = "Buy".to_string();
        self.t0 = t0;
        self.precio_entrada = precio_entrada + self.random_spread();
        self.tp = tp.unwrap_or(0.0);
        self.sl = sl.unwrap_or(0.0);

        match gestion {
            GestionStrategy::Fijo => self.lotaje = parametros.lotaje_fijo,
            GestionStrategy::Formula => {
                self.multiplicador = parametros.multiplicador;
                self.lotaje_quantia(precio_entrada, backtest);
            }
            // GestionStrategy::Kelly => 0.0,
            // GestionStrategy::PocertajeEquity => 0.0,
            // GestionStrategy::PorcentajeBalance => 0.0,
            _ => self.lotaje = 0.01,
        }
    }

    pub fn sell(
        &mut self,
        t0: String,
        precio_entrada: f64,
        gestion: GestionStrategy,
        parametros: GestionParams,
        backtest: &Backtest,
        tp: Option<f64>,
        sl: Option<f64>,
    ) {
        self.tipo = "Sell".to_string();
        self.t0 = t0;
        self.precio_entrada = precio_entrada;
        self.tp = tp.unwrap_or(0.0);
        self.sl = sl.unwrap_or(0.0);

        match gestion {
            GestionStrategy::Fijo => self.lotaje = parametros.lotaje_fijo,
            GestionStrategy::Formula => {
                self.multiplicador = parametros.multiplicador;
                self.lotaje_quantia(precio_entrada, backtest);
            }
            // GestionStrategy::Kelly => 0.0,
            // GestionStrategy::PocertajeEquity => 0.0,
            // GestionStrategy::PorcentajeBalance => 0.0,
            _ => self.lotaje = 0.01,
        }
    }

    pub fn close(&mut self, t1: String, precio_cierre: f64) {
        self.t1 = t1;
        self.precio_cierre = precio_cierre;
        self.calcular_duración();
        self.calcular_pl();

        if self.pl > 0.0 {
            self.label = 1;
        } else {
            self.label = 0;
        }
    }

    ///Calcula el lotaje óptimo para una operación basado en el balance actual, el precio de entrada,
    ///el valor de los contratos y un multiplicador. Si se han definido límites máximos o mínimos de lotaje,
    ///ajusta el resultado para no excederlos. El resultado se redondea hacia abajo a dos decimales.
    ///Args:
    ///    precio (float): El precio de entrada de la operación.
    ///Returns:
    ///    float: El lotaje calculado, ajustado a los límites y redondeado a dos decimales.
    pub fn lotaje_quantia(&mut self, precio: f64, backtest: &Backtest) {
        let lotaje =
            (backtest.balance / (precio * self.symbol.valor_contrato)) * self.multiplicador;

        self.lotaje_formato(lotaje);
    }

    fn calcular_duración(&mut self) {
        let date_time_t0 = NaiveDateTime::parse_from_str(&self.t0, "%Y-%m-%d %H:%M:%S").unwrap();
        let date_time_t1 = NaiveDateTime::parse_from_str(&self.t1, "%Y-%m-%d %H:%M:%S").unwrap();
        let duration = date_time_t1 - date_time_t0;
        self.duracion_segundos = format!("{:02}", duration.num_seconds());
        self.duracion_minutos = format!("{:02}", duration.num_minutes());
        self.duracion_horas = format!("{:02}", duration.num_hours());
        self.duracion_dias = format!("{:02}", duration.num_days());
    }

    fn calcular_pl(&mut self) {
        if self.tipo == "Sell" {
            self.plsc = (self.precio_entrada - self.precio_cierre)
                * self.lotaje
                * self.symbol.valor_contrato;
            self.pips_pl = (self.precio_entrada - self.precio_cierre) * 10000.0;
            self.pl = (self.plsc + (self.symbol.comision_lote * self.lotaje))
                + self.calcular_comision_swap();
        } else if self.tipo == "Buy" {
            self.plsc = (self.precio_cierre - self.precio_entrada)
                * self.lotaje
                * self.symbol.valor_contrato;
            self.pips_pl = (self.precio_cierre - self.precio_entrada) * 10000.0;
            self.pl = (self.plsc + (self.symbol.comision_lote * self.lotaje))
                + self.calcular_comision_swap();
        }
    }

    fn calcular_comision_swap(&mut self) -> f64 {
        let mut comision_swap = 0.0;

        if self.duracion_dias == "00".to_string() {
            return 0.0;
        }

        let t0 = NaiveDateTime::parse_from_str(&self.t0, "%Y-%m-%d %H:%M:%S").unwrap();
        let t1 = NaiveDateTime::parse_from_str(&self.t1, "%Y-%m-%d %H:%M:%S").unwrap();

        let swap_diario = match self.tipo.as_str() {
            "Buy" => self.symbol.swap_long * self.lotaje,
            "Sell" => self.symbol.swap_short * self.lotaje,
            _ => return 0.0,
        };

        let mut fecha_actual = t0.date();
        let fecha_fin = t1.date();

        while fecha_actual <= fecha_fin {
            let dia_semana = match fecha_actual.weekday() {
                Weekday::Mon => Dias::Lu,
                Weekday::Tue => Dias::Ma,
                Weekday::Wed => Dias::Mi,
                Weekday::Thu => Dias::Ju,
                Weekday::Fri => Dias::Vi,
                Weekday::Sat => Dias::Sa,
                Weekday::Sun => Dias::Do,
            };

            let multiplicador = if dia_semana == self.symbol.dia_triple_swap {
                3
            } else {
                1
            };

            if dia_semana == Dias::Do || dia_semana == Dias::Sa {
                if self.symbol.open_weekend {
                    comision_swap += swap_diario * multiplicador as f64;
                }
            } else {
                comision_swap += swap_diario * multiplicador as f64;
            }

            // println!("Swap individual: {}", comision_swap);

            fecha_actual = fecha_actual.succ_opt().unwrap();
        }

        // println!("Swap total: {}", comision_swap);

        comision_swap
    }
}
