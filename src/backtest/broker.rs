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
        self.id = insert_broker_cfd(self.clone()).await.unwrap();
    }

    ///Getters
    pub fn get_id(&self) -> i32 {
        self.id
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn get_symbol_info(&self) -> &Vec<SymbolInfoCFD> {
        &self.symbol_info
    }

    ///Setters
    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    ///Funciones

    pub fn add_symbol_info(&mut self, symbol_info: SymbolInfoCFD) {
        self.symbol_info.push(symbol_info);
    }
}
