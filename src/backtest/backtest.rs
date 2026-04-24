use crate::api::backtests::{insert_backtest_cfd, table_backtests_cfd};
use crate::api::strategies::{
    get_strategies_actions_by_strategy_id, get_strategies_by_id,
    get_strategies_conditions_by_strategy_id, get_strategies_indicators_by_strategy_id,
};
use crate::api::trades::insert_trades;
use crate::backtest::datos::Datos;
use crate::backtest::trade::Trade;
use crate::indicators::cycle::*;
use crate::indicators::momentum::*;
use crate::indicators::overlap::*;
use crate::indicators::pattern::*;
use crate::indicators::price::*;
use crate::indicators::statistic::*;
use crate::indicators::volatility::*;
use crate::indicators::volume::*;
use crate::strategy::strategy::Strategy;
use crate::strategy::strategy_options::{StrategyOptions, TradingDirection};
use serde::Serialize;
use serde_json::Value;
// use polars::datatypes::DataType;
use polars::prelude::*;
// use std::collections::HashMap;
use std::time::Instant;

#[derive(Debug, Clone)]
pub enum GestionStrategy {
    Formula,
}

impl ToString for GestionStrategy {
    fn to_string(&self) -> String {
        match self {
            GestionStrategy::Formula => "Formula".to_string(),
        }
    }
}

#[derive(Serialize, Debug)]
pub struct GestionFormula {
    pub multiplicador: f64,
}

