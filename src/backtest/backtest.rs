use crate::api::backtests::{insert_backtest_cfd, table_backtests_cfd};
use crate::api::strategies::{
    get_strategies_actions_by_strategy_id, get_strategies_by_id,
    get_strategies_indicators_by_strategy_id,
};

use crate::api::trades::insert_trades;
use crate::backtest::datos::Datos;
use crate::backtest::resultados::Resultados;
use crate::backtest::symbol::SymbolInfoCFD;
use crate::backtest::trade::Trade;
use crate::enums::actions::Action;
use crate::enums::entry::EntryDirection;
use crate::indicators::cycle::*;
use crate::indicators::momentum::*;
use crate::indicators::overlap::*;
use crate::indicators::pattern::*;
use crate::indicators::price::*;
use crate::indicators::statistic::*;
use crate::indicators::volatility::*;
use crate::indicators::volume::*;
use crate::strategy::strategy::Strategy;
use crate::strategy::strategy_condition::StrategyCondition;
use crate::strategy::strategy_options::TradingDirection;
use crate::utils::configuracion::LOGS_REGISTRO;

use chrono::DateTime;
use polars::prelude::*;
use std::time::Instant;

use crate::enums::activos::Activo;
use crate::enums::gestion::GestionStrategy;
use crate::enums::logics::Logic;
use crate::enums::tipos::{BeTipo, TlTipo};
use crate::structs::logs::RegistroLog;
use crate::structs::options::NBarsOptions;
use crate::structs::parametros::{BeParams, GestionParams, LimitParams, TlParams};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Backtest {
    pub id: i32,
    pub titulo: String,
    pub balance: f64,
    pub tipo: Activo, // Tipo de activo ej: Forex, Crypto, Futuros...etc
    pub gestion_strategy: GestionStrategy,
    pub parametros_gestion: GestionParams,
    pub trades: Vec<Trade>,
    pub datos: Vec<Datos>,
    pub estrategia: Strategy,
}

