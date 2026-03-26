use crate::api::dbsqlite::DbSqlite;
use crate::backtest::datos::Datos;
use crate::backtest::trade::Trade;

use polars::prelude::*;

#[derive(Debug, Clone)]
pub struct Backtest {
    id: i32,
    titulo: String,
    balance: f64,
    tipo: String, // Tipo de activo ej: Forex, Crypto, Futuros...etc
    trades: Vec<Trade>,
    datos: Vec<Datos>,
}

impl Backtest {
    pub async fn new(titulo: String, balance: f64, tipo: String) -> Self {
        let db: DbSqlite = DbSqlite::new("sqlite:db/quantia_db.sqlite3").await.unwrap();

        let table = db.table_backtest().await;

        let mut backtest: Backtest = Backtest {
            id: 0,
            titulo,
            balance,
            tipo,
            trades: Vec::<Trade>::new(),
            datos: Vec::<Datos>::new(),
        };

        match table {
            Ok(_) => {
                backtest.id = db.insert_backtest(&backtest).await.unwrap();
            }
            Err(e) => println!("TABLE: backtest error: {}", e),
        }

        backtest
    }

    ///Getters
    pub fn get_id(&self) -> i32 {
        self.id
    }

    pub fn get_titulo(&self) -> &str {
        &self.titulo
    }

    pub fn get_balance(&self) -> f64 {
        self.balance
    }

    pub fn get_tipo(&self) -> &str {
        &self.tipo
    }

    pub fn get_trades(&self) -> &Vec<Trade> {
        &self.trades
    }

    pub fn get_datos(&self) -> &Vec<Datos> {
        &self.datos
    }

    /// Setters
    pub fn set_id(&mut self, id: i32) {
        self.id = id;
    }

    pub fn set_balance(&mut self, balance: f64) {
        self.balance = balance;
    }

    pub fn set_tipo(&mut self, tipo: String) {
        self.tipo = tipo;
    }

    /// Funciones Core Backtest
    pub fn add_datos(&mut self, ruta: &str) -> Result<Datos, Box<dyn std::error::Error>> {
        let df: DataFrame = CsvReadOptions::default()
            .try_into_reader_with_file_path(Some(ruta.into()))?
            .finish()?;

        let data = Datos::new(df);
        self.datos.push(data.clone());
        Ok(data)
    }

    pub fn add_trade(&mut self, trade: Trade) {
        self.trades.push(trade);
    }

    pub fn add_datos_tbl(&self) {
        // TODO: Implementar la función para agregar datos al dataframe con indicadores de volatilidad
        // Entropía de Shannon y Exponente de hurst.
    }

    pub fn tiple_barrier_data(&self) {
        // TODO: Implementar la funcionalidad que devuelva un dataframe con los datos de los trades
        // para entrenar el modelo de machine learning con TBL.
    }

    pub fn informe_resultados(&self) {
        // TODO: Implementar la función para generar un informe de resultados del backtest
        // Recivirá el dataframe con los trades y calcula las metricas.
    }

    pub fn curva_equidad(&self) {
        // TODO: Implementar la función para generar la curva de equidad del backtest
        // Recivirá el dataframe con los trades y calcula las metricas.
    }

    pub fn split_data(&self) {
        // TODO: Implementar la función para dividir los datos en conjuntos de entrenamiento y prueba
        // Recivirá el dataframe con los trades y calcula las metricas.
    }

    pub fn solapamiento_trades(&self) {
        // TODO: Implementar la función para calcular el solapamiento entre los trades y mostrar un grafico.
    }

    pub async fn guardar_trades(&self) {
        let db: DbSqlite = DbSqlite::new("sqlite:db/quantia_db.sqlite3").await.unwrap();

        for trade in &self.trades {
            db.insert_trades(self.id, trade).await.unwrap();
        }
    }
}
