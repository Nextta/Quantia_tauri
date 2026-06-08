use crate::api::brokers::{insert_broker_cfd, table_brokers_cfd};
use crate::backtest::symbol::SymbolInfoCFD;

#[derive(Debug, Clone)]
pub struct BrokerCFD {
    pub id: i32,
    pub name: String,
    pub symbol_info: Vec<SymbolInfoCFD>,
}

impl BrokerCFD {
    pub async fn new(id: i32, name: String) -> Self {
        let table = table_brokers_cfd().await;

        let broker: BrokerCFD = BrokerCFD {
            id,
            name,
            symbol_info: Vec::<SymbolInfoCFD>::new(),
        };
        match table {
            Ok(_) => {
                println!("Broker creado: {:?}", broker);
            }
            Err(e) => println!("Error al crear broker: {:?}", e),
        }

        broker
    }

    pub async fn create_broker_cfd(&mut self) {
        self.id = insert_broker_cfd(&self).await.unwrap();
    }

    ///Funciones

    pub fn add_symbol_info(&mut self, symbol_info: SymbolInfoCFD) {
        self.symbol_info.push(symbol_info);
    }
}
