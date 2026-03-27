use crate::api::trades::{insert_trades, table_trades};
use crate::backtest::backtest::Backtest;
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
            id_symbol: symbol.get_id(),
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

    ///Getters
    pub fn get_id(&self) -> i32 {
        self.id
    }

    pub fn get_id_backtest(&self) -> i32 {
        self.id_backtest
    }

    pub fn get_id_symbol(&self) -> i32 {
        self.id_symbol
    }

    pub fn get_symbol(&self) -> &SymbolInfoCFD {
        &self.symbol
    }

    pub fn get_tipo(&self) -> &String {
        &self.tipo
    }

    pub fn get_lotaje(&self) -> f64 {
        self.lotaje
    }

    pub fn get_multiplier(&self) -> f64 {
        self.multiplicador
    }

    pub fn get_t0(&self) -> String {
        self.t0.clone()
    }

    pub fn get_precio_entrada(&self) -> f64 {
        self.precio_entrada
    }

    pub fn get_tp(&self) -> f64 {
        self.tp
    }

    pub fn get_sl(&self) -> f64 {
        self.sl
    }

    pub fn get_t1(&self) -> String {
        self.t1.clone()
    }

    pub fn get_precio_cierre(&self) -> f64 {
        self.precio_cierre
    }

    pub fn get_precio_maximo(&self) -> f64 {
        self.precio_maximo
    }

    pub fn get_precio_minimo(&self) -> f64 {
        self.precio_minimo
    }

    pub fn get_duracion_segundos(&self) -> &String {
        &self.duracion_segundos
    }

    pub fn get_duracion_minutos(&self) -> &String {
        &self.duracion_minutos
    }

    pub fn get_duracion_horas(&self) -> &String {
        &self.duracion_horas
    }

    pub fn get_duracion_dias(&self) -> &String {
        &self.duracion_dias
    }

    pub fn get_label(&self) -> u32 {
        self.label
    }

    pub fn get_pl(&self) -> f64 {
        self.pl
    }

    pub fn get_plsc(&self) -> f64 {
        self.plsc
    }

    pub fn get_pip_pl(&self) -> f64 {
        self.pips_pl
    }

    /// Setters
    pub fn set_id(&mut self, id: i32) {
        self.id = id;
    }

    pub fn set_id_backtest(&mut self, id_backtest: i32) {
        self.id_backtest = id_backtest;
    }

    pub fn set_id_symbol(&mut self, id_symbol: i32) {
        self.id_symbol = id_symbol;
    }

    pub fn set_multiplicador(&mut self, multiplicador: f64) {
        self.multiplicador = multiplicador;
    }

    pub fn set_pips_pl(&mut self, pips_pl: f64) {
        self.pips_pl = pips_pl;
    }

    pub fn set_symbol(&mut self, symbol: SymbolInfoCFD) {
        self.symbol = symbol;
    }

    pub fn set_tipo(&mut self, tipo: String) {
        self.tipo = tipo;
    }

    pub fn set_t0(&mut self, t0: String) {
        self.t0 = t0;
    }

    pub fn set_lotaje(&mut self, lotaje: f64, precio: f64, backtest: &Backtest) {
        if lotaje <= 0.0 {
            let m_lote = self.lotaje_quantia(precio, backtest);

            let mut lote: Decimal = m_lote.to_string().parse().unwrap();

            lote = truncate_decimal(lote, 2);

            self.lotaje = lote.to_string().parse().unwrap();
        } else {
            self.lotaje = lotaje;
        };
    }

    pub fn set_lotaje_fijo(&mut self, lotaje: f64) {
        self.lotaje = lotaje;
    }

    pub fn set_multiplier(&mut self, multiplier: f64) {
        self.multiplicador = multiplier;
    }

    pub fn set_precio_entrada(&mut self, precio_entrada: f64) {
        self.precio_entrada = precio_entrada;
    }

    pub fn set_tp(&mut self, tp: f64) {
        self.tp = tp;
    }

    pub fn set_sl(&mut self, sl: f64) {
        self.sl = sl;
    }

    pub fn set_t1(&mut self, t1: String) {
        self.t1 = t1;
    }

    pub fn set_precio_cierre(&mut self, precio_cierre: f64) {
        self.precio_cierre = precio_cierre;
    }

    pub fn set_precio_maximo(&mut self, precio_maximo: f64) {
        self.precio_maximo = precio_maximo;
    }

    pub fn set_precio_minimo(&mut self, precio_minimo: f64) {
        self.precio_minimo = precio_minimo;
    }

    pub fn set_duracion_segundos(&mut self, duracion: String) {
        self.duracion_segundos = duracion;
    }

    pub fn set_duracion_minutos(&mut self, duracion: String) {
        self.duracion_minutos = duracion;
    }

    pub fn set_duracion_horas(&mut self, duracion: String) {
        self.duracion_horas = duracion;
    }

    pub fn set_duracion_dias(&mut self, duracion: String) {
        self.duracion_dias = duracion;
    }

    pub fn set_label(&mut self, label: u32) {
        self.label = label;
    }

    pub fn set_pl(&mut self, pl: f64) {
        self.pl = pl;
    }

    pub fn set_plsc(&mut self, plsc: f64) {
        self.plsc = plsc;
    }

    pub fn set_pip_pl(&mut self, pip_pl: f64) {
        self.pips_pl = pip_pl;
    }

    /// Funciones
    fn random_spread(&self, precio_entrada: f64) -> f64 {
        let mut rng = rand::rng();
        let spread = self.symbol.get_spread();
        let mut numero: f64 = rng.random_range((spread / 2.0)..=spread);

        let numero_truncado: Decimal = truncate_decimal(
            numero.to_string().parse().unwrap(),
            self.symbol.get_digitos() as u32,
        );
        // println!("{}", numero_truncado);
        numero = numero_truncado.to_string().parse().unwrap();
        // println!("{}", numero);
        numero
    }

    pub fn buy(
        &mut self,
        lotaje: f64,
        multiplicador: f64,
        t0: String,
        precio_entrada: f64,
        tp: f64,
        sl: f64,
        backtest: &Backtest,
    ) {
        self.tipo = "Buy".to_string();
        self.multiplicador = multiplicador;
        self.t0 = t0;
        self.precio_entrada = precio_entrada + self.random_spread(precio_entrada);
        self.tp = tp;
        self.sl = sl;

        if lotaje <= 0.0 {
            self.set_lotaje(lotaje, precio_entrada, backtest);
        } else {
            self.lotaje = lotaje;
        }
    }

    pub fn sell(
        &mut self,
        lotaje: f64,
        multiplicador: f64,
        t0: String,
        precio_entrada: f64,
        tp: f64,
        sl: f64,
        backtest: &Backtest,
    ) {
        self.tipo = "Sell".to_string();
        self.multiplicador = multiplicador;
        self.t0 = t0;
        self.precio_entrada = precio_entrada;
        self.tp = tp;
        self.sl = sl;

        if lotaje <= 0.0 {
            self.set_lotaje(lotaje, precio_entrada, backtest);
        } else {
            self.lotaje = lotaje;
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
    pub fn lotaje_quantia(&self, precio: f64, backtest: &Backtest) -> f64 {
        let mut lotaje = (backtest.get_balance() / (precio * self.symbol.get_valor_contrato()))
            * self.multiplicador;

        if self.symbol.get_lotaje_maximo() < lotaje {
            lotaje = self.symbol.get_lotaje_maximo();
        }

        if lotaje < self.symbol.get_lotaje_minimo() {
            lotaje = self.symbol.get_lotaje_minimo()
        }

        lotaje
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
                * self.symbol.get_valor_contrato();
            self.pips_pl = (self.precio_entrada - self.precio_cierre) * 10000.0;
            self.pl = (self.plsc + (self.symbol.get_comision_lote() * self.lotaje))
                + self.calcular_comision_swap();
        } else if self.tipo == "Buy" {
            self.plsc = (self.precio_cierre - self.precio_entrada)
                * self.lotaje
                * self.symbol.get_valor_contrato();
            self.pips_pl = (self.precio_cierre - self.precio_entrada) * 10000.0;
            self.pl = (self.plsc + (self.symbol.get_comision_lote() * self.lotaje))
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
            "Buy" => self.symbol.get_swap_long() * self.lotaje,
            "Sell" => self.symbol.get_swap_short() * self.lotaje,
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

            let multiplicador = if dia_semana == self.symbol.get_dia_triple_swap() {
                3
            } else {
                1
            };

            if dia_semana == Dias::Do || dia_semana == Dias::Sa {
                if self.symbol.get_open_weekend() {
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