impl Backtest {
    pub async fn new(
        titulo: String,
        balance: f64,
        tipo: Activo,
        gestion_strategy: GestionStrategy,
        parametros_gestion: GestionParams,
    ) -> Self {
        let table = table_backtests_cfd().await;

        let mut backtest: Backtest = Backtest {
            id: 0,
            titulo,
            balance,
            tipo,
            gestion_strategy,
            parametros_gestion,
            trades: Vec::<Trade>::new(),
            datos: Vec::<Datos>::new(),
            estrategia: Strategy::new_empty(),
        };
        if LOGS_REGISTRO {
            backtest.add_registro("Backtest creado".to_string());
        }
        match table {
            Ok(_) => {
                backtest.id = insert_backtest_cfd(&backtest).await.unwrap();
                if LOGS_REGISTRO {
                    backtest.add_registro("Backtest guardado en la base de datos".to_string());
                }
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
        if LOGS_REGISTRO {
            self.add_registro(format!("Datos agregados: {}", ruta));
        }
        Ok(data)
    }

    pub fn add_trade(&mut self, trade: Trade) {
        if LOGS_REGISTRO {
            self.add_registro(format!("Trade agregado: {}", trade.id.clone()));
        }
        self.trades.push(trade);
    }

    fn add_registro(&mut self, message: String) {
        let entry = RegistroLog::new(message.to_string());
        entry.guardar_logs().unwrap();
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

    /// Añade los indicadores de la estrategia al dataframe de datos.
    ///
    /// # Parametros
    /// datos: Dataframe con los datos.
    ///
    /// # Retorna
    /// DataFrame con los nuevos datos.
    fn set_indicators_strategy(&mut self, datos: &mut DataFrame) {
        let mut df: DataFrame = datos.clone();
        let indicadores = self.estrategia.indicadores.clone();

        for indicator in &indicadores {
            match indicator.tipo.as_str() {
                "HT_DCPERIOD" => {
                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador HT_DCPERIOD agregado: {}",
                            indicator.nombre
                        ));
                    }

                    ht_dcperiod(&mut df, Some(&indicator.nombre));
                }
                "HT_DCPHASE" => {
                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador HT_DCPHASE agregado: {}",
                            indicator.nombre
                        ));
                    }

                    ht_dcphase(&mut df, Some(&indicator.nombre));
                }
                "HT_PHASOR" => {
                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador HT_PHASOR agregado: {}",
                            indicator.nombre
                        ));
                    }

                    ht_phasor(
                        &mut df,
                        Some(format!("{}_in_phase", &indicator.nombre).as_str()),
                        Some(format!("{}_quadrature", &indicator.nombre).as_str()),
                    );
                }
                "HT_SINE" => {
                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador HT_SINE agregado: {}",
                            indicator.nombre
                        ));
                    }

                    ht_sine(
                        &mut df,
                        Some(format!("{}_sine", &indicator.nombre).as_str()),
                        Some(format!("{}_lead_sine", &indicator.nombre).as_str()),
                    );
                }
                "HT_TRENDMODE" => {
                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador HT_TRENDMODE agregado: {}",
                            indicator.nombre
                        ));
                    }

                    ht_trendmode(&mut df, Some(&indicator.nombre));
                }
                "BBANDS" => {
                    let parametros =
                        serde_json::from_value::<BbandsParams>(indicator.parametros.clone())
                            .unwrap();

                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador BBANDS agregado: {}",
                            indicator.nombre
                        ));
                    }

                    bbands(
                        &mut df,
                        Some(parametros.timeperiod),
                        Some(parametros.nbdevup),
                        Some(parametros.nbdevdn),
                        Some(parametros.matype),
                        Some(format!("{}_bb_upper", &indicator.nombre).as_str()),
                        Some(format!("{}_bb_upper", &indicator.nombre).as_str()),
                        Some(format!("{}_bb_upper", &indicator.nombre).as_str()),
                    );
                }
                "DEMA" => {
                    let parametros =
                        serde_json::from_value::<DemaParams>(indicator.parametros.clone()).unwrap();

                    if LOGS_REGISTRO {
                        self.add_registro(format!("Indicador DEMA agregado: {}", indicator.nombre));
                    }
                    dema(
                        &mut df,
                        Some(parametros.timeperiod),
                        Some(&indicator.nombre),
                    );
                }
                "EMA" => {
                    let parametros =
                        serde_json::from_value::<EmaParams>(indicator.parametros.clone()).unwrap();

                    if LOGS_REGISTRO {
                        self.add_registro(format!("Indicador EMA agregado: {}", indicator.nombre));
                    }
                    ema(
                        &mut df,
                        Some(parametros.timeperiod),
                        Some(&indicator.nombre),
                    );
                }
                "KAMA" => {
                    let parametros =
                        serde_json::from_value::<KamaParams>(indicator.parametros.clone()).unwrap();

                    if LOGS_REGISTRO {
                        self.add_registro(format!("Indicador KAMA agregado: {}", indicator.nombre));
                    }
                    kama(
                        &mut df,
                        Some(parametros.timeperiod),
                        Some(&indicator.nombre),
                    );
                }
                "MA" => {
                    let parametros =
                        serde_json::from_value::<MaParams>(indicator.parametros.clone()).unwrap();

                    if LOGS_REGISTRO {
                        self.add_registro(format!("Indicador MA agregado: {}", indicator.nombre));
                    }
                    ma(
                        &mut df,
                        Some(parametros.timeperiod),
                        Some(parametros.matype),
                        Some(&indicator.nombre),
                    );
                }
                "MAMA" => {
                    let parametros =
                        serde_json::from_value::<MamaParams>(indicator.parametros.clone()).unwrap();

                    if LOGS_REGISTRO {
                        self.add_registro(format!("Indicador MAMA agregado: {}", indicator.nombre));
                    }

                    mama(
                        &mut df,
                        Some(parametros.fastlimit),
                        Some(parametros.slowlimit),
                        Some(format!("{}_mama", &indicator.nombre).as_str()),
                        Some(format!("{}_fama", &indicator.nombre).as_str()),
                    );
                }
                "MIDPOINT" => {
                    let parametros =
                        serde_json::from_value::<MidpointParams>(indicator.parametros.clone())
                            .unwrap();

                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador MIDPOINT agregado: {}",
                            indicator.nombre
                        ));
                    }

                    midpoint(
                        &mut df,
                        Some(parametros.timeperiod),
                        Some(&indicator.nombre),
                    );
                }
                "MIDPRICE" => {
                    let parametros =
                        serde_json::from_value::<MidpriceParams>(indicator.parametros.clone())
                            .unwrap();

                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador MIDPRICE agregado: {}",
                            indicator.nombre
                        ));
                    }

                    midprice(
                        &mut df,
                        Some(parametros.timeperiod),
                        Some(&indicator.nombre),
                    );
                }
                "SAR" => {
                    let parametros =
                        serde_json::from_value::<SarParams>(indicator.parametros.clone()).unwrap();

                    if LOGS_REGISTRO {
                        self.add_registro(format!("Indicador SAR agregado: {}", indicator.nombre));
                    }

                    sar(
                        &mut df,
                        Some(parametros.acceleration),
                        Some(parametros.maximum),
                        Some(&indicator.nombre),
                    );
                }
                "SAREXT" => {
                    let parametros =
                        serde_json::from_value::<SarextParams>(indicator.parametros.clone())
                            .unwrap();

                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador SAREXT agregado: {}",
                            indicator.nombre
                        ));
                    }

                    sarext(
                        &mut df,
                        Some(parametros.startvalue),
                        Some(parametros.offsetonlong),
                        Some(parametros.offsetonshort),
                        Some(parametros.blockonlong),
                        Some(parametros.blockonshort),
                        Some(&indicator.nombre),
                    );
                }
                "SMA" => {
                    let parametros =
                        serde_json::from_value::<SmaParams>(indicator.parametros.clone()).unwrap();

                    if LOGS_REGISTRO {
                        self.add_registro(format!("Indicador SMA agregado: {}", indicator.nombre));
                    }

                    sma(
                        &mut df,
                        Some(parametros.timeperiod),
                        Some(&indicator.nombre),
                    );
                }
                "T3" => {
                    let parametros =
                        serde_json::from_value::<T3Params>(indicator.parametros.clone()).unwrap();

                    if LOGS_REGISTRO {
                        self.add_registro(format!("Indicador T3 agregado: {}", indicator.nombre));
                    }

                    t3(
                        &mut df,
                        Some(parametros.timeperiod),
                        Some(parametros.vfactor),
                        Some(&indicator.nombre),
                    );
                }
                "TEMA" => {
                    let parametros =
                        serde_json::from_value::<TemaParams>(indicator.parametros.clone()).unwrap();

                    if LOGS_REGISTRO {
                        self.add_registro(format!("Indicador TEMA agregado: {}", indicator.nombre));
                    }

                    tema(
                        &mut df,
                        Some(parametros.timeperiod),
                        Some(&indicator.nombre),
                    );
                }
                "TRIMA" => {
                    let parametros =
                        serde_json::from_value::<TrimaParams>(indicator.parametros.clone())
                            .unwrap();

                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador TRIMA agregado: {}",
                            indicator.nombre
                        ));
                    }

                    trima(
                        &mut df,
                        Some(parametros.timeperiod),
                        Some(&indicator.nombre),
                    );
                }
                "WMA" => {
                    let parametros =
                        serde_json::from_value::<WmaParams>(indicator.parametros.clone()).unwrap();

                    if LOGS_REGISTRO {
                        self.add_registro(format!("Indicador WMA agregado: {}", indicator.nombre));
                    }

                    wma(
                        &mut df,
                        Some(parametros.timeperiod),
                        Some(&indicator.nombre),
                    );
                }
                "CDL2CROWS" => {
                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador CDL2CROWS agregado: {}",
                            indicator.nombre
                        ));
                    }

                    cdlupsidegap2crows(&mut df, Some(&indicator.nombre));
                }
                "CDL3BLACKCROWS" => {
                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador CDL3BLACKCROWS agregado: {}",
                            indicator.nombre
                        ));
                    }

                    cdl3blackcrows(&mut df, Some(&indicator.nombre));
                }
                "CDL3INSIDE" => {
                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador CDL3INSIDE agregado: {}",
                            indicator.nombre
                        ));
                    }

                    cdl3inside(&mut df, Some(&indicator.nombre));
                }
                "CDL3LINESTRIKE" => {
                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador CDL3LINESTRIKE agregado: {}",
                            indicator.nombre
                        ));
                    }

                    cdl3linestrike(&mut df, Some(&indicator.nombre));
                }
                "CDL3OUTSIDE" => {
                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador CDL3OUTSIDE agregado: {}",
                            indicator.nombre
                        ));
                    }

                    cdl3outside(&mut df, Some(&indicator.nombre));
                }
                "CDL3STARSINSOUTH" => {
                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador CDL3STARSINSOUTH agregado: {}",
                            indicator.nombre
                        ));
                    }

                    cdl3starsinsouth(&mut df, Some(&indicator.nombre));
                }
                "CDL3WHITESOLDIERS" => {
                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador CDL3WHITESOLDIERS agregado: {}",
                            indicator.nombre
                        ));
                    }

                    cdl3whitesoldiers(&mut df, Some(&indicator.nombre));
                }
                "CDLABANDONEDBABY" => {
                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador CDLABANDONEDBABY agregado: {}",
                            indicator.nombre
                        ));
                    }

                    cdlabandonedbaby(&mut df, Some(&indicator.nombre));
                }
                "CDLADVANCEBLOCK" => {
                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador CDLADVANCEBLOCK agregado: {}",
                            indicator.nombre
                        ));
                    }

                    cdladvanceblock(&mut df, Some(&indicator.nombre));
                }
                "CDLBELTHOLD" => {
                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador CDLBELTHOLD agregado: {}",
                            indicator.nombre
                        ));
                    }

                    cdlbelthold(&mut df, Some(&indicator.nombre));
                }
                "CDLBREAKAWAY" => {
                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador CDLBREAKAWAY agregado: {}",
                            indicator.nombre
                        ));
                    }

                    cdlbreakaway(&mut df, Some(&indicator.nombre));
                }
                "CDLCLOSINGMARUBOZU" => {
                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador CDLCLOSINGMARUBOZU agregado: {}",
                            indicator.nombre
                        ));
                    }

                    cdlclosingmarubuzo(&mut df, Some(&indicator.nombre));
                }
                "CDLCONCEALBABYSWALL" => {
                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador CDLCONCEALBABYSWALL agregado: {}",
                            indicator.nombre
                        ));
                    }

                    cdlconcealbabyswall(&mut df, Some(&indicator.nombre));
                }
                "CDLCOUNTERATTACK" => {
                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador CDLCOUNTERATTACK agregado: {}",
                            indicator.nombre
                        ));
                    }

                    cdlcounterattack(&mut df, Some(&indicator.nombre));
                }
                "CDLDARKCLOUDCOVER" => {
                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador CDLDARKCLOUDCOVER agregado: {}",
                            indicator.nombre
                        ));
                    }

                    cdldarkcloudcover(&mut df, Some(&indicator.nombre));
                }
                "CDLDOJI" => {
                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador CDLDOJI agregado: {}",
                            indicator.nombre
                        ));
                    }

                    cdldoji(&mut df, Some(&indicator.nombre));
                }
                "CDLDOJISTAR" => {
                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador CDLDOJISTAR agregado: {}",
                            indicator.nombre
                        ));
                    }

                    cdldojistar(&mut df, Some(&indicator.nombre));
                }
                "CDLDRAGONFLYDOJI" => {
                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador CDLDRAGONFLYDOJI agregado: {}",
                            indicator.nombre
                        ));
                    }

                    cdldragonflydoji(&mut df, Some(&indicator.nombre));
                }
                "CDLENGULFING" => {
                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador CDLENGULFING agregado: {}",
                            indicator.nombre
                        ));
                    }

                    cdlengulfing(&mut df, Some(&indicator.nombre));
                }
                "CDLEVENINGDOJISTAR" => {
                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador CDLEVENINGDOJISTAR agregado: {}",
                            indicator.nombre
                        ));
                    }

                    cdleveningdojistar(&mut df, Some(&indicator.nombre));
                }
                "CDLEVENINGSTAR" => {
                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador CDLEVENINGSTAR agregado: {}",
                            indicator.nombre
                        ));
                    }

                    cdleveningstar(&mut df, Some(&indicator.nombre));
                }
                "CDLGAPSIDESIDEWHITE" => {
                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador CDLGAPSIDESIDEWHITE agregado: {}",
                            indicator.nombre
                        ));
                    }

                    cdlgapsidesidewhite(&mut df, Some(&indicator.nombre));
                }
                "CDLGRAVESTONEDOJI" => {
                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador CDLGRAVESTONEDOJI agregado: {}",
                            indicator.nombre
                        ));
                    }

                    cdlgravestonedoji(&mut df, Some(&indicator.nombre));
                }
                "CDLHAMMER" => {
                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador CDLHAMMER agregado: {}",
                            indicator.nombre
                        ));
                    }

                    cdlhammer(&mut df, Some(&indicator.nombre));
                }
                "CDLHANGINGMAN" => {
                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador CDLHANGINGMAN agregado: {}",
                            indicator.nombre
                        ));
                    }

                    cdlhangingman(&mut df, Some(&indicator.nombre));
                }
                "CDLHARAMI" => {
                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador CDLHARAMI agregado: {}",
                            indicator.nombre
                        ));
                    }

                    cdlharami(&mut df, Some(&indicator.nombre));
                }
                "CDLHARAMICROSS" => {
                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador CDLHARAMICROSS agregado: {}",
                            indicator.nombre
                        ));
                    }

                    cdlharamicross(&mut df, Some(&indicator.nombre));
                }
                "CDLHIGHWAVE" => {
                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador CDLHIGHWAVE agregado: {}",
                            indicator.nombre
                        ));
                    }

                    cdlhighwave(&mut df, Some(&indicator.nombre));
                }
                "CDLHIKKAKE" => {
                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador CDLHIKKAKE agregado: {}",
                            indicator.nombre
                        ));
                    }

                    cdlhikkake(&mut df, Some(&indicator.nombre));
                }
                "CDLHIKKAKEMOD" => {
                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador CDLHIKKAKEMOD agregado: {}",
                            indicator.nombre
                        ));
                    }

                    cdlhikkakemod(&mut df, Some(&indicator.nombre));
                }
                "CDLHOMINGPIGEON" => {
                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador CDLHOMINGPIGEON agregado: {}",
                            indicator.nombre
                        ));
                    }

                    cdlhomingpigeon(&mut df, Some(&indicator.nombre));
                }
                "CDLIDENTICAL3CROWS" => {
                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador CDLIDENTICAL3CROWS agregado: {}",
                            indicator.nombre
                        ));
                    }

                    cdlidentical3crows(&mut df, Some(&indicator.nombre));
                }
                "CDLINNECK" => {
                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador CDLINNECK agregado: {}",
                            indicator.nombre
                        ));
                    }

                    cdlinneck(&mut df, Some(&indicator.nombre));
                }
                "CDLINVERTEDHAMMER" => {
                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador CDLINVERTEDHAMMER agregado: {}",
                            indicator.nombre
                        ));
                    }

                    cdlinvertedhammer(&mut df, Some(&indicator.nombre));
                }
                "CDLKICKING" => {
                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador CDLKICKING agregado: {}",
                            indicator.nombre
                        ));
                    }

                    cdlkicking(&mut df, Some(&indicator.nombre));
                }
                "CDLKICKINGBYLENGTH" => {
                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador CDLKICKINGBYLENGTH agregado: {}",
                            indicator.nombre
                        ));
                    }

                    cdlkickingbylength(&mut df, Some(&indicator.nombre));
                }
                "CDLLADDERBOTTOM" => {
                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador CDLLADDERBOTTOM agregado: {}",
                            indicator.nombre
                        ));
                    }

                    cdladderbottom(&mut df, Some(&indicator.nombre));
                }
                "CDLLONGLEGGEDDOJI" => {
                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador CDLLONGLEGGEDDOJI agregado: {}",
                            indicator.nombre
                        ));
                    }

                    cdllongleggeddoji(&mut df, Some(&indicator.nombre));
                }
                "CDLLONGLINE" => {
                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador CDLLONGLINE agregado: {}",
                            indicator.nombre
                        ));
                    }

                    cdllongline(&mut df, Some(&indicator.nombre));
                }
                "CDLMARUBOZU" => {
                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador CDLMARUBOZU agregado: {}",
                            indicator.nombre
                        ));
                    }

                    cdlmarubozu(&mut df, Some(&indicator.nombre));
                }
                "CDLMATCHINGLOW" => {
                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador CDLMATCHINGLOW agregado: {}",
                            indicator.nombre
                        ));
                    }

                    cdlmatchinglow(&mut df, Some(&indicator.nombre));
                }
                "CDLMATHOLD" => {
                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador CDLMATHOLD agregado: {}",
                            indicator.nombre
                        ));
                    }

                    cdlmathold(&mut df, Some(&indicator.nombre));
                }
                "CDLMORNINGDOJISTAR" => {
                    let parametros =
                        serde_json::from_value::<MorningDojiStar>(indicator.parametros.clone())
                            .unwrap();

                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador CDLMORNINGDOJISTAR agregado: {}",
                            indicator.nombre
                        ));
                    }

                    cdlmorningdojistar(
                        &mut df,
                        Some(parametros.penetration),
                        Some(&indicator.nombre),
                    );
                }
                "CDLMORNINGSTAR" => {
                    let parametros =
                        serde_json::from_value::<MorningStar>(indicator.parametros.clone())
                            .unwrap();

                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador CDLMORNINGSTAR agregado: {}",
                            indicator.nombre
                        ));
                    }

                    cdlmorningstar(
                        &mut df,
                        Some(parametros.penetration),
                        Some(&indicator.nombre),
                    );
                }
                "CDLONNECK" => {
                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador CDLONNECK agregado: {}",
                            indicator.nombre
                        ));
                    }

                    cdlonneck(&mut df, Some(&indicator.nombre));
                }
                "CDLPIERCING" => {
                    let parametros =
                        serde_json::from_value::<Piercing>(indicator.parametros.clone()).unwrap();

                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador CDLPIERCING agregado: {}",
                            indicator.nombre
                        ));
                    }

                    cdlpiercing(
                        &mut df,
                        Some(parametros.penetration),
                        Some(&indicator.nombre),
                    );
                }
                "CDLRICKSHAWMAN" => {
                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador CDLRICKSHAWMAN agregado: {}",
                            indicator.nombre
                        ));
                    }

                    cdlrickshawman(&mut df, Some(&indicator.nombre));
                }
                "CDLRISEFALL3METHODS" => {
                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador CDLRISEFALL3METHODS agregado: {}",
                            indicator.nombre
                        ));
                    }

                    cdlrisefall3methods(&mut df, Some(&indicator.nombre));
                }
                "CDLSEPARATINGLINES" => {
                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador CDLSEPARATINGLINES agregado: {}",
                            indicator.nombre
                        ));
                    }

                    cdlseparatinglines(&mut df, Some(&indicator.nombre));
                }
                "CDLSHOOTINGSTAR" => {
                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador CDLSHOOTINGSTAR agregado: {}",
                            indicator.nombre
                        ));
                    }

                    cdlshootingstar(&mut df, Some(&indicator.nombre));
                }
                "CDLSHORTLINE" => {
                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador CDLSHORTLINE agregado: {}",
                            indicator.nombre
                        ));
                    }

                    cdlshortline(&mut df, Some(&indicator.nombre));
                }
                "CDLSPINNINGTOP" => {
                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador CDLSPINNINGTOP agregado: {}",
                            indicator.nombre
                        ));
                    }

                    cdlspinningtop(&mut df, Some(&indicator.nombre));
                }
                "CDLSTALLEDPATTERN" => {
                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador CDLSTALLEDPATTERN agregado: {}",
                            indicator.nombre
                        ));
                    }

                    cdlstalledpattern(&mut df, Some(&indicator.nombre));
                }
                "CDLSTICKSANDWICH" => {
                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador CDLSTICKSANDWICH agregado: {}",
                            indicator.nombre
                        ));
                    }

                    cdlsticksandwich(&mut df, Some(&indicator.nombre));
                }
                "CDLTAKURI" => {
                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador CDLTAKURI agregado: {}",
                            indicator.nombre
                        ));
                    }

                    cdltakuri(&mut df, Some(&indicator.nombre));
                }
                "CDLTASUKIGAP" => {
                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador CDLTASUKIGAP agregado: {}",
                            indicator.nombre
                        ));
                    }

                    cdltasukigap(&mut df, Some(&indicator.nombre));
                }
                "CDLTHRUSTING" => {
                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador CDLTHRUSTING agregado: {}",
                            indicator.nombre
                        ));
                    }

                    cdlthrusting(&mut df, Some(&indicator.nombre));
                }
                "CDLTRISTAR" => {
                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador CDLTRISTAR agregado: {}",
                            indicator.nombre
                        ));
                    }

                    cdltristar(&mut df, Some(&indicator.nombre));
                }
                "CDLUNIQUE3RIVER" => {
                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador CDLUNIQUE3RIVER agregado: {}",
                            indicator.nombre
                        ));
                    }

                    cdlunique3river(&mut df, Some(&indicator.nombre));
                }
                "CDLUPSIDEGAP2CROWS" => {
                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador CDLUPSIDEGAP2CROWS agregado: {}",
                            indicator.nombre
                        ));
                    }

                    cdlupsidegap2crows(&mut df, Some(&indicator.nombre));
                }
                "CDLXSIDEGAP3METHODS" => {
                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador CDLXSIDEGAP3METHODS agregado: {}",
                            indicator.nombre
                        ));
                    }

                    cdlxsidegap3methods(&mut df, Some(&indicator.nombre));
                }
                "ADX" => {
                    let params: AdxParams =
                        serde_json::from_value::<AdxParams>(indicator.parametros.clone()).unwrap();

                    if LOGS_REGISTRO {
                        self.add_registro(format!("Indicador ADX agregado: {}", indicator.nombre));
                    }

                    adx(&mut df, Some(params.timeperiod), Some(&indicator.nombre));
                }
                "ADXR" => {
                    let params: AdxrParams =
                        serde_json::from_value::<AdxrParams>(indicator.parametros.clone()).unwrap();

                    if LOGS_REGISTRO {
                        self.add_registro(format!("Indicador ADXR agregado: {}", indicator.nombre));
                    }

                    adxr(&mut df, Some(params.timeperiod), Some(&indicator.nombre));
                }
                "APO" => {
                    let params: ApoParams =
                        serde_json::from_value::<ApoParams>(indicator.parametros.clone()).unwrap();

                    if LOGS_REGISTRO {
                        self.add_registro(format!("Indicador APO agregado: {}", indicator.nombre));
                    }

                    apo(
                        &mut df,
                        Some(params.fastperiod),
                        Some(params.slowperiod),
                        Some(&indicator.nombre),
                    );
                }
                "AROON" => {
                    let params: AroonParams =
                        serde_json::from_value::<AroonParams>(indicator.parametros.clone())
                            .unwrap();

                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador AROON agregado: {}",
                            indicator.nombre
                        ));
                    }
                    aroon(
                        &mut df,
                        Some(params.timeperiod),
                        Some(format!("{}_col_up", &indicator.nombre).as_str()),
                        Some(format!("{}_col_down", &indicator.nombre).as_str()),
                    );
                }
                "AROONOSC" => {
                    let params: AroonoscParams =
                        serde_json::from_value::<AroonoscParams>(indicator.parametros.clone())
                            .unwrap();

                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador AROONOSC agregado: {}",
                            indicator.nombre
                        ));
                    }

                    aroonosc(&mut df, Some(params.timeperiod), Some(&indicator.nombre));
                }
                "BOP" => {
                    let params: BopParams =
                        serde_json::from_value::<BopParams>(indicator.parametros.clone()).unwrap();

                    if LOGS_REGISTRO {
                        self.add_registro(format!("Indicador BOP agregado: {}", indicator.nombre));
                    }

                    bop(&mut df, Some(params.timeperiod), Some(&indicator.nombre));
                }
                "CCI" => {
                    let params: CciParams =
                        serde_json::from_value::<CciParams>(indicator.parametros.clone()).unwrap();

                    if LOGS_REGISTRO {
                        self.add_registro(format!("Indicador CCI agregado: {}", indicator.nombre));
                    }

                    cci(&mut df, Some(params.timeperiod), Some(&indicator.nombre));
                }
                "CMO" => {
                    let params: CmoParams =
                        serde_json::from_value::<CmoParams>(indicator.parametros.clone()).unwrap();

                    if LOGS_REGISTRO {
                        self.add_registro(format!("Indicador CMO agregado: {}", indicator.nombre));
                    }

                    cmo(&mut df, Some(params.timeperiod), Some(&indicator.nombre));
                }
                "DX" => {
                    let params: DxParams =
                        serde_json::from_value::<DxParams>(indicator.parametros.clone()).unwrap();

                    if LOGS_REGISTRO {
                        self.add_registro(format!("Indicador DX agregado: {}", indicator.nombre));
                    }

                    dx(&mut df, Some(params.timeperiod), Some(&indicator.nombre));
                }
                "MACD" => {
                    let params: MacdParams =
                        serde_json::from_value::<MacdParams>(indicator.parametros.clone()).unwrap();

                    if LOGS_REGISTRO {
                        self.add_registro(format!("Indicador MACD agregado: {}", indicator.nombre));
                    }

                    macd(
                        &mut df,
                        Some(params.timeperiod),
                        Some(params.slowperiod),
                        Some(params.signalperiod),
                        Some(&indicator.nombre),
                        Some(format!("{}_col_signal", &indicator.nombre).as_str()),
                        Some(format!("{}_col_hist", &indicator.nombre).as_str()),
                    );
                }
                "MACDEXT" => {
                    let params: MacdextParams =
                        serde_json::from_value::<MacdextParams>(indicator.parametros.clone())
                            .unwrap();

                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador MACDEXT agregado: {}",
                            indicator.nombre
                        ));
                    }

                    macdext(
                        &mut df,
                        Some(params.fastperiod),
                        Some(params.slowperiod),
                        Some(params.signalperiod),
                        Some(params.fastmatype),
                        Some(params.slowmatype),
                        Some(params.signalmatype),
                        Some(&indicator.nombre),
                        Some(format!("{}_col_signal", &indicator.nombre).as_str()),
                        Some(format!("{}_col_hist", &indicator.nombre).as_str()),
                    );
                }
                "MACDFIX" => {
                    let params: MacdfixParams =
                        serde_json::from_value::<MacdfixParams>(indicator.parametros.clone())
                            .unwrap();

                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador MAC&mut dfIX agregado: {}",
                            indicator.nombre
                        ));
                    }

                    macdfix(
                        &mut df,
                        Some(params.signalperiod),
                        Some(&indicator.nombre),
                        Some(format!("{}_col_signal", &indicator.nombre).as_str()),
                        Some(format!("{}_col_hist", &indicator.nombre).as_str()),
                    );
                }
                "MFI" => {
                    let params: MfiParams =
                        serde_json::from_value::<MfiParams>(indicator.parametros.clone()).unwrap();

                    if LOGS_REGISTRO {
                        self.add_registro(format!("Indicador MFI agregado: {}", indicator.nombre));
                    }

                    mfi(&mut df, Some(params.timeperiod), Some(&indicator.nombre));
                }
                "MINUS_DI" => {
                    let params: MinusDiParams =
                        serde_json::from_value::<MinusDiParams>(indicator.parametros.clone())
                            .unwrap();

                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador MINUS_DI agregado: {}",
                            indicator.nombre
                        ));
                    }

                    minus_di(&mut df, Some(params.timeperiod), Some(&indicator.nombre));
                }
                "MINUS_DM" => {
                    let params: MinusDmParams =
                        serde_json::from_value::<MinusDmParams>(indicator.parametros.clone())
                            .unwrap();

                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador MINUS_DM agregado: {}",
                            indicator.nombre
                        ));
                    }

                    minus_dm(&mut df, Some(params.timeperiod), Some(&indicator.nombre));
                }
                "MOM" => {
                    let params: MomParams =
                        serde_json::from_value::<MomParams>(indicator.parametros.clone()).unwrap();

                    if LOGS_REGISTRO {
                        self.add_registro(format!("Indicador MOM agregado: {}", indicator.nombre));
                    }

                    mom(&mut df, Some(params.timeperiod), Some(&indicator.nombre));
                }
                "PLUS_DI" => {
                    let params: PlusDiParams =
                        serde_json::from_value::<PlusDiParams>(indicator.parametros.clone())
                            .unwrap();

                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador PLUS_DI agregado: {}",
                            indicator.nombre
                        ));
                    }

                    plus_di(&mut df, Some(params.timeperiod), Some(&indicator.nombre));
                }
                "PLUS_DM" => {
                    let params: PlusDmParams =
                        serde_json::from_value::<PlusDmParams>(indicator.parametros.clone())
                            .unwrap();

                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador PLUS_DM agregado: {}",
                            indicator.nombre
                        ));
                    }

                    plus_dm(&mut df, Some(params.timeperiod), Some(&indicator.nombre));
                }
                "PPO" => {
                    let params: PpoParams =
                        serde_json::from_value::<PpoParams>(indicator.parametros.clone()).unwrap();

                    if LOGS_REGISTRO {
                        self.add_registro(format!("Indicador PPO agregado: {}", indicator.nombre));
                    }

                    ppo(
                        &mut df,
                        Some(params.fastperiod),
                        Some(params.slowperiod),
                        Some(&indicator.nombre),
                    );
                }
                "ROC" => {
                    let params: RocParams =
                        serde_json::from_value::<RocParams>(indicator.parametros.clone()).unwrap();

                    if LOGS_REGISTRO {
                        self.add_registro(format!("Indicador ROC agregado: {}", indicator.nombre));
                    }

                    roc(&mut df, Some(params.timeperiod), Some(&indicator.nombre));
                }
                "ROCP" => {
                    let params: RocpParams =
                        serde_json::from_value::<RocpParams>(indicator.parametros.clone()).unwrap();

                    if LOGS_REGISTRO {
                        self.add_registro(format!("Indicador ROCP agregado: {}", indicator.nombre));
                    }

                    rocp(&mut df, Some(params.timeperiod), Some(&indicator.nombre));
                }
                "ROCR" => {
                    let params: RocrParams =
                        serde_json::from_value::<RocrParams>(indicator.parametros.clone()).unwrap();

                    if LOGS_REGISTRO {
                        self.add_registro(format!("Indicador ROCR agregado: {}", indicator.nombre));
                    }

                    rocr(&mut df, Some(params.timeperiod), Some(&indicator.nombre));
                }
                "ROCR100" => {
                    let params: Roc100Params =
                        serde_json::from_value::<Roc100Params>(indicator.parametros.clone())
                            .unwrap();

                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador ROCR100 agregado: {}",
                            indicator.nombre
                        ));
                    }

                    rocr100(&mut df, Some(params.timeperiod), Some(&indicator.nombre));
                }
                "RSI" => {
                    let params: RsiParams =
                        serde_json::from_value::<RsiParams>(indicator.parametros.clone()).unwrap();

                    if LOGS_REGISTRO {
                        self.add_registro(format!("Indicador RSI agregado: {}", indicator.nombre));
                    }

                    rsi(&mut df, Some(params.timeperiod), Some(&indicator.nombre));
                }
                "STOCH" => {
                    let params: StochParams =
                        serde_json::from_value::<StochParams>(indicator.parametros.clone())
                            .unwrap();

                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador STOCH agregado: {}",
                            indicator.nombre
                        ));
                    }

                    stoch(
                        &mut df,
                        Some(params.fastk_period),
                        Some(params.slowk_period),
                        Some(params.slowk_matype),
                        Some(params.slowd_period),
                        Some(format!("{}_col_k", &indicator.nombre).as_str()),
                        Some(format!("{}_col_d", &indicator.nombre).as_str()),
                    );
                }
                "STOCHF" => {
                    let params: StochfParams =
                        serde_json::from_value::<StochfParams>(indicator.parametros.clone())
                            .unwrap();

                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador STOCHF agregado: {}",
                            indicator.nombre
                        ));
                    }

                    stochf(
                        &mut df,
                        Some(params.fastk_period),
                        Some(params.fastd_period),
                        Some(params.fastd_matype),
                        Some(format!("{}_col_k", &indicator.nombre).as_str()),
                        Some(format!("{}_col_d", &indicator.nombre).as_str()),
                    );
                }
                "STOCHRSI" => {
                    let params: StochRsiParams =
                        serde_json::from_value::<StochRsiParams>(indicator.parametros.clone())
                            .unwrap();

                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador STOCHRSI agregado: {}",
                            indicator.nombre
                        ));
                    }

                    stochrsi(
                        &mut df,
                        Some(params.timeperiod),
                        Some(params.fastk_period),
                        Some(params.fastd_period),
                        Some(params.fastd_matype),
                        Some(format!("{}_col_k", &indicator.nombre).as_str()),
                        Some(format!("{}_col_d", &indicator.nombre).as_str()),
                    );
                }
                "TRIX" => {
                    let params: TrixParams =
                        serde_json::from_value::<TrixParams>(indicator.parametros.clone()).unwrap();

                    if LOGS_REGISTRO {
                        self.add_registro(format!("Indicador TRIX agregado: {}", indicator.nombre));
                    }

                    trix(&mut df, Some(params.timeperiod), Some(&indicator.nombre));
                }
                "ULTOSC" => {
                    let params: UltoscParams =
                        serde_json::from_value::<UltoscParams>(indicator.parametros.clone())
                            .unwrap();

                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador ULTOSC agregado: {}",
                            indicator.nombre
                        ));
                    }

                    ultosc(
                        &mut df,
                        Some(params.timeperiod1),
                        Some(params.timeperiod2),
                        Some(params.timeperiod3),
                        Some(&indicator.nombre),
                    );
                }
                "WILLR" => {
                    let params: WillrParams =
                        serde_json::from_value::<WillrParams>(indicator.parametros.clone())
                            .unwrap();

                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador WILLR agregado: {}",
                            indicator.nombre
                        ));
                    }

                    willr(&mut df, Some(params.timeperiod), Some(&indicator.nombre));
                }
                "AVGPRICE" => {
                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador AVGPRICE agregado: {}",
                            indicator.nombre
                        ));
                    }

                    avgprice(&mut df, Some(&indicator.nombre));
                }
                "MEDPRICE" => {
                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador MEDPRICE agregado: {}",
                            indicator.nombre
                        ));
                    }

                    medprice(&mut df, Some(&indicator.nombre));
                }
                "TYPPRICE" => {
                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador TYPPRICE agregado: {}",
                            indicator.nombre
                        ));
                    }

                    typprice(&mut df, Some(&indicator.nombre));
                }
                "WCLPRICE" => {
                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador WCLPRICE agregado: {}",
                            indicator.nombre
                        ));
                    }

                    wclprice(&mut df, Some(&indicator.nombre));
                }
                "BETA" => {
                    let params: BetaParams =
                        serde_json::from_value::<BetaParams>(indicator.parametros.clone()).unwrap();

                    if LOGS_REGISTRO {
                        self.add_registro(format!("Indicador BETA agregado: {}", indicator.nombre));
                    }

                    beta(
                        &mut df,
                        &params.col_real0,
                        &params.col_real1,
                        Some(params.timeperiod),
                        Some(&indicator.nombre),
                    );
                }
                "CORREL" => {
                    let params: CorrelParams =
                        serde_json::from_value::<CorrelParams>(indicator.parametros.clone())
                            .unwrap();

                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador CORREL agregado: {}",
                            indicator.nombre
                        ));
                    }

                    correl(
                        &mut df,
                        &params.col_real0,
                        &params.col_real1,
                        Some(params.timeperiod),
                        Some(&indicator.nombre),
                    );
                }
                "LINEARREG" => {
                    let params: LinearRegParams =
                        serde_json::from_value::<LinearRegParams>(indicator.parametros.clone())
                            .unwrap();

                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador LINEARREG agregado: {}",
                            indicator.nombre
                        ));
                    }

                    linearreg(
                        &mut df,
                        &params.col_real,
                        Some(params.timeperiod),
                        Some(&indicator.nombre),
                    );
                }
                "LINEARREG_ANGLE" => {
                    let params: LinearRegAngleParams =
                        serde_json::from_value::<LinearRegAngleParams>(
                            indicator.parametros.clone(),
                        )
                        .unwrap();

                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador LINEARREG_ANGLE agregado: {}",
                            indicator.nombre
                        ));
                    }

                    linearreg_angle(
                        &mut df,
                        &params.col_real,
                        Some(params.timeperiod),
                        Some(&indicator.nombre),
                    );
                }
                "LINEARREG_INTERCEPT" => {
                    let params: LinearRegInterceptParams =
                        serde_json::from_value::<LinearRegInterceptParams>(
                            indicator.parametros.clone(),
                        )
                        .unwrap();

                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador LINEARREG_INTERCEPT agregado: {}",
                            indicator.nombre
                        ));
                    }

                    linearreg_intercept(
                        &mut df,
                        &params.col_real,
                        Some(params.timeperiod),
                        Some(&indicator.nombre),
                    );
                }
                "LINEARREG_SLOPE" => {
                    let params: LinearRegSlopeParams =
                        serde_json::from_value::<LinearRegSlopeParams>(
                            indicator.parametros.clone(),
                        )
                        .unwrap();

                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador LINEARREG_SLOPE agregado: {}",
                            indicator.nombre
                        ));
                    }

                    linearreg_slope(
                        &mut df,
                        &params.col_real,
                        Some(params.timeperiod),
                        Some(&indicator.nombre),
                    );
                }
                "STDDEV" => {
                    let params: StddevParams =
                        serde_json::from_value::<StddevParams>(indicator.parametros.clone())
                            .unwrap();

                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador STDDEV agregado: {}",
                            indicator.nombre
                        ));
                    }

                    stddev(
                        &mut df,
                        &params.col_real,
                        Some(params.timeperiod),
                        Some(params.nbdev),
                        Some(&indicator.nombre),
                    );
                }
                "TSF" => {
                    let params: TsfParams =
                        serde_json::from_value::<TsfParams>(indicator.parametros.clone()).unwrap();

                    if LOGS_REGISTRO {
                        self.add_registro(format!("Indicador TSF agregado: {}", indicator.nombre));
                    }

                    tsf(
                        &mut df,
                        &params.col_real,
                        Some(params.timeperiod),
                        Some(&indicator.nombre),
                    );
                }
                "VAR" => {
                    let params: VarParams =
                        serde_json::from_value::<VarParams>(indicator.parametros.clone()).unwrap();

                    if LOGS_REGISTRO {
                        self.add_registro(format!("Indicador VAR agregado: {}", indicator.nombre));
                    }

                    var(
                        &mut df,
                        &params.col_real,
                        Some(params.timeperiod),
                        Some(params.nbdev),
                        Some(&indicator.nombre),
                    );
                }
                "TRANGE" => {
                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador TRANGE agregado: {}",
                            indicator.nombre
                        ));
                    }
                    trange(&mut df, Some(&indicator.nombre));
                }
                "ATR" => {
                    let params: AtrParams =
                        serde_json::from_value::<AtrParams>(indicator.parametros.clone()).unwrap();

                    if LOGS_REGISTRO {
                        self.add_registro(format!("Indicador ATR agregado: {}", indicator.nombre));
                    }

                    atr(
                        &mut df,
                        Some(params.timeperiod),
                        Some(params.multiplier),
                        Some(&indicator.nombre),
                    );
                }
                "NATR" => {
                    let params: NatrParams =
                        serde_json::from_value::<NatrParams>(indicator.parametros.clone()).unwrap();

                    if LOGS_REGISTRO {
                        self.add_registro(format!("Indicador NATR agregado: {}", indicator.nombre));
                    }

                    natr(&mut df, Some(params.timeperiod), Some(&indicator.nombre));
                }
                "AD" => {
                    if LOGS_REGISTRO {
                        self.add_registro(format!("Indicador AD agregado: {}", indicator.nombre));
                    }

                    ad(&mut df, Some(&indicator.nombre));
                }
                "ADOSC" => {
                    let params: AdoscParams =
                        serde_json::from_value::<AdoscParams>(indicator.parametros.clone())
                            .unwrap();

                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Indicador ADOSC agregado: {}",
                            indicator.nombre
                        ));
                    }

                    adosc(
                        &mut df,
                        Some(params.fastperiod),
                        Some(params.slowperiod),
                        Some(&indicator.nombre),
                    );
                }
                "OBV" => {
                    if LOGS_REGISTRO {
                        self.add_registro(format!("Indicador OBV agregado: {}", indicator.nombre));
                    }

                    obv(&mut df, Some(&indicator.nombre));
                }
                _ => {
                    if LOGS_REGISTRO {
                        self.add_registro(format!("El indicador no existe."));
                    }
                }
            }
        }
    }

    /// Confirma si se puede operar en una dirección de compra o venta.
    ///
    /// # Parametro
    /// tipo: Si es buy o sell.
    ///
    /// # Retorna
    /// True si se puede operar en la dirección indicada, false en caso contrario.
    fn verificar_direccion(&mut self, tipo: EntryDirection) -> bool {
        match tipo {
            EntryDirection::Buy => {
                if LOGS_REGISTRO {
                    self.add_registro(format!("Verificando: Buy"));
                }
                if self.estrategia.opciones.trading_direccion == TradingDirection::Long
                    || self.estrategia.opciones.trading_direccion == TradingDirection::Both
                {
                    if LOGS_REGISTRO {
                        self.add_registro(format!("Direccion de Buy: OK"));
                    }
                    true
                } else {
                    false
                }
            }
            EntryDirection::Sell => {
                if LOGS_REGISTRO {
                    self.add_registro(format!("Verificando: Sell"));
                }
                if self.estrategia.opciones.trading_direccion == TradingDirection::Short
                    || self.estrategia.opciones.trading_direccion == TradingDirection::Both
                {
                    if LOGS_REGISTRO {
                        self.add_registro(format!("Direccion de Sell: OK"));
                    }
                    true
                } else {
                    false
                }
            }
        }
    }

    /// Verifica las condiciones de una acción de estrategia.
    ///
    /// # Arguments
    ///
    /// * `df` - DataFrame con los datos de la estrategia.
    /// * `accion` - Acción de estrategia a verificar.
    /// * `i` - Índice del DataFrame.
    ///
    /// # Returns
    ///
    /// `true` si las condiciones se cumplen, `false` en caso contrario.
    fn check_conditions(
        &mut self,
        df: &DataFrame,
        condiciones: &StrategyCondition,
        i: &usize,
    ) -> bool {
        let check_condition = |campo_a: f64, campo_b: f64, operador: &str| match operador {
            ">" => campo_a > campo_b,
            "<" => campo_a < campo_b,
            ">=" => campo_a >= campo_b,
            "<=" => campo_a <= campo_b,
            "==" => campo_a == campo_b,
            "!=" => campo_a != campo_b,
            _ => false,
        };

        if (i.clone() as i32 - condiciones.shift_b) >= 0
            && (i.clone() as i32 - condiciones.shift_a) >= 0
        {
            let campo_a = df
                .column(&condiciones.campo_a)
                .unwrap()
                .f64()
                .unwrap()
                .get(i - condiciones.shift_a as usize)
                .unwrap_or(0.0);

            let campo_b = df
                .column(&condiciones.campo_b)
                .unwrap()
                .f64()
                .unwrap()
                .get(i - condiciones.shift_b as usize)
                .unwrap_or(0.0);

            if let Some(next_condition) = &condiciones.next_condition {
                let result = self.check_conditions(df, &*next_condition, i);

                let resultado = check_condition(campo_a, campo_b, &condiciones.operador);

                match condiciones.logica {
                    Some(Logic::AND) => {
                        if resultado && result {
                            if LOGS_REGISTRO {
                                self.add_registro(format!(
                                    "La condición {} se cumple",
                                    condiciones.logica.unwrap().to_string()
                                ));
                            }
                            return true;
                        } else {
                            if LOGS_REGISTRO {
                                self.add_registro(format!(
                                    "La condición {} no se cumple",
                                    condiciones.logica.unwrap().to_string()
                                ));
                            }
                            return false;
                        }
                    }
                    Some(Logic::OR) => {
                        if resultado || result {
                            if LOGS_REGISTRO {
                                self.add_registro(format!(
                                    "La condición {} se cumple",
                                    condiciones.logica.unwrap().to_string()
                                ));
                            }
                            return true;
                        } else {
                            if LOGS_REGISTRO {
                                self.add_registro(format!(
                                    "La condición {} no se cumple",
                                    condiciones.logica.unwrap().to_string()
                                ));
                            }
                            return false;
                        }
                    }
                    _ => resultado,
                }
            } else {
                let resultado = check_condition(campo_a, campo_b, &condiciones.operador);

                if !resultado {
                    if LOGS_REGISTRO {
                        self.add_registro(format!("La condición no se cumple",));
                    }
                    return false;
                } else {
                    if LOGS_REGISTRO {
                        self.add_registro(format!("La condición se cumple",));
                    }
                    return true;
                }
            }
        } else {
            if LOGS_REGISTRO {
                self.add_registro(format!(
                    "El indice {} del shift es incorrecto",
                    (i.clone() as i32 - condiciones.shift_b)
                ));
            }
            return false;
        }
    }

    /// Comprueba las opciones de entrada de una acción en un índice dado.
    ///
    /// # Parametros
    /// open_trades: Vector con los trades abiertos.
    ///
    /// # Retorna
    /// True si se puede operar en la dirección indicada, false en caso contrario.
    fn entry_options(&mut self, open_trades: &Vec<Trade>) -> bool {
        let entry_options: bool;

        if LOGS_REGISTRO {
            self.add_registro(format!("Comprobando opciones de entrada"));
        }

        if !self.estrategia.opciones.multiples_trades && !open_trades.is_empty() {
            entry_options = false;
            if LOGS_REGISTRO {
                self.add_registro(format!("No se puede operar con mas de un trade"));
            }
        } else {
            entry_options = true;
            if LOGS_REGISTRO {
                self.add_registro(format!("Se puede operar con mas de un trade"));
            }
        }
        entry_options
    }

    /// Obtiene el límite de una acción en un índice dado.
    ///
    /// # Parametros
    /// df: DataFrame con los datos del índice.
    /// params: String con los parametros del limite.
    /// i: Indice del limite a obtener.
    ///
    /// # Retorna
    /// El límite de la acción en el índice dado.
    fn get_limit(
        &mut self,
        df: DataFrame,
        params: String,
        i: usize,
    ) -> Result<f64, serde_json::Error> {
        if LOGS_REGISTRO {
            self.add_registro("Obteniendo limite con la función get_limit".to_string());
        }

        let params: LimitParams = serde_json::from_str(&params).unwrap();
        if LOGS_REGISTRO {
            self.add_registro(format!("Parametros de la limitada {:?}", params));
        }

        let mut valor: f64 = 0.0;

        if (i as i32 - params.shift.clone() as i32) >= 0 {
            valor = df
                .column(&params.nombre_col)
                .unwrap()
                .f64()
                .unwrap()
                .get(i - params.shift)
                .unwrap_or(0.0);
        }

        let limit: f64 = match params.tipo.as_str() {
            "pip" => {
                if params.direccion == EntryDirection::Buy {
                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Obteniendo limite de compra por pip: {}",
                            valor + params.valor
                        ));
                    }
                    valor + params.valor
                } else if params.direccion == EntryDirection::Sell {
                    if valor - params.valor >= 0.00000 {
                        if LOGS_REGISTRO {
                            self.add_registro(format!(
                                "Obteniendo limite de venta por pip: {}",
                                valor - params.valor
                            ));
                        }
                        valor - params.valor
                    } else {
                        if LOGS_REGISTRO {
                            self.add_registro(format!("Limite por pip no valido"));
                        }
                        0.0
                    }
                } else {
                    if LOGS_REGISTRO {
                        self.add_registro(format!("Limite por pip no valido"));
                    }
                    0.0
                }
            }
            "tick" => {
                if params.direccion == EntryDirection::Buy {
                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Obteniendo limite de compra por tick: {}",
                            valor + params.valor
                        ));
                    }
                    valor + params.valor
                } else if params.direccion == EntryDirection::Sell {
                    if valor - params.valor >= 0.00000 {
                        if LOGS_REGISTRO {
                            self.add_registro(format!(
                                "Obteniendo limite de venta por tick: {}",
                                valor - params.valor
                            ));
                        }
                        valor - params.valor
                    } else {
                        if LOGS_REGISTRO {
                            self.add_registro(format!("Limite por tick no valido"));
                        }
                        0.0
                    }
                } else {
                    if LOGS_REGISTRO {
                        self.add_registro(format!("Limite por tick no valido"));
                    }
                    0.0
                }
            }
            "punto" => {
                if params.direccion == EntryDirection::Buy {
                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Obteniendo limite de compra por punto: {}",
                            valor + params.valor
                        ));
                    }
                    valor + params.valor
                } else if params.direccion == EntryDirection::Sell {
                    if valor - params.valor >= 0.00000 {
                        if LOGS_REGISTRO {
                            self.add_registro(format!(
                                "Obteniendo limite de venta por punto: {}",
                                valor - params.valor
                            ));
                        }
                        valor - params.valor
                    } else {
                        if LOGS_REGISTRO {
                            self.add_registro(format!("Limite por punto no valido"));
                        }
                        0.0
                    }
                } else {
                    if LOGS_REGISTRO {
                        self.add_registro(format!("Limite por punto no valido"));
                    }
                    0.0
                }
            }
            "porcentaje" => {
                if params.direccion == EntryDirection::Buy {
                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Obteniendo limite de compra por porcentaje: {}",
                            valor + (valor * params.valor)
                        ));
                    }
                    valor + (valor * params.valor)
                } else if params.direccion == EntryDirection::Sell {
                    if valor - (valor * params.valor) >= 0.00000 {
                        if LOGS_REGISTRO {
                            self.add_registro(format!(
                                "Obteniendo limite de venta por porcentaje: {}",
                                valor - (valor * params.valor)
                            ));
                        }
                        valor - (valor * params.valor)
                    } else {
                        if LOGS_REGISTRO {
                            self.add_registro(format!("Limite por porcentaje no valido"));
                        }
                        0.0
                    }
                } else {
                    if LOGS_REGISTRO {
                        self.add_registro(format!("Limite por porcentaje no valido"));
                    }
                    0.0
                }
            }
            "atr" => {
                if params.direccion == EntryDirection::Buy {
                    if LOGS_REGISTRO {
                        self.add_registro(format!(
                            "Obteniendo limite de compra por atr: {}",
                            valor + params.valor
                        ));
                    }
                    valor + params.valor
                } else if params.direccion == EntryDirection::Sell {
                    if valor - params.valor >= 0.00000 {
                        if LOGS_REGISTRO {
                            self.add_registro(format!(
                                "Obteniendo limite de venta por atr: {}",
                                valor - params.valor
                            ));
                        }
                        valor - params.valor
                    } else {
                        if LOGS_REGISTRO {
                            self.add_registro(format!("Limite por atr no valido"));
                        }
                        0.0
                    }
                } else {
                    if LOGS_REGISTRO {
                        self.add_registro(format!("Limite por atr no valido"));
                    }
                    0.0
                }
            }
            _ => valor,
        };

        Ok(limit)
    }

    /// Obtiene el stop loss para una operación en un índice dado.
    ///
    /// # Arguments
    ///
    /// * `df` - El DataFrame con los datos de la operación.
    /// * `i` - El índice en el DataFrame.
    ///
    /// # Returns
    ///
    /// El valor del stop loss.
    fn get_stoploss(
        &mut self,
        precio_entrada: f64,
        df: DataFrame,
        i: usize,
        direccion: EntryDirection,
    ) -> f64 {
        if LOGS_REGISTRO {
            self.add_registro(format!("Obteniendo stop loss"));
        }

        let params = &self
            .estrategia
            .opciones
            .parametros_stoploss
            .clone()
            .unwrap();

        if LOGS_REGISTRO {
            self.add_registro(format!(
                "Obteniendo los parametros de stop loss: {:?}",
                params
            ));
        }

        let mut valor: f64 = 0.0;

        if (i as i32 - params.shift.clone() as i32) >= 0 {
            valor = df
                .column(&params.nombre_col)
                .unwrap()
                .get(i - &params.shift)
                .unwrap()
                .try_extract::<f64>()
                .unwrap_or(0.0);

            if LOGS_REGISTRO {
                self.add_registro(format!(
                    "Valor optenido para calcular el stop loss: {}",
                    valor
                ));
            }

            let sl: f64 = match params.tipo.as_str() {
                "pip" => {
                    if direccion == EntryDirection::Buy {
                        if valor - params.valor >= 0.00000 {
                            if LOGS_REGISTRO {
                                self.add_registro(format!(
                                    "Obteniendo stop loss por pip: {}",
                                    valor - params.valor
                                ));
                            }
                            valor - params.valor
                        } else {
                            if LOGS_REGISTRO {
                                self.add_registro(format!("stop loss por pip no valido"));
                            }
                            0.0
                        }
                    } else if direccion == EntryDirection::Sell {
                        if LOGS_REGISTRO {
                            self.add_registro(format!(
                                "Obteniendo stop loss por pip: {}",
                                valor + params.valor
                            ));
                        }
                        valor + params.valor
                    } else {
                        if LOGS_REGISTRO {
                            self.add_registro(format!("stop loss por pip no valido"));
                        }
                        0.0
                    }
                }
                "tick" => {
                    if direccion == EntryDirection::Buy {
                        if valor - params.valor >= 0.00000 {
                            if LOGS_REGISTRO {
                                self.add_registro(format!(
                                    "Obteniendo stop loss por tick: {}",
                                    valor - params.valor
                                ));
                            }
                            valor - params.valor
                        } else {
                            if LOGS_REGISTRO {
                                self.add_registro(format!("stop loss por tick no valido"));
                            }
                            0.0
                        }
                    } else if direccion == EntryDirection::Sell {
                        if LOGS_REGISTRO {
                            self.add_registro(format!(
                                "Obteniendo stop loss por tick: {}",
                                valor + params.valor
                            ));
                        }
                        valor + params.valor
                    } else {
                        if LOGS_REGISTRO {
                            self.add_registro(format!("stop loss por tick no valido"));
                        }
                        0.0
                    }
                }
                "punto" => {
                    if direccion == EntryDirection::Buy {
                        if valor - params.valor >= 0.00000 {
                            if LOGS_REGISTRO {
                                self.add_registro(format!(
                                    "Obteniendo stop loss por punto: {}",
                                    valor - params.valor
                                ));
                            }
                            valor - params.valor
                        } else {
                            if LOGS_REGISTRO {
                                self.add_registro(format!("stop loss por punto no valido"));
                            }
                            0.0
                        }
                    } else if direccion == EntryDirection::Sell {
                        if LOGS_REGISTRO {
                            self.add_registro(format!(
                                "Obteniendo stop loss por punto: {}",
                                valor + params.valor
                            ));
                        }
                        valor + params.valor
                    } else {
                        if LOGS_REGISTRO {
                            self.add_registro(format!("stop loss por punto no valido"));
                        }
                        0.0
                    }
                }
                "porcentaje" => {
                    if direccion == EntryDirection::Buy {
                        if valor - (valor * params.valor) >= 0.00000 {
                            if LOGS_REGISTRO {
                                self.add_registro(format!(
                                    "Obteniendo stop loss por porcentaje: {}",
                                    valor - (valor * params.valor)
                                ));
                            }
                            valor - (valor * params.valor)
                        } else {
                            if LOGS_REGISTRO {
                                self.add_registro(format!("stop loss por porcentaje no valido"));
                            }
                            0.0
                        }
                    } else if direccion == EntryDirection::Sell {
                        if LOGS_REGISTRO {
                            self.add_registro(format!(
                                "Obteniendo stop loss por porcentaje: {}",
                                valor + (valor * params.valor)
                            ));
                        }
                        valor + (valor * params.valor)
                    } else {
                        if LOGS_REGISTRO {
                            self.add_registro(format!("stop loss por porcentaje no valido"));
                        }
                        0.0
                    }
                }
                "atr" => {
                    if direccion == EntryDirection::Buy {
                        if precio_entrada - valor >= 0.00000 {
                            if LOGS_REGISTRO {
                                self.add_registro(format!(
                                    "Obteniendo stop loss por atr: {}",
                                    precio_entrada - valor
                                ));
                            }
                            precio_entrada - valor
                        } else {
                            if LOGS_REGISTRO {
                                self.add_registro(format!("stop loss por atr no valido"));
                            }
                            0.0
                        }
                    } else if direccion == EntryDirection::Sell {
                        if LOGS_REGISTRO {
                            self.add_registro(format!(
                                "Obteniendo stop loss por atr: {}",
                                precio_entrada + valor
                            ));
                        }
                        precio_entrada + valor
                    } else {
                        if LOGS_REGISTRO {
                            self.add_registro(format!("stop loss por atr no valido"));
                        }
                        0.0
                    }
                }
                _ => valor,
            };
            return sl;
        }

        valor
    }

    /// Obtiene el take profit para una operación en un índice dado.
    ///
    /// # Arguments
    ///
    /// * `df` - El DataFrame con los datos de la operación.
    /// * `i` - El índice en el DataFrame.
    ///
    /// # Returns
    ///
    /// El valor del take profit.
    fn get_takeprofit(
        &mut self,
        precio_entrada: f64,
        df: DataFrame,
        i: usize,
        direccion: EntryDirection,
    ) -> f64 {
        if LOGS_REGISTRO {
            self.add_registro(format!("Obteniendo take profit"));
        }

        let params = &self
            .estrategia
            .opciones
            .parametros_takeprofit
            .clone()
            .unwrap();

        if LOGS_REGISTRO {
            self.add_registro(format!(
                "Obteniendo los parametros de take profit: {:?}",
                params
            ));
        }

        let mut valor: f64 = 0.0;

        if (i as i32 - params.shift.clone() as i32) >= 0 {
            valor = df
                .column(&params.nombre_col)
                .unwrap()
                .get(i - &params.shift)
                .unwrap()
                .try_extract::<f64>()
                .unwrap_or(0.0);

            if LOGS_REGISTRO {
                self.add_registro(format!(
                    "Valor optenido para calcular el take profit: {}",
                    valor
                ));
            }

            let tp: f64 = match params.tipo.as_str() {
                "pip" => {
                    if direccion == EntryDirection::Buy {
                        if LOGS_REGISTRO {
                            self.add_registro(format!(
                                "Obteniendo limite de compra por pip: {}",
                                valor + params.valor
                            ));
                        }
                        valor + params.valor
                    } else if direccion == EntryDirection::Sell {
                        if valor - params.valor >= 0.00000 {
                            if LOGS_REGISTRO {
                                self.add_registro(format!(
                                    "Obteniendo limite de venta por pip: {}",
                                    valor - params.valor
                                ));
                            }
                            valor - params.valor
                        } else {
                            if LOGS_REGISTRO {
                                self.add_registro(format!("limite de venta por pip no valido"));
                            }
                            0.0
                        }
                    } else {
                        if LOGS_REGISTRO {
                            self.add_registro(format!("limite de venta por pip no valido"));
                        }
                        0.0
                    }
                }
                "tick" => {
                    if direccion == EntryDirection::Buy {
                        if LOGS_REGISTRO {
                            self.add_registro(format!(
                                "Obteniendo limite de compra por tick: {}",
                                valor + params.valor
                            ));
                        }
                        valor + params.valor
                    } else if direccion == EntryDirection::Sell {
                        if valor - params.valor >= 0.00000 {
                            if LOGS_REGISTRO {
                                self.add_registro(format!(
                                    "Obteniendo limite de venta por tick: {}",
                                    valor - params.valor
                                ));
                            }
                            valor - params.valor
                        } else {
                            if LOGS_REGISTRO {
                                self.add_registro(format!("limite de venta por tick no valido"));
                            }
                            0.0
                        }
                    } else {
                        if LOGS_REGISTRO {
                            self.add_registro(format!("limite de venta por tick no valido"));
                        }
                        0.0
                    }
                }
                "punto" => {
                    if direccion == EntryDirection::Buy {
                        if LOGS_REGISTRO {
                            self.add_registro(format!(
                                "Obteniendo limite de compra por punto: {}",
                                valor + params.valor
                            ));
                        }
                        valor + params.valor
                    } else if direccion == EntryDirection::Sell {
                        if valor - params.valor >= 0.00000 {
                            if LOGS_REGISTRO {
                                self.add_registro(format!(
                                    "Obteniendo limite de venta por punto: {}",
                                    valor - params.valor
                                ));
                            }
                            valor - params.valor
                        } else {
                            if LOGS_REGISTRO {
                                self.add_registro(format!("limite de venta por punto no valido"));
                            }
                            0.0
                        }
                    } else {
                        if LOGS_REGISTRO {
                            self.add_registro(format!("limite de venta por punto no valido"));
                        }
                        0.0
                    }
                }
                "porcentaje" => {
                    if direccion == EntryDirection::Buy {
                        if LOGS_REGISTRO {
                            self.add_registro(format!(
                                "Obteniendo limite de compra por porcentaje: {}",
                                valor + (valor * params.valor)
                            ));
                        }
                        valor + (valor * params.valor)
                    } else if direccion == EntryDirection::Sell {
                        if valor - (valor * params.valor) >= 0.00000 {
                            if LOGS_REGISTRO {
                                self.add_registro(format!(
                                    "Obteniendo limite de venta por porcentaje: {}",
                                    valor - (valor * params.valor)
                                ));
                            }
                            valor - (valor * params.valor)
                        } else {
                            if LOGS_REGISTRO {
                                self.add_registro(format!(
                                    "limite de venta por porcentaje no valido"
                                ));
                            }
                            0.0
                        }
                    } else {
                        if LOGS_REGISTRO {
                            self.add_registro(format!("limite de venta por porcentaje no valido"));
                        }
                        0.0
                    }
                }
                "atr" => {
                    if direccion == EntryDirection::Buy {
                        if LOGS_REGISTRO {
                            self.add_registro(format!(
                                "Obteniendo limite de compra por atr: {}",
                                precio_entrada + valor
                            ));
                        }
                        precio_entrada + valor
                    } else if direccion == EntryDirection::Sell {
                        if precio_entrada - valor >= 0.00000 {
                            if LOGS_REGISTRO {
                                self.add_registro(format!(
                                    "Obteniendo limite de venta por atr: {}",
                                    precio_entrada - valor
                                ));
                            }
                            precio_entrada - valor
                        } else {
                            if LOGS_REGISTRO {
                                self.add_registro(format!("limite de atr no valido"));
                            }
                            0.0
                        }
                    } else {
                        if LOGS_REGISTRO {
                            self.add_registro(format!("limite de atr no valido"));
                        }
                        0.0
                    }
                }
                _ => valor,
            };

            return tp;
        }
        valor
    }

    /// Ejecuta una operación de entrada de una acción en un índice dado.
    ///
    /// # Parametros
    /// timestamp: Timestamp de la operación.
    /// symbol: SymbolInfoCFD con la información del símbolo.
    /// signal: String con la señal de entrada.
    /// precio_entrada: Precio de entrada de la acción.
    /// stoploss: Opcional. Precio de stoploss de la acción.
    /// takeprofit: Opcional. Precio de takeprofit de la acción.
    ///
    /// # Retorna
    /// El trade ejecutado, si se pudo realizar.
    async fn ejecutar_entry(
        &mut self,
        timestamp: i64,
        symbol: SymbolInfoCFD,
        signal: EntryDirection,
        precio_entrada: f64,
        stoploss: Option<f64>,
        takeprofit: Option<f64>,
    ) -> Option<Trade> {
        let mut trade: Trade = Trade::new(self.id.clone(), symbol.clone()).await;

        let sl = stoploss.unwrap_or(0.0);
        let mut tp = takeprofit.unwrap_or(0.0);

        let naive_time = DateTime::from_timestamp_millis(timestamp).expect("timestamp inválido");
        let t0 = naive_time.format("%Y-%m-%d %H:%M:%S").to_string();

        match signal {
            EntryDirection::Buy => {
                let spread = trade.random_spread();
                let precio = precio_entrada + spread;

                if tp > 0.0 {
                    tp += spread;
                }

                trade.buy(
                    t0.clone(),
                    precio,
                    self.gestion_strategy.clone(),
                    self.parametros_gestion.clone(),
                    &self,
                    Some(tp),
                    Some(sl),
                );
                if LOGS_REGISTRO {
                    self.add_registro(format!(
                        "Trade buy ejecutado: t0={} precio_entrada={} tp={} sl={} spread={}",
                        t0, precio, tp, sl, spread
                    ));
                }
                return Some(trade);
            }
            EntryDirection::Sell => {
                trade.sell(
                    t0.clone(),
                    precio_entrada,
                    self.gestion_strategy.clone(),
                    self.parametros_gestion.clone(),
                    &self,
                    Some(tp),
                    Some(sl),
                );
                if LOGS_REGISTRO {
                    self.add_registro(format!(
                        "Trade sell ejecutado: t0={} precio_entrada={} tp={} sl={}",
                        t0, precio_entrada, tp, sl
                    ));
                }
                return Some(trade);
            }
        }
    }

    /// Identifica si se da la condición para activar el Breakevent
    ///
    /// # Arguments
    ///
    /// * `tipo` - Tipo de operación ("buy" o "sell")
    /// * `unidad` - Unidad de medida ("tick" o "pip")
    /// * `symbol` - Información del símbolo
    /// * `valor` - Valor en unidad para saber cuaando activar el BreakEvent
    /// * `precio_entrada` - Precio de entrada
    /// * `precio_actual` - Precio actual
    ///
    /// # Returns
    ///
    /// `true` si se da la condición para activar el Breakevent, `false` en caso contrario
    fn colocar_be(
        &mut self,
        tipo: EntryDirection,
        unidad: BeTipo,
        symbol: SymbolInfoCFD,
        valor: f64,
        precio_entrada: f64,
        precio_actual: f64,
    ) -> bool {
        let diferencia = match tipo {
            EntryDirection::Buy => precio_actual - precio_entrada,
            EntryDirection::Sell => precio_entrada - precio_actual,
        };

        let resultado = match symbol.digitos {
            1 => match unidad {
                BeTipo::Tick => diferencia / 0.1,
                BeTipo::Pip => diferencia,
                BeTipo::Punto => diferencia / 0.1,
                BeTipo::Porcentaje => (diferencia / precio_entrada) * 100.0,
                _ => diferencia,
            },
            2 => match unidad {
                BeTipo::Tick => diferencia / 0.01,
                BeTipo::Pip => diferencia / 0.1,
                BeTipo::Punto => diferencia / 0.01,
                BeTipo::Porcentaje => (diferencia / precio_entrada) * 100.0,
                _ => diferencia,
            },
            3 => match unidad {
                BeTipo::Tick => diferencia / 0.001,
                BeTipo::Pip => diferencia / 0.01,
                BeTipo::Punto => diferencia / 0.001,
                BeTipo::Porcentaje => (diferencia / precio_entrada) * 100.0,
                _ => diferencia,
            },
            4 => match unidad {
                BeTipo::Tick => diferencia / 0.0001,
                BeTipo::Pip => diferencia / 0.001,
                BeTipo::Punto => diferencia / 0.0001,
                BeTipo::Porcentaje => (diferencia / precio_entrada) * 100.0,
                _ => diferencia,
            },
            5 => match unidad {
                BeTipo::Tick => diferencia / 0.00001,
                BeTipo::Pip => diferencia / 0.0001,
                BeTipo::Punto => diferencia / 0.00001,
                BeTipo::Porcentaje => (diferencia / precio_entrada) * 100.0,
                _ => diferencia,
            },
            _ => diferencia,
        };

        if resultado >= valor {
            if LOGS_REGISTRO {
                self.add_registro(format!("Se va a colocar BE en precio={}", resultado));
            }
            true
        } else {
            false
        }
    }

    /// Calcula el BE+ de un trade basado en la unidad y el símbolo.
    ///
    /// # Arguments
    ///
    /// * `unidad` - Unidad de medida para el cálculo (tick, pip, punto, porcentaje).
    /// * `symbol` - Información del símbolo del activo.
    /// * `valor` - Valor del trade.
    /// * `precio_entrada` - Precio de entrada del trade.
    ///
    /// # Returns
    ///
    /// El BE+ calculado como un valor f64.
    fn calcular_be_plus(
        &mut self,
        unidad: BeTipo,
        symbol: SymbolInfoCFD,
        valor: f64,
        precio_entrada: f64,
    ) -> f64 {
        let resultado: f64 = match symbol.digitos {
            1 => match unidad {
                BeTipo::Tick => valor * 0.1,
                BeTipo::Pip => valor,
                BeTipo::Punto => valor * 0.1,
                BeTipo::Porcentaje => (valor / precio_entrada) * 100.0,
                _ => 0.0,
            },
            2 => match unidad {
                BeTipo::Tick => valor * 0.01,
                BeTipo::Pip => valor * 0.1,
                BeTipo::Punto => valor * 0.01,
                BeTipo::Porcentaje => (valor / precio_entrada) * 100.0,
                _ => 0.0,
            },
            3 => match unidad {
                BeTipo::Tick => valor * 0.001,
                BeTipo::Pip => valor * 0.01,
                BeTipo::Punto => valor * 0.001,
                BeTipo::Porcentaje => (valor / precio_entrada) * 100.0,
                _ => 0.0,
            },
            4 => match unidad {
                BeTipo::Tick => valor * 0.0001,
                BeTipo::Pip => valor * 0.001,
                BeTipo::Punto => valor * 0.0001,
                BeTipo::Porcentaje => (valor / precio_entrada) * 100.0,
                _ => 0.0,
            },
            5 => match unidad {
                BeTipo::Tick => valor * 0.00001,
                BeTipo::Pip => valor * 0.0001,
                BeTipo::Punto => valor * 0.00001,
                BeTipo::Porcentaje => (valor / precio_entrada) * 100.0,
                _ => 0.0,
            },
            _ => 0.0,
        };

        if LOGS_REGISTRO {
            self.add_registro(format!("Se va a colocar BE+ en precio={}", resultado));
        }
        resultado
    }

    /// Calcula el TLS (Take Loss Stop) basado en el tipo de entrada y los parámetros.
    ///
    /// # Arguments
    ///
    /// * `tipo` - El tipo de entrada (compra o venta).
    /// * `symbol` - Información del símbolo.
    /// * `parametros` - Parámetros del TLS.
    /// * `trade` - El trade actual.
    /// * `precio_actual` - Precio actual del símbolo.
    /// * `precio_anterior` - Precio anterior del símbolo.
    ///
    /// # Returns
    ///
    /// El valor del TLS calculado.
    fn calcular_tls(
        &mut self,
        tipo: EntryDirection,
        symbol: SymbolInfoCFD,
        parametros: TlParams,
        trade: Trade,
        precio_actual: f64,
        precio_anterior: f64,
    ) -> f64 {
        if LOGS_REGISTRO {
            self.add_registro("Calculando TLS...".to_string());
        }

        let resultado: f64 = match parametros.tipo {
            TlTipo::Pip => match tipo {
                EntryDirection::Buy => {
                    if LOGS_REGISTRO {
                        self.add_registro("Calculando TLS en pips para Buy".to_string());
                    }

                    let diff = precio_actual - precio_anterior;
                    if LOGS_REGISTRO {
                        self.add_registro(format!("Diferencia: {}", diff));
                    }

                    if diff > 0.0 {
                        let pips = diff / parametros.valor;
                        if pips > 0.0 {
                            match symbol.digitos {
                                1 => trade.sl + (pips * 1.0),
                                2 => trade.sl + (pips * 0.1),
                                3 => trade.sl + (pips * 0.01),
                                4 => trade.sl + (pips * 0.001),
                                5 => trade.sl + (pips * 0.0001),
                                _ => 0.0,
                            }
                        } else {
                            0.0
                        }
                    } else {
                        0.0
                    }
                }
                EntryDirection::Sell => {
                    if LOGS_REGISTRO {
                        self.add_registro("Calculando TLS en pips para Sell".to_string());
                    }
                    let diff = precio_anterior - precio_actual;
                    if LOGS_REGISTRO {
                        self.add_registro(format!("Diferencia: {}", diff));
                    }

                    if diff > 0.0 {
                        let pips = diff / parametros.valor;
                        if pips > 0.0 {
                            match symbol.digitos {
                                1 => trade.sl - (pips * 1.0),
                                2 => trade.sl - (pips * 0.1),
                                3 => trade.sl - (pips * 0.01),
                                4 => trade.sl - (pips * 0.001),
                                5 => trade.sl - (pips * 0.0001),
                                _ => 0.0,
                            }
                        } else {
                            0.0
                        }
                    } else {
                        0.0
                    }
                }
            },
            TlTipo::Tick => match tipo {
                EntryDirection::Buy => {
                    if LOGS_REGISTRO {
                        self.add_registro("Calculando TLS en ticks para Buy".to_string());
                    }
                    let diff = precio_actual - precio_anterior;
                    if LOGS_REGISTRO {
                        self.add_registro(format!("Diferencia: {}", diff));
                    }

                    if diff > 0.0 {
                        let ticks = diff / parametros.valor;
                        if ticks > 0.0 {
                            match symbol.digitos {
                                1 => trade.sl + (ticks * 0.1),
                                2 => trade.sl + (ticks * 0.01),
                                3 => trade.sl + (ticks * 0.001),
                                4 => trade.sl + (ticks * 0.0001),
                                5 => trade.sl + (ticks * 0.00001),
                                _ => 0.0,
                            }
                        } else {
                            0.0
                        }
                    } else {
                        0.0
                    }
                }
                EntryDirection::Sell => {
                    if LOGS_REGISTRO {
                        self.add_registro("Calculando TLS en ticks para Sell".to_string());
                    }
                    let diff = precio_anterior - precio_actual;
                    if LOGS_REGISTRO {
                        self.add_registro(format!("Diferencia: {}", diff));
                    }

                    if diff > 0.0 {
                        let ticks = diff / parametros.valor;
                        if ticks > 0.0 {
                            match symbol.digitos {
                                1 => trade.sl - (ticks * 0.1),
                                2 => trade.sl - (ticks * 0.01),
                                3 => trade.sl - (ticks * 0.001),
                                4 => trade.sl - (ticks * 0.0001),
                                5 => trade.sl - (ticks * 0.00001),
                                _ => 0.0,
                            }
                        } else {
                            0.0
                        }
                    } else {
                        0.0
                    }
                }
            },
            TlTipo::Punto => match tipo {
                EntryDirection::Buy => {
                    if LOGS_REGISTRO {
                        self.add_registro("Calculando TLS en puntos para Buy".to_string());
                    }
                    let diff = precio_actual - precio_anterior;
                    if LOGS_REGISTRO {
                        self.add_registro(format!("Diferencia: {}", diff));
                    }

                    if diff > 0.0 {
                        let punto = diff / parametros.valor;
                        if punto > 0.0 {
                            match symbol.digitos {
                                1 => trade.sl + (punto * 0.1),
                                2 => trade.sl + (punto * 0.01),
                                3 => trade.sl + (punto * 0.001),
                                4 => trade.sl + (punto * 0.0001),
                                5 => trade.sl + (punto * 0.00001),
                                _ => 0.0,
                            }
                        } else {
                            0.0
                        }
                    } else {
                        0.0
                    }
                }
                EntryDirection::Sell => {
                    if LOGS_REGISTRO {
                        self.add_registro("Calculando TLS en puntos para Sell".to_string());
                    }
                    let diff = precio_anterior - precio_actual;
                    if LOGS_REGISTRO {
                        self.add_registro(format!("Diferencia: {}", diff));
                    }

                    if diff > 0.0 {
                        let punto = diff / parametros.valor;
                        if punto > 0.0 {
                            match symbol.digitos {
                                1 => trade.sl - (punto * 0.1),
                                2 => trade.sl - (punto * 0.01),
                                3 => trade.sl - (punto * 0.001),
                                4 => trade.sl - (punto * 0.0001),
                                5 => trade.sl - (punto * 0.00001),
                                _ => 0.0,
                            }
                        } else {
                            0.0
                        }
                    } else {
                        0.0
                    }
                }
            },
            _ => 0.0,
        };

        if LOGS_REGISTRO {
            self.add_registro(format!("Resultado del calculo de TLS: {}", resultado));
        }

        resultado
    }

    /// Activa el TSL (Take Stop Loss) basado en los datos y los parámetros proporcionados.
    ///
    /// # Arguments
    ///
    /// * `data` - El DataFrame que contiene los datos de mercado.
    /// * `i` - El índice del dato actual.
    /// * `tipo` - La dirección del trade (compra o venta).
    /// * `parametros` - Los parámetros de gestión de TSL.
    /// * `symbol` - La información del símbolo del activo.
    /// * `precio_entrada` - El precio de entrada del trade.
    /// * `precio_actual` - El precio actual del activo.
    ///
    /// # Returns
    ///
    /// `true` si el TSL se activa, `false` en caso contrario.
    fn activar_tsl(
        &mut self,
        data: DataFrame,
        i: usize,
        tipo: EntryDirection,
        parametros: TlParams,
        symbol: SymbolInfoCFD,
        precio_entrada: f64,
        precio_actual: f64,
    ) -> bool {
        if LOGS_REGISTRO {
            self.add_registro("Activando TSL".to_string());
        }
        let diferencia = match tipo {
            EntryDirection::Buy => precio_actual - precio_entrada,
            EntryDirection::Sell => precio_entrada - precio_actual,
        };

        let mut indicador: bool = false;

        let resultado = match symbol.digitos {
            1 => match parametros.activacion_tipo {
                TlTipo::Tick => diferencia / 0.1,
                TlTipo::Pip => diferencia,
                TlTipo::Punto => diferencia / 0.1,
                TlTipo::Porcentaje => (diferencia / precio_entrada) * 100.0,
                TlTipo::Indicador => {
                    let indicador_price: f64 = data
                        .column(parametros.columna_nombre.as_str())
                        .unwrap()
                        .get(i)
                        .unwrap()
                        .try_extract::<f64>()
                        .unwrap();

                    let resultado: f64 = match tipo {
                        EntryDirection::Buy => {
                            if indicador_price <= precio_actual {
                                indicador = true;
                                indicador_price
                            } else {
                                0.0
                            }
                        }
                        EntryDirection::Sell => {
                            if indicador_price >= precio_actual {
                                indicador = true;
                                indicador_price
                            } else {
                                0.0
                            }
                        }
                    };

                    resultado
                }
            },
            2 => match parametros.activacion_tipo {
                TlTipo::Tick => diferencia / 0.01,
                TlTipo::Pip => diferencia / 0.1,
                TlTipo::Punto => diferencia / 0.01,
                TlTipo::Porcentaje => (diferencia / precio_entrada) * 100.0,
                TlTipo::Indicador => {
                    let indicador_price: f64 = data
                        .column(parametros.columna_nombre.as_str())
                        .unwrap()
                        .get(i)
                        .unwrap()
                        .try_extract::<f64>()
                        .unwrap();

                    let resultado: f64 = match tipo {
                        EntryDirection::Buy => {
                            if indicador_price <= precio_actual {
                                indicador = true;
                                indicador_price
                            } else {
                                0.0
                            }
                        }
                        EntryDirection::Sell => {
                            if indicador_price >= precio_actual {
                                indicador = true;
                                indicador_price
                            } else {
                                0.0
                            }
                        }
                    };

                    resultado
                }
            },
            3 => match parametros.activacion_tipo {
                TlTipo::Tick => diferencia / 0.001,
                TlTipo::Pip => diferencia / 0.01,
                TlTipo::Punto => diferencia / 0.001,
                TlTipo::Porcentaje => (diferencia / precio_entrada) * 100.0,
                TlTipo::Indicador => {
                    let indicador_price: f64 = data
                        .column(parametros.columna_nombre.as_str())
                        .unwrap()
                        .get(i)
                        .unwrap()
                        .try_extract::<f64>()
                        .unwrap();

                    let resultado: f64 = match tipo {
                        EntryDirection::Buy => {
                            if indicador_price <= precio_actual {
                                indicador = true;
                                indicador_price
                            } else {
                                0.0
                            }
                        }
                        EntryDirection::Sell => {
                            if indicador_price >= precio_actual {
                                indicador = true;
                                indicador_price
                            } else {
                                0.0
                            }
                        }
                    };

                    resultado
                }
            },
            4 => match parametros.activacion_tipo {
                TlTipo::Tick => diferencia / 0.0001,
                TlTipo::Pip => diferencia / 0.001,
                TlTipo::Punto => diferencia / 0.0001,
                TlTipo::Porcentaje => (diferencia / precio_entrada) * 100.0,
                TlTipo::Indicador => {
                    let indicador_price: f64 = data
                        .column(parametros.columna_nombre.as_str())
                        .unwrap()
                        .get(i)
                        .unwrap()
                        .try_extract::<f64>()
                        .unwrap();

                    let resultado: f64 = match tipo {
                        EntryDirection::Buy => {
                            if indicador_price <= precio_actual {
                                indicador = true;
                                indicador_price
                            } else {
                                0.0
                            }
                        }
                        EntryDirection::Sell => {
                            if indicador_price >= precio_actual {
                                indicador = true;
                                indicador_price
                            } else {
                                0.0
                            }
                        }
                    };

                    resultado
                }
            },
            5 => match parametros.activacion_tipo {
                TlTipo::Tick => diferencia / 0.00001,
                TlTipo::Pip => diferencia / 0.0001,
                TlTipo::Punto => diferencia / 0.00001,
                TlTipo::Porcentaje => (diferencia / precio_entrada) * 100.0,
                TlTipo::Indicador => {
                    let indicador_price: f64 = data
                        .column(parametros.columna_nombre.as_str())
                        .unwrap()
                        .get(i)
                        .unwrap()
                        .try_extract::<f64>()
                        .unwrap();

                    let resultado: f64 = match tipo {
                        EntryDirection::Buy => {
                            if indicador_price <= precio_actual {
                                indicador = true;
                                indicador_price
                            } else {
                                0.0
                            }
                        }
                        EntryDirection::Sell => {
                            if indicador_price >= precio_actual {
                                indicador = true;
                                indicador_price
                            } else {
                                0.0
                            }
                        }
                    };

                    resultado
                }
            },
            _ => diferencia,
        };

        if resultado >= parametros.activacion_valor && !indicador {
            if LOGS_REGISTRO {
                self.add_registro("TSL activado".to_string());
            }
            true
        } else {
            if indicador {
                if LOGS_REGISTRO {
                    self.add_registro("TSL activado".to_string());
                }
                true
            } else {
                if LOGS_REGISTRO {
                    self.add_registro("TSL desactivado".to_string());
                }
                false
            }
        }
    }

    /// Realiza el backtest de una estrategia en un DataFrame dado.
    ///
    /// # Parametros
    /// df: DataFrame con los datos del índice.
    /// symbol: SymbolInfoCFD con la información del símbolo.
    ///
    /// # Retorna
    /// Un string con el resultado del backtest.
    async fn backtest(
        &mut self,
        df: &mut DataFrame,
        symbol: SymbolInfoCFD,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let mut open_trades: Vec<Trade> = Vec::new();

        let mut buy_limits: Vec<f64> = Vec::new();
        let mut sell_limits: Vec<f64> = Vec::new();

        let mut buy_stops: Vec<f64> = Vec::new();
        let mut sell_stops: Vec<f64> = Vec::new();

        // Forma de optener un dato: df.column(&columna)?.get(row_idx)?;
        //
        // Con tipado:
        // df.column("close").unwrap().get(i).unwrap().try_extract::<f64>().unwrap();
        for i in 0..df.height() {
            if !buy_limits.is_empty() {
                if LOGS_REGISTRO {
                    self.add_registro(format!("Recorriendo los buy limits..."));
                }

                let mut indices: Vec<usize> = Vec::new();
                let low: f64 = df
                    .column("low")
                    .unwrap()
                    .get(i)
                    .unwrap()
                    .try_extract::<f64>()
                    .unwrap();

                for (idx, limit) in buy_limits
                    .iter()
                    .enumerate()
                    .filter(|(_, limit)| limit > &&low)
                {
                    let timestamp = df
                        .column("timestamp")
                        .unwrap()
                        .get(i + 1)
                        .unwrap()
                        .try_extract::<i64>()
                        .unwrap();

                    let tp: f64 =
                        self.get_takeprofit(limit.clone(), df.clone(), i, EntryDirection::Buy);
                    let sl: f64 =
                        self.get_stoploss(limit.clone(), df.clone(), i, EntryDirection::Buy);

                    let trade: Option<Trade> = self
                        .ejecutar_entry(
                            timestamp,
                            symbol.clone(),
                            EntryDirection::Buy,
                            limit.clone(),
                            Some(sl),
                            Some(tp),
                        )
                        .await;

                    if let Some(trade) = trade {
                        if LOGS_REGISTRO {
                            self.add_registro(format!(
                                "Buy limit activado en índice: {}. Trade ejecutado {:?}.",
                                idx, &trade
                            ));
                        }
                        open_trades.push(trade);
                    }

                    indices.push(idx);
                }

                for idx in indices.iter().rev() {
                    if LOGS_REGISTRO {
                        self.add_registro(format!("Buy limit eliminado en índice: {}.", *idx));
                    }
                    buy_limits.remove(*idx);
                }
            }

            if !buy_stops.is_empty() {
                if LOGS_REGISTRO {
                    self.add_registro(format!("Recorriendo los buy stops..."));
                }

                let mut indices: Vec<usize> = Vec::new();
                let high: f64 = df
                    .column("high")
                    .unwrap()
                    .get(i)
                    .unwrap()
                    .try_extract::<f64>()
                    .unwrap();

                for (idx, limit) in buy_stops
                    .iter()
                    .enumerate()
                    .filter(|(_, limit)| limit < &&high)
                {
                    let timestamp = df
                        .column("timestamp")
                        .unwrap()
                        .get(i + 1)
                        .unwrap()
                        .try_extract::<i64>()
                        .unwrap();

                    let tp: f64 =
                        self.get_takeprofit(limit.clone(), df.clone(), i, EntryDirection::Buy);
                    let sl: f64 =
                        self.get_stoploss(limit.clone(), df.clone(), i, EntryDirection::Buy);

                    let trade: Option<Trade> = self
                        .ejecutar_entry(
                            timestamp,
                            symbol.clone(),
                            EntryDirection::Buy,
                            limit.clone(),
                            Some(sl),
                            Some(tp),
                        )
                        .await;

                    if let Some(trade) = trade {
                        if LOGS_REGISTRO {
                            self.add_registro(format!(
                                "Buy stop activado en índice: {}. Trade ejecutado {:?}.",
                                idx, &trade
                            ));
                        }

                        open_trades.push(trade);
                    }

                    indices.push(idx);
                }

                for idx in indices.iter().rev() {
                    if LOGS_REGISTRO {
                        self.add_registro(format!("Buy stop eliminado en índice: {}.", *idx));
                    }

                    buy_stops.remove(*idx);
                }
            }

            if !sell_limits.is_empty() {
                if LOGS_REGISTRO {
                    self.add_registro(format!("Recorriendo los sell limits..."));
                }

                let mut indices: Vec<usize> = Vec::new();
                let high: f64 = df
                    .column("high")
                    .unwrap()
                    .get(i)
                    .unwrap()
                    .try_extract::<f64>()
                    .unwrap();

                for (idx, limit) in sell_limits
                    .iter()
                    .enumerate()
                    .filter(|(_, limit)| limit < &&high)
                {
                    let timestamp = df
                        .column("timestamp")
                        .unwrap()
                        .get(i + 1)
                        .unwrap()
                        .try_extract::<i64>()
                        .unwrap();

                    let tp: f64 =
                        self.get_takeprofit(limit.clone(), df.clone(), i, EntryDirection::Sell);
                    let sl: f64 =
                        self.get_stoploss(limit.clone(), df.clone(), i, EntryDirection::Sell);

                    let trade: Option<Trade> = self
                        .ejecutar_entry(
                            timestamp,
                            symbol.clone(),
                            EntryDirection::Sell,
                            limit.clone(),
                            Some(sl),
                            Some(tp),
                        )
                        .await;

                    if let Some(trade) = trade {
                        if LOGS_REGISTRO {
                            self.add_registro(format!(
                                "Sell limit activado en índice: {}. Trade ejecutado {:?}.",
                                idx, &trade
                            ));
                        }

                        open_trades.push(trade);
                    }

                    indices.push(idx);
                }

                for idx in indices.iter().rev() {
                    if LOGS_REGISTRO {
                        self.add_registro(format!("Sell limit eliminado en índice: {}.", *idx));
                    }

                    sell_limits.remove(*idx);
                }
            }

            if !sell_stops.is_empty() {
                if LOGS_REGISTRO {
                    self.add_registro(format!("Recorriendo los sell stops..."));
                }

                let mut indices: Vec<usize> = Vec::new();
                let low: f64 = df
                    .column("low")
                    .unwrap()
                    .get(i)
                    .unwrap()
                    .try_extract::<f64>()
                    .unwrap();

                for (idx, limit) in sell_stops
                    .iter()
                    .enumerate()
                    .filter(|(_, limit)| limit > &&low)
                {
                    let timestamp = df
                        .column("timestamp")
                        .unwrap()
                        .get(i + 1)
                        .unwrap()
                        .try_extract::<i64>()
                        .unwrap();

                    let tp: f64 =
                        self.get_takeprofit(limit.clone(), df.clone(), i, EntryDirection::Sell);
                    let sl: f64 =
                        self.get_stoploss(limit.clone(), df.clone(), i, EntryDirection::Sell);

                    let trade: Option<Trade> = self
                        .ejecutar_entry(
                            timestamp,
                            symbol.clone(),
                            EntryDirection::Sell,
                            limit.clone(),
                            Some(sl),
                            Some(tp),
                        )
                        .await;

                    if let Some(trade) = trade {
                        if LOGS_REGISTRO {
                            self.add_registro(format!(
                                "Sell stop activado en índice: {}. Trade ejecutado {:?}.",
                                idx, &trade
                            ));
                        }

                        open_trades.push(trade);
                    }

                    indices.push(idx);
                }

                for idx in indices.iter().rev() {
                    if LOGS_REGISTRO {
                        self.add_registro(format!("Sell stop eliminado en índice: {}.", *idx));
                    }

                    sell_stops.remove(*idx);
                }
            }

            if !open_trades.is_empty() {
                if LOGS_REGISTRO {
                    self.add_registro(format!("Recorriendo los trades abiertos..."));
                }
                let precio_actual: f64 = df
                    .column("close")
                    .unwrap()
                    .get(i)
                    .unwrap()
                    .try_extract::<f64>()
                    .unwrap();
                // Recorremos el vactor de operaciones abiertas(open_trades) y comprobamos si
                // alcanzan los stop-loss o take-profit para cerrarlas.
                let mut indices: Vec<usize> = Vec::new();
                for (idx, trade) in open_trades.iter_mut().enumerate() {
                    if LOGS_REGISTRO {
                        self.add_registro(format!("Procesando trade abierto: {:?}.", &trade));
                    }
                    match trade.tipo {
                        EntryDirection::Buy => {
                            if trade.sl > 0.00000 && precio_actual <= trade.sl {
                                let timestamp: i64 = df
                                    .column("timestamp")
                                    .unwrap()
                                    .get(i + 1)
                                    .unwrap()
                                    .try_extract::<i64>()
                                    .unwrap();

                                let naive_time = DateTime::from_timestamp_millis(timestamp)
                                    .expect("timestamp inválido");
                                let t1 = naive_time.format("%Y-%m-%d %H:%M:%S").to_string();

                                self.balance += trade.close(t1, trade.sl);
                                indices.push(idx);
                                if LOGS_REGISTRO {
                                    self.add_registro(format!(
                                        "Se ha ejecutado el Stop loss en : {}.",
                                        &trade.sl
                                    ));
                                }
                            }
                            if trade.tp > 0.00000 && precio_actual >= trade.tp {
                                let timestamp: i64 = df
                                    .column("timestamp")
                                    .unwrap()
                                    .get(i + 1)
                                    .unwrap()
                                    .try_extract::<i64>()
                                    .unwrap();

                                let naive_time = DateTime::from_timestamp_millis(timestamp)
                                    .expect("timestamp inválido");
                                let t1 = naive_time.format("%Y-%m-%d %H:%M:%S").to_string();

                                self.balance += trade.close(t1, trade.tp);
                                indices.push(idx);
                                if LOGS_REGISTRO {
                                    self.add_registro(format!(
                                        "Se ha ejecutado el Take profit en : {}.",
                                        &trade.tp
                                    ));
                                }
                            }
                        }
                        EntryDirection::Sell => {
                            if trade.sl > 0.00000 && precio_actual >= trade.sl {
                                let timestamp: i64 = df
                                    .column("timestamp")
                                    .unwrap()
                                    .get(i + 1)
                                    .unwrap()
                                    .try_extract::<i64>()
                                    .unwrap();

                                let naive_time = DateTime::from_timestamp_millis(timestamp)
                                    .expect("timestamp inválido");
                                let t1 = naive_time.format("%Y-%m-%d %H:%M:%S").to_string();

                                self.balance += trade.close(t1, trade.sl);
                                indices.push(idx);
                                if LOGS_REGISTRO {
                                    self.add_registro(format!(
                                        "Se ha ejecutado el Stop loss en : {}.",
                                        &trade.sl
                                    ));
                                }
                            }
                            if trade.tp > 0.00000 && precio_actual <= trade.tp {
                                let timestamp: i64 = df
                                    .column("timestamp")
                                    .unwrap()
                                    .get(i + 1)
                                    .unwrap()
                                    .try_extract::<i64>()
                                    .unwrap();

                                let naive_time = DateTime::from_timestamp_millis(timestamp)
                                    .expect("timestamp inválido");
                                let t1 = naive_time.format("%Y-%m-%d %H:%M:%S").to_string();

                                self.balance += trade.close(t1, trade.tp);
                                indices.push(idx);
                                if LOGS_REGISTRO {
                                    self.add_registro(format!(
                                        "Se ha ejecutado el Take profit en : {}.",
                                        &trade.tp
                                    ));
                                }
                            }
                        }
                    }
                }

                // Cerrar trades abiertos basados en las condiciones de salida
                let acciones = self.estrategia.acciones.clone();

                acciones
                .iter()
                .filter(|acc| acc.tipo_signal == "Exit")
                .for_each(|accion| match accion.tipo {
                        Action::ExitBuy => {
                            match &accion.conditions {
                                Some(condition) => {
                                    if self.check_conditions(&df, &condition, &i) {
                                        let timestamp: i64 = df
                                            .column("timestamp")
                                            .unwrap()
                                            .get(i + 1)
                                            .unwrap()
                                            .try_extract::<i64>()
                                            .unwrap();

                                        let precio_cierre: f64 = df
                                            .column("open")
                                            .unwrap()
                                            .get(i + 1)
                                            .unwrap()
                                            .try_extract::<f64>()
                                            .unwrap();

                                        let naive_time = DateTime::from_timestamp_millis(timestamp)
                                            .expect("timestamp inválido");
                                        let t1 = naive_time.format("%Y-%m-%d %H:%M:%S").to_string();

                                        for (idx, trade) in open_trades.iter_mut().enumerate() {
                                            match trade.tipo {
                                                EntryDirection::Buy => {
                                                    self.balance += trade.close(t1.clone(), precio_cierre);
                                                    indices.push(idx);
                                                    if LOGS_REGISTRO {
                                                        self.add_registro(format!("Condición de salida Exit Buy activada. Cerramos el trade: {:?}", &trade));
                                                    }
                                                }
                                                _ => {}
                                            }
                                        }
                                    }
                                }
                                None => {
                                    if LOGS_REGISTRO {
                                        self.add_registro(format!("No hay condiciones de salida en esta accion: {:?}", &accion));
                                    }
                                }
                            }
                        }
                        Action::ExitSell => {
                            match &accion.conditions {
                                Some(condition) => {
                                    if self.check_conditions(&df, &condition, &i) {
                                        let timestamp: i64 = df
                                            .column("timestamp")
                                            .unwrap()
                                            .get(i + 1)
                                            .unwrap()
                                            .try_extract::<i64>()
                                            .unwrap();

                                        let precio_cierre: f64 = df
                                            .column("open")
                                            .unwrap()
                                            .get(i + 1)
                                            .unwrap()
                                            .try_extract::<f64>()
                                            .unwrap();

                                        let naive_time = DateTime::from_timestamp_millis(timestamp)
                                            .expect("timestamp inválido");
                                        let t1 = naive_time.format("%Y-%m-%d %H:%M:%S").to_string();

                                        for (idx, trade) in open_trades.iter_mut().enumerate() {
                                            match trade.tipo {
                                                EntryDirection::Sell => {
                                                    self.balance += trade.close(t1.clone(), precio_cierre);
                                                    indices.push(idx);
                                                    if LOGS_REGISTRO {
                                                        self.add_registro(format!("Condición de salida Exit Sell activada. Cerramos el trade: {:?}", &trade));
                                                    }
                                                }

                                                _ => {}
                                            }
                                        }
                                    }
                                }
                                None => {
                                    if LOGS_REGISTRO {
                                        self.add_registro(format!("No hay condiciones de salida en esta accion: {:?}", &accion));
                                    }
                                }
                            }
                        }
                        Action::Nbars => {
                            let n_bars: NBarsOptions =
                                serde_json::from_value(accion.parametros.clone()).unwrap();

                            let timestamp: i64 = df
                                .column("timestamp")
                                .unwrap()
                                .get(i - n_bars.valor)
                                .unwrap()
                                .try_extract::<i64>()
                                .unwrap();

                            let naive_time = DateTime::from_timestamp_millis(timestamp)
                                .expect("timestamp inválido");
                            let time_actual = naive_time.format("%Y-%m-%d %H:%M:%S").to_string();

                            for (idx, trade) in open_trades.iter_mut().enumerate() {
                                if trade.t0 == time_actual {
                                    let timestamp: i64 = df
                                        .column("timestamp")
                                        .unwrap()
                                        .get(i + 1)
                                        .unwrap()
                                        .try_extract::<i64>()
                                        .unwrap();

                                    let precio_cierre: f64 = df
                                        .column("open")
                                        .unwrap()
                                        .get(i + 1)
                                        .unwrap()
                                        .try_extract::<f64>()
                                        .unwrap();

                                    let naive_time = DateTime::from_timestamp_millis(timestamp)
                                        .expect("timestamp inválido");
                                    let t1 = naive_time.format("%Y-%m-%d %H:%M:%S").to_string();

                                    self.balance += trade.close(t1, precio_cierre);
                                    indices.push(idx);
                                    if LOGS_REGISTRO {
                                        self.add_registro(format!("Condición de salida en NBars({}) activada. Cerramos el trade: {:?}", n_bars.valor.clone(), &trade));
                                    }
                                }
                            }
                        }
                        Action::CloseAllRules => {
                            match &accion.conditions{
                                Some(condition) => {
                                    if self.check_conditions(&df, &condition, &i) {
                                        let timestamp: i64 = df
                                            .column("timestamp")
                                            .unwrap()
                                            .get(i + 1)
                                            .unwrap()
                                            .try_extract::<i64>()
                                            .unwrap();

                                        let precio_cierre: f64 = df
                                            .column("open")
                                            .unwrap()
                                            .get(i + 1)
                                            .unwrap()
                                            .try_extract::<f64>()
                                            .unwrap();

                                        let naive_time = DateTime::from_timestamp_millis(timestamp)
                                            .expect("timestamp inválido");
                                        let t1 = naive_time.format("%Y-%m-%d %H:%M:%S").to_string();

                                        for (idx, trade) in open_trades.iter_mut().enumerate() {
                                            self.balance += trade.close(t1.clone(), precio_cierre);
                                            indices.push(idx);
                                            if LOGS_REGISTRO {
                                                self.add_registro(format!("Condición de salida activada. Cerramos el trade: {:?}", &trade));
                                            }
                                        }
                                    }
                                }
                                None => {}
                            }
                        }
                        _ => {}
                    });

                // Activamos el Breakeven segun las condiciones definidas en las acciones
                acciones
                    .iter()
                    .filter(|acc| acc.tipo_signal == "BE")
                    .for_each(|accion| {
                        let parametros: BeParams =
                            serde_json::from_value(accion.parametros.clone()).unwrap();

                        let precio_cierre: f64 = df
                            .column("close")
                            .unwrap()
                            .get(i)
                            .unwrap()
                            .try_extract::<f64>()
                            .unwrap();

                        match parametros.tipo {
                            BeTipo::Tick => {
                                for trade in open_trades.iter_mut() {
                                    match trade.tipo {
                                        EntryDirection::Buy => {
                                            if self.colocar_be(
                                                EntryDirection::Buy,
                                                BeTipo::Tick,
                                                symbol.clone(),
                                                parametros.valor,
                                                trade.precio_entrada,
                                                precio_cierre,
                                            ) {
                                                if parametros.be_plus > 0.0
                                                    && trade.sl < trade.precio_entrada
                                                {
                                                    trade.sl = trade.precio_entrada
                                                        + self.calcular_be_plus(
                                                            BeTipo::Tick,
                                                            symbol.clone(),
                                                            parametros.be_plus,
                                                            trade.precio_entrada,
                                                        );
                                                } else if trade.sl != trade.precio_entrada {
                                                    trade.sl = trade.precio_entrada;
                                                }
                                                if LOGS_REGISTRO {
                                                    self.add_registro(format!(
                                                        "BE activado por ticks a precio de entrada: {}",
                                                        trade.sl
                                                    ));
                                                }
                                            }
                                        }
                                        EntryDirection::Sell => {
                                            if self.colocar_be(
                                                EntryDirection::Sell,
                                                BeTipo::Tick,
                                                symbol.clone(),
                                                parametros.valor,
                                                trade.precio_entrada,
                                                precio_cierre,
                                            ) {
                                                if parametros.be_plus > 0.0
                                                    && trade.sl > trade.precio_entrada
                                                {
                                                    trade.sl = trade.precio_entrada
                                                        - self.calcular_be_plus(
                                                            BeTipo::Tick,
                                                            symbol.clone(),
                                                            parametros.be_plus,
                                                            trade.precio_entrada,
                                                        );
                                                } else if trade.sl != trade.precio_entrada {
                                                    trade.sl = trade.precio_entrada;
                                                }

                                                if LOGS_REGISTRO {
                                                    self.add_registro(format!(
                                                        "BE activado por ticks a precio de entrada: {:?}",
                                                        trade.sl
                                                    ));
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            BeTipo::Pip => {
                                for trade in open_trades.iter_mut() {
                                    match trade.tipo {
                                        EntryDirection::Buy => {
                                            if self.colocar_be(
                                                EntryDirection::Buy,
                                                BeTipo::Pip,
                                                symbol.clone(),
                                                parametros.valor,
                                                trade.precio_entrada,
                                                precio_cierre,
                                            ) {
                                                if parametros.be_plus > 0.0
                                                    && trade.sl < trade.precio_entrada
                                                {
                                                    trade.sl = trade.precio_entrada
                                                        + self.calcular_be_plus(
                                                            BeTipo::Pip,
                                                            symbol.clone(),
                                                            parametros.be_plus,
                                                            trade.precio_entrada,
                                                        );
                                                } else if trade.sl != trade.precio_entrada {
                                                    trade.sl = trade.precio_entrada;
                                                }

                                                if LOGS_REGISTRO {
                                                    self.add_registro(format!(
                                                        "BE activado por pips a precio de entrada: {}",
                                                        trade.sl
                                                    ));
                                                }
                                            }
                                        }
                                        EntryDirection::Sell => {
                                            if self.colocar_be(
                                                EntryDirection::Sell,
                                                BeTipo::Pip,
                                                symbol.clone(),
                                                parametros.valor,
                                                trade.precio_entrada,
                                                precio_cierre,
                                            ) {
                                                if parametros.be_plus > 0.0
                                                    && trade.sl > trade.precio_entrada
                                                {
                                                    trade.sl = trade.precio_entrada
                                                        - self.calcular_be_plus(
                                                            BeTipo::Pip,
                                                            symbol.clone(),
                                                            parametros.be_plus,
                                                            trade.precio_entrada,
                                                        );
                                                } else if trade.sl != trade.precio_entrada {
                                                    trade.sl = trade.precio_entrada;
                                                }

                                                if LOGS_REGISTRO {
                                                    self.add_registro(format!(
                                                        "BE activado por pips a precio de entrada: {}",
                                                        trade.sl
                                                    ));
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            BeTipo::Punto => {
                                for trade in open_trades.iter_mut() {
                                    match trade.tipo {
                                        EntryDirection::Buy => {
                                            if self.colocar_be(
                                                EntryDirection::Buy,
                                                BeTipo::Punto,
                                                symbol.clone(),
                                                parametros.valor,
                                                trade.precio_entrada,
                                                precio_cierre,
                                            ) {
                                                if parametros.be_plus > 0.0
                                                    && trade.sl < trade.precio_entrada
                                                {
                                                    trade.sl = trade.precio_entrada
                                                        + self.calcular_be_plus(
                                                            BeTipo::Punto,
                                                            symbol.clone(),
                                                            parametros.be_plus,
                                                            trade.precio_entrada,
                                                        );
                                                } else if trade.sl != trade.precio_entrada {
                                                    trade.sl = trade.precio_entrada;
                                                }

                                                if LOGS_REGISTRO {
                                                    self.add_registro(format!(
                                                        "BE activado por puntos a precio de entrada: {}",
                                                        trade.sl
                                                    ));
                                                }
                                            }
                                        }
                                        EntryDirection::Sell => {
                                            if self.colocar_be(
                                                EntryDirection::Sell,
                                                BeTipo::Punto,
                                                symbol.clone(),
                                                parametros.valor,
                                                trade.precio_entrada,
                                                precio_cierre,
                                            ) {
                                                if parametros.be_plus > 0.0
                                                    && trade.sl > trade.precio_entrada
                                                {
                                                    trade.sl = trade.precio_entrada
                                                        - self.calcular_be_plus(
                                                            BeTipo::Punto,
                                                            symbol.clone(),
                                                            parametros.be_plus,
                                                            trade.precio_entrada,
                                                        );
                                                } else if trade.sl != trade.precio_entrada {
                                                    trade.sl = trade.precio_entrada;
                                                }

                                                if LOGS_REGISTRO {
                                                    self.add_registro(format!(
                                                        "BE activado por puntos a precio de entrada: {}",
                                                        trade.sl
                                                    ));
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            BeTipo::Porcentaje => {
                                for trade in open_trades.iter_mut() {
                                    match trade.tipo {
                                        EntryDirection::Buy => {
                                            if self.colocar_be(
                                                EntryDirection::Buy,
                                                BeTipo::Porcentaje,
                                                symbol.clone(),
                                                parametros.valor,
                                                trade.precio_entrada,
                                                precio_cierre,
                                            ) {
                                                if parametros.be_plus > 0.0
                                                    && trade.sl < trade.precio_entrada
                                                {
                                                    trade.sl = trade.precio_entrada
                                                        + self.calcular_be_plus(
                                                            BeTipo::Porcentaje,
                                                            symbol.clone(),
                                                            parametros.be_plus,
                                                            trade.precio_entrada,
                                                        );
                                                } else if trade.sl != trade.precio_entrada {
                                                    trade.sl = trade.precio_entrada;
                                                }

                                                if LOGS_REGISTRO {
                                                    self.add_registro(format!(
                                                        "BE activado por porcentaje a precio de entrada: {}",
                                                        trade.sl
                                                    ));
                                                }
                                            }
                                        }
                                        EntryDirection::Sell => {
                                            if self.colocar_be(
                                                EntryDirection::Sell,
                                                BeTipo::Porcentaje,
                                                symbol.clone(),
                                                parametros.valor,
                                                trade.precio_entrada,
                                                precio_cierre,
                                            ) {
                                                if parametros.be_plus > 0.0
                                                    && trade.sl > trade.precio_entrada
                                                {
                                                    trade.sl = trade.precio_entrada
                                                        - self.calcular_be_plus(
                                                            BeTipo::Porcentaje,
                                                            symbol.clone(),
                                                            parametros.be_plus,
                                                            trade.precio_entrada,
                                                        );
                                                } else if trade.sl != trade.precio_entrada {
                                                    trade.sl = trade.precio_entrada;
                                                }

                                                if LOGS_REGISTRO {
                                                    self.add_registro(format!(
                                                        "BE activado por porcentaje a precio de entrada: {}",
                                                        trade.sl
                                                    ));
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            BeTipo::Indicador => {
                                if let Some(col_name) = &parametros.col_name {
                                    let valor = df
                                        .column(col_name)
                                        .unwrap()
                                        .get(i)
                                        .unwrap()
                                        .try_extract::<f64>()
                                        .unwrap();

                                    for trade in open_trades.iter_mut() {
                                        match trade.tipo {
                                            EntryDirection::Buy => {
                                                if valor <= precio_cierre {
                                                    if parametros.be_plus > 0.0
                                                        && trade.sl < trade.precio_entrada
                                                    {
                                                        trade.sl = trade.precio_entrada
                                                            + self.calcular_be_plus(
                                                                BeTipo::Tick,
                                                                symbol.clone(),
                                                                parametros.be_plus,
                                                                trade.precio_entrada,
                                                            );
                                                    } else if trade.sl != trade.precio_entrada {
                                                        trade.sl = trade.precio_entrada;
                                                    }

                                                    if LOGS_REGISTRO {
                                                        self.add_registro(format!(
                                                            "BE activado por indicador a precio de entrada: {}",
                                                            trade.sl
                                                        ));
                                                    }
                                                }
                                            }
                                            EntryDirection::Sell => {
                                                if parametros.be_plus > 0.0
                                                    && trade.sl > trade.precio_entrada
                                                {
                                                    trade.sl = trade.precio_entrada
                                                        - self.calcular_be_plus(
                                                            BeTipo::Tick,
                                                            symbol.clone(),
                                                            parametros.be_plus,
                                                            trade.precio_entrada,
                                                        );
                                                } else if trade.sl != trade.precio_entrada {
                                                    trade.sl = trade.precio_entrada;
                                                }

                                                if LOGS_REGISTRO {
                                                    self.add_registro(format!(
                                                        "BE activado por indicador a precio de entrada: {}",
                                                        trade.sl
                                                    ));
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    });

                // Activamos las opciones de trailing stoploss segun la configuracion de las acciones
                acciones
                    .iter()
                    .filter(|acc| acc.tipo_signal == "TSL")
                    .for_each(|accion| {
                        let parametros: TlParams =
                            serde_json::from_value(accion.parametros.clone()).unwrap();

                        let precio_cierre: f64 = df
                            .column("close")
                            .unwrap()
                            .get(i)
                            .unwrap()
                            .try_extract::<f64>()
                            .unwrap();

                        match parametros.activacion_tipo {
                            TlTipo::Tick => {
                                for trade in open_trades.iter_mut() {
                                    match trade.tipo {
                                        EntryDirection::Buy => {
                                            if self.activar_tsl(
                                                df.clone(),
                                                i.clone(),
                                                EntryDirection::Buy,
                                                parametros.clone(),
                                                symbol.clone(),
                                                trade.precio_entrada,
                                                precio_cierre,
                                            ) {
                                                let precio_actual: f64 = df
                                                    .column("high")
                                                    .unwrap()
                                                    .get(i)
                                                    .unwrap()
                                                    .try_extract::<f64>()
                                                    .unwrap();

                                                let precio_anterior: f64 = df
                                                    .column("high")
                                                    .unwrap()
                                                    .get(i - 1)
                                                    .unwrap()
                                                    .try_extract::<f64>()
                                                    .unwrap();

                                                trade.sl = self.calcular_tls(
                                                    EntryDirection::Buy,
                                                    symbol.clone(),
                                                    parametros.clone(),
                                                    trade.clone(),
                                                    precio_actual,
                                                    precio_anterior,
                                                );

                                                if LOGS_REGISTRO {
                                                    self.add_registro(format!(
                                                        "TSL activado por ticks. Precio SL: {}",
                                                        trade.sl
                                                    ));
                                                }
                                            }
                                        }
                                        EntryDirection::Sell => {
                                            if self.activar_tsl(
                                                df.clone(),
                                                i.clone(),
                                                EntryDirection::Sell,
                                                parametros.clone(),
                                                symbol.clone(),
                                                trade.precio_entrada,
                                                precio_cierre,
                                            ) {
                                                let precio_actual: f64 = df
                                                    .column("low")
                                                    .unwrap()
                                                    .get(i)
                                                    .unwrap()
                                                    .try_extract::<f64>()
                                                    .unwrap();

                                                let precio_anterior: f64 = df
                                                    .column("low")
                                                    .unwrap()
                                                    .get(i - 1)
                                                    .unwrap()
                                                    .try_extract::<f64>()
                                                    .unwrap();

                                                trade.sl = self.calcular_tls(
                                                    EntryDirection::Sell,
                                                    symbol.clone(),
                                                    parametros.clone(),
                                                    trade.clone(),
                                                    precio_actual,
                                                    precio_anterior,
                                                );

                                                if LOGS_REGISTRO {
                                                    self.add_registro(format!(
                                                        "TSL activado por ticks. Precio SL: {}",
                                                        trade.sl
                                                    ));
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            TlTipo::Pip => {
                                for trade in open_trades.iter_mut() {
                                    match trade.tipo {
                                        EntryDirection::Buy => {
                                            if self.activar_tsl(
                                                df.clone(),
                                                i.clone(),
                                                EntryDirection::Buy,
                                                parametros.clone(),
                                                symbol.clone(),
                                                trade.precio_entrada,
                                                precio_cierre,
                                            ) {
                                                let precio_actual: f64 = df
                                                    .column("high")
                                                    .unwrap()
                                                    .get(i)
                                                    .unwrap()
                                                    .try_extract::<f64>()
                                                    .unwrap();

                                                let precio_anterior: f64 = df
                                                    .column("high")
                                                    .unwrap()
                                                    .get(i - 1)
                                                    .unwrap()
                                                    .try_extract::<f64>()
                                                    .unwrap();

                                                trade.sl = self.calcular_tls(
                                                    EntryDirection::Buy,
                                                    symbol.clone(),
                                                    parametros.clone(),
                                                    trade.clone(),
                                                    precio_actual,
                                                    precio_anterior,
                                                );

                                                if LOGS_REGISTRO {
                                                    self.add_registro(format!(
                                                        "TSL activado por pips. Precio SL: {}",
                                                        trade.sl
                                                    ));
                                                }
                                            }
                                        }
                                        EntryDirection::Sell => {
                                            if self.activar_tsl(
                                                df.clone(),
                                                i.clone(),
                                                EntryDirection::Sell,
                                                parametros.clone(),
                                                symbol.clone(),
                                                trade.precio_entrada,
                                                precio_cierre,
                                            ) {
                                                let precio_actual: f64 = df
                                                    .column("low")
                                                    .unwrap()
                                                    .get(i)
                                                    .unwrap()
                                                    .try_extract::<f64>()
                                                    .unwrap();

                                                let precio_anterior: f64 = df
                                                    .column("low")
                                                    .unwrap()
                                                    .get(i - 1)
                                                    .unwrap()
                                                    .try_extract::<f64>()
                                                    .unwrap();

                                                trade.sl = self.calcular_tls(
                                                    EntryDirection::Sell,
                                                    symbol.clone(),
                                                    parametros.clone(),
                                                    trade.clone(),
                                                    precio_actual,
                                                    precio_anterior,
                                                );

                                                if LOGS_REGISTRO {
                                                    self.add_registro(format!(
                                                        "TSL activado por pips. Precio SL: {}",
                                                        trade.sl
                                                    ));
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            TlTipo::Punto => {
                                for trade in open_trades.iter_mut() {
                                    match trade.tipo {
                                        EntryDirection::Buy => {
                                            if self.activar_tsl(
                                                df.clone(),
                                                i.clone(),
                                                EntryDirection::Buy,
                                                parametros.clone(),
                                                symbol.clone(),
                                                trade.precio_entrada,
                                                precio_cierre,
                                            ) {
                                                let precio_actual: f64 = df
                                                    .column("high")
                                                    .unwrap()
                                                    .get(i)
                                                    .unwrap()
                                                    .try_extract::<f64>()
                                                    .unwrap();

                                                let precio_anterior: f64 = df
                                                    .column("high")
                                                    .unwrap()
                                                    .get(i - 1)
                                                    .unwrap()
                                                    .try_extract::<f64>()
                                                    .unwrap();

                                                trade.sl = self.calcular_tls(
                                                    EntryDirection::Buy,
                                                    symbol.clone(),
                                                    parametros.clone(),
                                                    trade.clone(),
                                                    precio_actual,
                                                    precio_anterior,
                                                );

                                                if LOGS_REGISTRO {
                                                    self.add_registro(format!(
                                                        "TSL activado por punto. Precio SL: {}",
                                                        trade.sl
                                                    ));
                                                }
                                            }
                                        }
                                        EntryDirection::Sell => {
                                            if self.activar_tsl(
                                                df.clone(),
                                                i.clone(),
                                                EntryDirection::Sell,
                                                parametros.clone(),
                                                symbol.clone(),
                                                trade.precio_entrada,
                                                precio_cierre,
                                            ) {
                                                let precio_actual: f64 = df
                                                    .column("low")
                                                    .unwrap()
                                                    .get(i)
                                                    .unwrap()
                                                    .try_extract::<f64>()
                                                    .unwrap();

                                                let precio_anterior: f64 = df
                                                    .column("low")
                                                    .unwrap()
                                                    .get(i - 1)
                                                    .unwrap()
                                                    .try_extract::<f64>()
                                                    .unwrap();

                                                trade.sl = self.calcular_tls(
                                                    EntryDirection::Sell,
                                                    symbol.clone(),
                                                    parametros.clone(),
                                                    trade.clone(),
                                                    precio_actual,
                                                    precio_anterior,
                                                );

                                                if LOGS_REGISTRO {
                                                    self.add_registro(format!(
                                                        "TSL activado por punto. Precio SL: {}",
                                                        trade.sl
                                                    ));
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            TlTipo::Porcentaje => {
                                for trade in open_trades.iter_mut() {
                                    match trade.tipo {
                                        EntryDirection::Buy => {
                                            if self.activar_tsl(
                                                df.clone(),
                                                i.clone(),
                                                EntryDirection::Buy,
                                                parametros.clone(),
                                                symbol.clone(),
                                                trade.precio_entrada,
                                                precio_cierre,
                                            ) {
                                                let precio_actual: f64 = df
                                                    .column("high")
                                                    .unwrap()
                                                    .get(i)
                                                    .unwrap()
                                                    .try_extract::<f64>()
                                                    .unwrap();

                                                let precio_anterior: f64 = df
                                                    .column("high")
                                                    .unwrap()
                                                    .get(i - 1)
                                                    .unwrap()
                                                    .try_extract::<f64>()
                                                    .unwrap();

                                                trade.sl = self.calcular_tls(
                                                    EntryDirection::Buy,
                                                    symbol.clone(),
                                                    parametros.clone(),
                                                    trade.clone(),
                                                    precio_actual,
                                                    precio_anterior,
                                                );

                                                if LOGS_REGISTRO {
                                                    self.add_registro(format!(
                                                        "TSL activado por porcentaje. Precio SL: {}",
                                                        trade.sl
                                                    ));
                                                }
                                            }
                                        }
                                        EntryDirection::Sell => {
                                            if self.activar_tsl(
                                                df.clone(),
                                                i.clone(),
                                                EntryDirection::Sell,
                                                parametros.clone(),
                                                symbol.clone(),
                                                trade.precio_entrada,
                                                precio_cierre,
                                            ) {
                                                let precio_actual: f64 = df
                                                    .column("low")
                                                    .unwrap()
                                                    .get(i)
                                                    .unwrap()
                                                    .try_extract::<f64>()
                                                    .unwrap();

                                                let precio_anterior: f64 = df
                                                    .column("low")
                                                    .unwrap()
                                                    .get(i - 1)
                                                    .unwrap()
                                                    .try_extract::<f64>()
                                                    .unwrap();

                                                trade.sl = self.calcular_tls(
                                                    EntryDirection::Sell,
                                                    symbol.clone(),
                                                    parametros.clone(),
                                                    trade.clone(),
                                                    precio_actual,
                                                    precio_anterior,
                                                );

                                                if LOGS_REGISTRO {
                                                    self.add_registro(format!(
                                                        "TSL activado por porcentaje. Precio SL: {}",
                                                        trade.sl
                                                    ));
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            TlTipo::Indicador => {
                                for trade in open_trades.iter_mut() {
                                    match trade.tipo {
                                        EntryDirection::Buy => {
                                            if self.activar_tsl(
                                                df.clone(),
                                                i.clone(),
                                                EntryDirection::Buy,
                                                parametros.clone(),
                                                symbol.clone(),
                                                trade.precio_entrada,
                                                precio_cierre,
                                            ) {
                                                let precio_actual: f64 = df
                                                    .column("high")
                                                    .unwrap()
                                                    .get(i)
                                                    .unwrap()
                                                    .try_extract::<f64>()
                                                    .unwrap();

                                                let precio_anterior: f64 = df
                                                    .column("high")
                                                    .unwrap()
                                                    .get(i - 1)
                                                    .unwrap()
                                                    .try_extract::<f64>()
                                                    .unwrap();

                                                trade.sl = self.calcular_tls(
                                                    EntryDirection::Buy,
                                                    symbol.clone(),
                                                    parametros.clone(),
                                                    trade.clone(),
                                                    precio_actual,
                                                    precio_anterior,
                                                );

                                                if LOGS_REGISTRO {
                                                    self.add_registro(format!(
                                                        "TSL activado por indicador. Precio SL: {}",
                                                        trade.sl
                                                    ));
                                                }
                                            }
                                        }
                                        EntryDirection::Sell => {
                                            if self.activar_tsl(
                                                df.clone(),
                                                i.clone(),
                                                EntryDirection::Sell,
                                                parametros.clone(),
                                                symbol.clone(),
                                                trade.precio_entrada,
                                                precio_cierre,
                                            ) {
                                                let precio_actual: f64 = df
                                                    .column("low")
                                                    .unwrap()
                                                    .get(i)
                                                    .unwrap()
                                                    .try_extract::<f64>()
                                                    .unwrap();

                                                let precio_anterior: f64 = df
                                                    .column("low")
                                                    .unwrap()
                                                    .get(i - 1)
                                                    .unwrap()
                                                    .try_extract::<f64>()
                                                    .unwrap();

                                                trade.sl = self.calcular_tls(
                                                    EntryDirection::Sell,
                                                    symbol.clone(),
                                                    parametros.clone(),
                                                    trade.clone(),
                                                    precio_actual,
                                                    precio_anterior,
                                                );

                                                if LOGS_REGISTRO {
                                                    self.add_registro(format!(
                                                        "TSL activado por indicador. Precio SL: {}",
                                                        trade.sl
                                                    ));
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    });

                if !indices.is_empty() {
                    for idx in &indices {
                        self.add_trade(open_trades[*idx].clone());
                    }

                    for idx in indices.iter().rev() {
                        if LOGS_REGISTRO {
                            self.add_registro(format!("Trade cerrado: {:?}", open_trades[*idx]));
                        }
                        open_trades.remove(*idx);
                    }
                }
            }

            let acciones = self.estrategia.acciones.clone();
            // Optenemos las acciones de entrada.
            for accion in acciones.iter().filter(|acc| acc.tipo_signal == "Entry") {
                match &accion.conditions {
                    Some(condition) => {
                        match accion.tipo {
                            Action::Buy => {
                                if self.verificar_direccion(EntryDirection::Buy)
                                    && self.entry_options(&open_trades)
                                    && self.check_conditions(&df, &condition, &i)
                                {
                                    let precio_entrada: f64 = df
                                        .column("open")
                                        .unwrap()
                                        .get(i + 1)
                                        .unwrap()
                                        .try_extract::<f64>()
                                        .unwrap();

                                    let timestamp = df
                                        .column("timestamp")
                                        .unwrap()
                                        .get(i + 1)
                                        .unwrap()
                                        .try_extract::<i64>()
                                        .unwrap();

                                    let tp: f64 = self.get_takeprofit(
                                        precio_entrada,
                                        df.clone(),
                                        i,
                                        EntryDirection::Buy,
                                    );
                                    let sl: f64 = self.get_stoploss(
                                        precio_entrada,
                                        df.clone(),
                                        i,
                                        EntryDirection::Buy,
                                    );

                                    let trade: Option<Trade> = self
                                        .ejecutar_entry(
                                            timestamp,
                                            symbol.clone(),
                                            EntryDirection::Buy,
                                            precio_entrada,
                                            Some(sl),
                                            Some(tp),
                                        )
                                        .await;

                                    if let Some(trade) = trade {
                                        open_trades.push(trade);
                                        break;
                                    }
                                }
                            }
                            Action::Sell => {
                                if self.verificar_direccion(EntryDirection::Sell)
                                    && self.entry_options(&open_trades)
                                    && self.check_conditions(&df, &condition, &i)
                                {
                                    let precio_entrada: f64 = df
                                        .column("open")
                                        .unwrap()
                                        .get(i + 1)
                                        .unwrap()
                                        .try_extract::<f64>()
                                        .unwrap();

                                    let timestamp = df
                                        .column("timestamp")
                                        .unwrap()
                                        .get(i + 1)
                                        .unwrap()
                                        .try_extract::<i64>()
                                        .unwrap();

                                    let tp: f64 = self.get_takeprofit(
                                        precio_entrada.clone(),
                                        df.clone(),
                                        i,
                                        EntryDirection::Sell,
                                    );
                                    let sl: f64 = self.get_stoploss(
                                        precio_entrada.clone(),
                                        df.clone(),
                                        i,
                                        EntryDirection::Sell,
                                    );

                                    let trade: Option<Trade> = self
                                        .ejecutar_entry(
                                            timestamp,
                                            symbol.clone(),
                                            EntryDirection::Sell,
                                            precio_entrada.clone(),
                                            Some(sl),
                                            Some(tp),
                                        )
                                        .await;

                                    if let Some(trade) = trade {
                                        open_trades.push(trade);
                                        break;
                                    }
                                }
                            }
                            Action::BuyLimit => {
                                if self.verificar_direccion(EntryDirection::Buy)
                                    && self.entry_options(&open_trades)
                                    && self.check_conditions(&df, &condition, &i)
                                {
                                    let precio_limite = self.get_limit(
                                        df.clone(),
                                        accion.parametros.to_string(),
                                        i,
                                    )?;
                                    buy_limits.push(precio_limite);
                                    break;
                                }
                            }
                            Action::SellLimit => {
                                if self.verificar_direccion(EntryDirection::Sell)
                                    && self.entry_options(&open_trades)
                                    && self.check_conditions(&df, &condition, &i)
                                {
                                    let precio_limite = self.get_limit(
                                        df.clone(),
                                        accion.parametros.to_string(),
                                        i,
                                    )?;
                                    sell_limits.push(precio_limite);
                                    break;
                                }
                            }
                            Action::BuyStop => {
                                if self.verificar_direccion(EntryDirection::Buy)
                                    && self.entry_options(&open_trades)
                                    && self.check_conditions(&df, &condition, &i)
                                {
                                    let precio_limite = self.get_limit(
                                        df.clone(),
                                        accion.parametros.to_string(),
                                        i,
                                    )?;
                                    buy_stops.push(precio_limite);
                                    break;
                                }
                            }
                            Action::SellStop => {
                                if self.verificar_direccion(EntryDirection::Sell)
                                    && self.entry_options(&open_trades)
                                    && self.check_conditions(&df, &condition, &i)
                                {
                                    let precio_limite = self.get_limit(
                                        df.clone(),
                                        accion.parametros.to_string(),
                                        i,
                                    )?;
                                    sell_stops.push(precio_limite);
                                    break;
                                }
                            }
                            _ => {}
                        };
                    }
                    None => {
                        if LOGS_REGISTRO {
                            self.add_registro(format!(
                                "No hay condiciones para la acción: {:?}",
                                accion
                            ));
                        }
                    }
                }
            }
        }

        if LOGS_REGISTRO {
            self.add_registro("Backtest ejecutado correctamente".to_string());
        }
        Ok("Backtest ejecutado correctamente".to_string())
    }

    pub async fn run(
        &mut self,
        id_startegy: i32,
        symbol: SymbolInfoCFD,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let inicio = Instant::now();

        if LOGS_REGISTRO {
            self.add_registro("Iniciando backtest...".to_string());
        }

        self.estrategia = match get_strategies_by_id(id_startegy).await {
            Ok(strategy) => {
                let mut estrategia: Strategy = strategy;

                match get_strategies_actions_by_strategy_id(estrategia.id).await {
                    Ok(acciones) => {
                        estrategia.acciones = acciones.clone();
                        if LOGS_REGISTRO {
                            self.add_registro(format!("Acciones cargadas: {:?}", acciones));
                        }
                    }
                    Err(e) => {
                        println!("Error al obtener acciones: {:?}", e);
                    }
                }

                match get_strategies_indicators_by_strategy_id(estrategia.id).await {
                    Ok(indicadores) => {
                        estrategia.indicadores = indicadores.clone();
                        if LOGS_REGISTRO {
                            self.add_registro(format!("Indicadores cargados: {:?}", indicadores));
                        }
                    }
                    Err(e) => {
                        println!("Error al obtener indicadores: {:?}", e);
                    }
                }

                // match get_strategies_conditions_by_strategy_id(estrategia.id).await {
                //     Ok(condiciones) => {
                //         estrategia.condiciones = condiciones.clone();
                //         if LOGS_REGISTRO {
                //             self.add_registro(format!("Condiciones cargadas: {:?}", condiciones));
                //         }
                //     }
                //     Err(e) => {
                //         println!("Error al obtener condiciones: {:?}", e);
                //     }
                // }

                if LOGS_REGISTRO {
                    self.add_registro(format!(
                        "Estrategia cargada correctamente: {:?}",
                        estrategia
                    ));
                }
                estrategia
            }
            Err(e) => {
                let error = format!("Error al obtener estrategia: {:?}", e);
                return Err(Box::new(error)).unwrap();
            }
        };

        if self.datos.is_empty() {
            if LOGS_REGISTRO {
                self.add_registro("No hay datos para ejecutar el backtest".to_string());
            }
            return Ok("No hay datos para ejecutar el backtest".to_string());
        }

        for data in self.datos.clone() {
            // Verificamos los indicadores que tiene la estrategia para añadirlos a los datos del DataFrame
            let mut df = data.get_datos();
            self.set_indicators_strategy(&mut df);
            df = df
                .lazy()
                .fill_nan(lit(NULL))
                .drop_nulls(None) // elimina filas con cualquier null
                .collect()?;

            if LOGS_REGISTRO {
                self.add_registro(format!("{:?}", &df.head(Some(20))));
            }

            self.backtest(&mut df, symbol.clone()).await.unwrap();
        }

        if !self.trades.is_empty() {
            if LOGS_REGISTRO {
                self.add_registro("Guardando los trades en la base de datos.".to_string());
            }

            self.guardar_trades().await;

            if LOGS_REGISTRO {
                self.add_registro("Trades guardados en la base de datos.".to_string());
            }

            let mut resultados = Resultados::new(self.id).await;

            if LOGS_REGISTRO {
                self.add_registro("Calculando los resultados.".to_string());
            }

            resultados.calcular_resultados(self.trades.clone(), self.balance.clone());

            if LOGS_REGISTRO {
                self.add_registro(format!("Resultados calculados: {:?}", resultados));
            }

            if LOGS_REGISTRO {
                self.add_registro("Guardando los resultados en la base de datos.".to_string());
            }

            resultados.guardar_resultados().await;
        } else {
            if LOGS_REGISTRO {
                self.add_registro("No se ha registrado ningun trade.".to_string());
            }
        }

        let duracion = inicio.elapsed();

        if LOGS_REGISTRO {
            self.add_registro(format!("Backtest finalizado en {}", duracion.as_secs_f64()));
        }
        Ok(format!("Backtest finalizado en {}", duracion.as_secs_f64()))
    }

    pub async fn guardar_trades(&self) {
        for trade in &self.trades {
            insert_trades(self.id, trade).await.unwrap();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::symbols::get_symbol_cfd_by_id;
    use crate::backtest::dias::Dias;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_bt_crucemedias() {
        let symbol: SymbolInfoCFD = get_symbol_cfd_by_id(1).await.unwrap_or(SymbolInfoCFD {
            id: 1,
            broker_id: 1,
            name: "EURUSD".to_string(),
            valor_contrato: 100000.0,
            comision_lote: 6.0,
            swap_long: -7.0,
            swap_short: 6.0,
            dia_triple_swap: Dias::Mi,
            lotaje_minimo: 0.01,
            lotaje_maximo: 100.0,
            digitos: 5,
            open_weekend: false,
            spread: 0.00020,
        });

        let gestion: GestionStrategy = GestionStrategy::Formula;
        let parametros_gestion: GestionParams = GestionParams {
            multiplicador: 1.0,
            lotaje_fijo: 0.10,
        };

        let mut bt: Backtest = Backtest::new(
            "UnitTest: CruceMedias".to_string(),
            10000.0,
            Activo::CDF,
            gestion,
            parametros_gestion,
        )
        .await;

        let _ = bt.add_datos("download/xauusd-h1.csv").unwrap();

        match bt.run(1, symbol).await {
            Ok(_) => assert!(true),
            Err(e) => assert!(false, "Error: {}", e),
        }
    }
}
