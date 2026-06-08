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
                println!("Symbol creado: {:?}", symbol);
            }
            Err(e) => println!("Error al crear symbol: {:?}", e),
        }

        symbol
    }

    pub async fn create_symbol(&mut self) {
        self.id = insert_symbol_cfd(&self).await.unwrap();
    }
}
