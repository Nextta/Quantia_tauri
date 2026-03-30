use crate::api::symbols::{insert_symbol_cfd, table_symbols_cfd};
use crate::backtest::dias::Dias;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SymbolInfoCFD {
    pub id: i32,
    pub broker_id: i32,
    pub name: String,
    pub valor_contrato: f64,
    pub comision_lote: f64,
    pub swap_long: f64,
    pub swap_short: f64,
    pub dia_triple_swap: Dias,
    pub lotaje_minimo: f64,
    pub lotaje_maximo: f64,
    pub digitos: u32,
    pub open_weekend: bool,
    pub spread: f64,
}

impl SymbolInfoCFD {
    pub async fn new(
        id: i32,
        broker_id: i32,
        name: String,
        valor_contrato: f64,
        comision_lote: f64,
        swap_long: f64,
        swap_short: f64,
        dia_triple_swap: Dias,
        lotaje_minimo: f64,
        lotaje_maximo: f64,
        digitos: u32,
        open_weekend: bool,
        spread: f64,
    ) -> Self {
        let table = table_symbols_cfd().await;

        let symbol: SymbolInfoCFD = SymbolInfoCFD {
            id,
            broker_id,
            name,
            valor_contrato,
            comision_lote,
            swap_long,
            swap_short,
            dia_triple_swap,
            lotaje_minimo,
            lotaje_maximo,
            digitos,
            open_weekend,
            spread,
        };
        match table {
            Ok(_) => {
                //symbol.id = db.insert_symbol_cfd(symbol.clone()).await.unwrap();
                println!("Symbol creado: {:?}", symbol);
            }
            Err(e) => println!("Error al crear symbol: {:?}", e),
        }

        symbol
    }

    pub async fn create_symbol(&mut self) {
        self.id = insert_symbol_cfd(self.clone()).await.unwrap();
    }

    ///Getters
    pub fn get_id(&self) -> i32 {
        self.id
    }

    pub fn get_broker_id(&self) -> i32 {
        self.broker_id
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn get_open_weekend(&self) -> bool {
        self.open_weekend
    }

    pub fn get_valor_contrato(&self) -> f64 {
        self.valor_contrato
    }

    pub fn get_comision_lote(&self) -> f64 {
        self.comision_lote
    }

    pub fn get_swap_long(&self) -> f64 {
        self.swap_long
    }

    pub fn get_swap_short(&self) -> f64 {
        self.swap_short
    }

    pub fn get_dia_triple_swap(&self) -> Dias {
        self.dia_triple_swap.clone()
    }

    pub fn get_lotaje_minimo(&self) -> f64 {
        self.lotaje_minimo
    }

    pub fn get_lotaje_maximo(&self) -> f64 {
        self.lotaje_maximo
    }

    pub fn get_digitos(&self) -> u32 {
        self.digitos
    }

    pub fn get_spread(&self) -> f64 {
        self.spread
    }

    ///Setters
    pub fn set_broker_id(&mut self, broker_id: i32) {
        self.broker_id = broker_id;
    }

    pub fn set_open_weekend(&mut self, open_weekend: bool) {
        self.open_weekend = open_weekend;
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn set_valor_contrato(&mut self, valor_contrato: f64) {
        self.valor_contrato = valor_contrato;
    }

    pub fn set_comision_lote(&mut self, comision_lote: f64) {
        self.comision_lote = comision_lote;
    }

    pub fn set_swap_long(&mut self, swap_long: f64) {
        self.swap_long = swap_long;
    }

    pub fn set_swap_short(&mut self, swap_short: f64) {
        self.swap_short = swap_short;
    }

    pub fn set_dia_triple_swap(&mut self, dia_triple_swap: Dias) {
        self.dia_triple_swap = dia_triple_swap;
    }

    pub fn set_lotaje_minimo(&mut self, lotaje_minimo: f64) {
        self.lotaje_minimo = lotaje_minimo;
    }

    pub fn set_lotaje_maximo(&mut self, lotaje_maximo: f64) {
        self.lotaje_maximo = lotaje_maximo;
    }

    pub fn set_digitos(&mut self, digitos: u32) {
        self.digitos = digitos;
    }

    pub fn set_spread(&mut self, spread: f64) {
        self.spread = spread;
    }
}
