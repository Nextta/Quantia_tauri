use crate::backtest::backtest::Backtest;
use crate::structs::parametros::GestionParams;

use crate::backtest::datos::Datos;
use crate::backtest::resultados::Resultados;
use crate::backtest::symbol::SymbolInfoCFD;
use crate::backtest::trade::Trade;
use crate::enums::gestion::GestionStrategy;

use crate::enums::activos::Activo;
use chrono::DateTime;
use polars::prelude::*;
use std::time::Instant;

// Definimos la estructura de los datos del Dataframe
struct BacktestData {
    pub open: Vec<f64>,
    pub high: Vec<f64>,
    pub low: Vec<f64>,
    pub close: Vec<f64>,
    pub time: Vec<i64>,
}

fn prepare_backtest_slices(df: &DataFrame) -> PolarsResult<BacktestData> {
    // 1. IMPORTANTE: Rechunk para asegurar memoria contigua en todo el DF
    // Sin esto, podrías tener punteros rotos o errores al intentar obtener el slice.
    //*df = df.rechunk();

    // Función auxiliar interna para extraer slice de f32 de forma segura
    let get_slice = |name: &str, df: &DataFrame| -> PolarsResult<Vec<f64>> {
        let col = df.column(name)?;
        let chunked_array = col.f64()?;

        // Rechunkear para hacerlo un solo chunk
        let rechunked = chunked_array.rechunk();

        // Obtenemos el slice contiguo. Si hay nulls, Polars avisará.
        Ok(rechunked.cont_slice()?.to_vec())
    };

    // Función auxiliar interna para extraer slice de i64 de forma segura
    let get_slice_i64 = |name: &str, df: &DataFrame| -> PolarsResult<Vec<i64>> {
        let col = df.column(name)?;
        let chunked_array = col.i64()?;

        // Rechunkear para hacerlo un solo chunk
        let rechunked = chunked_array.rechunk();
        Ok(rechunked.cont_slice()?.to_vec())
    };

    // 3. Mapeo de columnas
    Ok(BacktestData {
        open: get_slice("open", df)?,
        high: get_slice("high", df)?,
        low: get_slice("low", df)?,
        close: get_slice("close", df)?,
        time: get_slice_i64("timestamp", df)?,
    })
}

// Definimos el estado del backtest
struct State {
    in_position: bool,
    entry_price: f64,
    sl: f64,
    tp: f64,
    // vertical_barrier: i32,
    trade_type: String,
}

pub async fn run_backtest(
    titulo: &str,
    balance: f64,
    tipo: Activo,
    datos_path: &str,
    symbol: SymbolInfoCFD,
) -> PolarsResult<()> {
    // Creamos el backtest
    let mut backtest: Backtest = Backtest::new(
        titulo.to_string(),
        balance,
        tipo,
        GestionStrategy::Formula,
        serde_json::from_str("{}").unwrap(),
    )
    .await; // Hacer un Enum para los tipos de activos

    // Añadimos los datos al backtest
    let df: Datos = backtest
        .add_datos(datos_path)
        .expect("Error al cargar datos");

    // Extraemos los slices (Los datos del Dataframe para recorrerlos)
    let data = prepare_backtest_slices(&df.get_datos())?;

    // Inicializamos el estado
    let mut state = State {
        in_position: false,
        entry_price: 0.0,
        sl: 0.0,
        tp: 0.0,
        // vertical_barrier: 0,
        trade_type: "None".to_string(),
    };

    // Creamos el struct de trade para realizar las operaciones
    let mut trade: Trade = Trade::new(backtest.id, symbol).await;

    println!("Iniciando Backtest...");
    let inicio = Instant::now();
    // El iterador zip es extremadamente eficiente (se vectoriza con SIMD)
    for i in 0..data.close.len() {
        let _ = data.open[i];
        let close = data.close[i];
        // let high = data.high[i];
        // let low = data.low[i];
        let time = data.time[i];

        // Formateamos Unix timestamp en **milisegundos**
        let naive_time = DateTime::from_timestamp_millis(time).expect("timestamp inválido");
        let time_str = naive_time.format("%Y-%m-%d %H:%M:%S").to_string();

        // Logica de la estrategia:
        if !state.in_position && i > 3 {
            if data.close[i - 1] > data.high[i - 2]
                && close < data.low[i - 1]
                && close > data.low[i - 2]
            {
                state.in_position = true;
                state.entry_price = close;
                state.sl = data.high[i - 1];
                state.tp = data.low[i - 2];
                state.trade_type = "Sell".to_string();

                trade.sell(
                    time_str.as_str(),
                    &close,
                    &GestionStrategy::Fijo,
                    &GestionParams {
                        lotaje_fijo: 0.01,
                        multiplicador: 1.0,
                    },
                    &backtest,
                    Some(state.tp),
                    Some(state.sl),
                );
                // println!("Operación en Short abierta");
            } else if data.close[i - 1] < data.low[i - 2]
                && close > data.high[i - 1]
                && close < data.high[i - 2]
            {
                state.in_position = true;
                state.entry_price = close;
                state.sl = data.low[i - 1];
                state.tp = data.high[i - 2];
                state.trade_type = "Buy".to_string();

                trade.buy(
                    time_str.as_str(),
                    &close,
                    &GestionStrategy::Fijo,
                    &GestionParams {
                        lotaje_fijo: 0.01,
                        multiplicador: 1.0,
                    },
                    &backtest,
                    Some(state.tp),
                    Some(state.sl),
                );
                // println!("Operación en Long abierta");
            }
        } else if state.in_position {
            // Control de SL/TP usando data.high[i] y data.low[i]
            if state.trade_type == "Sell" && data.low[i] <= state.tp {
                trade.close(time_str.clone(), state.tp);
                backtest.add_trade(trade.clone());

                state.in_position = false;
                state.trade_type = "None".to_string();
                // println!("Operación en Short Ganada");
            } else if state.trade_type == "Sell" && data.high[i] >= state.sl {
                trade.close(time_str.clone(), state.sl);
                backtest.add_trade(trade.clone());

                state.in_position = false;
                state.trade_type = "None".to_string();
                // println!("Operación en Short perdida");
            } else if state.trade_type == "Buy" && data.high[i] >= state.tp {
                trade.close(time_str.clone(), state.tp);
                backtest.add_trade(trade.clone());

                state.in_position = false;
                state.trade_type = "None".to_string();
                // println!("Operación en Long Ganada");
            } else if state.trade_type == "Buy" && data.low[i] <= state.sl {
                trade.close(time_str.clone(), state.sl);
                backtest.add_trade(trade.clone());

                state.in_position = false;
                state.trade_type = "None".to_string();
                // println!("Operación en Long Perdida");
            }
        }
    }
    let duracion = inicio.elapsed();
    println!("Backtest finalizado en {:?}", duracion);

    backtest.guardar_trades().await;

    // Creamos los resultados
    let mut resultados: Resultados = Resultados::new(backtest.id).await;
    resultados.calcular_resultados(backtest.trades.clone(), backtest.balance);
    resultados.guardar_resultados().await;

    Ok(())
}
