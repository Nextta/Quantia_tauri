use crate::api::backtests::{insert_backtest_cfd, table_backtests_cfd};
use crate::api::strategies::{
    get_strategies_actions_by_strategy_id, get_strategies_by_id,
    get_strategies_conditions_by_strategy_id, get_strategies_indicators_by_strategy_id,
};
use crate::api::trades::insert_trades;
use crate::backtest::datos::Datos;
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
use crate::strategy::strategy_action::StrategyAction;
use crate::strategy::strategy_options::TradingDirection;

use chrono::DateTime;
use polars::prelude::*;
// use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;

use crate::enums::gestion::GestionStrategy;
use crate::enums::tipos::{BeTipo, TlTipo};
use crate::structs::options::NBarsOptions;
use crate::structs::parametros::{BeParams, GestionParams, LimitParams, TlParams};

#[derive(Debug, Clone)]
pub struct Backtest {
    pub id: i32,
    pub titulo: String,
    pub balance: f64,
    pub tipo: String, // Tipo de activo ej: Forex, Crypto, Futuros...etc
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
        tipo: String,
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

    /// Añade los indicadores de la estrategia al dataframe de datos.
    ///
    /// # Parametros
    /// datos: Dataframe con los datos.
    ///
    /// # Retorna
    /// DataFrame con los nuevos datos.
    fn set_indicators_strategy(&mut self, datos: DataFrame) -> PolarsResult<DataFrame> {
        let mut df: DataFrame = datos.clone();
        //TODO: Añadir los indicadores de la estrategia al dataframe de datos.
        for indicator in &self.estrategia.indicadores {
            df = match indicator.tipo.as_str() {
                "HT_DCPERIOD" => ht_dcperiod(df, Some(&indicator.nombre))?,
                "HT_DCPHASE" => ht_dcphase(df, Some(&indicator.nombre))?,
                "HT_PHASOR" => ht_phasor(
                    df,
                    Some(format!("{}_in_phase", &indicator.nombre).as_str()),
                    Some(format!("{}_quadrature", &indicator.nombre).as_str()),
                )?,
                "HT_SINE" => ht_sine(
                    df,
                    Some(format!("{}_sine", &indicator.nombre).as_str()),
                    Some(format!("{}_lead_sine", &indicator.nombre).as_str()),
                )?,
                "HT_TRENDMODE" => ht_trendmode(df, Some(&indicator.nombre))?,
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
                    )?
                }
                "DEMA" => {
                    let parametros =
                        serde_json::from_value::<DemaParams>(indicator.parametros.clone()).unwrap();
                    dema(df, Some(parametros.timeperiod), Some(&indicator.nombre))?
                }
                "EMA" => {
                    let parametros =
                        serde_json::from_value::<EmaParams>(indicator.parametros.clone()).unwrap();
                    ema(df, Some(parametros.timeperiod), Some(&indicator.nombre))?
                }
                "KAMA" => {
                    let parametros =
                        serde_json::from_value::<KamaParams>(indicator.parametros.clone()).unwrap();
                    kama(df, Some(parametros.timeperiod), Some(&indicator.nombre))?
                }
                "MA" => {
                    let parametros =
                        serde_json::from_value::<MaParams>(indicator.parametros.clone()).unwrap();
                    ma(
                        df,
                        Some(parametros.timeperiod),
                        Some(parametros.matype),
                        Some(&indicator.nombre),
                    )?
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
                    )?
                }
                "MIDPOINT" => {
                    let parametros =
                        serde_json::from_value::<MidpointParams>(indicator.parametros.clone())
                            .unwrap();
                    midpoint(df, Some(parametros.timeperiod), Some(&indicator.nombre))?
                }
                "MIDPRICE" => {
                    let parametros =
                        serde_json::from_value::<MidpriceParams>(indicator.parametros.clone())
                            .unwrap();
                    midprice(df, Some(parametros.timeperiod), Some(&indicator.nombre))?
                }
                "SAR" => {
                    let parametros =
                        serde_json::from_value::<SarParams>(indicator.parametros.clone()).unwrap();
                    sar(
                        df,
                        Some(parametros.acceleration),
                        Some(parametros.maximum),
                        Some(&indicator.nombre),
                    )?
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
                    )?
                }
                "SMA" => {
                    let parametros =
                        serde_json::from_value::<SmaParams>(indicator.parametros.clone()).unwrap();
                    sma(df, Some(parametros.timeperiod), Some(&indicator.nombre))?
                }
                "T3" => {
                    let parametros =
                        serde_json::from_value::<T3Params>(indicator.parametros.clone()).unwrap();
                    t3(
                        df,
                        Some(parametros.timeperiod),
                        Some(parametros.vfactor),
                        Some(&indicator.nombre),
                    )?
                }
                "TEMA" => {
                    let parametros =
                        serde_json::from_value::<TemaParams>(indicator.parametros.clone()).unwrap();
                    tema(df, Some(parametros.timeperiod), Some(&indicator.nombre))?
                }
                "TRIMA" => {
                    let parametros =
                        serde_json::from_value::<TrimaParams>(indicator.parametros.clone())
                            .unwrap();
                    trima(df, Some(parametros.timeperiod), Some(&indicator.nombre))?
                }
                "WMA" => {
                    let parametros =
                        serde_json::from_value::<WmaParams>(indicator.parametros.clone()).unwrap();
                    wma(df, Some(parametros.timeperiod), Some(&indicator.nombre))?
                }
                "CDL2CROWS" => cdlupsidegap2crows(df, Some(&indicator.nombre))?,
                "CDL3BLACKCROWS" => cdl3blackcrows(df, Some(&indicator.nombre))?,
                "CDL3INSIDE" => cdl3inside(df, Some(&indicator.nombre))?,
                "CDL3LINESTRIKE" => cdl3linestrike(df, Some(&indicator.nombre))?,
                "CDL3OUTSIDE" => cdl3outside(df, Some(&indicator.nombre))?,
                "CDL3STARSINSOUTH" => cdl3starsinsouth(df, Some(&indicator.nombre))?,
                "CDL3WHITESOLDIERS" => cdl3whitesoldiers(df, Some(&indicator.nombre))?,
                "CDLABANDONEDBABY" => cdlabandonedbaby(df, Some(&indicator.nombre))?,
                "CDLADVANCEBLOCK" => cdladvanceblock(df, Some(&indicator.nombre))?,
                "CDLBELTHOLD" => cdlbelthold(df, Some(&indicator.nombre))?,
                "CDLBREAKAWAY" => cdlbreakaway(df, Some(&indicator.nombre))?,
                "CDLCLOSINGMARUBOZU" => cdlclosingmarubuzo(df, Some(&indicator.nombre))?,
                "CDLCONCEALBABYSWALL" => cdlconcealbabyswall(df, Some(&indicator.nombre))?,
                "CDLCOUNTERATTACK" => cdlcounterattack(df, Some(&indicator.nombre))?,
                "CDLDARKCLOUDCOVER" => cdldarkcloudcover(df, Some(&indicator.nombre))?,
                "CDLDOJI" => cdldoji(df, Some(&indicator.nombre))?,
                "CDLDOJISTAR" => cdldojistar(df, Some(&indicator.nombre))?,
                "CDLDRAGONFLYDOJI" => cdldragonflydoji(df, Some(&indicator.nombre))?,
                "CDLENGULFING" => cdlengulfing(df, Some(&indicator.nombre))?,
                "CDLEVENINGDOJISTAR" => cdleveningdojistar(df, Some(&indicator.nombre))?,
                "CDLEVENINGSTAR" => cdleveningstar(df, Some(&indicator.nombre))?,
                "CDLGAPSIDESIDEWHITE" => cdlgapsidesidewhite(df, Some(&indicator.nombre))?,
                "CDLGRAVESTONEDOJI" => cdlgravestonedoji(df, Some(&indicator.nombre))?,
                "CDLHAMMER" => cdlhammer(df, Some(&indicator.nombre))?,
                "CDLHANGINGMAN" => cdlhangingman(df, Some(&indicator.nombre))?,
                "CDLHARAMI" => cdlharami(df, Some(&indicator.nombre))?,
                "CDLHARAMICROSS" => cdlharamicross(df, Some(&indicator.nombre))?,
                "CDLHIGHWAVE" => cdlhighwave(df, Some(&indicator.nombre))?,
                "CDLHIKKAKE" => cdlhikkake(df, Some(&indicator.nombre))?,
                "CDLHIKKAKEMOD" => cdlhikkakemod(df, Some(&indicator.nombre))?,
                "CDLHOMINGPIGEON" => cdlhomingpigeon(df, Some(&indicator.nombre))?,
                "CDLIDENTICAL3CROWS" => cdlidentical3crows(df, Some(&indicator.nombre))?,
                "CDLINNECK" => cdlinneck(df, Some(&indicator.nombre))?,
                "CDLINVERTEDHAMMER" => cdlinvertedhammer(df, Some(&indicator.nombre))?,
                "CDLKICKING" => cdlkicking(df, Some(&indicator.nombre))?,
                "CDLKICKINGBYLENGTH" => cdlkickingbylength(df, Some(&indicator.nombre))?,
                "CDLLADDERBOTTOM" => cdladderbottom(df, Some(&indicator.nombre))?,
                "CDLLONGLEGGEDDOJI" => cdllongleggeddoji(df, Some(&indicator.nombre))?,
                "CDLLONGLINE" => cdllongline(df, Some(&indicator.nombre))?,
                "CDLMARUBOZU" => cdlmarubozu(df, Some(&indicator.nombre))?,
                "CDLMATCHINGLOW" => cdlmatchinglow(df, Some(&indicator.nombre))?,
                "CDLMATHOLD" => cdlmathold(df, Some(&indicator.nombre))?,
                "CDLMORNINGDOJISTAR" => {
                    let parametros =
                        serde_json::from_value::<MorningDojiStar>(indicator.parametros.clone())
                            .unwrap();
                    cdlmorningdojistar(df, Some(parametros.penetration), Some(&indicator.nombre))?
                }
                "CDLMORNINGSTAR" => {
                    let parametros =
                        serde_json::from_value::<MorningStar>(indicator.parametros.clone())
                            .unwrap();
                    cdlmorningstar(df, Some(parametros.penetration), Some(&indicator.nombre))?
                }
                "CDLONNECK" => cdlonneck(df, Some(&indicator.nombre))?,
                "CDLPIERCING" => {
                    let parametros =
                        serde_json::from_value::<Piercing>(indicator.parametros.clone()).unwrap();
                    cdlpiercing(df, Some(parametros.penetration), Some(&indicator.nombre))?
                }
                "CDLRICKSHAWMAN" => cdlrickshawman(df, Some(&indicator.nombre))?,
                "CDLRISEFALL3METHODS" => cdlrisefall3methods(df, Some(&indicator.nombre))?,
                "CDLSEPARATINGLINES" => cdlseparatinglines(df, Some(&indicator.nombre))?,
                "CDLSHOOTINGSTAR" => cdlshootingstar(df, Some(&indicator.nombre))?,
                "CDLSHORTLINE" => cdlshortline(df, Some(&indicator.nombre))?,
                "CDLSPINNINGTOP" => cdlspinningtop(df, Some(&indicator.nombre))?,
                "CDLSTALLEDPATTERN" => cdlstalledpattern(df, Some(&indicator.nombre))?,
                "CDLSTICKSANDWICH" => cdlsticksandwich(df, Some(&indicator.nombre))?,
                "CDLTAKURI" => cdltakuri(df, Some(&indicator.nombre))?,
                "CDLTASUKIGAP" => cdltasukigap(df, Some(&indicator.nombre))?,
                "CDLTHRUSTING" => cdlthrusting(df, Some(&indicator.nombre))?,
                "CDLTRISTAR" => cdltristar(df, Some(&indicator.nombre))?,
                "CDLUNIQUE3RIVER" => cdlunique3river(df, Some(&indicator.nombre))?,
                "CDLUPSIDEGAP2CROWS" => cdlupsidegap2crows(df, Some(&indicator.nombre))?,
                "CDLXSIDEGAP3METHODS" => cdlxsidegap3methods(df, Some(&indicator.nombre))?,
                "ADX" => {
                    let params: AdxParams =
                        serde_json::from_value::<AdxParams>(indicator.parametros.clone()).unwrap();
                    adx(df, Some(params.timeperiod), Some(&indicator.nombre))?
                }
                "ADXR" => {
                    let params: AdxrParams =
                        serde_json::from_value::<AdxrParams>(indicator.parametros.clone()).unwrap();
                    adxr(df, Some(params.timeperiod), Some(&indicator.nombre))?
                }
                "APO" => {
                    let params: ApoParams =
                        serde_json::from_value::<ApoParams>(indicator.parametros.clone()).unwrap();
                    apo(
                        df,
                        Some(params.fastperiod),
                        Some(params.slowperiod),
                        Some(&indicator.nombre),
                    )?
                }
                "AROON" => {
                    let params: AroonParams =
                        serde_json::from_value::<AroonParams>(indicator.parametros.clone())
                            .unwrap();
                    aroon(
                        df,
                        Some(params.timeperiod),
                        Some(format!("{}_col_up", &indicator.nombre).as_str()),
                        Some(format!("{}_col_down", &indicator.nombre).as_str()),
                    )?
                }
                "AROONOSC" => {
                    let params: AroonoscParams =
                        serde_json::from_value::<AroonoscParams>(indicator.parametros.clone())
                            .unwrap();
                    aroonosc(df, Some(params.timeperiod), Some(&indicator.nombre))?
                }
                "BOP" => {
                    let params: BopParams =
                        serde_json::from_value::<BopParams>(indicator.parametros.clone()).unwrap();
                    bop(df, Some(params.timeperiod), Some(&indicator.nombre))?
                }
                "CCI" => {
                    let params: CciParams =
                        serde_json::from_value::<CciParams>(indicator.parametros.clone()).unwrap();
                    cci(df, Some(params.timeperiod), Some(&indicator.nombre))?
                }
                "CMO" => {
                    let params: CmoParams =
                        serde_json::from_value::<CmoParams>(indicator.parametros.clone()).unwrap();
                    cmo(df, Some(params.timeperiod), Some(&indicator.nombre))?
                }
                "DX" => {
                    let params: DxParams =
                        serde_json::from_value::<DxParams>(indicator.parametros.clone()).unwrap();
                    dx(df, Some(params.timeperiod), Some(&indicator.nombre))?
                }
                "MACD" => {
                    let params: MacdParams =
                        serde_json::from_value::<MacdParams>(indicator.parametros.clone()).unwrap();
                    macd(
                        df,
                        Some(params.timeperiod),
                        Some(params.slowperiod),
                        Some(params.signalperiod),
                        Some(&indicator.nombre),
                        Some(format!("{}_col_signal", &indicator.nombre).as_str()),
                        Some(format!("{}_col_hist", &indicator.nombre).as_str()),
                    )?
                }
                "MACDEXT" => {
                    let params: MacdextParams =
                        serde_json::from_value::<MacdextParams>(indicator.parametros.clone())
                            .unwrap();
                    macdext(
                        df,
                        Some(params.fastperiod),
                        Some(params.slowperiod),
                        Some(params.signalperiod),
                        Some(params.fastmatype),
                        Some(params.slowmatype),
                        Some(params.signalmatype),
                        Some(&indicator.nombre),
                        Some(format!("{}_col_signal", &indicator.nombre).as_str()),
                        Some(format!("{}_col_hist", &indicator.nombre).as_str()),
                    )?
                }
                "MACDFIX" => {
                    let params: MacdfixParams =
                        serde_json::from_value::<MacdfixParams>(indicator.parametros.clone())
                            .unwrap();
                    macdfix(
                        df,
                        Some(params.signalperiod),
                        Some(&indicator.nombre),
                        Some(format!("{}_col_signal", &indicator.nombre).as_str()),
                        Some(format!("{}_col_hist", &indicator.nombre).as_str()),
                    )?
                }
                "MFI" => {
                    let params: MfiParams =
                        serde_json::from_value::<MfiParams>(indicator.parametros.clone()).unwrap();
                    mfi(df, Some(params.timeperiod), Some(&indicator.nombre))?
                }
                "MINUS_DI" => {
                    let params: MinusDiParams =
                        serde_json::from_value::<MinusDiParams>(indicator.parametros.clone())
                            .unwrap();
                    minus_di(df, Some(params.timeperiod), Some(&indicator.nombre))?
                }
                "MINUS_DM" => {
                    let params: MinusDmParams =
                        serde_json::from_value::<MinusDmParams>(indicator.parametros.clone())
                            .unwrap();
                    minus_dm(df, Some(params.timeperiod), Some(&indicator.nombre))?
                }
                "MOM" => {
                    let params: MomParams =
                        serde_json::from_value::<MomParams>(indicator.parametros.clone()).unwrap();
                    mom(df, Some(params.timeperiod), Some(&indicator.nombre))?
                }
                "PLUS_DI" => {
                    let params: PlusDiParams =
                        serde_json::from_value::<PlusDiParams>(indicator.parametros.clone())
                            .unwrap();
                    plus_di(df, Some(params.timeperiod), Some(&indicator.nombre))?
                }
                "PLUS_DM" => {
                    let params: PlusDmParams =
                        serde_json::from_value::<PlusDmParams>(indicator.parametros.clone())
                            .unwrap();
                    plus_dm(df, Some(params.timeperiod), Some(&indicator.nombre))?
                }
                "PPO" => {
                    let params: PpoParams =
                        serde_json::from_value::<PpoParams>(indicator.parametros.clone()).unwrap();
                    ppo(
                        df,
                        Some(params.fastperiod),
                        Some(params.slowperiod),
                        Some(&indicator.nombre),
                    )?
                }
                "ROC" => {
                    let params: RocParams =
                        serde_json::from_value::<RocParams>(indicator.parametros.clone()).unwrap();
                    roc(df, Some(params.timeperiod), Some(&indicator.nombre))?
                }
                "ROCP" => {
                    let params: RocpParams =
                        serde_json::from_value::<RocpParams>(indicator.parametros.clone()).unwrap();
                    rocp(df, Some(params.timeperiod), Some(&indicator.nombre))?
                }
                "ROCR" => {
                    let params: RocrParams =
                        serde_json::from_value::<RocrParams>(indicator.parametros.clone()).unwrap();
                    rocr(df, Some(params.timeperiod), Some(&indicator.nombre))?
                }
                "ROCR100" => {
                    let params: Roc100Params =
                        serde_json::from_value::<Roc100Params>(indicator.parametros.clone())
                            .unwrap();
                    rocr100(df, Some(params.timeperiod), Some(&indicator.nombre))?
                }
                "RSI" => {
                    let params: RsiParams =
                        serde_json::from_value::<RsiParams>(indicator.parametros.clone()).unwrap();
                    rsi(df, Some(params.timeperiod), Some(&indicator.nombre))?
                }
                "STOCH" => {
                    let params: StochParams =
                        serde_json::from_value::<StochParams>(indicator.parametros.clone())
                            .unwrap();
                    stoch(
                        df,
                        Some(params.fastk_period),
                        Some(params.slowk_period),
                        Some(params.slowk_matype),
                        Some(params.slowd_period),
                        Some(format!("{}_col_k", &indicator.nombre).as_str()),
                        Some(format!("{}_col_d", &indicator.nombre).as_str()),
                    )?
                }
                "STOCHF" => {
                    let params: StochfParams =
                        serde_json::from_value::<StochfParams>(indicator.parametros.clone())
                            .unwrap();
                    stochf(
                        df,
                        Some(params.fastk_period),
                        Some(params.fastd_period),
                        Some(params.fastd_matype),
                        Some(format!("{}_col_k", &indicator.nombre).as_str()),
                        Some(format!("{}_col_d", &indicator.nombre).as_str()),
                    )?
                }
                "STOCHRSI" => {
                    let params: StochRsiParams =
                        serde_json::from_value::<StochRsiParams>(indicator.parametros.clone())
                            .unwrap();
                    stochrsi(
                        df,
                        Some(params.timeperiod),
                        Some(params.fastk_period),
                        Some(params.fastd_period),
                        Some(params.fastd_matype),
                        Some(format!("{}_col_k", &indicator.nombre).as_str()),
                        Some(format!("{}_col_d", &indicator.nombre).as_str()),
                    )?
                }
                "TRIX" => {
                    let params: TrixParams =
                        serde_json::from_value::<TrixParams>(indicator.parametros.clone()).unwrap();
                    trix(df, Some(params.timeperiod), Some(&indicator.nombre))?
                }
                "ULTOSC" => {
                    let params: UltoscParams =
                        serde_json::from_value::<UltoscParams>(indicator.parametros.clone())
                            .unwrap();
                    ultosc(
                        df,
                        Some(params.timeperiod1),
                        Some(params.timeperiod2),
                        Some(params.timeperiod3),
                        Some(&indicator.nombre),
                    )?
                }
                "WILLR" => {
                    let params: WillrParams =
                        serde_json::from_value::<WillrParams>(indicator.parametros.clone())
                            .unwrap();
                    willr(df, Some(params.timeperiod), Some(&indicator.nombre))?
                }
                "AVGPRICE" => avgprice(df, Some(&indicator.nombre))?,
                "MEDPRICE" => medprice(df, Some(&indicator.nombre))?,
                "TYPPRICE" => typprice(df, Some(&indicator.nombre))?,
                "WCLPRICE" => wclprice(df, Some(&indicator.nombre))?,
                "BETA" => {
                    let params: BetaParams =
                        serde_json::from_value::<BetaParams>(indicator.parametros.clone()).unwrap();
                    beta(
                        df,
                        &params.col_real0,
                        &params.col_real1,
                        Some(params.timeperiod),
                        Some(&indicator.nombre),
                    )?
                }
                "CORREL" => {
                    let params: CorrelParams =
                        serde_json::from_value::<CorrelParams>(indicator.parametros.clone())
                            .unwrap();
                    correl(
                        df,
                        &params.col_real0,
                        &params.col_real1,
                        Some(params.timeperiod),
                        Some(&indicator.nombre),
                    )?
                }
                "LINEARREG" => {
                    let params: LinearRegParams =
                        serde_json::from_value::<LinearRegParams>(indicator.parametros.clone())
                            .unwrap();
                    linearreg(
                        df,
                        &params.col_real,
                        Some(params.timeperiod),
                        Some(&indicator.nombre),
                    )?
                }
                "LINEARREG_ANGLE" => {
                    let params: LinearRegAngleParams =
                        serde_json::from_value::<LinearRegAngleParams>(
                            indicator.parametros.clone(),
                        )
                        .unwrap();
                    linearreg_angle(
                        df,
                        &params.col_real,
                        Some(params.timeperiod),
                        Some(&indicator.nombre),
                    )?
                }
                "LINEARREG_INTERCEPT" => {
                    let params: LinearRegInterceptParams =
                        serde_json::from_value::<LinearRegInterceptParams>(
                            indicator.parametros.clone(),
                        )
                        .unwrap();
                    linearreg_intercept(
                        df,
                        &params.col_real,
                        Some(params.timeperiod),
                        Some(&indicator.nombre),
                    )?
                }
                "LINEARREG_SLOPE" => {
                    let params: LinearRegSlopeParams =
                        serde_json::from_value::<LinearRegSlopeParams>(
                            indicator.parametros.clone(),
                        )
                        .unwrap();
                    linearreg_slope(
                        df,
                        &params.col_real,
                        Some(params.timeperiod),
                        Some(&indicator.nombre),
                    )?
                }
                "STDDEV" => {
                    let params: StddevParams =
                        serde_json::from_value::<StddevParams>(indicator.parametros.clone())
                            .unwrap();
                    stddev(
                        df,
                        &params.col_real,
                        Some(params.timeperiod),
                        Some(params.nbdev),
                        Some(&indicator.nombre),
                    )?
                }
                "TSF" => {
                    let params: TsfParams =
                        serde_json::from_value::<TsfParams>(indicator.parametros.clone()).unwrap();
                    tsf(
                        df,
                        &params.col_real,
                        Some(params.timeperiod),
                        Some(&indicator.nombre),
                    )?
                }
                "VAR" => {
                    let params: VarParams =
                        serde_json::from_value::<VarParams>(indicator.parametros.clone()).unwrap();
                    var(
                        df,
                        &params.col_real,
                        Some(params.timeperiod),
                        Some(params.nbdev),
                        Some(&indicator.nombre),
                    )?
                }
                "TRANGE" => trange(df, Some(&indicator.nombre))?,
                "ATR" => {
                    let params: AtrParams =
                        serde_json::from_value::<AtrParams>(indicator.parametros.clone()).unwrap();
                    atr(df, Some(params.timeperiod), Some(&indicator.nombre))?
                }
                "NATR" => {
                    let params: NatrParams =
                        serde_json::from_value::<NatrParams>(indicator.parametros.clone()).unwrap();
                    natr(df, Some(params.timeperiod), Some(&indicator.nombre))?
                }
                "AD" => ad(df, Some(&indicator.nombre))?,
                "ADOSC" => {
                    let params: AdoscParams =
                        serde_json::from_value::<AdoscParams>(indicator.parametros.clone())
                            .unwrap();
                    adosc(
                        df,
                        Some(params.fastperiod),
                        Some(params.slowperiod),
                        Some(&indicator.nombre),
                    )?
                }
                "OBV" => obv(df, Some(&indicator.nombre))?,
                _ => df,
            }
        }

        Ok(df)
    }

    /// Confirma si se puede operar en una dirección de compra o venta.
    ///
    /// # Parametro
    /// tipo: Si es buy o sell.
    ///
    /// # Retorna
    /// True si se puede operar en la dirección indicada, false en caso contrario.
    fn verificar_direccion(&self, tipo: EntryDirection) -> bool {
        match tipo {
            EntryDirection::Buy => {
                self.estrategia.opciones.trading_direccion == TradingDirection::Long
                    || self.estrategia.opciones.trading_direccion == TradingDirection::Both
            }
            EntryDirection::Sell => {
                self.estrategia.opciones.trading_direccion == TradingDirection::Short
                    || self.estrategia.opciones.trading_direccion == TradingDirection::Both
            }
        }
    }

    /// Comprueba todas las condiciones de una acción en un índice dado.
    ///
    /// # Parametros
    /// df: Dataframe con los datos.
    /// accion: Acción a confirmar.
    /// i: Índice del dataframe.
    ///
    /// # Retorna
    /// True si se puede operar en la dirección indicada, false en caso contrario.
    fn test_conditions(&self, df: DataFrame, accion: StrategyAction, i: usize) -> bool {
        let mut condiciones_map: HashMap<String, bool> = HashMap::new();
        self.estrategia
            .condiciones
            .iter()
            .filter(|condicion| condicion.action_id == accion.id)
            .for_each(|condicion| {
                let campo_a = df
                    .column(&condicion.campo_a)
                    .unwrap()
                    .get(i - condicion.shift_a as usize)
                    .unwrap();
                let campo_b = df
                    .column(&condicion.campo_b)
                    .unwrap()
                    .get(i - condicion.shift_b as usize)
                    .unwrap();
                let resultado = match condicion.operador.as_str() {
                    ">" => campo_a > campo_b,
                    "<" => campo_a < campo_b,
                    "==" => campo_a == campo_b,
                    _ => false,
                };

                condiciones_map.insert(condicion.logica.clone(), resultado);
            });

        let mut all_true = true;
        let mut key_anterior = "none".to_string();
        let mut resultado_anterior = true;
        for (key, resultado) in condiciones_map.iter() {
            if key_anterior == "none".to_string() {
                key_anterior = key.clone();
                resultado_anterior = *resultado;
            } else {
                match key.as_str() {
                    "AND" => all_true = resultado_anterior == *resultado,
                    "OR" => {
                        all_true =
                            resultado_anterior != *resultado || resultado_anterior == *resultado
                    }
                    _ => all_true = false,
                }

                if !all_true {
                    break;
                }
            }
        }

        all_true
    }

    /// Comprueba las opciones de entrada de una acción en un índice dado.
    ///
    /// # Parametros
    /// open_trades: Vector con los trades abiertos.
    ///
    /// # Retorna
    /// True si se puede operar en la dirección indicada, false en caso contrario.
    fn entry_options(&self, open_trades: &Vec<Trade>) -> bool {
        let entry_options: bool;
        if !self.estrategia.opciones.multiples_tardes && !open_trades.is_empty() {
            entry_options = false;
        } else {
            entry_options = true;
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
    fn get_limit(&self, df: DataFrame, params: String, i: usize) -> Result<f64, serde_json::Error> {
        let params: LimitParams = serde_json::from_str(&params).unwrap();

        let valor: f64 = df
            .column(&params.nombre_col)
            .unwrap()
            .f64()
            .unwrap()
            .get(i - params.shift)
            .unwrap_or(0.0);

        let limit: f64 = match params.tipo.as_str() {
            "pip" => {
                if params.direccion == EntryDirection::Buy {
                    valor + params.valor
                } else if params.direccion == EntryDirection::Sell {
                    valor - params.valor
                } else {
                    0.0
                }
            }
            "tick" => {
                if params.direccion == EntryDirection::Buy {
                    valor + params.valor
                } else if params.direccion == EntryDirection::Sell {
                    valor - params.valor
                } else {
                    0.0
                }
            }
            "punto" => {
                if params.direccion == EntryDirection::Buy {
                    valor + params.valor
                } else if params.direccion == EntryDirection::Sell {
                    valor - params.valor
                } else {
                    0.0
                }
            }
            "porcentaje" => {
                if params.direccion == EntryDirection::Buy {
                    valor + (valor * params.valor)
                } else if params.direccion == EntryDirection::Sell {
                    valor - (valor * params.valor)
                } else {
                    0.0
                }
            }
            "atr" => {
                if params.direccion == EntryDirection::Buy {
                    valor + params.valor
                } else if params.direccion == EntryDirection::Sell {
                    valor - params.valor
                } else {
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
    fn get_stoploss(&self, df: DataFrame, i: usize) -> f64 {
        let params = &self.estrategia.opciones.parametros_stoploss.clone();

        let valor: f64 = df
            .column(&params.nombre_col)
            .unwrap()
            .f64()
            .unwrap()
            .get(i - &params.shift)
            .unwrap_or(0.0);

        let sl: f64 = match params.tipo.as_str() {
            "pip" => {
                if params.direccion == EntryDirection::Buy {
                    valor + params.valor
                } else if params.direccion == EntryDirection::Sell {
                    valor - params.valor
                } else {
                    0.0
                }
            }
            "tick" => {
                if params.direccion == EntryDirection::Buy {
                    valor + params.valor
                } else if params.direccion == EntryDirection::Sell {
                    valor - params.valor
                } else {
                    0.0
                }
            }
            "punto" => {
                if params.direccion == EntryDirection::Buy {
                    valor + params.valor
                } else if params.direccion == EntryDirection::Sell {
                    valor - params.valor
                } else {
                    0.0
                }
            }
            "porcentaje" => {
                if params.direccion == EntryDirection::Buy {
                    valor + (valor * params.valor)
                } else if params.direccion == EntryDirection::Sell {
                    valor - (valor * params.valor)
                } else {
                    0.0
                }
            }
            "atr" => {
                if params.direccion == EntryDirection::Buy {
                    valor + params.valor
                } else if params.direccion == EntryDirection::Sell {
                    valor - params.valor
                } else {
                    0.0
                }
            }
            _ => valor,
        };

        sl
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
    fn get_takeprofit(&self, df: DataFrame, i: usize) -> f64 {
        let params = &self.estrategia.opciones.parametros_takeprofit.clone();

        let valor: f64 = df
            .column(&params.nombre_col)
            .unwrap()
            .f64()
            .unwrap()
            .get(i - &params.shift)
            .unwrap_or(0.0);

        let tp: f64 = match params.tipo.as_str() {
            "pip" => {
                if params.direccion == EntryDirection::Buy {
                    valor + params.valor
                } else if params.direccion == EntryDirection::Sell {
                    valor - params.valor
                } else {
                    0.0
                }
            }
            "tick" => {
                if params.direccion == EntryDirection::Buy {
                    valor + params.valor
                } else if params.direccion == EntryDirection::Sell {
                    valor - params.valor
                } else {
                    0.0
                }
            }
            "punto" => {
                if params.direccion == EntryDirection::Buy {
                    valor + params.valor
                } else if params.direccion == EntryDirection::Sell {
                    valor - params.valor
                } else {
                    0.0
                }
            }
            "porcentaje" => {
                if params.direccion == EntryDirection::Buy {
                    valor + (valor * params.valor)
                } else if params.direccion == EntryDirection::Sell {
                    valor - (valor * params.valor)
                } else {
                    0.0
                }
            }
            "atr" => {
                if params.direccion == EntryDirection::Buy {
                    valor + params.valor
                } else if params.direccion == EntryDirection::Sell {
                    valor - params.valor
                } else {
                    0.0
                }
            }
            _ => valor,
        };

        tp
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
        &self,
        timestamp: i64,
        symbol: SymbolInfoCFD,
        signal: EntryDirection,
        precio_entrada: f64,
        stoploss: Option<f64>,
        takeprofit: Option<f64>,
    ) -> Option<Trade> {
        let mut trade: Trade = Trade::new(self.id.clone(), symbol.clone()).await;

        let sl = stoploss.unwrap_or(0.0);
        let tp = takeprofit.unwrap_or(0.0);

        let naive_time = DateTime::from_timestamp_millis(timestamp).expect("timestamp inválido");
        let t0 = naive_time.format("%Y-%m-%d %H:%M:%S").to_string();

        match signal {
            EntryDirection::Buy => {
                trade.buy(
                    t0,
                    precio_entrada,
                    self.gestion_strategy.clone(),
                    self.parametros_gestion.clone(),
                    &self,
                    Some(tp),
                    Some(sl),
                );

                return Some(trade);
            }
            EntryDirection::Sell => {
                trade.sell(
                    t0,
                    precio_entrada,
                    self.gestion_strategy.clone(),
                    self.parametros_gestion.clone(),
                    &self,
                    Some(tp),
                    Some(sl),
                );

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
        &self,
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
        &self,
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
        &self,
        data: DataFrame,
        i: usize,
        tipo: EntryDirection,
        parametros: TlParams,
        symbol: SymbolInfoCFD,
        precio_entrada: f64,
        precio_actual: f64,
    ) -> bool {
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
            true
        } else {
            if indicador {
                true
            } else {
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
        df: DataFrame,
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

                    let tp: f64 = self.get_takeprofit(df.clone(), i);
                    let sl: f64 = self.get_stoploss(df.clone(), i);

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
                        open_trades.push(trade);
                    }

                    indices.push(idx);
                }

                for idx in indices.iter().rev() {
                    buy_limits.remove(*idx);
                }
            }

            if !buy_stops.is_empty() {
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

                    let tp: f64 = self.get_takeprofit(df.clone(), i);
                    let sl: f64 = self.get_stoploss(df.clone(), i);

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
                        open_trades.push(trade);
                    }

                    indices.push(idx);
                }

                for idx in indices.iter().rev() {
                    buy_stops.remove(*idx);
                }
            }

            if !sell_limits.is_empty() {
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

                    let tp: f64 = self.get_takeprofit(df.clone(), i);
                    let sl: f64 = self.get_stoploss(df.clone(), i);

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
                        open_trades.push(trade);
                    }

                    indices.push(idx);
                }

                for idx in indices.iter().rev() {
                    sell_limits.remove(*idx);
                }
            }

            if !sell_stops.is_empty() {
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

                    let tp: f64 = self.get_takeprofit(df.clone(), i);
                    let sl: f64 = self.get_stoploss(df.clone(), i);

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
                        open_trades.push(trade);
                    }

                    indices.push(idx);
                }

                for idx in indices.iter().rev() {
                    sell_stops.remove(*idx);
                }
            }

            if !open_trades.is_empty() {
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
                    match trade.tipo {
                        EntryDirection::Buy => {
                            if precio_actual <= trade.sl {
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

                                trade.close(t1, trade.sl);
                                indices.push(idx);
                                self.add_trade(trade.clone());
                            }
                            if precio_actual >= trade.tp {
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

                                trade.close(t1, trade.tp);
                                indices.push(idx);
                                self.add_trade(trade.clone());
                            }
                        }
                        EntryDirection::Sell => {
                            if precio_actual >= trade.sl {
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

                                trade.close(t1, trade.sl);
                                indices.push(idx);
                                self.add_trade(trade.clone());
                            }
                            if precio_actual <= trade.tp {
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

                                trade.close(t1, trade.tp);
                                indices.push(idx);
                                self.add_trade(trade.clone());
                            }
                        }
                    }
                }

                // Cerrar trades abiertos basados en las condiciones de salida
                self.estrategia
                    .acciones
                    .iter()
                    .filter(|acc| acc.tipo_signal == "Exit")
                    .for_each(|accion| match accion.tipo.as_str() {
                        "exit_buy" => {
                            if self.test_conditions(df.clone(), accion.clone(), i) {
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
                                            trade.close(t1.clone(), precio_cierre);
                                            indices.push(idx);
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                        "exit_sell" => {
                            if self.test_conditions(df.clone(), accion.clone(), i) {
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
                                            trade.close(t1.clone(), precio_cierre);
                                            indices.push(idx);
                                        }

                                        _ => {}
                                    }
                                }
                            }
                        }
                        "N_bars" => {
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

                                    trade.close(t1, precio_cierre);
                                    indices.push(idx);
                                }
                            }
                        }
                        "Close_all_rule" => {
                            if self.test_conditions(df.clone(), accion.clone(), i) {
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
                                    trade.close(t1.clone(), precio_cierre);
                                    indices.push(idx);
                                }
                            }
                        }
                        _ => {}
                    });

                // Activamos el Breakeven segun las condiciones definidas en las acciones
                self.estrategia
                    .acciones
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
                                            }
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    });

                // Activamos las opciones de trailing stoploss segun la configuracion de las acciones
                self.estrategia
                    .acciones
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
                                            ) {}
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
                                            ) {}
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
                                            ) {}
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
                                            ) {}
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
                                            ) {}
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
                                            ) {}
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
                                            ) {}
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
                                            ) {}
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
                                            ) {}
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
                                            ) {}
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
                        open_trades.remove(*idx);
                    }
                }
            }

            // Optenemos las acciones de entrada.
            for accion in self
                .estrategia
                .acciones
                .iter()
                .filter(|acc| acc.tipo_signal == "Entry")
            {
                match accion.tipo {
                    Action::Buy => {
                        if self.verificar_direccion(EntryDirection::Buy)
                            && self.entry_options(&open_trades)
                            && self.test_conditions(df.clone(), accion.clone(), i)
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

                            let tp: f64 = self.get_takeprofit(df.clone(), i);
                            let sl: f64 = self.get_stoploss(df.clone(), i);

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
                            }
                        }
                    }
                    Action::Sell => {
                        if self.verificar_direccion(EntryDirection::Sell)
                            && self.entry_options(&open_trades)
                            && self.test_conditions(df.clone(), accion.clone(), i)
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

                            let tp: f64 = self.get_takeprofit(df.clone(), i);
                            let sl: f64 = self.get_stoploss(df.clone(), i);

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
                            }
                        }
                    }
                    Action::BuyLimit => {
                        if self.verificar_direccion(EntryDirection::Buy)
                            && self.entry_options(&open_trades)
                            && self.test_conditions(df.clone(), accion.clone(), i)
                        {
                            let precio_limite =
                                self.get_limit(df.clone(), accion.parametros.to_string(), i)?;
                            buy_limits.push(precio_limite);
                        }
                    }
                    Action::SellLimit => {
                        if self.verificar_direccion(EntryDirection::Sell)
                            && self.entry_options(&open_trades)
                            && self.test_conditions(df.clone(), accion.clone(), i)
                        {
                            let precio_limite =
                                self.get_limit(df.clone(), accion.parametros.to_string(), i)?;
                            sell_limits.push(precio_limite);
                        }
                    }
                    Action::BuyStop => {
                        if self.verificar_direccion(EntryDirection::Buy)
                            && self.entry_options(&open_trades)
                            && self.test_conditions(df.clone(), accion.clone(), i)
                        {
                            let precio_limite =
                                self.get_limit(df.clone(), accion.parametros.to_string(), i)?;
                            buy_stops.push(precio_limite);
                        }
                    }
                    Action::SellStop => {
                        if self.verificar_direccion(EntryDirection::Sell)
                            && self.entry_options(&open_trades)
                            && self.test_conditions(df.clone(), accion.clone(), i)
                        {
                            let precio_limite =
                                self.get_limit(df.clone(), accion.parametros.to_string(), i)?;
                            sell_stops.push(precio_limite);
                        }
                    }
                    _ => {}
                };
            }
        }
        Ok("Backtest ejecutado correctamente".to_string())
    }

    pub async fn run(
        &mut self,
        id_startegy: i32,
        symbol: SymbolInfoCFD,
    ) -> Result<String, Box<dyn std::error::Error>> {
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
                let error = format!("Error al obtener estrategia: {:?}", e);
                return Err(Box::new(error)).unwrap();
            }
        };

        if self.datos.is_empty() {
            return Ok("No hay datos para ejecutar el backtest".to_string());
        }

        for data in self.datos.clone() {
            // Verificamos los indicadores que tiene la estrategia para añadirlos a los datos del DataFrame
            let df = match self.set_indicators_strategy(data.get_datos()) {
                Ok(df_result) => df_result,
                Err(e) => {
                    return Err(Box::new(e));
                }
            };

            //TODO: Backtest de la estrategia con los datos del DataFrame.
            self.backtest(df.clone(), symbol.clone()).await.unwrap();
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