#[derive(Debug, Clone)]
pub struct Backtest {
    pub id: i32,
    pub titulo: String,
    pub balance: f64,
    pub tipo: String, // Tipo de activo ej: Forex, Crypto, Futuros...etc
    pub gestion_strategy: GestionStrategy,
    pub parametros_gestion: Value,
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
        parametros_gestion: Value,
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
                "CDL2CROWS" => cdlupsidegap2crows(df, Some(&indicator.nombre)).await?,
                "CDL3BLACKCROWS" => cdl3blackcrows(df, Some(&indicator.nombre)).await?,
                "CDL3INSIDE" => cdl3inside(df, Some(&indicator.nombre)).await?,
                "CDL3LINESTRIKE" => cdl3linestrike(df, Some(&indicator.nombre)).await?,
                "CDL3OUTSIDE" => cdl3outside(df, Some(&indicator.nombre)).await?,
                "CDL3STARSINSOUTH" => cdl3starsinsouth(df, Some(&indicator.nombre)).await?,
                "CDL3WHITESOLDIERS" => cdl3whitesoldiers(df, Some(&indicator.nombre)).await?,
                "CDLABANDONEDBABY" => cdlabandonedbaby(df, Some(&indicator.nombre)).await?,
                "CDLADVANCEBLOCK" => cdladvanceblock(df, Some(&indicator.nombre)).await?,
                "CDLBELTHOLD" => cdlbelthold(df, Some(&indicator.nombre)).await?,
                "CDLBREAKAWAY" => cdlbreakaway(df, Some(&indicator.nombre)).await?,
                "CDLCLOSINGMARUBOZU" => cdlclosingmarubuzo(df, Some(&indicator.nombre)).await?,
                "CDLCONCEALBABYSWALL" => cdlconcealbabyswall(df, Some(&indicator.nombre)).await?,
                "CDLCOUNTERATTACK" => cdlcounterattack(df, Some(&indicator.nombre)).await?,
                "CDLDARKCLOUDCOVER" => cdldarkcloudcover(df, Some(&indicator.nombre)).await?,
                "CDLDOJI" => cdldoji(df, Some(&indicator.nombre)).await?,
                "CDLDOJISTAR" => cdldojistar(df, Some(&indicator.nombre)).await?,
                "CDLDRAGONFLYDOJI" => cdldragonflydoji(df, Some(&indicator.nombre)).await?,
                "CDLENGULFING" => cdlengulfing(df, Some(&indicator.nombre)).await?,
                "CDLEVENINGDOJISTAR" => cdleveningdojistar(df, Some(&indicator.nombre)).await?,
                "CDLEVENINGSTAR" => cdleveningstar(df, Some(&indicator.nombre)).await?,
                "CDLGAPSIDESIDEWHITE" => cdlgapsidesidewhite(df, Some(&indicator.nombre)).await?,
                "CDLGRAVESTONEDOJI" => cdlgravestonedoji(df, Some(&indicator.nombre)).await?,
                "CDLHAMMER" => cdlhammer(df, Some(&indicator.nombre)).await?,
                "CDLHANGINGMAN" => cdlhangingman(df, Some(&indicator.nombre)).await?,
                "CDLHARAMI" => cdlharami(df, Some(&indicator.nombre)).await?,
                "CDLHARAMICROSS" => cdlharamicross(df, Some(&indicator.nombre)).await?,
                "CDLHIGHWAVE" => cdlhighwave(df, Some(&indicator.nombre)).await?,
                "CDLHIKKAKE" => cdlhikkake(df, Some(&indicator.nombre)).await?,
                "CDLHIKKAKEMOD" => cdlhikkakemod(df, Some(&indicator.nombre)).await?,
                "CDLHOMINGPIGEON" => cdlhomingpigeon(df, Some(&indicator.nombre)).await?,
                "CDLIDENTICAL3CROWS" => cdlidentical3crows(df, Some(&indicator.nombre)).await?,
                "CDLINNECK" => cdlinneck(df, Some(&indicator.nombre)).await?,
                "CDLINVERTEDHAMMER" => cdlinvertedhammer(df, Some(&indicator.nombre)).await?,
                "CDLKICKING" => cdlkicking(df, Some(&indicator.nombre)).await?,
                "CDLKICKINGBYLENGTH" => cdlkickingbylength(df, Some(&indicator.nombre)).await?,
                "CDLLADDERBOTTOM" => cdladderbottom(df, Some(&indicator.nombre)).await?,
                "CDLLONGLEGGEDDOJI" => cdllongleggeddoji(df, Some(&indicator.nombre)).await?,
                "CDLLONGLINE" => cdllongline(df, Some(&indicator.nombre)).await?,
                "CDLMARUBOZU" => cdlmarubozu(df, Some(&indicator.nombre)).await?,
                "CDLMATCHINGLOW" => cdlmatchinglow(df, Some(&indicator.nombre)).await?,
                "CDLMATHOLD" => cdlmathold(df, Some(&indicator.nombre)).await?,
                "CDLMORNINGDOJISTAR" => {
                    let parametros =
                        serde_json::from_value::<MorningDojiStar>(indicator.parametros.clone())
                            .unwrap();
                    cdlmorningdojistar(df, Some(parametros.penetration), Some(&indicator.nombre))
                        .await?
                }
                "CDLMORNINGSTAR" => {
                    let parametros =
                        serde_json::from_value::<MorningStar>(indicator.parametros.clone())
                            .unwrap();
                    cdlmorningstar(df, Some(parametros.penetration), Some(&indicator.nombre))
                        .await?
                }
                "CDLONNECK" => cdlonneck(df, Some(&indicator.nombre)).await?,
                "CDLPIERCING" => {
                    let parametros =
                        serde_json::from_value::<Piercing>(indicator.parametros.clone()).unwrap();
                    cdlpiercing(df, Some(parametros.penetration), Some(&indicator.nombre)).await?
                }
                "CDLRICKSHAWMAN" => cdlrickshawman(df, Some(&indicator.nombre)).await?,
                "CDLRISEFALL3METHODS" => cdlrisefall3methods(df, Some(&indicator.nombre)).await?,
                "CDLSEPARATINGLINES" => cdlseparatinglines(df, Some(&indicator.nombre)).await?,
                "CDLSHOOTINGSTAR" => cdlshootingstar(df, Some(&indicator.nombre)).await?,
                "CDLSHORTLINE" => cdlshortline(df, Some(&indicator.nombre)).await?,
                "CDLSPINNINGTOP" => cdlspinningtop(df, Some(&indicator.nombre)).await?,
                "CDLSTALLEDPATTERN" => cdlstalledpattern(df, Some(&indicator.nombre)).await?,
                "CDLSTICKSANDWICH" => cdlsticksandwich(df, Some(&indicator.nombre)).await?,
                "CDLTAKURI" => cdltakuri(df, Some(&indicator.nombre)).await?,
                "CDLTASUKIGAP" => cdltasukigap(df, Some(&indicator.nombre)).await?,
                "CDLTHRUSTING" => cdlthrusting(df, Some(&indicator.nombre)).await?,
                "CDLTRISTAR" => cdltristar(df, Some(&indicator.nombre)).await?,
                "CDLUNIQUE3RIVER" => cdlunique3river(df, Some(&indicator.nombre)).await?,
                "CDLUPSIDEGAP2CROWS" => cdlupsidegap2crows(df, Some(&indicator.nombre)).await?,
                "CDLXSIDEGAP3METHODS" => cdlxsidegap3methods(df, Some(&indicator.nombre)).await?,
                "ADX" => {
                    let params: AdxParams =
                        serde_json::from_value::<AdxParams>(indicator.parametros.clone()).unwrap();
                    adx(df, Some(params.timeperiod), Some(&indicator.nombre)).await?
                }
                "ADXR" => {
                    let params: AdxrParams =
                        serde_json::from_value::<AdxrParams>(indicator.parametros.clone()).unwrap();
                    adxr(df, Some(params.timeperiod), Some(&indicator.nombre)).await?
                }
                "APO" => {
                    let params: ApoParams =
                        serde_json::from_value::<ApoParams>(indicator.parametros.clone()).unwrap();
                    apo(
                        df,
                        Some(params.fastperiod),
                        Some(params.slowperiod),
                        Some(&indicator.nombre),
                    )
                    .await?
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
                    )
                    .await?
                }
                "AROONOSC" => {
                    let params: AroonoscParams =
                        serde_json::from_value::<AroonoscParams>(indicator.parametros.clone())
                            .unwrap();
                    aroonosc(df, Some(params.timeperiod), Some(&indicator.nombre)).await?
                }
                "BOP" => {
                    let params: BopParams =
                        serde_json::from_value::<BopParams>(indicator.parametros.clone()).unwrap();
                    bop(df, Some(params.timeperiod), Some(&indicator.nombre)).await?
                }
                "CCI" => {
                    let params: CciParams =
                        serde_json::from_value::<CciParams>(indicator.parametros.clone()).unwrap();
                    cci(df, Some(params.timeperiod), Some(&indicator.nombre)).await?
                }
                "CMO" => {
                    let params: CmoParams =
                        serde_json::from_value::<CmoParams>(indicator.parametros.clone()).unwrap();
                    cmo(df, Some(params.timeperiod), Some(&indicator.nombre)).await?
                }
                "DX" => {
                    let params: DxParams =
                        serde_json::from_value::<DxParams>(indicator.parametros.clone()).unwrap();
                    dx(df, Some(params.timeperiod), Some(&indicator.nombre)).await?
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
                    )
                    .await?
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
                    )
                    .await?
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
                    )
                    .await?
                }
                "MFI" => {
                    let params: MfiParams =
                        serde_json::from_value::<MfiParams>(indicator.parametros.clone()).unwrap();
                    mfi(df, Some(params.timeperiod), Some(&indicator.nombre)).await?
                }
                "MINUS_DI" => {
                    let params: MinusDiParams =
                        serde_json::from_value::<MinusDiParams>(indicator.parametros.clone())
                            .unwrap();
                    minus_di(df, Some(params.timeperiod), Some(&indicator.nombre)).await?
                }
                "MINUS_DM" => {
                    let params: MinusDmParams =
                        serde_json::from_value::<MinusDmParams>(indicator.parametros.clone())
                            .unwrap();
                    minus_dm(df, Some(params.timeperiod), Some(&indicator.nombre)).await?
                }
                "MOM" => {
                    let params: MomParams =
                        serde_json::from_value::<MomParams>(indicator.parametros.clone()).unwrap();
                    mom(df, Some(params.timeperiod), Some(&indicator.nombre)).await?
                }
                "PLUS_DI" => {
                    let params: PlusDiParams =
                        serde_json::from_value::<PlusDiParams>(indicator.parametros.clone())
                            .unwrap();
                    plus_di(df, Some(params.timeperiod), Some(&indicator.nombre)).await?
                }
                "PLUS_DM" => {
                    let params: PlusDmParams =
                        serde_json::from_value::<PlusDmParams>(indicator.parametros.clone())
                            .unwrap();
                    plus_dm(df, Some(params.timeperiod), Some(&indicator.nombre)).await?
                }
                "PPO" => {
                    let params: PpoParams =
                        serde_json::from_value::<PpoParams>(indicator.parametros.clone()).unwrap();
                    ppo(
                        df,
                        Some(params.fastperiod),
                        Some(params.slowperiod),
                        Some(&indicator.nombre),
                    )
                    .await?
                }
                "ROC" => {
                    let params: RocParams =
                        serde_json::from_value::<RocParams>(indicator.parametros.clone()).unwrap();
                    roc(df, Some(params.timeperiod), Some(&indicator.nombre)).await?
                }
                "ROCP" => {
                    let params: RocpParams =
                        serde_json::from_value::<RocpParams>(indicator.parametros.clone()).unwrap();
                    rocp(df, Some(params.timeperiod), Some(&indicator.nombre)).await?
                }
                "ROCR" => {
                    let params: RocrParams =
                        serde_json::from_value::<RocrParams>(indicator.parametros.clone()).unwrap();
                    rocr(df, Some(params.timeperiod), Some(&indicator.nombre)).await?
                }
                "ROCR100" => {
                    let params: Roc100Params =
                        serde_json::from_value::<Roc100Params>(indicator.parametros.clone())
                            .unwrap();
                    rocr100(df, Some(params.timeperiod), Some(&indicator.nombre)).await?
                }
                "RSI" => {
                    let params: RsiParams =
                        serde_json::from_value::<RsiParams>(indicator.parametros.clone()).unwrap();
                    rsi(df, Some(params.timeperiod), Some(&indicator.nombre)).await?
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
                    )
                    .await?
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
                    )
                    .await?
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
                    )
                    .await?
                }
                "TRIX" => {
                    let params: TrixParams =
                        serde_json::from_value::<TrixParams>(indicator.parametros.clone()).unwrap();
                    trix(df, Some(params.timeperiod), Some(&indicator.nombre)).await?
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
                    )
                    .await?
                }
                "WILLR" => {
                    let params: WillrParams =
                        serde_json::from_value::<WillrParams>(indicator.parametros.clone())
                            .unwrap();
                    willr(df, Some(params.timeperiod), Some(&indicator.nombre)).await?
                }
                "AVGPRICE" => avgprice(df, Some(&indicator.nombre)).await?,
                "MEDPRICE" => medprice(df, Some(&indicator.nombre)).await?,
                "TYPPRICE" => typprice(df, Some(&indicator.nombre)).await?,
                "WCLPRICE" => wclprice(df, Some(&indicator.nombre)).await?,
                "BETA" => {
                    let params: BetaParams =
                        serde_json::from_value::<BetaParams>(indicator.parametros.clone()).unwrap();
                    beta(
                        df,
                        &params.col_real0,
                        &params.col_real1,
                        Some(params.timeperiod),
                        Some(&indicator.nombre),
                    )
                    .await?
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
                    )
                    .await?
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
                    )
                    .await?
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
                    )
                    .await?
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
                    )
                    .await?
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
                    )
                    .await?
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
                    )
                    .await?
                }
                "TSF" => {
                    let params: TsfParams =
                        serde_json::from_value::<TsfParams>(indicator.parametros.clone()).unwrap();
                    tsf(
                        df,
                        &params.col_real,
                        Some(params.timeperiod),
                        Some(&indicator.nombre),
                    )
                    .await?
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
                    )
                    .await?
                }
                "TRANGE" => trange(df, Some(&indicator.nombre)).await?,
                "ATR" => {
                    let params: AtrParams =
                        serde_json::from_value::<AtrParams>(indicator.parametros.clone()).unwrap();
                    atr(df, Some(params.timeperiod), Some(&indicator.nombre)).await?
                }
                "NATR" => {
                    let params: NatrParams =
                        serde_json::from_value::<NatrParams>(indicator.parametros.clone()).unwrap();
                    natr(df, Some(params.timeperiod), Some(&indicator.nombre)).await?
                }
                "AD" => ad(df, Some(&indicator.nombre)).await?,
                "ADOSC" => {
                    let params: AdoscParams =
                        serde_json::from_value::<AdoscParams>(indicator.parametros.clone())
                            .unwrap();
                    adosc(
                        df,
                        Some(params.fastperiod),
                        Some(params.slowperiod),
                        Some(&indicator.nombre),
                    )
                    .await?
                }
                "OBV" => obv(df, Some(&indicator.nombre)).await?,
                _ => df,
            }
        }

        Ok(df)
    }

    async fn backtest(&mut self, df: DataFrame) -> Result<String, Box<dyn std::error::Error>> {
        // Foma de optener un dato: df.column(&columna)?.get(row_idx)?;
        for i in 0..df.height() {}
        Ok("Backtest ejecutado correctamente".to_string())
    }

    async fn backtest_long(&mut self, df: DataFrame) -> Result<String, Box<dyn std::error::Error>> {
        // Foma de optener un dato: df.column(&columna)?.get(row_idx)?;
        for i in 0..df.height() {}
        Ok("Backtest ejecutado correctamente".to_string())
    }

    async fn backtest_short(
        &mut self,
        df: DataFrame,
    ) -> Result<String, Box<dyn std::error::Error>> {
        // Foma de optener un dato: df.column(&columna)?.get(row_idx)?;
        for i in 0..df.height() {}
        Ok("Backtest ejecutado correctamente".to_string())
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
            // Verificamos los indicadores que tiene la estrategia para añadirlos a los datos del DataFrame
            let df = match self.set_indicators_strategy(data.get_datos()).await {
                Ok(df_result) => df_result,
                Err(e) => {
                    return Err(Box::new(e));
                }
            };

            //TODO: 2-Verificamos la direccion operativa que la estrategia nos permite operar. Long, Short o Both.
            match self.estrategia.opciones.trading_direccion {
                TradingDirection::Long => {
                    self.backtest_long(df.clone()).await.unwrap();
                }
                TradingDirection::Short => {
                    self.backtest_short(df.clone()).await.unwrap();
                }
                _ => {
                    self.backtest(df.clone()).await.unwrap();
                }
            }
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
