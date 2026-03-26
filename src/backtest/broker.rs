use crate::api::dbsqlite::DbSqlite;
use crate::backtest::symbol::SymbolInfoCFD;

#[derive(Debug, Clone)]
pub struct BrokerCFD {
    id: i32,
    name: String,
    symbol_info: Vec<SymbolInfoCFD>,
}

impl BrokerCFD {
    pub async fn new(id: i32, name: String) -> Self {
        let db: DbSqlite = DbSqlite::new("sqlite:db/quantia_db.sqlite3").await.unwrap();

        let table = db.table_broker_cfd().await;

        let broker: BrokerCFD = BrokerCFD {
            id,
            name,
            symbol_info: Vec::<SymbolInfoCFD>::new(),
        };
        match table {
            Ok(_) => {
                println!("Broker creado: {:?}", broker);
            }
            Err(e) => println!("Error al crear broker: {}", e),
        }

        broker
    }

    pub async fn create_broker_cfd(&mut self) {
        let db: DbSqlite = DbSqlite::new("sqlite:db/quantia_db.sqlite3").await.unwrap();
        self.id = db.insert_broker_cfd(self.clone()).await.unwrap();
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
