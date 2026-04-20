use crate::api::backtests::{insert_backtest_cfd, table_backtests_cfd};
use crate::api::strategies::{
    get_strategies_actions_by_strategy_id, get_strategies_by_id,
    get_strategies_conditions_by_strategy_id, get_strategies_indicators_by_strategy_id,
};
use crate::api::trades::insert_trades;
use crate::backtest::datos::Datos;
use crate::backtest::trade::Trade;
use crate::indicators::cycle::*;
use crate::indicators::overlap::*;
use crate::indicators::pattern::*;
use crate::strategy::strategy::Strategy;
use crate::strategy::strategy_options::StrategyOptions;
use polars::datatypes::DataType;
use polars::prelude::*;
use std::collections::HashMap;
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct Backtest {
    pub id: i32,
    pub titulo: String,
    pub balance: f64,
    pub tipo: String, // Tipo de activo ej: Forex, Crypto, Futuros...etc
    pub trades: Vec<Trade>,
    pub datos: Vec<Datos>,
    pub estrategia: Strategy,
}

impl Backtest {
    pub async fn new(titulo: String, balance: f64, tipo: String) -> Self {
        let table = table_backtests_cfd().await;

        let mut backtest: Backtest = Backtest {
            id: 0,
            titulo,
            balance,
            tipo,
            trades: Vec::<Trade>::new(),
            datos: Vec::<Datos>::new(),
            estrategia: Strategy::new_empty(),
        };

        match table {
            Ok(_) => {
                backtest.id = insert_backtest_cfd(backtest.clone()).await.unwrap();
            }
            Err(e) => println!("TABLE: backtest error: {:?}", e),
        }

        backtest
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

    async fn set_indicators_strategy(&mut self, datos: DataFrame) -> PolarsResult<DataFrame> {
        let mut df: DataFrame = datos.clone();
        //TODO: Añadir los indicadores de la estrategia al dataframe de datos.
        for indicator in &self.estrategia.indicadores {
            df = match indicator.tipo.as_str() {
                "HT_DCPERIOD" => ht_dcperiod(df, Some(&indicator.nombre)).await?,
                "HT_DCPHASE" => ht_dcphase(df, Some(&indicator.nombre)).await?,
                "HT_PHASOR" => {
                    ht_phasor(
                        df,
                        Some(format!("{}_in_phase", &indicator.nombre).as_str()),
                        Some(format!("{}_quadrature", &indicator.nombre).as_str()),
                    )
                    .await?
                }
                "HT_SINE" => {
                    ht_sine(
                        df,
                        Some(format!("{}_sine", &indicator.nombre).as_str()),
                        Some(format!("{}_lead_sine", &indicator.nombre).as_str()),
                    )
                    .await?
                }
                "HT_TRENDMODE" => ht_trendmode(df, Some(&indicator.nombre)).await?,
                "BBANDS" => {
                    let parametros =
                        serde_json::from_value::<BbandsParams>(indicator.parametros.clone())
                            .unwrap();
                    bbands(
                        df,
                        Some(parametros.timeperiod),
                        Some(parametros.nbdevup),
                        Some(parametros.nbdevdn),
                        Some(parametros.matype),
                        Some(format!("{}_bb_upper", &indicator.nombre).as_str()),
                        Some(format!("{}_bb_upper", &indicator.nombre).as_str()),
                        Some(format!("{}_bb_upper", &indicator.nombre).as_str()),
                    )
                    .await?
                }
                "DEMA" => {
                    let parametros =
                        serde_json::from_value::<DemaParams>(indicator.parametros.clone()).unwrap();
                    dema(df, Some(parametros.timeperiod), Some(&indicator.nombre)).await?
                }
                "EMA" => {
                    let parametros =
                        serde_json::from_value::<EmaParams>(indicator.parametros.clone()).unwrap();
                    ema(df, Some(parametros.timeperiod), Some(&indicator.nombre)).await?
                }
                "KAMA" => {
                    let parametros =
                        serde_json::from_value::<KamaParams>(indicator.parametros.clone()).unwrap();
                    kama(df, Some(parametros.timeperiod), Some(&indicator.nombre)).await?
                }
                "MA" => {
                    let parametros =
                        serde_json::from_value::<MaParams>(indicator.parametros.clone()).unwrap();
                    ma(
                        df,
                        Some(parametros.timeperiod),
                        Some(parametros.matype),
                        Some(&indicator.nombre),
                    )
                    .await?
                }
                "MAMA" => {
                    let parametros =
                        serde_json::from_value::<MamaParams>(indicator.parametros.clone()).unwrap();
                    mama(
                        df,
                        Some(parametros.fastlimit),
                        Some(parametros.slowlimit),
                        Some(format!("{}_mama", &indicator.nombre).as_str()),
                        Some(format!("{}_fama", &indicator.nombre).as_str()),
                    )
                    .await?
                }
                "MIDPOINT" => {
                    let parametros =
                        serde_json::from_value::<MidpointParams>(indicator.parametros.clone())
                            .unwrap();
                    midpoint(df, Some(parametros.timeperiod), Some(&indicator.nombre)).await?
                }
                "MIDPRICE" => {
                    let parametros =
                        serde_json::from_value::<MidpriceParams>(indicator.parametros.clone())
                            .unwrap();
                    midprice(df, Some(parametros.timeperiod), Some(&indicator.nombre)).await?
                }
                "SAR" => {
                    let parametros =
                        serde_json::from_value::<SarParams>(indicator.parametros.clone()).unwrap();
                    sar(
                        df,
                        Some(parametros.acceleration),
                        Some(parametros.maximum),
                        Some(&indicator.nombre),
                    )
                    .await?
                }
                "SAREXT" => {
                    let parametros =
                        serde_json::from_value::<SarextParams>(indicator.parametros.clone())
                            .unwrap();
                    sarext(
                        df,
                        Some(parametros.startvalue),
                        Some(parametros.offsetonlong),
                        Some(parametros.offsetonshort),
                        Some(parametros.blockonlong),
                        Some(parametros.blockonshort),
                        Some(&indicator.nombre),
                    )
                    .await?
                }
                "SMA" => {
                    let parametros =
                        serde_json::from_value::<SmaParams>(indicator.parametros.clone()).unwrap();
                    sma(df, Some(parametros.timeperiod), Some(&indicator.nombre)).await?
                }
                "T3" => {
                    let parametros =
                        serde_json::from_value::<T3Params>(indicator.parametros.clone()).unwrap();
                    t3(
                        df,
                        Some(parametros.timeperiod),
                        Some(parametros.vfactor),
                        Some(&indicator.nombre),
                    )
                    .await?
                }
                "TEMA" => {
                    let parametros =
                        serde_json::from_value::<TemaParams>(indicator.parametros.clone()).unwrap();
                    tema(df, Some(parametros.timeperiod), Some(&indicator.nombre)).await?
                }
                "TRIMA" => {
                    let parametros =
                        serde_json::from_value::<TrimaParams>(indicator.parametros.clone())
                            .unwrap();
                    trima(df, Some(parametros.timeperiod), Some(&indicator.nombre)).await?
                }
                "WMA" => {
                    let parametros =
                        serde_json::from_value::<WmaParams>(indicator.parametros.clone()).unwrap();
                    wma(df, Some(parametros.timeperiod), Some(&indicator.nombre)).await?
                }
                _ => df,
            }
        }

        Ok(df)
    }

    pub async fn run(&mut self, id_startegy: i32) -> Result<String, Box<dyn std::error::Error>> {
        let inicio = Instant::now();
        self.estrategia = match get_strategies_by_id(id_startegy).await {
            Ok(strategy) => {
                let mut estrategia: Strategy = strategy;

                match get_strategies_actions_by_strategy_id(estrategia.id).await {
                    Ok(acciones) => {
                        estrategia.acciones = acciones;
                    }
                    Err(e) => {
                        println!("Error al obtener acciones: {:?}", e);
                    }
                }

                match get_strategies_indicators_by_strategy_id(estrategia.id).await {
                    Ok(indicadores) => {
                        estrategia.indicadores = indicadores;
                    }
                    Err(e) => {
                        println!("Error al obtener indicadores: {:?}", e);
                    }
                }

                match get_strategies_conditions_by_strategy_id(estrategia.id).await {
                    Ok(condiciones) => {
                        estrategia.condiciones = condiciones;
                    }
                    Err(e) => {
                        println!("Error al obtener condiciones: {:?}", e);
                    }
                }

                estrategia
            }
            Err(e) => {
                println!("Error al obtener estrategia: {:?}", e);
                Strategy {
                    id: 0,
                    id_user: 0,
                    nombre: String::new(),
                    descripcion: None,
                    activa: false,
                    creada_en: String::new(),
                    indicadores: Vec::new(),
                    condiciones: Vec::new(),
                    acciones: Vec::new(),
                    opciones: StrategyOptions::new_empty(),
                }
            }
        };

        if self.datos.is_empty() {
            return Ok("No hay datos para ejecutar el backtest".to_string());
        }

        for data in self.datos.clone() {
            //TODO: 1-Verificamos los indicadores que tiene la estrategia para añadirlos a los datos del DataFrame
            let df = match self.set_indicators_strategy(data.get_datos()).await {
                Ok(df_result) => df_result,
                Err(e) => {
                    return Err(Box::new(e));
                }
            };

            let schema = df.schema();
            let data_types = schema
                .iter()
                .map(|(name, dtype)| (name.clone().to_string(), dtype.clone()))
                .collect::<HashMap<String, DataType>>();

            //TODO: 2-Verificamos la direccion operativa que la estrategia nos permite operar. Long, Short o Both.

            // Foma de optener un dato: df.column(&columna)?.get(row_idx)?;
            for i in 0..df.height() {}
        }

        let duracion = inicio.elapsed();
        Ok(format!("Backtest finalizado en {}", duracion.as_secs_f64()))
    }

    pub async fn guardar_trades(&self) {
        for trade in &self.trades {
            insert_trades(self.id, trade).await.unwrap();
        }
    }
}
