use polars::prelude::*;
use serde::{Deserialize, Serialize};

pub const PATTERN_BULLISH: i32 = 100;
pub const PATTERN_BEARISH: i32 = -100;
pub const PATTERN_NONE: i32 = 0;

/// Obtiene las series de Open, High, Low, Close de un DataFrame.
fn get_ohlc(df: &DataFrame) -> PolarsResult<(Series, Series, Series, Series)> {
    let open = df.column("open")?.as_materialized_series().clone();
    let high = df.column("high")?.as_materialized_series().clone();
    let low = df.column("low")?.as_materialized_series().clone();
    let close = df.column("close")?.as_materialized_series().clone();
    Ok((open, high, low, close))
}

/// Determina si una vela es blanca (alcista) o negra (bajista).
fn candle_color(open: f64, close: f64) -> i32 {
    if close > open {
        1
    } else if close < open {
        -1
    } else {
        0
    }
}

/// CDL2CROWS - Two Crows
///
/// El patrón Two Crows es un patrón de reversión bajista que ocurre en una tendencia alcista.
/// Se compone de tres velas:
/// 1. Una vela blanca larga.
/// 2. Una vela negra pequeña que abre con un gap alcista.
/// 3. Una vela negra que abre dentro del cuerpo de la segunda vela y cierra dentro del cuerpo de la primera.
///
/// # Parámetros
/// * `df` - DataFrame con columnas open, high, low, close.
/// * `output_col` - Nombre opcional para la columna de salida (defecto: "cdl2crows").
///
/// # Retorno
/// PolarsResult<DataFrame> con la columna del indicador añadida.
pub fn cdl2crows(df: &mut DataFrame, output_col: Option<&str>) {
    let output_name = output_col.unwrap_or("cdl2crows");
    let (open_s, _high_s, _low_s, close_s) = get_ohlc(&df).unwrap();
    let open = open_s.f64().unwrap();
    let close = close_s.f64().unwrap();

    let len = open.len();
    let mut result: Vec<i32> = vec![0; len];

    if len < 3 {
        df.with_column(Series::new(output_name.into(), &result).into());
    }

    for i in 2..len {
        let o0 = open.get(i - 2).unwrap();
        let c0 = close.get(i - 2).unwrap();
        let o1 = open.get(i - 1).unwrap();
        let c1 = close.get(i - 1).unwrap();
        let o2 = open.get(i).unwrap();
        let c2 = close.get(i).unwrap();

        // 1. Vela blanca larga (simplificado como vela blanca)
        let is_white0 = candle_color(o0, c0) == 1;
        // 2. Vela negra que abre con gap alcista
        let is_black1 = candle_color(o1, c1) == -1;
        let gaps_up1 = o1 > c0;
        // 3. Vela negra que abre dentro del cuerpo de la 2da y cierra dentro del de la 1ra
        let is_black2 = candle_color(o2, c2) == -1;
        let opens_inside_day2 = o2 < o1 && o2 > c1;
        let closes_inside_day1 = c2 > o0 && c2 < c0;

        if is_white0
            && is_black1
            && gaps_up1
            && is_black2
            && opens_inside_day2
            && closes_inside_day1
        {
            result[i] = PATTERN_BEARISH;
        }
    }

    df.with_column(Series::new(output_name.into(), result).into());
}

/// CDL3BLACKCROWS - Three Black Crows
///
/// El patrón Three Black Crows (Tres Cuervos Negros) es un patrón de reversión bajista
/// que consiste en tres velas negras consecutivas con cierres progresivamente más bajos.
///
/// # Parámetros
/// * `df` - DataFrame con columnas open, high, low, close.
/// * `output_col` - Nombre opcional para la columna de salida (defecto: "cdl3blackcrows").
///
/// # Retorno
/// PolarsResult<DataFrame> con la columna del indicador añadida.
pub fn cdl3blackcrows(df: &mut DataFrame, output_col: Option<&str>) {
    let output_name = output_col.unwrap_or("cdl3blackcrows");
    let (open_s, _high_s, _low_s, close_s) = get_ohlc(&df).unwrap();
    let open = open_s.f64().unwrap();
    let close = close_s.f64().unwrap();

    let len = open.len();
    let mut result: Vec<i32> = vec![0; len];

    if len < 3 {
        df.with_column(Series::new(output_name.into(), &result).into());
    }

    for i in 2..len {
        let o0 = open.get(i - 2).unwrap();
        let c0 = close.get(i - 2).unwrap();
        let o1 = open.get(i - 1).unwrap();
        let c1 = close.get(i - 1).unwrap();
        let o2 = open.get(i).unwrap();
        let c2 = close.get(i).unwrap();

        // Las tres velas deben ser negras
        let all_black =
            candle_color(o0, c0) == -1 && candle_color(o1, c1) == -1 && candle_color(o2, c2) == -1;

        // Cada vela abre dentro del cuerpo de la anterior
        let open_within_prev1 = o1 < o0 && o1 > c0;
        let open_within_prev2 = o2 < o1 && o2 > c1;

        // Cada vela cierra por debajo de la anterior
        let lower_closes = c1 < c0 && c2 < c1;

        if all_black && open_within_prev1 && open_within_prev2 && lower_closes {
            result[i] = PATTERN_BEARISH;
        }
    }

    df.with_column(Series::new(output_name.into(), result).into());
}

/// CDL3INSIDE - Three Inside Up/Down
///
/// El patrón Three Inside es un patrón de reversión de tres velas.
/// - Up (Alcista): Un Harami alcista confirmado por una tercera vela blanca que cierra más alto.
/// - Down (Bajista): Un Harami bajista confirmado por una tercera vela negra que cierra más bajo.
///
/// # Parámetros
/// * `df` - DataFrame con columnas open, high, low, close.
/// * `output_col` - Nombre opcional para la columna de salida (defecto: "cdl3inside").
///
/// # Retorno
/// PolarsResult<DataFrame> con la columna del indicador añadida.
pub fn cdl3inside(df: &mut DataFrame, output_col: Option<&str>) {
    let output_name = output_col.unwrap_or("cdl3inside");
    let (open_s, _high_s, _low_s, close_s) = get_ohlc(&df).unwrap();
    let open = open_s.f64().unwrap();
    let close = close_s.f64().unwrap();

    let len = open.len();
    let mut result: Vec<i32> = vec![0; len];

    if len < 3 {
        df.with_column(Series::new(output_name.into(), &result).into());
    }

    for i in 2..len {
        let o0 = open.get(i - 2).unwrap();
        let c0 = close.get(i - 2).unwrap();
        let o1 = open.get(i - 1).unwrap();
        let c1 = close.get(i - 1).unwrap();
        let o2 = open.get(i).unwrap();
        let c2 = close.get(i).unwrap();

        // Harami Bullish (Día 1 y 2)
        let is_harami_bullish =
            candle_color(o0, c0) == -1 && candle_color(o1, c1) == 1 && o1 > c0 && c1 < o0;

        // Confirmation Bullish (Día 3)
        if is_harami_bullish && candle_color(o2, c2) == 1 && c2 > c1 {
            result[i] = PATTERN_BULLISH;
            continue;
        }

        // Harami Bearish (Día 1 y 2)
        let is_harami_bearish =
            candle_color(o0, c0) == 1 && candle_color(o1, c1) == -1 && o1 < c0 && c1 > o0;

        // Confirmation Bearish (Día 3)
        if is_harami_bearish && candle_color(o2, c2) == -1 && c2 < c1 {
            result[i] = PATTERN_BEARISH;
        }
    }

    df.with_column(Series::new(output_name.into(), result).into());
}

/// CDL3LINESTRIKE - Three-Line Strike
///
/// El patrón Three-Line Strike consta de cuatro velas:
/// - Bullish (Alcista): Tres velas negras que descienden, seguidas de una vela blanca larga
///   que envuelve las tres anteriores.
/// - Bearish (Bajista): Tres velas blancas que ascienden, seguidas de una vela negra larga
///   que envuelve las tres anteriores.
///
/// # Parámetros
/// * `df` - DataFrame con columnas open, high, low, close.
/// * `output_col` - Nombre opcional para la columna de salida (defecto: "cdl3linestrike").
///
/// # Retorno
/// PolarsResult<DataFrame> con la columna del indicador añadida.
pub fn cdl3linestrike(df: &mut DataFrame, output_col: Option<&str>) {
    let output_name = output_col.unwrap_or("cdl3linestrike");
    let (open_s, _high_s, _low_s, close_s) = get_ohlc(&df).unwrap();
    let open = open_s.f64().unwrap();
    let close = close_s.f64().unwrap();

    let len = open.len();
    let mut result: Vec<i32> = vec![0; len];

    if len < 4 {
        df.with_column(Series::new(output_name.into(), &result).into());
    }

    for i in 3..len {
        let o0 = open.get(i - 3).unwrap();
        let c0 = close.get(i - 3).unwrap();
        let o1 = open.get(i - 2).unwrap();
        let c1 = close.get(i - 2).unwrap();
        let o2 = open.get(i - 1).unwrap();
        let c2 = close.get(i - 1).unwrap();
        let o3 = open.get(i).unwrap();
        let c3 = close.get(i).unwrap();

        // Bullish Three-Line Strike
        let three_black =
            candle_color(o0, c0) == -1 && candle_color(o1, c1) == -1 && candle_color(o2, c2) == -1;
        let descending = c1 < c0 && c2 < c1;
        let strike_up = candle_color(o3, c3) == 1 && o3 <= c2 && c3 > o0;

        if three_black && descending && strike_up {
            result[i] = PATTERN_BULLISH;
            continue;
        }

        // Bearish Three-Line Strike
        let three_white =
            candle_color(o0, c0) == 1 && candle_color(o1, c1) == 1 && candle_color(o2, c2) == 1;
        let ascending = c1 > c0 && c2 > c1;
        let strike_down = candle_color(o3, c3) == -1 && o3 >= c2 && c3 < o0;

        if three_white && ascending && strike_down {
            result[i] = PATTERN_BEARISH;
        }
    }

    df.with_column(Series::new(output_name.into(), result).into());
}

/// CDL3OUTSIDE - Three Outside Up/Down
///
/// El patrón Three Outside es un patrón de reversión de tres velas:
/// - Up (Alcista): Un Engulfing alcista confirmado por una tercera vela blanca que cierra más alto.
/// - Down (Bajista): Un Engulfing bajista confirmado por una tercera vela negra que cierra más bajo.
///
/// # Parámetros
/// * `df` - DataFrame con columnas open, high, low, close.
/// * `output_col` - Nombre opcional para la columna de salida (defecto: "cdl3outside").
///
/// # Retorno
/// PolarsResult<DataFrame> con la columna del indicador añadida.
pub fn cdl3outside(df: &mut DataFrame, output_col: Option<&str>) {
    let output_name = output_col.unwrap_or("cdl3outside");
    let (open_s, _high_s, _low_s, close_s) = get_ohlc(&df).unwrap();
    let open = open_s.f64().unwrap();
    let close = close_s.f64().unwrap();

    let len = open.len();
    let mut result: Vec<i32> = vec![0; len];

    if len < 3 {
        df.with_column(Series::new(output_name.into(), &result).into());
    }

    for i in 2..len {
        let o0 = open.get(i - 2).unwrap();
        let c0 = close.get(i - 2).unwrap();
        let o1 = open.get(i - 1).unwrap();
        let c1 = close.get(i - 1).unwrap();
        let o2 = open.get(i).unwrap();
        let c2 = close.get(i).unwrap();

        // Bullish Engulfing (Día 1 y 2)
        let is_engulfing_bullish =
            candle_color(o0, c0) == -1 && candle_color(o1, c1) == 1 && o1 < c0 && c1 > o0;

        // Confirmation Bullish (Día 3)
        if is_engulfing_bullish && candle_color(o2, c2) == 1 && c2 > c1 {
            result[i] = PATTERN_BULLISH;
            continue;
        }

        // Bearish Engulfing (Día 1 y 2)
        let is_engulfing_bearish =
            candle_color(o0, c0) == 1 && candle_color(o1, c1) == -1 && o1 > c0 && c1 < o0;

        // Confirmation Bearish (Día 3)
        if is_engulfing_bearish && candle_color(o2, c2) == -1 && c2 < c1 {
            result[i] = PATTERN_BEARISH;
        }
    }

    df.with_column(Series::new(output_name.into(), result).into());
}

/// CDL3STARSINSOUTH - Three Stars In The South
///
/// El patrón Three Stars In The South es un patrón de reversión alcista que ocurre
/// en una tendencia bajista. Indica que la fuerza bajista se está agotando.
///
/// # Parámetros
/// * `df` - DataFrame con columnas open, high, low, close.
/// * `output_col` - Nombre opcional para la columna de salida (defecto: "cdl3starsinsouth").
///
/// # Retorno
/// PolarsResult<DataFrame> con la columna del indicador añadida.
pub fn cdl3starsinsouth(df: &mut DataFrame, output_col: Option<&str>) {
    let output_name = output_col.unwrap_or("cdl3starsinsouth");
    let (open_s, high_s, low_s, close_s) = get_ohlc(&df).unwrap();
    let open = open_s.f64().unwrap();
    let high = high_s.f64().unwrap();
    let low = low_s.f64().unwrap();
    let close = close_s.f64().unwrap();

    let len = open.len();
    let mut result: Vec<i32> = vec![0; len];

    if len < 3 {
        df.with_column(Series::new(output_name.into(), &result).into());
    }

    for i in 2..len {
        let o0 = open.get(i - 2).unwrap();
        let h0 = high.get(i - 2).unwrap();
        let l0 = low.get(i - 2).unwrap();
        let c0 = close.get(i - 2).unwrap();

        let o1 = open.get(i - 1).unwrap();
        let _ = high.get(i - 1).unwrap();
        let l1 = low.get(i - 1).unwrap();
        let c1 = close.get(i - 1).unwrap();

        let o2 = open.get(i).unwrap();
        let h2 = high.get(i).unwrap();
        let l2 = low.get(i).unwrap();
        let c2 = close.get(i).unwrap();

        // Día 1: Vela negra larga con sombra inferior larga
        let is_black0 = candle_color(o0, c0) == -1;
        let body0 = o0 - c0;
        let lower_shadow0 = c0 - l0;
        let is_long_body0 = body0 > (h0 - l0) * 0.5;
        let is_long_shadow0 = lower_shadow0 > body0 * 0.5;

        // Día 2: Vela negra, cuerpo más pequeño que día 1, mínimo mayor que día 1
        let is_black1 = candle_color(o1, c1) == -1;
        let body1 = o1 - c1;
        let is_smaller_body1 = body1 < body0;
        let is_higher_low1 = l1 > l0;

        // Día 3: Vela negra pequeña (marubozu), abre dentro de cuerpo día 2, rango dentro de día 2
        let is_black2 = candle_color(o2, c2) == -1;
        let body2 = o2 - c2;
        let is_small_body2 = body2 < body1 * 0.5;
        let no_shadows2 = (h2 - o2).abs() < body2 * 0.1 && (c2 - l2).abs() < body2 * 0.1;
        let within_prev2 = o2 < o1 && o2 > c1 && h2 <= o1 && l2 >= c1;

        if is_black0
            && is_long_body0
            && is_long_shadow0
            && is_black1
            && is_smaller_body1
            && is_higher_low1
            && is_black2
            && is_small_body2
            && no_shadows2
            && within_prev2
        {
            result[i] = PATTERN_BULLISH;
        }
    }

    df.with_column(Series::new(output_name.into(), result).into());
}

/// CDL3WHITESOLDIERS - Three Advancing White Soldiers
///
/// El patrón Three Advancing White Soldiers (Tres Soldados Blancos) es un patrón
/// de reversión alcista compuesto por tres velas blancas consecutivas con cierres
/// progresivamente más altos.
///
/// # Parámetros
/// * `df` - DataFrame con columnas open, high, low, close.
/// * `output_col` - Nombre opcional para la columna de salida (defecto: "cdl3whitesoldiers").
///
/// # Retorno
/// PolarsResult<DataFrame> con la columna del indicador añadida.
pub fn cdl3whitesoldiers(df: &mut DataFrame, output_col: Option<&str>) {
    let output_name = output_col.unwrap_or("cdl3whitesoldiers");
    let (open_s, high_s, _low_s, close_s) = get_ohlc(&df).unwrap();
    let open = open_s.f64().unwrap();
    let high = high_s.f64().unwrap();
    let close = close_s.f64().unwrap();

    let len = open.len();
    let mut result: Vec<i32> = vec![0; len];

    if len < 3 {
        df.with_column(Series::new(output_name.into(), &result).into());
    }

    for i in 2..len {
        let o0 = open.get(i - 2).unwrap();
        let c0 = close.get(i - 2).unwrap();
        let o1 = open.get(i - 1).unwrap();
        let h1 = high.get(i - 1).unwrap();
        let c1 = close.get(i - 1).unwrap();
        let o2 = open.get(i).unwrap();
        let h2 = high.get(i).unwrap();
        let c2 = close.get(i).unwrap();

        // Las tres velas deben ser blancas
        let all_white =
            candle_color(o0, c0) == 1 && candle_color(o1, c1) == 1 && candle_color(o2, c2) == 1;

        // Cierres progresivamente más altos
        let higher_closes = c1 > c0 && c2 > c1;

        // Aperturas dentro del cuerpo anterior
        let open_within_prev1 = o1 > o0 && o1 < c0;
        let open_within_prev2 = o2 > o1 && o2 < c1;

        // Sombras superiores pequeñas (cierre cerca del máximo)
        let body1 = c1 - o1;
        let body2 = c2 - o2;
        let short_shadow1 = (h1 - c1) < body1 * 0.2;
        let short_shadow2 = (h2 - c2) < body2 * 0.2;

        if all_white
            && higher_closes
            && open_within_prev1
            && open_within_prev2
            && short_shadow1
            && short_shadow2
        {
            result[i] = PATTERN_BULLISH;
        }
    }

    df.with_column(Series::new(output_name.into(), result).into());
}

/// CDLABANDONEDBABY - Abandoned Baby
///
/// El patrón Abandoned Baby (Bebé Abandonado) es un patrón de reversión muy raro y potente
/// compuesto por tres velas, donde la vela central (un Doji) tiene gaps significativos
/// que la separan completamente de las velas de los lados (incluyendo las sombras).
///
/// # Parámetros
/// * `df` - DataFrame con columnas open, high, low, close.
/// * `output_col` - Nombre opcional para la columna de salida (defecto: "cdlabandonedbaby").
///
/// # Retorno
/// PolarsResult<DataFrame> con la columna del indicador añadida.
pub fn cdlabandonedbaby(df: &mut DataFrame, output_col: Option<&str>) {
    let output_name = output_col.unwrap_or("cdlabandonedbaby");
    let (open_s, high_s, low_s, close_s) = get_ohlc(&df).unwrap();
    let open = open_s.f64().unwrap();
    let high = high_s.f64().unwrap();
    let low = low_s.f64().unwrap();
    let close = close_s.f64().unwrap();

    let len = open.len();
    let mut result: Vec<i32> = vec![0; len];

    if len < 3 {
        df.with_column(Series::new(output_name.into(), &result).into());
    }

    for i in 2..len {
        let o0 = open.get(i - 2).unwrap();
        let h0 = high.get(i - 2).unwrap();
        let l0 = low.get(i - 2).unwrap();
        let c0 = close.get(i - 2).unwrap();

        let o1 = open.get(i - 1).unwrap();
        let h1 = high.get(i - 1).unwrap();
        let l1 = low.get(i - 1).unwrap();
        let c1 = close.get(i - 1).unwrap();

        let o2 = open.get(i).unwrap();
        let h2 = high.get(i).unwrap();
        let l2 = low.get(i).unwrap();
        let c2 = close.get(i).unwrap();

        // Identificar Doji (Cuerpo muy pequeño respecto al rango)
        let body1 = (o1 - c1).abs();
        let range1 = h1 - l1;
        let is_doji1 = if range1 > 0.0 {
            body1 / range1 < 0.1
        } else {
            true
        };

        // Bullish Abandoned Baby
        let is_bullish = candle_color(o0, c0) == -1  // Día 1 Negro
                      && is_doji1                   // Día 2 Doji
                      && candle_color(o2, c2) == 1   // Día 3 Blanco
                      && l0 > h1                     // Gap hacia abajo (incluye sombras)
                      && l2 > h1; // Gap hacia arriba (incluye sombras)

        if is_bullish {
            result[i] = PATTERN_BULLISH;
            continue;
        }

        // Bearish Abandoned Baby
        let is_bearish = candle_color(o0, c0) == 1    // Día 1 Blanco
                      && is_doji1                    // Día 2 Doji
                      && candle_color(o2, c2) == -1  // Día 3 Negro
                      && h0 < l1                     // Gap hacia arriba (incluye sombras)
                      && h2 < l1; // Gap hacia abajo (incluye sombras)

        if is_bearish {
            result[i] = PATTERN_BEARISH;
        }
    }

    df.with_column(Series::new(output_name.into(), result).into());
}

/// CDLADVANCEBLOCK - Advance Block
///
/// El patrón Advance Block es un patrón de reversión bajista que ocurre en una tendencia alcista.
/// Indica una pérdida de impulso mediante tres velas blancas con cuerpos decrecientes
/// y sombras superiores más largas.
///
/// # Parámetros
/// * `df` - DataFrame con columnas open, high, low, close.
/// * `output_col` - Nombre opcional para la columna de salida (defecto: "cdladvanceblock").
///
/// # Retorno
/// PolarsResult<DataFrame> con la columna del indicador añadida.
pub fn cdladvanceblock(df: &mut DataFrame, output_col: Option<&str>) {
    let output_name = output_col.unwrap_or("cdladvanceblock");
    let (open_s, high_s, _low_s, close_s) = get_ohlc(&df).unwrap();
    let open = open_s.f64().unwrap();
    let high = high_s.f64().unwrap();
    let close = close_s.f64().unwrap();

    let len = open.len();
    let mut result: Vec<i32> = vec![0; len];

    if len < 3 {
        df.with_column(Series::new(output_name.into(), &result).into());
    }

    for i in 2..len {
        let o0 = open.get(i - 2).unwrap();
        let c0 = close.get(i - 2).unwrap();
        let o1 = open.get(i - 1).unwrap();
        let h1 = high.get(i - 1).unwrap();
        let c1 = close.get(i - 1).unwrap();
        let o2 = open.get(i).unwrap();
        let h2 = high.get(i).unwrap();
        let c2 = close.get(i).unwrap();

        // Las tres velas deben ser blancas
        let all_white =
            candle_color(o0, c0) == 1 && candle_color(o1, c1) == 1 && candle_color(o2, c2) == 1;

        // Cierres ascendentes
        let higher_closes = c1 > c0 && c2 > c1;

        // Aperturas dentro del cuerpo anterior
        let open_within_prev1 = o1 > o0 && o1 < c0;
        let open_within_prev2 = o2 > o1 && o2 < c1;

        // Cuerpos decrecientes
        let body0 = c0 - o0;
        let body1 = c1 - o1;
        let body2 = c2 - o2;
        let shrinking_bodies = body1 < body0 && body2 < body1;

        // Sombras superiores largas en las últimas dos velas
        let shadow1 = h1 - c1;
        let shadow2 = h2 - c2;
        let long_upper_shadows = shadow1 > body1 * 0.5 && shadow2 > body2 * 0.5;

        if all_white
            && higher_closes
            && open_within_prev1
            && open_within_prev2
            && shrinking_bodies
            && long_upper_shadows
        {
            result[i] = PATTERN_BEARISH;
        }
    }

    df.with_column(Series::new(output_name.into(), result).into());
}

/// CDLBELTHOLD - Belt-hold
///
/// El patrón Belt-hold es un patrón de una sola vela:
/// - Bullish (Alcista): Una vela blanca larga que abre en su mínimo (sin sombra inferior).
/// - Bearish (Bajista): Una vela negra larga que abre en su máximo (sin sombra superior).
///
/// # Parámetros
/// * `df` - DataFrame con columnas open, high, low, close.
/// * `output_col` - Nombre opcional para la columna de salida (defecto: "cdlbelthold").
///
/// # Retorno
/// PolarsResult<DataFrame> con la columna del indicador añadida.
pub fn cdlbelthold(df: &mut DataFrame, output_col: Option<&str>) {
    let output_name = output_col.unwrap_or("cdlbelthold");
    let (open_s, high_s, low_s, close_s) = get_ohlc(&df).unwrap();
    let open = open_s.f64().unwrap();
    let high = high_s.f64().unwrap();
    let low = low_s.f64().unwrap();
    let close = close_s.f64().unwrap();

    let len = open.len();
    let mut result: Vec<i32> = vec![0; len];

    for i in 0..len {
        let o = open.get(i).unwrap();
        let h = high.get(i).unwrap();
        let l = low.get(i).unwrap();
        let c = close.get(i).unwrap();

        let body = (c - o).abs();
        let range = h - l;

        if range == 0.0 {
            continue;
        }

        // Definición de "cuerpo largo": más del 70% del rango total
        let is_long_body = body > range * 0.7;

        // Bullish Belt-hold: Abre en el mínimo y cierra al alza
        let is_bullish = candle_color(o, c) == 1
                      && (o - l).abs() < range * 0.05 // Sombra inferior mínima
                      && is_long_body;

        if is_bullish {
            result[i] = PATTERN_BULLISH;
            continue;
        }

        // Bearish Belt-hold: Abre en el máximo y cierra a la baja
        let is_bearish = candle_color(o, c) == -1
                       && (h - o).abs() < range * 0.05 // Sombra superior mínima
                       && is_long_body;

        if is_bearish {
            result[i] = PATTERN_BEARISH;
        }
    }

    df.with_column(Series::new(output_name.into(), result).into());
}

/// CDLBREAKAWAY - Breakaway
///
/// El patrón Breakaway es un patrón de reversión de cinco velas:
/// - Bullish (Alcista): Ocurre en tendencia bajista, con un gap inicial seguido de tres velas
///   pequeñas y una vela blanca larga que cierra el gap.
/// - Bearish (Bajista): Ocurre en tendencia alcista, con un gap inicial seguido de tres velas
///   pequeñas y una vela negra larga que cierra el gap.
///
/// # Parámetros
/// * `df` - DataFrame con columnas open, high, low, close.
/// * `output_col` - Nombre opcional para la columna de salida (defecto: "cdlbreakaway").
///
/// # Retorno
/// PolarsResult<DataFrame> con la columna del indicador añadida.
pub fn cdlbreakaway(df: &mut DataFrame, output_col: Option<&str>) {
    let output_name = output_col.unwrap_or("cdlbreakaway");
    let (open_s, _high_s, _low_s, close_s) = get_ohlc(&df).unwrap();
    let open = open_s.f64().unwrap();
    let close = close_s.f64().unwrap();

    let len = open.len();
    let mut result: Vec<i32> = vec![0; len];

    if len < 5 {
        df.with_column(Series::new(output_name.into(), &result).into());
    }

    for i in 4..len {
        let o0 = open.get(i - 4).unwrap();
        let c0 = close.get(i - 4).unwrap();
        let o1 = open.get(i - 3).unwrap();
        let c1 = close.get(i - 3).unwrap();
        let c2 = close.get(i - 2).unwrap();
        let c3 = close.get(i - 1).unwrap();
        let o4 = open.get(i).unwrap();
        let c4 = close.get(i).unwrap();

        // Bullish Breakaway
        let is_bullish = candle_color(o0, c0) == -1    // Día 1: Negro largo
                      && candle_color(o1, c1) == -1    // Día 2: Negro
                      && o1 < c0                       // Gap hacia abajo
                      && c2 < c1 && c3 < c2            // Días 3 y 4: Bajistas
                      && candle_color(o4, c4) == 1     // Día 5: Blanco largo
                      && c4 > c1 && c4 < c0; // Cierra dentro del gap

        if is_bullish {
            result[i] = PATTERN_BULLISH;
            continue;
        }

        // Bearish Breakaway
        let is_bearish = candle_color(o0, c0) == 1     // Día 1: Blanco largo
                       && candle_color(o1, c1) == 1     // Día 2: Blanco
                       && o1 > c0                       // Gap hacia arriba
                       && c2 > c1 && c3 > c2            // Días 3 y 4: Alcistas
                       && candle_color(o4, c4) == -1    // Día 5: Negro largo
                       && c4 < c1 && c4 > c0; // Cierra dentro del gap

        if is_bearish {
            result[i] = PATTERN_BEARISH;
        }
    }

    df.with_column(Series::new(output_name.into(), result).into());
}

/// CDLCLOSINGMARUBOZU - Closing Marubozu
///
/// El Closing Marubozu es una vela con un cuerpo largo y sin sombra en el extremo del cierre.
/// - Bullish (Alcista): Vela blanca donde el cierre es igual al máximo.
/// - Bearish (Bajista): Vela negra donde el cierre es igual al mínimo.
///
/// # Parámetros
/// * `df` - DataFrame con columnas open, high, low, close.
/// * `output_col` - Nombre opcional para la columna de salida (defecto: "cdlclosingmarubozu").
///
/// # Retorno
/// PolarsResult<DataFrame> con la columna del indicador añadida.
pub fn cdlclosingmarubuzo(df: &mut DataFrame, output_col: Option<&str>) {
    let output_name = output_col.unwrap_or("cdlclosingmarubozu");
    let (open_s, high_s, low_s, close_s) = get_ohlc(&df).unwrap();
    let open = open_s.f64().unwrap();
    let high = high_s.f64().unwrap();
    let low = low_s.f64().unwrap();
    let close = close_s.f64().unwrap();

    let len = open.len();
    let mut result: Vec<i32> = vec![0; len];

    for i in 0..len {
        let o = open.get(i).unwrap();
        let h = high.get(i).unwrap();
        let l = low.get(i).unwrap();
        let c = close.get(i).unwrap();

        let body = (c - o).abs();
        let range = h - l;

        if range == 0.0 {
            continue;
        }

        // Definición de cuerpo largo (ej: > 70% del rango total)
        let is_long_body = body > range * 0.7;

        // Bullish: Blanco y Cierre = Máximo
        if candle_color(o, c) == 1 && (c - h).abs() < f64::EPSILON && is_long_body {
            result[i] = PATTERN_BULLISH;
            continue;
        }

        // Bearish: Negro y Cierre = Mínimo
        if candle_color(o, c) == -1 && (c - l).abs() < f64::EPSILON && is_long_body {
            result[i] = PATTERN_BEARISH;
        }
    }

    df.with_column(Series::new(output_name.into(), result).into());
}

/// CDLCONCEALBABYSWALL - Concealing Baby Swallow
///
/// El Concealing Baby Swallow es un patrón de reversión alcista de cuatro velas:
/// 1. Dos Marubozus negros seguidos.
/// 2. Una tercera vela negra con gap bajista y sombra superior larga.
/// 3. Una cuarta vela negra que envuelve completamente a la tercera.
///
/// # Parámetros
/// * `df` - DataFrame con columnas open, high, low, close.
/// * `output_col` - Nombre opcional para la columna de salida (defecto: "cdlconcealbabyswall").
///
/// # Retorno
/// PolarsResult<DataFrame> con la columna del indicador añadida.
pub fn cdlconcealbabyswall(df: &mut DataFrame, output_col: Option<&str>) {
    let output_name = output_col.unwrap_or("cdlconcealbabyswall");
    let (open_s, high_s, low_s, close_s) = get_ohlc(&df).unwrap();
    let open = open_s.f64().unwrap();
    let high = high_s.f64().unwrap();
    let low = low_s.f64().unwrap();
    let close = close_s.f64().unwrap();

    let len = open.len();
    let mut result: Vec<i32> = vec![0; len];

    if len < 4 {
        df.with_column(Series::new(output_name.into(), &result).into());
    }

    for i in 3..len {
        let o0 = open.get(i - 3).unwrap();
        let c0 = close.get(i - 3).unwrap();
        let o1 = open.get(i - 2).unwrap();
        let c1 = close.get(i - 2).unwrap();
        let o2 = open.get(i - 1).unwrap();
        let h2 = high.get(i - 1).unwrap();
        let c2 = close.get(i - 1).unwrap();
        let o3 = open.get(i).unwrap();
        let h3 = high.get(i).unwrap();
        let l3 = low.get(i).unwrap();
        let c3 = close.get(i).unwrap();

        // Día 1 y 2: Marubozus negros
        let marubozu0 = candle_color(o0, c0) == -1
            && (o0 - c0) > (high.get(i - 3).unwrap() - low.get(i - 3).unwrap()) * 0.8;
        let marubozu1 = candle_color(o1, c1) == -1
            && (o1 - c1) > (high.get(i - 2).unwrap() - low.get(i - 2).unwrap()) * 0.8;

        // Día 3: Vela negra, gap abajo, sombra superior larga
        let day3 = candle_color(o2, c2) == -1 && o2 < c1 && h2 > o2;

        // Día 4: Vela negra que envuelve al día 3
        let day4 = candle_color(o3, c3) == -1 && o3 > h2 && c3 < l3 + (h3 - l3) * 0.1;

        if marubozu0 && marubozu1 && day3 && day4 {
            result[i] = PATTERN_BULLISH;
        }
    }

    df.with_column(Series::new(output_name.into(), result).into());
}

/// CDLCOUNTERATTACK - Counterattack
///
/// El Counterattack es un patrón de reversión de dos velas con colores opuestos
/// y cierres iguales o muy cercanos.
/// - Bullish (Alcista): Vela negra seguida de una blanca que cierra al mismo nivel.
/// - Bearish (Bajista): Vela blanca seguida de una negra que cierra al mismo nivel.
///
/// # Parámetros
/// * `df` - DataFrame con columnas open, high, low, close.
/// * `output_col` - Nombre opcional para la columna de salida (defecto: "cdlcounterattack").
///
/// # Retorno
/// PolarsResult<DataFrame> con la columna del indicador añadida.
pub fn cdlcounterattack(df: &mut DataFrame, output_col: Option<&str>) {
    let output_name = output_col.unwrap_or("cdlcounterattack");
    let (open_s, _high_s, _low_s, close_s) = get_ohlc(&df).unwrap();
    let open = open_s.f64().unwrap();
    let close = close_s.f64().unwrap();

    let len = open.len();
    let mut result: Vec<i32> = vec![0; len];

    if len < 2 {
        df.with_column(Series::new(output_name.into(), &result).into());
    }

    for i in 1..len {
        let o0 = open.get(i - 1).unwrap();
        let c0 = close.get(i - 1).unwrap();
        let o1 = open.get(i).unwrap();
        let c1 = close.get(i).unwrap();

        // Diferencia aceptable para "cierres iguales" (0.1% del precio)
        let diff_limit = c0 * 0.001;

        // Bullish Counterattack
        if candle_color(o0, c0) == -1
            && candle_color(o1, c1) == 1
            && (c1 - c0).abs() <= diff_limit
            && o1 < c0
        {
            result[i] = PATTERN_BULLISH;
            continue;
        }

        // Bearish Counterattack
        if candle_color(o0, c0) == 1
            && candle_color(o1, c1) == -1
            && (c1 - c0).abs() <= diff_limit
            && o1 > c0
        {
            result[i] = PATTERN_BEARISH;
        }
    }

    df.with_column(Series::new(output_name.into(), result).into());
}

/// CDLDARKCLOUDCOVER - Dark Cloud Cover
///
/// El Dark Cloud Cover (Nube Oscura) es un patrón de reversión bajista de dos velas.
/// - Día 1: Vela blanca larga.
/// - Día 2: Vela negra que abre por encima del cierre del día 1 y cierra por debajo
///   del punto medio del cuerpo del día 1.
///
/// # Parámetros
/// * `df` - DataFrame con columnas open, high, low, close.
/// * `output_col` - Nombre opcional para la columna de salida (defecto: "cdldarkcloudcover").
///
/// # Retorno
/// PolarsResult<DataFrame> con la columna del indicador añadida.
pub fn cdldarkcloudcover(df: &mut DataFrame, output_col: Option<&str>) {
    let output_name = output_col.unwrap_or("cdldarkcloudcover");
    let (open_s, _high_s, _low_s, close_s) = get_ohlc(&df).unwrap();
    let open = open_s.f64().unwrap();
    let close = close_s.f64().unwrap();

    let len = open.len();
    let mut result: Vec<i32> = vec![0; len];

    if len < 2 {
        df.with_column(Series::new(output_name.into(), &result).into());
    }

    for i in 1..len {
        let o0 = open.get(i - 1).unwrap();
        let c0 = close.get(i - 1).unwrap();
        let o1 = open.get(i).unwrap();
        let c1 = close.get(i).unwrap();

        // Día 1: Blanco
        let is_white0 = candle_color(o0, c0) == 1;
        // Día 2: Negro
        let is_black1 = candle_color(o1, c1) == -1;

        // Apertura por encima del cierre anterior
        let open_above = o1 > c0;

        // Cierre por debajo del punto medio del cuerpo anterior
        let mid_point = (o0 + c0) / 2.0;
        let deep_penetration = c1 < mid_point && c1 > o0;

        if is_white0 && is_black1 && open_above && deep_penetration {
            result[i] = PATTERN_BEARISH;
        }
    }

    df.with_column(Series::new(output_name.into(), result).into());
}

/// CDLDOJI - Doji
///
/// El Doji es una vela donde la apertura y el cierre son iguales o muy cercanos,
/// indicando indecisión en el mercado.
///
/// # Parámetros
/// * `df` - DataFrame con columnas open, high, low, close.
/// * `output_col` - Nombre opcional para la columna de salida (defecto: "cdldoji").
///
/// # Retorno
/// PolarsResult<DataFrame> con la columna del indicador añadida.
pub fn cdldoji(df: &mut DataFrame, output_col: Option<&str>) {
    let output_name = output_col.unwrap_or("cdldoji");
    let (open_s, high_s, low_s, close_s) = get_ohlc(&df).unwrap();
    let open = open_s.f64().unwrap();
    let high = high_s.f64().unwrap();
    let low = low_s.f64().unwrap();
    let close = close_s.f64().unwrap();

    let len = open.len();
    let mut result: Vec<i32> = vec![0; len];

    for i in 0..len {
        let o = open.get(i).unwrap();
        let h = high.get(i).unwrap();
        let l = low.get(i).unwrap();
        let c = close.get(i).unwrap();

        let body = (o - c).abs();
        let range = h - l;

        // Se considera Doji si el cuerpo es menor al 10% del rango total
        if range > 0.0 && body <= range * 0.1 {
            result[i] = PATTERN_BULLISH; // TA-Lib suele usar 100 para Doji
        } else if range == 0.0 && body == 0.0 {
            result[i] = PATTERN_BULLISH;
        }
    }

    df.with_column(Series::new(output_name.into(), result).into());
}

/// CDLDOJISTAR - Doji Star
///
/// El Doji Star es un patrón de reversión de dos velas que indica un posible cambio de tendencia.
/// - Bullish (Alcista): Vela negra larga seguida de un Doji con gap bajista.
/// - Bearish (Bajista): Vela blanca larga seguida de un Doji con gap alcista.
///
/// # Parámetros
/// * `df` - DataFrame con columnas open, high, low, close.
/// * `output_col` - Nombre opcional para la columna de salida (defecto: "cdldojistar").
///
/// # Retorno
/// PolarsResult<DataFrame> con la columna del indicador añadida.
pub fn cdldojistar(df: &mut DataFrame, output_col: Option<&str>) {
    let output_name = output_col.unwrap_or("cdldojistar");
    let (open_s, high_s, low_s, close_s) = get_ohlc(&df).unwrap();
    let open = open_s.f64().unwrap();
    let high = high_s.f64().unwrap();
    let low = low_s.f64().unwrap();
    let close = close_s.f64().unwrap();

    let len = open.len();
    let mut result: Vec<i32> = vec![0; len];

    if len < 2 {
        df.with_column(Series::new(output_name.into(), &result).into());
    }

    for i in 1..len {
        let o0 = open.get(i - 1).unwrap();
        let c0 = close.get(i - 1).unwrap();
        let o1 = open.get(i).unwrap();
        let h1 = high.get(i).unwrap();
        let l1 = low.get(i).unwrap();
        let c1 = close.get(i).unwrap();

        let body0 = (o0 - c0).abs();
        let body1 = (o1 - c1).abs();
        let range1 = h1 - l1;

        // Día 1: Cuerpo largo
        let is_long0 = body0 > (high.get(i - 1).unwrap() - low.get(i - 1).unwrap()) * 0.7;

        // Día 2: Doji (Cuerpo <= 10% del rango)
        let is_doji1 = if range1 > 0.0 {
            body1 <= range1 * 0.1
        } else {
            true
        };

        // Bullish Doji Star
        if candle_color(o0, c0) == -1 && is_long0 && is_doji1 && o1 < c0 {
            result[i] = PATTERN_BULLISH;
            continue;
        }

        // Bearish Doji Star
        if candle_color(o0, c0) == 1 && is_long0 && is_doji1 && o1 > c0 {
            result[i] = PATTERN_BEARISH;
        }
    }

    df.with_column(Series::new(output_name.into(), result).into());
}

/// CDLDRAGONFLYDOJI - Dragonfly Doji
///
/// El Dragonfly Doji (Doji Libélula) es una vela donde la apertura, el máximo y el cierre
/// son iguales o muy cercanos, con una sombra inferior larga. Indica una posible
/// reversión alcista.
///
/// # Parámetros
/// * `df` - DataFrame con columnas open, high, low, close.
/// * `output_col` - Nombre opcional para la columna de salida (defecto: "cdldragonflydoji").
///
/// # Retorno
/// PolarsResult<DataFrame> con la columna del indicador añadida.
pub fn cdldragonflydoji(df: &mut DataFrame, output_col: Option<&str>) {
    let output_name = output_col.unwrap_or("cdldragonflydoji");
    let (open_s, high_s, low_s, close_s) = get_ohlc(&df).unwrap();
    let open = open_s.f64().unwrap();
    let high = high_s.f64().unwrap();
    let low = low_s.f64().unwrap();
    let close = close_s.f64().unwrap();

    let len = open.len();
    let mut result: Vec<i32> = vec![0; len];

    for i in 0..len {
        let o = open.get(i).unwrap();
        let h = high.get(i).unwrap();
        let l = low.get(i).unwrap();
        let c = close.get(i).unwrap();

        let body = (o - c).abs();
        let range = h - l;

        if range == 0.0 {
            continue;
        }

        let is_doji = body <= range * 0.1;
        let small_upper_shadow = (h - o.max(c)) <= range * 0.1;
        let long_lower_shadow = (o.min(c) - l) > range * 0.6;

        if is_doji && small_upper_shadow && long_lower_shadow {
            result[i] = PATTERN_BULLISH;
        }
    }

    df.with_column(Series::new(output_name.into(), result).into());
}

/// CDLENGULFING - Engulfing Pattern
///
/// El Engulfing Pattern (Patrón Envolvente) es un patrón de reversión de dos velas:
/// - Bullish (Alcista): Vela negra seguida de una blanca que envuelve su cuerpo.
/// - Bearish (Bajista): Vela blanca seguida de una negra que envuelve su cuerpo.
///
/// # Parámetros
/// * `df` - DataFrame con columnas open, high, low, close.
/// * `output_col` - Nombre opcional para la columna de salida (defecto: "cdlengulfing").
///
/// # Retorno
/// PolarsResult<DataFrame> con la columna del indicador añadida.
pub fn cdlengulfing(df: &mut DataFrame, output_col: Option<&str>) {
    let output_name = output_col.unwrap_or("cdlengulfing");
    let (open_s, _high_s, _low_s, close_s) = get_ohlc(&df).unwrap();
    let open = open_s.f64().unwrap();
    let close = close_s.f64().unwrap();

    let len = open.len();
    let mut result: Vec<i32> = vec![0; len];

    if len < 2 {
        df.with_column(Series::new(output_name.into(), &result).into());
    }

    for i in 1..len {
        let o0 = open.get(i - 1).unwrap();
        let c0 = close.get(i - 1).unwrap();
        let o1 = open.get(i).unwrap();
        let c1 = close.get(i).unwrap();

        // Bullish Engulfing
        if candle_color(o0, c0) == -1 && candle_color(o1, c1) == 1 && o1 < c0 && c1 > o0 {
            result[i] = PATTERN_BULLISH;
            continue;
        }

        // Bearish Engulfing
        if candle_color(o0, c0) == 1 && candle_color(o1, c1) == -1 && o1 > c0 && c1 < o0 {
            result[i] = PATTERN_BEARISH;
        }
    }

    df.with_column(Series::new(output_name.into(), result).into());
}

/// CDLEVENINGDOJISTAR - Evening Doji Star
///
/// El Evening Doji Star es un patrón de reversión bajista de tres velas que ocurre
/// en una tendencia alcista:
/// 1. Vela blanca larga.
/// 2. Doji con gap alcista.
/// 3. Vela negra que cierra dentro del cuerpo de la primera vela.
///
/// # Parámetros
/// * `df` - DataFrame con columnas open, high, low, close.
/// * `output_col` - Nombre opcional para la columna de salida (defecto: "cdleveningdojistar").
///
/// # Retorno
/// PolarsResult<DataFrame> con la columna del indicador añadida.
pub fn cdleveningdojistar(df: &mut DataFrame, output_col: Option<&str>) {
    let output_name = output_col.unwrap_or("cdleveningdojistar");
    let (open_s, high_s, low_s, close_s) = get_ohlc(&df).unwrap();
    let open = open_s.f64().unwrap();
    let high = high_s.f64().unwrap();
    let low = low_s.f64().unwrap();
    let close = close_s.f64().unwrap();

    let len = open.len();
    let mut result: Vec<i32> = vec![0; len];

    if len < 3 {
        df.with_column(Series::new(output_name.into(), &result).into());
    }

    for i in 2..len {
        let o0 = open.get(i - 2).unwrap();
        let c0 = close.get(i - 2).unwrap();
        let o1 = open.get(i - 1).unwrap();
        let h1 = high.get(i - 1).unwrap();
        let l1 = low.get(i - 1).unwrap();
        let c1 = close.get(i - 1).unwrap();
        let o2 = open.get(i).unwrap();
        let c2 = close.get(i).unwrap();

        // Día 1: Blanco largo
        let is_white0 = candle_color(o0, c0) == 1;

        // Día 2: Doji con gap
        let body1 = (o1 - c1).abs();
        let range1 = h1 - l1;
        let is_doji1 = if range1 > 0.0 {
            body1 <= range1 * 0.1
        } else {
            true
        };
        let gap_up1 = o1 > c0 && c1 > c0;

        // Día 3: Negro que cierra dentro del cuerpo del Día 1
        let is_black2 = candle_color(o2, c2) == -1;
        let closes_deep = c2 < (o0 + c0) / 2.0 && c2 > o0;

        if is_white0 && is_doji1 && gap_up1 && is_black2 && closes_deep {
            result[i] = PATTERN_BEARISH;
        }
    }

    df.with_column(Series::new(output_name.into(), result).into());
}

/// CDLEVENINGSTAR - Evening Star
///
/// El Evening Star (Estrella del Atardecer) es un patrón de reversión bajista de tres velas:
/// 1. Vela blanca larga.
/// 2. Vela de cuerpo pequeño con gap alcista.
/// 3. Vela negra que cierra dentro del cuerpo de la primera vela.
///
/// # Parámetros
/// * `df` - DataFrame con columnas open, high, low, close.
/// * `output_col` - Nombre opcional para la columna de salida (defecto: "cdleveningstar").
///
/// # Retorno
/// PolarsResult<DataFrame> con la columna del indicador añadida.
pub fn cdleveningstar(df: &mut DataFrame, output_col: Option<&str>) {
    let output_name = output_col.unwrap_or("cdleveningstar");
    let (open_s, high_s, low_s, close_s) = get_ohlc(&df).unwrap();
    let open = open_s.f64().unwrap();
    let high = high_s.f64().unwrap();
    let low = low_s.f64().unwrap();
    let close = close_s.f64().unwrap();

    let len = open.len();
    let mut result: Vec<i32> = vec![0; len];

    if len < 3 {
        df.with_column(Series::new(output_name.into(), &result).into());
    }

    for i in 2..len {
        let o0 = open.get(i - 2).unwrap();
        let c0 = close.get(i - 2).unwrap();
        let o1 = open.get(i - 1).unwrap();
        let c1 = close.get(i - 1).unwrap();
        let o2 = open.get(i).unwrap();
        let c2 = close.get(i).unwrap();

        let body0 = (o0 - c0).abs();
        let body1 = (o1 - c1).abs();
        let range0 = high.get(i - 2).unwrap() - low.get(i - 2).unwrap();

        // Día 1: Blanco largo
        let is_white0 = candle_color(o0, c0) == 1 && body0 > range0 * 0.6;

        // Día 2: Cuerpo pequeño con gap
        let is_small1 = body1 < body0 * 0.3;
        let gap_up1 = o1 > c0 && c1 > c0;

        // Día 3: Negro que cierra dentro del cuerpo del Día 1
        let is_black2 = candle_color(o2, c2) == -1;
        let closes_deep = c2 < (o0 + c0) / 2.0 && c2 > o0;

        if is_white0 && is_small1 && gap_up1 && is_black2 && closes_deep {
            result[i] = PATTERN_BEARISH;
        }
    }

    df.with_column(Series::new(output_name.into(), result).into());
}

/// CDLGAPSIDESIDEWHITE - Up/Down-gap side-by-side white lines
///
/// Este es un patrón de continuación de tres velas:
/// - Bullish (Alcista): Una vela blanca seguida de un gap alcista y dos velas blancas "lado a lado" con cuerpos similares.
/// - Bearish (Bajista): Una vela negra seguida de un gap bajista y dos velas blancas "lado a lado" con cuerpos similares.
///
/// # Parámetros
/// * `df` - DataFrame con columnas open, high, low, close.
/// * `output_col` - Nombre opcional para la columna de salida (defecto: "cdlgapsidesidewhite").
///
/// # Retorno
/// PolarsResult<DataFrame> con la columna del indicador añadida.
pub fn cdlgapsidesidewhite(df: &mut DataFrame, output_col: Option<&str>) {
    let output_name = output_col.unwrap_or("cdlgapsidesidewhite");
    let (open_s, _high_s, _low_s, close_s) = get_ohlc(&df).unwrap();
    let open = open_s.f64().unwrap();
    let close = close_s.f64().unwrap();

    let len = open.len();
    let mut result: Vec<i32> = vec![0; len];

    if len < 3 {
        df.with_column(Series::new(output_name.into(), &result).into());
    }

    for i in 2..len {
        let o0 = open.get(i - 2).unwrap();
        let c0 = close.get(i - 2).unwrap();
        let o1 = open.get(i - 1).unwrap();
        let c1 = close.get(i - 1).unwrap();
        let o2 = open.get(i).unwrap();
        let c2 = close.get(i).unwrap();

        let is_white1 = candle_color(o1, c1) == 1;
        let is_white2 = candle_color(o2, c2) == 1;

        // Las dos velas finales deben ser blancas y estar "lado a lado" (aperturas similares)
        let side_by_side = (o1 - o2).abs() <= o1 * 0.005;
        // Cuerpos similares (diferencia menor al 10%)
        let body1 = c1 - o1;
        let body2 = c2 - o2;
        let similar_bodies = (body1 - body2).abs() <= body1 * 0.1;

        if is_white1 && is_white2 && side_by_side && similar_bodies {
            // Escenario Alcista (Up-gap)
            if candle_color(o0, c0) == 1 && o1 > c0 {
                result[i] = PATTERN_BULLISH;
            }
            // Escenario Bajista (Down-gap)
            else if candle_color(o0, c0) == -1 && c1 < c0 {
                result[i] = PATTERN_BEARISH;
            }
        }
    }

    df.with_column(Series::new(output_name.into(), result).into());
}

/// CDLGRAVESTONEDOJI - Gravestone Doji
///
/// El Gravestone Doji (Doji Lápida) es una vela donde la apertura, el mínimo y el cierre
/// son iguales o muy cercanos, con una sombra superior larga. Indica una posible
/// reversión bajista.
///
/// # Parámetros
/// * `df` - DataFrame con columnas open, high, low, close.
/// * `output_col` - Nombre opcional para la columna de salida (defecto: "cdlgravestonedoji").
///
/// # Retorno
/// PolarsResult<DataFrame> con la columna del indicador añadida.
pub fn cdlgravestonedoji(df: &mut DataFrame, output_col: Option<&str>) {
    let output_name = output_col.unwrap_or("cdlgravestonedoji");
    let (open_s, high_s, low_s, close_s) = get_ohlc(&df).unwrap();
    let open = open_s.f64().unwrap();
    let high = high_s.f64().unwrap();
    let low = low_s.f64().unwrap();
    let close = close_s.f64().unwrap();

    let len = open.len();
    let mut result: Vec<i32> = vec![0; len];

    for i in 0..len {
        let o = open.get(i).unwrap();
        let h = high.get(i).unwrap();
        let l = low.get(i).unwrap();
        let c = close.get(i).unwrap();

        let body = (o - c).abs();
        let range = h - l;

        if range == 0.0 {
            continue;
        }

        let is_doji = body <= range * 0.1;
        let small_lower_shadow = (o.min(c) - l) <= range * 0.1;
        let long_upper_shadow = (h - o.max(c)) > range * 0.6;

        if is_doji && small_lower_shadow && long_upper_shadow {
            result[i] = PATTERN_BULLISH; // TA-Lib usa 100 para marcar la presencia del patrón
        }
    }

    df.with_column(Series::new(output_name.into(), result).into());
}

/// CDLHAMMER - Hammer
///
/// El Hammer (Martillo) es un patrón de reversión alcista de una sola vela que ocurre
/// al final de una tendencia bajista. Tiene un cuerpo pequeño y una sombra inferior larga.
///
/// # Parámetros
/// * `df` - DataFrame con columnas open, high, low, close.
/// * `output_col` - Nombre opcional para la columna de salida (defecto: "cdlhammer").
///
/// # Retorno
/// PolarsResult<DataFrame> con la columna del indicador añadida.
pub fn cdlhammer(df: &mut DataFrame, output_col: Option<&str>) {
    let output_name = output_col.unwrap_or("cdlhammer");
    let (open_s, high_s, low_s, close_s) = get_ohlc(&df).unwrap();
    let open = open_s.f64().unwrap();
    let high = high_s.f64().unwrap();
    let low = low_s.f64().unwrap();
    let close = close_s.f64().unwrap();

    let len = open.len();
    let mut result: Vec<i32> = vec![0; len];

    for i in 0..len {
        let o = open.get(i).unwrap();
        let h = high.get(i).unwrap();
        let l = low.get(i).unwrap();
        let c = close.get(i).unwrap();

        let body = (o - c).abs();
        let range = h - l;

        if range == 0.0 {
            continue;
        }

        let lower_shadow = o.min(c) - l;
        let upper_shadow = h - o.max(c);

        // Martillo: Sombra inferior >= 2x Cuerpo, Sombra superior muy pequeña
        let is_hammer = lower_shadow >= 2.0 * body
            && upper_shadow <= range * 0.1
            && body > 0.0
            && body < range * 0.3;

        if is_hammer {
            result[i] = PATTERN_BULLISH;
        }
    }

    df.with_column(Series::new(output_name.into(), result).into());
}

/// CDLHANGINGMAN - Hanging Man
///
/// El Hanging Man (Hombre Colgado) es un patrón de reversión bajista de una sola vela
/// que ocurre al final de una tendencia alcista. Tiene la misma forma que un Hammer.
///
/// # Parámetros
/// * `df` - DataFrame con columnas open, high, low, close.
/// * `output_col` - Nombre opcional para la columna de salida (defecto: "cdlhangingman").
///
/// # Retorno
/// PolarsResult<DataFrame> con la columna del indicador añadida.
pub fn cdlhangingman(df: &mut DataFrame, output_col: Option<&str>) {
    let output_name = output_col.unwrap_or("cdlhangingman");
    let (open_s, high_s, low_s, close_s) = get_ohlc(&df).unwrap();
    let open = open_s.f64().unwrap();
    let high = high_s.f64().unwrap();
    let low = low_s.f64().unwrap();
    let close = close_s.f64().unwrap();

    let len = open.len();
    let mut result: Vec<i32> = vec![0; len];

    for i in 0..len {
        let o = open.get(i).unwrap();
        let h = high.get(i).unwrap();
        let l = low.get(i).unwrap();
        let c = close.get(i).unwrap();

        let body = (o - c).abs();
        let range = h - l;

        if range == 0.0 {
            continue;
        }

        let lower_shadow = o.min(c) - l;
        let upper_shadow = h - o.max(c);

        // Hombre colgado: Misma geometría que el martillo (Sombra inferior >= 2x Cuerpo)
        // pero se identifica como señal bajista (-100)
        let is_hanging_man = lower_shadow >= 2.0 * body
            && upper_shadow <= range * 0.1
            && body > 0.0
            && body < range * 0.3;

        if is_hanging_man {
            result[i] = PATTERN_BEARISH;
        }
    }

    df.with_column(Series::new(output_name.into(), result).into());
}

/// CDLHARAMI - Harami Pattern
///
/// El Harami es un patrón de reversión de dos velas donde el cuerpo de la segunda vela
/// está completamente contenido dentro del cuerpo de la primera.
/// - Bullish (Alcista): Vela negra seguida de una blanca pequeña contenida en ella.
/// - Bearish (Bajista): Vela blanca seguida de una negra pequeña contenida en ella.
///
/// # Parámetros
/// * `df` - DataFrame con columnas open, high, low, close.
/// * `output_col` - Nombre opcional para la columna de salida (defecto: "cdlharami").
///
/// # Retorno
/// PolarsResult<DataFrame> con la columna del indicador añadida.
pub fn cdlharami(df: &mut DataFrame, output_col: Option<&str>) {
    let output_name = output_col.unwrap_or("cdlharami");
    let (open_s, _high_s, _low_s, close_s) = get_ohlc(&df).unwrap();
    let open = open_s.f64().unwrap();
    let close = close_s.f64().unwrap();

    let len = open.len();
    let mut result: Vec<i32> = vec![0; len];

    if len < 2 {
        df.with_column(Series::new(output_name.into(), &result).into());
    }

    for i in 1..len {
        let o0 = open.get(i - 1).unwrap();
        let c0 = close.get(i - 1).unwrap();
        let o1 = open.get(i).unwrap();
        let c1 = close.get(i).unwrap();

        let body0_top = o0.max(c0);
        let body0_bottom = o0.min(c0);
        let body1_top = o1.max(c1);
        let body1_bottom = o1.min(c1);

        // La segunda vela debe estar dentro del cuerpo de la primera
        let is_inside = body1_top <= body0_top && body1_bottom >= body0_bottom;

        if is_inside {
            // Bullish: Negro -> Blanco
            if candle_color(o0, c0) == -1 && candle_color(o1, c1) == 1 {
                result[i] = PATTERN_BULLISH;
            }
            // Bearish: Blanco -> Negro
            else if candle_color(o0, c0) == 1 && candle_color(o1, c1) == -1 {
                result[i] = PATTERN_BEARISH;
            }
        }
    }

    df.with_column(Series::new(output_name.into(), result).into());
}

/// CDLHARAMICROSS - Harami Cross Pattern
///
/// El Harami Cross es un patrón de reversión de dos velas donde la segunda es un Doji
/// contenido totalmente en el cuerpo de la primera vela. Es más potente que el Harami normal.
/// - Bullish (Alcista): Vela negra seguida de un Doji contenido en ella.
/// - Bearish (Bajista): Vela blanca seguida de un Doji contenido en ella.
///
/// # Parámetros
/// * `df` - DataFrame con columnas open, high, low, close.
/// * `output_col` - Nombre opcional para la columna de salida (defecto: "cdlharamicross").
///
/// # Retorno
/// PolarsResult<DataFrame> con la columna del indicador añadida.
pub fn cdlharamicross(df: &mut DataFrame, output_col: Option<&str>) {
    let output_name = output_col.unwrap_or("cdlharamicross");
    let (open_s, high_s, low_s, close_s) = get_ohlc(&df).unwrap();
    let open = open_s.f64().unwrap();
    let high = high_s.f64().unwrap();
    let low = low_s.f64().unwrap();
    let close = close_s.f64().unwrap();

    let len = open.len();
    let mut result: Vec<i32> = vec![0; len];

    if len < 2 {
        df.with_column(Series::new(output_name.into(), &result).into());
    }

    for i in 1..len {
        let o0 = open.get(i - 1).unwrap();
        let c0 = close.get(i - 1).unwrap();
        let o1 = open.get(i).unwrap();
        let h1 = high.get(i).unwrap();
        let l1 = low.get(i).unwrap();
        let c1 = close.get(i).unwrap();

        let body0_top = o0.max(c0);
        let body0_bottom = o0.min(c0);
        let body1_top = o1.max(c1);
        let body1_bottom = o1.min(c1);

        // Identificar si la segunda vela es un Doji
        let body1 = (o1 - c1).abs();
        let range1 = h1 - l1;
        let is_doji1 = if range1 > 0.0 {
            body1 <= range1 * 0.1
        } else {
            true
        };

        // La segunda vela debe estar dentro del cuerpo de la primera
        let is_inside = body1_top <= body0_top && body1_bottom >= body0_bottom;

        if is_inside && is_doji1 {
            // Bullish: Negro -> Doji
            if candle_color(o0, c0) == -1 {
                result[i] = PATTERN_BULLISH;
            }
            // Bearish: Blanco -> Doji
            else if candle_color(o0, c0) == 1 {
                result[i] = PATTERN_BEARISH;
            }
        }
    }

    df.with_column(Series::new(output_name.into(), result).into());
}

/// CDLHIGHWAVE - High-Wave Candle
///
/// La vela High-Wave es un patrón de indecisión que tiene un cuerpo muy pequeño
/// y sombras superior e inferior muy largas.
/// - Bullish (Alcista): Vela blanca con sombras largas.
/// - Bearish (Bajista): Vela negra con sombras largas.
///
/// # Parámetros
/// * `df` - DataFrame con columnas open, high, low, close.
/// * `output_col` - Nombre opcional para la columna de salida (defecto: "cdlhighwave").
///
/// # Retorno
/// PolarsResult<DataFrame> con la columna del indicador añadida.
pub fn cdlhighwave(df: &mut DataFrame, output_col: Option<&str>) {
    let output_name = output_col.unwrap_or("cdlhighwave");
    let (open_s, high_s, low_s, close_s) = get_ohlc(&df).unwrap();
    let open = open_s.f64().unwrap();
    let high = high_s.f64().unwrap();
    let low = low_s.f64().unwrap();
    let close = close_s.f64().unwrap();

    let len = open.len();
    let mut result: Vec<i32> = vec![0; len];

    for i in 0..len {
        let o = open.get(i).unwrap();
        let h = high.get(i).unwrap();
        let l = low.get(i).unwrap();
        let c = close.get(i).unwrap();

        let body = (o - c).abs();
        let range = h - l;

        if range == 0.0 {
            continue;
        }

        // Definición de High-Wave: cuerpo pequeño y sombras muy largas
        let is_small_body = body <= range * 0.15;
        let is_long_upper = (h - o.max(c)) >= range * 0.35;
        let is_long_lower = (o.min(c) - l) >= range * 0.35;

        if is_small_body && is_long_upper && is_long_lower {
            if candle_color(o, c) == 1 {
                result[i] = PATTERN_BULLISH;
            } else if candle_color(o, c) == -1 {
                result[i] = PATTERN_BEARISH;
            } else {
                result[i] = 100; // Doji High-Wave
            }
        }
    }

    df.with_column(Series::new(output_name.into(), result).into());
}

/// CDLHIKKAKE - Hikkake Pattern
///
/// El patrón Hikkake es un patrón de ruptura falsa basado en una vela interna (Inside Bar).
/// - Bullish (Alcista): Inside Bar seguido de una vela con máximo y mínimo menor (trampa bajista).
/// - Bearish (Bajista): Inside Bar seguido de una vela con máximo y mínimo mayor (trampa alcista).
///
/// # Parámetros
/// * `df` - DataFrame con columnas open, high, low, close.
/// * `output_col` - Nombre opcional para la columna de salida (defecto: "cdlhikkake").
///
/// # Retorno
/// PolarsResult<DataFrame> con la columna del indicador añadida.
pub fn cdlhikkake(df: &mut DataFrame, output_col: Option<&str>) {
    let output_name = output_col.unwrap_or("cdlhikkake");
    let (_, high_s, low_s, _) = get_ohlc(&df).unwrap();
    let high = high_s.f64().unwrap();
    let low = low_s.f64().unwrap();

    let len = high.len();
    let mut result: Vec<i32> = vec![0; len];

    if len < 3 {
        df.with_column(Series::new(output_name.into(), &result).into());
    }

    for i in 2..len {
        let h0 = high.get(i - 2).unwrap();
        let l0 = low.get(i - 2).unwrap();
        let h1 = high.get(i - 1).unwrap();
        let l1 = low.get(i - 1).unwrap();
        let h2 = high.get(i).unwrap();
        let l2 = low.get(i).unwrap();

        // 1. Identificar Inside Bar (i-1 dentro de i-2)
        let is_inside_bar = h1 < h0 && l1 > l0;

        if is_inside_bar {
            // 2. Identificar el Hikkake en la vela actual (i)

            // Bullish: Ruptura falsa hacia abajo (máximo y mínimo menor que la inside bar)
            if h2 < h1 && l2 < l1 {
                result[i] = PATTERN_BULLISH;
            }
            // Bearish: Ruptura falsa hacia arriba (máximo y mínimo mayor que la inside bar)
            else if h2 > h1 && l2 > l1 {
                result[i] = PATTERN_BEARISH;
            }
        }
    }

    df.with_column(Series::new(output_name.into(), result).into());
}

/// CDLHIKKAKEMOD - Modified Hikkake Pattern
///
/// El patrón Modified Hikkake es una variante del Hikkake estándar que busca aumentar la fiabilidad.
/// Se compone de una secuencia de cuatro velas:
/// 1. Vela de referencia (i-3).
/// 2. Inside bar relativa a la 1ra (i-2), con cierre cerca del extremo (bajo para alcista, alto para bajista).
/// 3. Inside bar relativa a la 2da (i-1).
/// 4. Vela de ruptura falsa o "trap" (i): máximo menor y mínimo menor que la 3ra para alcista; o máximo mayor y mínimo mayor para bajista.
///
/// La confirmación ocurre si en las siguientes 3 velas el precio rompe el extremo opuesto de la 3ra vela.
/// - +100: Setup alcista.
/// - -100: Setup bajista.
/// - +200: Confirmación alcista.
/// - -200: Confirmación bajista.
///
/// # Parámetros
/// * `df` - DataFrame con columnas open, high, low, close.
/// * `output_col` - Nombre opcional para la columna de salida (defecto: "cdlhikkakemod").
///
/// # Retorno
/// PolarsResult<DataFrame> con la columna del indicador añadida.
pub fn cdlhikkakemod(df: &mut DataFrame, output_col: Option<&str>) {
    let output_name = output_col.unwrap_or("cdlhikkakemod");
    let (_open_s, high_s, low_s, close_s) = get_ohlc(&df).unwrap();
    let high = high_s.f64().unwrap();
    let low = low_s.f64().unwrap();
    let close = close_s.f64().unwrap();

    let len = high.len();
    let mut result: Vec<i32> = vec![0; len];

    if len < 6 {
        df.with_column(Series::new(output_name.into(), &result).into());
    }

    let mut pattern_idx: i32 = -1;
    let mut pattern_res: i32 = 0;

    for i in 3..len {
        let h3 = high.get(i - 3).unwrap();
        let l3 = low.get(i - 3).unwrap();
        let h2 = high.get(i - 2).unwrap();
        let l2 = low.get(i - 2).unwrap();
        let c2 = close.get(i - 2).unwrap();
        let h1 = high.get(i - 1).unwrap();
        let l1 = low.get(i - 1).unwrap();
        let h0 = high.get(i).unwrap();
        let l0 = low.get(i).unwrap();
        let c0 = close.get(i).unwrap();

        // 1. Identificar si hay un nuevo patrón (Trap bar)
        // 2nd candle (i-2) inside 1st (i-3)
        let is_inside2 = h2 < h3 && l2 > l3;
        // 3rd candle (i-1) inside 2nd (i-2)
        let is_inside1 = h1 < h2 && l1 > l2;

        if is_inside2 && is_inside1 {
            let range2 = h2 - l2;

            // Bullish: close near low (i-2), fakeout (i) con HH y LL menor que i-1
            if h0 < h1 && l0 < l1 && c2 <= l2 + range2 * 0.25 {
                result[i] = PATTERN_BULLISH;
                pattern_idx = i as i32;
                pattern_res = PATTERN_BULLISH;
            }
            // Bearish: close near high (i-2), fakeout (i) con HH y LL mayor que i-1
            else if h0 > h1 && l0 > l1 && c2 >= h2 - range2 * 0.25 {
                result[i] = PATTERN_BEARISH;
                pattern_idx = i as i32;
                pattern_res = PATTERN_BEARISH;
            } else {
                result[i] = 0;
            }
        } else {
            // Si no hay un nuevo patrón, buscamos confirmación de uno previo (máximo 3 velas atrás)
            if pattern_idx != -1 && i as i32 <= pattern_idx + 3 {
                let h_trigger = high.get((pattern_idx - 1) as usize).unwrap();
                let l_trigger = low.get((pattern_idx - 1) as usize).unwrap();

                if pattern_res == PATTERN_BULLISH && c0 > h_trigger {
                    result[i] = 200;
                    pattern_idx = -1;
                } else if pattern_res == PATTERN_BEARISH && c0 < l_trigger {
                    result[i] = -200;
                    pattern_idx = -1;
                } else {
                    result[i] = 0;
                }
            } else {
                result[i] = 0;
            }
        }
    }

    df.with_column(Series::new(output_name.into(), result).into());
}

/// CDLHOMINGPIGEON - Homing Pigeon
///
/// El patrón Homing Pigeon es un patrón de reversión alcista que ocurre en una tendencia bajista.
/// Se compone de dos velas:
/// 1. Una vela negra larga.
/// 2. Una vela negra más pequeña que está completamente contenida dentro del cuerpo de la primera.
///
/// Es similar al Harami alcista, pero ambas velas son negras (bajistas).
///
/// # Parámetros
/// * `df` - DataFrame con columnas open, high, low, close.
/// * `output_col` - Nombre opcional para la columna de salida (defecto: "cdlhomingpigeon").
///
/// # Retorno
/// PolarsResult<DataFrame> con la columna del indicador añadida.
pub fn cdlhomingpigeon(df: &mut DataFrame, output_col: Option<&str>) {
    let output_name = output_col.unwrap_or("cdlhomingpigeon");
    let (open_s, _high_s, _low_s, close_s) = get_ohlc(&df).unwrap();
    let open = open_s.f64().unwrap();
    let close = close_s.f64().unwrap();

    let len = open.len();
    let mut result: Vec<i32> = vec![0; len];

    if len < 2 {
        df.with_column(Series::new(output_name.into(), &result).into());
    }

    for i in 1..len {
        let o0 = open.get(i - 1).unwrap();
        let c0 = close.get(i - 1).unwrap();
        let o1 = open.get(i).unwrap();
        let c1 = close.get(i).unwrap();

        let is_black0 = candle_color(o0, c0) == -1;
        let is_black1 = candle_color(o1, c1) == -1;

        let body0_top = o0.max(c0);
        let body0_bottom = o0.min(c0);
        let body1_top = o1.max(c1);
        let body1_bottom = o1.min(c1);

        // La segunda vela debe estar dentro del cuerpo de la primera
        let is_inside = body1_top < body0_top && body1_bottom > body0_bottom;

        if is_black0 && is_black1 && is_inside {
            result[i] = PATTERN_BULLISH;
        }
    }

    df.with_column(Series::new(output_name.into(), result).into());
}

/// CDLIDENTICAL3CROWS - Identical Three Crows
///
/// El patrón Identical Three Crows (Tres Cuervos Idénticos) es una variante más bajista de los Tres Cuervos Negros.
/// Se compone de tres velas negras largas donde cada una abre al mismo nivel (o muy cerca) del cierre de la vela anterior.
///
/// # Parámetros
/// * `df` - DataFrame con columnas open, high, low, close.
/// * `output_col` - Nombre opcional para la columna de salida (defecto: "cdlidentical3crows").
///
/// # Retorno
/// PolarsResult<DataFrame> con la columna del indicador añadida.
pub fn cdlidentical3crows(df: &mut DataFrame, output_col: Option<&str>) {
    let output_name = output_col.unwrap_or("cdlidentical3crows");
    let (open_s, high_s, low_s, close_s) = get_ohlc(&df).unwrap();
    let open = open_s.f64().unwrap();
    let high = high_s.f64().unwrap();
    let low = low_s.f64().unwrap();
    let close = close_s.f64().unwrap();

    let len = open.len();
    let mut result: Vec<i32> = vec![0; len];

    if len < 3 {
        df.with_column(Series::new(output_name.into(), &result).into());
    }

    for i in 2..len {
        let o0 = open.get(i - 2).unwrap();
        let h0 = high.get(i - 2).unwrap();
        let l0 = low.get(i - 2).unwrap();
        let c0 = close.get(i - 2).unwrap();

        let o1 = open.get(i - 1).unwrap();
        let h1 = high.get(i - 1).unwrap();
        let l1 = low.get(i - 1).unwrap();
        let c1 = close.get(i - 1).unwrap();

        let o2 = open.get(i).unwrap();
        let h2 = high.get(i).unwrap();
        let l2 = low.get(i).unwrap();
        let c2 = close.get(i).unwrap();

        // 1. Las tres velas deben ser negras
        let all_black =
            candle_color(o0, c0) == -1 && candle_color(o1, c1) == -1 && candle_color(o2, c2) == -1;

        if !all_black {
            continue;
        }

        // 2. Cada vela debe tener un cuerpo largo (cuerpo > 50% del rango total)
        let is_long0 = (o0 - c0) > (h0 - l0) * 0.5;
        let is_long1 = (o1 - c1) > (h1 - l1) * 0.5;
        let is_long2 = (o2 - c2) > (h2 - l2) * 0.5;

        // 3. Aperturas idénticas (o muy cercanas) al cierre anterior
        // Usamos una tolerancia pequeña relativa al rango de la vela anterior
        let tolerance1 = (h0 - l0) * 0.1;
        let tolerance2 = (h1 - l1) * 0.1;
        let identical_open1 = (o1 - c0).abs() <= tolerance1;
        let identical_open2 = (o2 - c1).abs() <= tolerance2;

        // 4. Cierres progresivamente más bajos
        let lower_closes = c1 < c0 && c2 < c1;

        if is_long0 && is_long1 && is_long2 && identical_open1 && identical_open2 && lower_closes {
            result[i] = PATTERN_BEARISH;
        }
    }

    df.with_column(Series::new(output_name.into(), result).into());
}

/// CDLINNECK - In-Neck Pattern
///
/// El patrón In-Neck es un patrón de continuación bajista de dos velas que ocurre en una tendencia bajista.
/// Se compone de:
/// 1. Una vela negra larga.
/// 2. Una vela blanca que abre por debajo del mínimo de la vela anterior y cierra casi al mismo nivel del cierre de la vela anterior.
///
/// Indica que aunque hubo un intento alcista, no tuvo la fuerza suficiente para entrar significativamente en el cuerpo de la vela bajista previa.
///
/// # Parámetros
/// * `df` - DataFrame con columnas open, high, low, close.
/// * `output_col` - Nombre opcional para la columna de salida (defecto: "cdlinneck").
///
/// # Retorno
/// PolarsResult<DataFrame> con la columna del indicador añadida.
pub fn cdlinneck(df: &mut DataFrame, output_col: Option<&str>) {
    let output_name = output_col.unwrap_or("cdlinneck");
    let (open_s, high_s, low_s, close_s) = get_ohlc(&df).unwrap();
    let open = open_s.f64().unwrap();
    let high = high_s.f64().unwrap();
    let low = low_s.f64().unwrap();
    let close = close_s.f64().unwrap();

    let len = open.len();
    let mut result: Vec<i32> = vec![0; len];

    if len < 2 {
        df.with_column(Series::new(output_name.into(), &result).into());
    }

    for i in 1..len {
        let o0 = open.get(i - 1).unwrap();
        let h0 = high.get(i - 1).unwrap();
        let l0 = low.get(i - 1).unwrap();
        let c0 = close.get(i - 1).unwrap();

        let o1 = open.get(i).unwrap();
        let c1 = close.get(i).unwrap();

        // 1. La primera vela debe ser negra y larga
        let is_black0 = candle_color(o0, c0) == -1;
        let is_long0 = (o0 - c0) > (h0 - l0) * 0.5;

        // 2. La segunda vela debe ser blanca
        let is_white1 = candle_color(o1, c1) == 1;

        // 3. La segunda vela abre por debajo del mínimo de la primera
        let opens_below0 = o1 < l0;

        // 4. La segunda vela cierra cerca del cierre de la primera (nivel del cuello)
        let tolerance = (h0 - l0) * 0.1;
        let closes_at_neck = (c1 - c0).abs() <= tolerance;

        if is_black0 && is_long0 && is_white1 && opens_below0 && closes_at_neck {
            result[i] = PATTERN_BEARISH;
        }
    }

    df.with_column(Series::new(output_name.into(), result).into());
}

/// CDLINVERTEDHAMMER - Inverted Hammer
///
/// El Inverted Hammer (Martillo Invertido) es un patrón de reversión alcista que ocurre en una tendencia bajista.
/// Se compone de una vela con un cuerpo pequeño, una sombra superior larga (al menos el doble del cuerpo)
/// y casi ninguna sombra inferior.
///
/// # Parámetros
/// * `df` - DataFrame con columnas open, high, low, close.
/// * `output_col` - Nombre opcional para la columna de salida (defecto: "cdlinvertedhammer").
///
/// # Retorno
/// PolarsResult<DataFrame> con la columna del indicador añadida.
pub fn cdlinvertedhammer(df: &mut DataFrame, output_col: Option<&str>) {
    let output_name = output_col.unwrap_or("cdlinvertedhammer");
    let (open_s, high_s, low_s, close_s) = get_ohlc(&df).unwrap();
    let open = open_s.f64().unwrap();
    let high = high_s.f64().unwrap();
    let low = low_s.f64().unwrap();
    let close = close_s.f64().unwrap();

    let len = open.len();
    let mut result: Vec<i32> = vec![0; len];

    for i in 0..len {
        let o = open.get(i).unwrap();
        let h = high.get(i).unwrap();
        let l = low.get(i).unwrap();
        let c = close.get(i).unwrap();

        let body = (o - c).abs();
        let range = h - l;

        if range == 0.0 {
            continue;
        }

        let upper_shadow = h - o.max(c);
        let lower_shadow = o.min(c) - l;

        // Martillo Invertido: Sombra superior >= 2x Cuerpo, Sombra inferior muy pequeña
        let is_inverted_hammer = upper_shadow >= 2.0 * body
            && lower_shadow <= range * 0.1
            && body > 0.0
            && body < range * 0.3;

        if is_inverted_hammer {
            result[i] = PATTERN_BULLISH;
        }
    }

    df.with_column(Series::new(output_name.into(), result).into());
}

/// CDLKICKING - Kicking
///
/// El patrón Kicking (Patada) es un patrón de reversión de dos velas altamente fiable que se caracteriza por un gap entre dos marubozus de signo opuesto.
///
/// Bullish:
/// 1. Un Marubozu negro.
/// 2. Un Marubozu blanco que abre con un gap alcista (abre por encima o igual a la apertura anterior).
///
/// Bearish:
/// 1. Un Marubozu blanco.
/// 2. Un Marubozu negro que abre con un gap bajista (abre por debajo o igual a la apertura anterior).
///
/// # Parámetros
/// * `df` - DataFrame con columnas open, high, low, close.
/// * `output_col` - Nombre opcional para la columna de salida (defecto: "cdlkicking").
///
/// # Retorno
/// PolarsResult<DataFrame> con la columna del indicador añadida.
pub fn cdlkicking(df: &mut DataFrame, output_col: Option<&str>) {
    let output_name = output_col.unwrap_or("cdlkicking");
    let (open_s, high_s, low_s, close_s) = get_ohlc(&df).unwrap();
    let open = open_s.f64().unwrap();
    let high = high_s.f64().unwrap();
    let low = low_s.f64().unwrap();
    let close = close_s.f64().unwrap();

    let len = open.len();
    let mut result: Vec<i32> = vec![0; len];

    if len < 2 {
        df.with_column(Series::new(output_name.into(), &result).into());
    }

    for i in 1..len {
        let o0 = open.get(i - 1).unwrap();
        let h0 = high.get(i - 1).unwrap();
        let l0 = low.get(i - 1).unwrap();
        let c0 = close.get(i - 1).unwrap();

        let o1 = open.get(i).unwrap();
        let h1 = high.get(i).unwrap();
        let l1 = low.get(i).unwrap();
        let c1 = close.get(i).unwrap();

        let body0 = (o0 - c0).abs();
        let range0 = h0 - l0;
        let body1 = (o1 - c1).abs();
        let range1 = h1 - l1;

        if range0 == 0.0 || range1 == 0.0 {
            continue;
        }

        // Marubozu: sombras muy pequeñas (ej. < 5% del rango)
        let is_marubozu0 = body0 > range0 * 0.95;
        let is_marubozu1 = body1 > range1 * 0.95;

        if is_marubozu0 && is_marubozu1 {
            // Bullish: Negro seguido de Blanco con Gap alcista
            if candle_color(o0, c0) == -1 && candle_color(o1, c1) == 1 && o1 >= o0 {
                result[i] = PATTERN_BULLISH;
            }
            // Bearish: Blanco seguido de Negro con Gap bajista
            else if candle_color(o0, c0) == 1 && candle_color(o1, c1) == -1 && o1 <= o0 {
                result[i] = PATTERN_BEARISH;
            }
        }
    }

    df.with_column(Series::new(output_name.into(), result).into());
}

/// CDLKICKINGBYLENGTH - Kicking - bull/bear determined by the longer marubozu
///
/// El patrón Kicking By Length es una variante del Kicking donde el resultado (alcista o bajista)
/// está determinado por cuál de los dos marubozus es más largo.
///
/// Se compone de:
/// 1. Dos Marubozus de signo opuesto.
/// 2. Un gap entre ellos (alcista si la 1ra es negra, bajista si la 1ra es blanca).
/// 3. El resultado es el color del marubozu más largo.
///
/// # Parámetros
/// * `df` - DataFrame con columnas open, high, low, close.
/// * `output_col` - Nombre opcional para la columna de salida (defecto: "cdlkickingbylength").
///
/// # Retorno
/// PolarsResult<DataFrame> con la columna del indicador añadida.
pub fn cdlkickingbylength(df: &mut DataFrame, output_col: Option<&str>) {
    let output_name = output_col.unwrap_or("cdlkickingbylength");
    let (open_s, high_s, low_s, close_s) = get_ohlc(&df).unwrap();
    let open = open_s.f64().unwrap();
    let high = high_s.f64().unwrap();
    let low = low_s.f64().unwrap();
    let close = close_s.f64().unwrap();

    let len = open.len();
    let mut result: Vec<i32> = vec![0; len];

    if len < 2 {
        df.with_column(Series::new(output_name.into(), &result).into());
    }

    for i in 1..len {
        let o0 = open.get(i - 1).unwrap();
        let h0 = high.get(i - 1).unwrap();
        let l0 = low.get(i - 1).unwrap();
        let c0 = close.get(i - 1).unwrap();

        let o1 = open.get(i).unwrap();
        let h1 = high.get(i).unwrap();
        let l1 = low.get(i).unwrap();
        let c1 = close.get(i).unwrap();

        let body0 = (o0 - c0).abs();
        let range0 = h0 - l0;
        let body1 = (o1 - c1).abs();
        let range1 = h1 - l1;

        if range0 == 0.0 || range1 == 0.0 {
            continue;
        }

        // Marubozu: sombras muy pequeñas (ej. < 5% del rango)
        let is_marubozu0 = body0 > range0 * 0.95;
        let is_marubozu1 = body1 > range1 * 0.95;

        if is_marubozu0 && is_marubozu1 && candle_color(o0, c0) == -candle_color(o1, c1) {
            // Verificar Gap
            let has_gap = (candle_color(o0, c0) == -1 && o1 >= o0)   // Bullish setup
                       || (candle_color(o0, c0) == 1 && o1 <= o0); // Bearish setup

            if has_gap {
                // El color de la vela más larga determina el resultado
                if body1 > body0 {
                    result[i] = candle_color(o1, c1) * 100;
                } else {
                    result[i] = candle_color(o0, c0) * 100;
                }
            }
        }
    }

    df.with_column(Series::new(output_name.into(), result).into());
}

/// CDLLADDERBOTTOM - Ladder Bottom
///
/// El Ladder Bottom es un patrón de reversión alcista de cinco velas que ocurre en una tendencia bajista.
/// Se compone de:
/// 1, 2, 3. Tres velas negras largas con aperturas y cierres sucesivamente más bajos (como tres cuervos negros).
/// 4. Una vela negra con un cuerpo pequeño y una sombra superior larga (forma de martillo invertido).
/// 5. Una vela blanca que abre por encima del cuerpo de la cuarta vela y cierra más alto.
///
/// # Parámetros
/// * `df` - DataFrame con columnas open, high, low, close.
/// * `output_col` - Nombre opcional para la columna de salida (defecto: "cdladderbottom").
///
/// # Retorno
/// PolarsResult<DataFrame> con la columna del indicador añadida.
pub fn cdladderbottom(df: &mut DataFrame, output_col: Option<&str>) {
    let output_name = output_col.unwrap_or("cdladderbottom");
    let (open_s, high_s, low_s, close_s) = get_ohlc(&df).unwrap();
    let open = open_s.f64().unwrap();
    let high = high_s.f64().unwrap();
    let low = low_s.f64().unwrap();
    let close = close_s.f64().unwrap();

    let len = open.len();
    let mut result: Vec<i32> = vec![0; len];

    if len < 5 {
        df.with_column(Series::new(output_name.into(), &result).into());
    }

    for i in 4..len {
        let o0 = open.get(i - 4).unwrap();
        let c0 = close.get(i - 4).unwrap();
        let h0 = high.get(i - 4).unwrap();
        let l0 = low.get(i - 4).unwrap();

        let o1 = open.get(i - 3).unwrap();
        let c1 = close.get(i - 3).unwrap();
        let h1 = high.get(i - 3).unwrap();
        let l1 = low.get(i - 3).unwrap();

        let o2 = open.get(i - 2).unwrap();
        let c2 = close.get(i - 2).unwrap();
        let h2 = high.get(i - 2).unwrap();
        let l2 = low.get(i - 2).unwrap();

        let o3 = open.get(i - 1).unwrap();
        let c3 = close.get(i - 1).unwrap();
        let h3 = high.get(i - 1).unwrap();
        let l3 = low.get(i - 1).unwrap();

        let o4 = open.get(i).unwrap();
        let c4 = close.get(i).unwrap();

        // 1, 2, 3: Velas negras largas, cierres descendentes
        let black0 = candle_color(o0, c0) == -1 && (o0 - c0) > (h0 - l0) * 0.5;
        let black1 = candle_color(o1, c1) == -1 && (o1 - c1) > (h1 - l1) * 0.5 && c1 < c0;
        let black2 = candle_color(o2, c2) == -1 && (o2 - c2) > (h2 - l2) * 0.5 && c2 < c1;

        // 4: Vela negra con sombra superior larga y cuerpo pequeño
        let body3 = (o3 - c3).abs();
        let range3 = h3 - l3;
        let upper_shadow3 = h3 - o3.max(c3);
        let black3 = candle_color(o3, c3) == -1 && upper_shadow3 > body3 && body3 < range3 * 0.4;

        // 5: Vela blanca, abre por encima del cuerpo de la 4ta
        let white4 = candle_color(o4, c4) == 1 && o4 > o3.max(c3);

        if black0 && black1 && black2 && black3 && white4 {
            result[i] = PATTERN_BULLISH;
        }
    }

    df.with_column(Series::new(output_name.into(), result).into());
}

/// CDLLONGLEGGEDDOJI - Long Legged Doji
///
/// El Long Legged Doji es una vela con sombras superiores e inferiores muy largas y un cuerpo muy pequeño (Doji).
/// Indica indecisión extrema en el mercado.
///
/// # Parámetros
/// * `df` - DataFrame con columnas open, high, low, close.
/// * `output_col` - Nombre opcional para la columna de salida (defecto: "cdllongleggeddoji").
///
/// # Retorno
/// PolarsResult<DataFrame> con la columna del indicador añadida.
pub fn cdllongleggeddoji(df: &mut DataFrame, output_col: Option<&str>) {
    let output_name = output_col.unwrap_or("cdllongleggeddoji");
    let (open_s, high_s, low_s, close_s) = get_ohlc(&df).unwrap();
    let open = open_s.f64().unwrap();
    let high = high_s.f64().unwrap();
    let low = low_s.f64().unwrap();
    let close = close_s.f64().unwrap();

    let len = open.len();
    let mut result: Vec<i32> = vec![0; len];

    for i in 0..len {
        let o = open.get(i).unwrap();
        let h = high.get(i).unwrap();
        let l = low.get(i).unwrap();
        let c = close.get(i).unwrap();

        let body = (o - c).abs();
        let range = h - l;

        if range == 0.0 {
            continue;
        }

        let is_doji = body <= range * 0.1;
        let upper_shadow = h - o.max(c);
        let lower_shadow = o.min(c) - l;

        // Long Legged: Doji con sombras largas (ej. >= 30% del rango total cada una)
        let is_long_shadows = upper_shadow >= range * 0.3 && lower_shadow >= range * 0.3;

        if is_doji && is_long_shadows {
            result[i] = 100;
        }
    }

    df.with_column(Series::new(output_name.into(), result).into());
}

/// CDLLONGLINE - Long Line Candle
///
/// La vela Long Line es una vela con un cuerpo real largo en comparación con su rango total.
/// A diferencia del Marubozu, puede tener sombras, pero el cuerpo debe ser la parte predominante.
///
/// # Parámetros
/// * `df` - DataFrame con columnas open, high, low, close.
/// * `output_col` - Nombre opcional para la columna de salida (defecto: "cdllongline").
///
/// # Retorno
/// PolarsResult<DataFrame> con la columna del indicador añadida.
pub fn cdllongline(df: &mut DataFrame, output_col: Option<&str>) {
    let output_name = output_col.unwrap_or("cdllongline");
    let (open_s, high_s, low_s, close_s) = get_ohlc(&df).unwrap();
    let open = open_s.f64().unwrap();
    let high = high_s.f64().unwrap();
    let low = low_s.f64().unwrap();
    let close = close_s.f64().unwrap();

    let len = open.len();
    let mut result: Vec<i32> = vec![0; len];

    for i in 0..len {
        let o = open.get(i).unwrap();
        let h = high.get(i).unwrap();
        let l = low.get(i).unwrap();
        let c = close.get(i).unwrap();

        let body = (o - c).abs();
        let range = h - l;

        if range == 0.0 {
            continue;
        }

        // Definición de cuerpo largo: > 70% del rango total de la vela
        let is_long_body = body > range * 0.7;

        if is_long_body {
            if candle_color(o, c) == 1 {
                result[i] = PATTERN_BULLISH;
            } else if candle_color(o, c) == -1 {
                result[i] = PATTERN_BEARISH;
            }
        }
    }

    df.with_column(Series::new(output_name.into(), result).into());
}

/// CDLMARUBOZU - Marubozu
///
/// El Marubozu es una vela con un cuerpo largo y sin (o casi sin) sombras superiores e inferiores.
///
/// # Parámetros
/// * `df` - DataFrame con columnas open, high, low, close.
/// * `output_col` - Nombre opcional para la columna de salida (defecto: "cdlmarubozu").
///
/// # Retorno
/// PolarsResult<DataFrame> con la columna del indicador añadida.
pub fn cdlmarubozu(df: &mut DataFrame, output_col: Option<&str>) {
    let output_name = output_col.unwrap_or("cdlmarubozu");
    let (open_s, high_s, low_s, close_s) = get_ohlc(&df).unwrap();
    let open = open_s.f64().unwrap();
    let high = high_s.f64().unwrap();
    let low = low_s.f64().unwrap();
    let close = close_s.f64().unwrap();

    let len = open.len();
    let mut result: Vec<i32> = vec![0; len];

    for i in 0..len {
        let o = open.get(i).unwrap();
        let h = high.get(i).unwrap();
        let l = low.get(i).unwrap();
        let c = close.get(i).unwrap();

        let body = (o - c).abs();
        let range = h - l;

        if range == 0.0 {
            continue;
        }

        // Marubozu: el cuerpo debe ocupar casi todo el rango (ej: > 95%)
        let is_marubozu = body >= range * 0.95;

        if is_marubozu {
            if candle_color(o, c) == 1 {
                result[i] = PATTERN_BULLISH;
            } else if candle_color(o, c) == -1 {
                result[i] = PATTERN_BEARISH;
            }
        }
    }

    df.with_column(Series::new(output_name.into(), result).into());
}

/// CDLMATCHINGLOW - Matching Low
///
/// El Matching Low es un patrón de reversión alcista de dos velas que ocurre en una tendencia bajista.
/// Se compone de:
/// 1. Una vela negra larga.
/// 2. Una segunda vela negra que cierra exactamente (o muy cerca) al mismo nivel que el cierre de la primera.
///
/// Indica que se ha alcanzado un nivel de soporte donde los vendedores no pueden empujar el precio más abajo.
///
/// # Parámetros
/// * `df` - DataFrame con columnas open, high, low, close.
/// * `output_col` - Nombre opcional para la columna de salida (defecto: "cdlmatchinglow").
///
/// # Retorno
/// PolarsResult<DataFrame> con la columna del indicador añadida.
pub fn cdlmatchinglow(df: &mut DataFrame, output_col: Option<&str>) {
    let output_name = output_col.unwrap_or("cdlmatchinglow");
    let (open_s, high_s, low_s, close_s) = get_ohlc(&df).unwrap();
    let open = open_s.f64().unwrap();
    let high = high_s.f64().unwrap();
    let low = low_s.f64().unwrap();
    let close = close_s.f64().unwrap();

    let len = open.len();
    let mut result: Vec<i32> = vec![0; len];

    if len < 2 {
        df.with_column(Series::new(output_name.into(), &result).into());
    }

    for i in 1..len {
        let o0 = open.get(i - 1).unwrap();
        let c0 = close.get(i - 1).unwrap();
        let h0 = high.get(i - 1).unwrap();
        let l0 = low.get(i - 1).unwrap();

        let o1 = open.get(i).unwrap();
        let c1 = close.get(i).unwrap();

        // 1. Ambas velas deben ser negras
        let black0 = candle_color(o0, c0) == -1;
        let black1 = candle_color(o1, c1) == -1;

        if !black0 || !black1 {
            continue;
        }

        // 2. La primera vela debe ser larga (cuerpo > 50% del rango)
        let is_long0 = (o0 - c0) > (h0 - l0) * 0.5;

        // 3. Los cierres deben ser iguales (o muy cercanos)
        let tolerance = (h0 - l0) * 0.05; // 5% de tolerancia respecto al rango de la primera vela
        let equal_closes = (c0 - c1).abs() <= tolerance;

        if is_long0 && equal_closes {
            result[i] = PATTERN_BULLISH;
        }
    }

    df.with_column(Series::new(output_name.into(), result).into());
}

/// CDLMATHOLD - Mat Hold
///
/// El Mat Hold es un patrón de continuación alcista de cinco velas que ocurre en una tendencia alcista.
/// Se compone de:
/// 1. Una vela blanca larga.
/// 2. Una vela negra pequeña que abre con un gap alcista.
/// 3, 4. Dos velas pequeñas (normalmente negras) que continúan la consolidación pero se mantienen dentro del rango de la primera vela.
/// 5. Una vela blanca larga que cierra por encima del máximo de la primera vela.
///
/// A diferencia del "Rising Three Methods", el Mat Hold permite un gap en la segunda vela y es considerado más robusto.
///
/// # Parámetros
/// * `df` - DataFrame con columnas open, high, low, close.
/// * `output_col` - Nombre opcional para la columna de salida (defecto: "cdlmathold").
///
/// # Retorno
/// PolarsResult<DataFrame> con la columna del indicador añadida.
pub fn cdlmathold(df: &mut DataFrame, output_col: Option<&str>) {
    let output_name = output_col.unwrap_or("cdlmathold");
    let (open_s, high_s, low_s, close_s) = get_ohlc(&df).unwrap();
    let open = open_s.f64().unwrap();
    let high = high_s.f64().unwrap();
    let low = low_s.f64().unwrap();
    let close = close_s.f64().unwrap();

    let len = open.len();
    let mut result: Vec<i32> = vec![0; len];

    if len < 5 {
        df.with_column(Series::new(output_name.into(), &result).into());
    }

    for i in 4..len {
        let o0 = open.get(i - 4).unwrap();
        let c0 = close.get(i - 4).unwrap();
        let h0 = high.get(i - 4).unwrap();
        let l0 = low.get(i - 4).unwrap();

        let o1 = open.get(i - 3).unwrap();
        let c1 = close.get(i - 3).unwrap();

        let _ = open.get(i - 2).unwrap();
        let _ = close.get(i - 2).unwrap();
        let l2 = low.get(i - 2).unwrap();

        let _ = open.get(i - 1).unwrap();
        let _ = close.get(i - 1).unwrap();
        let l3 = low.get(i - 1).unwrap();

        let o4 = open.get(i).unwrap();
        let c4 = close.get(i).unwrap();

        // 1. Vela blanca larga
        let body0 = (c0 - o0).abs();
        let range0 = h0 - l0;
        let white0 = candle_color(o0, c0) == 1 && body0 > range0 * 0.5;

        // 2. Vela negra pequeña con gap alcista
        let black1 = candle_color(o1, c1) == -1 && o1 > c0;

        // 3, 4. Velas pequeñas que se mantienen sobre el mínimo de la 1ra
        let consolidation = l2 > l0 && l3 > l0;

        // 5. Vela blanca larga que cierra sobre el máximo (o cierre) de la 1ra
        let white4 = candle_color(o4, c4) == 1 && c4 > h0;

        if white0 && black1 && consolidation && white4 {
            result[i] = PATTERN_BULLISH;
        }
    }

    df.with_column(Series::new(output_name.into(), result).into());
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MorningDojiStar {
    pub penetration: f64,
}

/// CDLMORNINGDOJISTAR - Morning Doji Star
///
/// El Morning Doji Star es un patrón de reversión alcista de tres velas.
/// Consiste en una vela negra larga, seguida de un Doji que abre con gap bajista,
/// y finalmente una vela blanca que cierra profundamente dentro del cuerpo de la primera vela.
///
/// # Parámetros
/// * `df`: DataFrame de Polars con columnas "open", "high", "low", "close".
/// * `penetration`: Porcentaje de penetración de la tercera vela en el cuerpo de la primera (default: 0.3).
/// * `output_col`: Nombre opcional para la columna de salida (default: "cdlmorningdojistar").
///
/// # Retorno
/// * DataFrame con una nueva columna con los valores 0 (nada) o 100 (alcista).
pub fn cdlmorningdojistar(df: &mut DataFrame, penetration: Option<f64>, output_col: Option<&str>) {
    let output_name = output_col.unwrap_or("cdlmorningdojistar");
    let penetration = penetration.unwrap_or(0.3);
    let (open_s, high_s, low_s, close_s) = get_ohlc(&df).unwrap();
    let open = open_s.f64().unwrap();
    let high = high_s.f64().unwrap();
    let low = low_s.f64().unwrap();
    let close = close_s.f64().unwrap();

    let len = open.len();
    let mut result: Vec<i32> = vec![0; len];

    if len < 3 {
        df.with_column(Series::new(output_name.into(), &result).into());
    }

    for i in 2..len {
        let o0 = open.get(i - 2).unwrap();
        let h0 = high.get(i - 2).unwrap();
        let l0 = low.get(i - 2).unwrap();
        let c0 = close.get(i - 2).unwrap();

        let o1 = open.get(i - 1).unwrap();
        let h1 = high.get(i - 1).unwrap();
        let l1 = low.get(i - 1).unwrap();
        let c1 = close.get(i - 1).unwrap();

        let o2 = open.get(i).unwrap();
        let c2 = close.get(i).unwrap();

        let body0 = (o0 - c0).abs();
        let range0 = h0 - l0;

        // Día 1: Negro largo
        let is_black0 = candle_color(o0, c0) == -1;
        let is_long0 = if range0 > 0.0 {
            body0 > range0 * 0.6
        } else {
            false
        };

        // Día 2: Doji con gap bajista
        let body1 = (o1 - c1).abs();
        let range1 = h1 - l1;
        let is_doji1 = if range1 > 0.0 {
            body1 <= range1 * 0.1
        } else {
            true
        };
        // Gap bajista entre cuerpos
        let gap_down1 = o1.max(c1) < o0.min(c0);

        // Día 3: Blanco con gap alcista y penetración
        let is_white2 = candle_color(o2, c2) == 1;
        // Gap alcista entre cuerpos respecto al Doji
        let gap_up2 = o2.min(c2) > o1.max(c1);

        // Penetración: el cierre de la vela 3 supera el nivel de penetración del cuerpo de la vela 1
        let target_level = c0 + body0 * penetration;
        let deep_penetration = c2 > target_level && c2 < o0;

        if is_black0
            && is_long0
            && is_doji1
            && gap_down1
            && is_white2
            && gap_up2
            && deep_penetration
        {
            result[i] = PATTERN_BULLISH;
        }
    }

    df.with_column(Series::new(output_name.into(), result).into());
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MorningStar {
    pub penetration: f64,
}

/// CDLMORNINGSTAR - Morning Star
///
/// El Morning Star es un patrón de reversión alcista de tres velas.
/// Consiste en una vela negra larga, seguida de una vela de cuerpo pequeño que abre con gap bajista,
/// y finalmente una vela blanca que cierra profundamente dentro del cuerpo de la primera vela.
///
/// # Parámetros
/// * `df`: DataFrame de Polars con columnas "open", "high", "low", "close".
/// * `penetration`: Porcentaje de penetración de la tercera vela en el cuerpo de la primera (default: 0.3).
/// * `output_col`: Nombre opcional para la columna de salida (default: "cdlmorningstar").
///
/// # Retorno
/// * DataFrame con una nueva columna con los valores 0 (nada) o 100 (alcista).
pub fn cdlmorningstar(df: &mut DataFrame, penetration: Option<f64>, output_col: Option<&str>) {
    let output_name = output_col.unwrap_or("cdlmorningstar");
    let penetration = penetration.unwrap_or(0.3);
    let (open_s, high_s, low_s, close_s) = get_ohlc(&df).unwrap();
    let open = open_s.f64().unwrap();
    let high = high_s.f64().unwrap();
    let low = low_s.f64().unwrap();
    let close = close_s.f64().unwrap();

    let len = open.len();
    let mut result: Vec<i32> = vec![0; len];

    if len < 3 {
        df.with_column(Series::new(output_name.into(), &result).into());
    }

    for i in 2..len {
        let o0 = open.get(i - 2).unwrap();
        let h0 = high.get(i - 2).unwrap();
        let l0 = low.get(i - 2).unwrap();
        let c0 = close.get(i - 2).unwrap();

        let o1 = open.get(i - 1).unwrap();
        let _ = high.get(i - 1).unwrap();
        let _ = low.get(i - 1).unwrap();
        let c1 = close.get(i - 1).unwrap();

        let o2 = open.get(i).unwrap();
        let c2 = close.get(i).unwrap();

        let body0 = (o0 - c0).abs();
        let range0 = h0 - l0;

        // Día 1: Negro largo
        let is_black0 = candle_color(o0, c0) == -1;
        let is_long0 = if range0 > 0.0 {
            body0 > range0 * 0.6
        } else {
            false
        };

        // Día 2: Cuerpo pequeño con gap bajista
        let body1 = (o1 - c1).abs();
        let is_small1 = body1 < body0 * 0.3;
        let gap_down1 = o1.max(c1) < o0.min(c0);

        // Día 3: Blanco con gap alcista y penetración
        let is_white2 = candle_color(o2, c2) == 1;
        // Gap alcista entre cuerpos respecto al cuerpo de la vela 2
        let gap_up2 = o2.min(c2) > o1.max(c1);

        // Penetración: el cierre de la vela 3 supera el nivel de penetración del cuerpo de la vela 1
        let target_level = c0 + body0 * penetration;
        let deep_penetration = c2 > target_level && c2 < o0;

        if is_black0
            && is_long0
            && is_small1
            && gap_down1
            && is_white2
            && gap_up2
            && deep_penetration
        {
            result[i] = PATTERN_BULLISH;
        }
    }

    df.with_column(Series::new(output_name.into(), result).into());
}

/// CDLONNECK - On-Neck Pattern
///
/// El patrón On-Neck es un patrón de continuación bajista que ocurre en una tendencia a la baja.
/// Consiste en una vela negra larga seguida de una vela blanca pequeña que abre con gap bajista
/// pero cierra al mismo nivel (o muy cerca) del mínimo de la vela anterior.
///
/// # Parámetros
/// * `df`: DataFrame de Polars con columnas "open", "high", "low", "close".
/// * `output_col`: Nombre opcional para la columna de salida (default: "cdlonneck").
///
/// # Retorno
/// * DataFrame con una nueva columna con los valores 0 (nada) o -100 (bajista).
pub fn cdlonneck(df: &mut DataFrame, output_col: Option<&str>) {
    let output_name = output_col.unwrap_or("cdlonneck");
    let (open_s, high_s, low_s, close_s) = get_ohlc(&df).unwrap();
    let open = open_s.f64().unwrap();
    let high = high_s.f64().unwrap();
    let low = low_s.f64().unwrap();
    let close = close_s.f64().unwrap();

    let len = open.len();
    let mut result: Vec<i32> = vec![0; len];

    if len < 2 {
        df.with_column(Series::new(output_name.into(), &result).into());
    }

    for i in 1..len {
        let o0 = open.get(i - 1).unwrap();
        let h0 = high.get(i - 1).unwrap();
        let l0 = low.get(i - 1).unwrap();
        let c0 = close.get(i - 1).unwrap();

        let o1 = open.get(i).unwrap();
        let c1 = close.get(i).unwrap();

        let body0 = (o0 - c0).abs();
        let range0 = h0 - l0;

        // Día 1: Vela negra larga
        let is_black0 = candle_color(o0, c0) == -1;
        let is_long0 = if range0 > 0.0 {
            body0 > range0 * 0.6
        } else {
            false
        };

        // Día 2: Vela blanca pequeña
        let is_white1 = candle_color(o1, c1) == 1;

        // El cierre del día 2 debe estar cerca del mínimo del día 1 (On-Neck)
        // Usamos una tolerancia pequeña relativa al rango de la primera vela
        let tolerance = range0 * 0.1;
        let on_neck = (c1 - l0).abs() <= tolerance;

        // Apertura con gap bajista (respecto al cierre anterior)
        let gap_down = o1 < c0;

        if is_black0 && is_long0 && is_white1 && on_neck && gap_down {
            result[i] = PATTERN_BEARISH;
        }
    }

    df.with_column(Series::new(output_name.into(), result).into());
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Piercing {
    pub penetration: f64,
}

/// CDLPIERCING - Piercing Pattern
///
/// El patrón Piercing es un patrón de reversión alcista de dos velas.
/// Consiste en una vela negra larga seguida de una vela blanca que abre por debajo del mínimo de la vela anterior
/// y cierra por encima del punto medio del cuerpo de la primera vela.
///
/// # Parámetros
/// * `df`: DataFrame de Polars con columnas "open", "high", "low", "close".
/// * `penetration`: Porcentaje de penetración de la segunda vela en el cuerpo de la primera (default: 0.5).
/// * `output_col`: Nombre opcional para la columna de salida (default: "cdlpiercing").
///
/// # Retorno
/// * DataFrame con una nueva columna con los valores 0 (nada) o 100 (alcista).
pub fn cdlpiercing(df: &mut DataFrame, penetration: Option<f64>, output_col: Option<&str>) {
    let output_name = output_col.unwrap_or("cdlpiercing");
    let penetration = penetration.unwrap_or(0.5);
    let (open_s, high_s, low_s, close_s) = get_ohlc(&df).unwrap();
    let open = open_s.f64().unwrap();
    let high = high_s.f64().unwrap();
    let low = low_s.f64().unwrap();
    let close = close_s.f64().unwrap();

    let len = open.len();
    let mut result: Vec<i32> = vec![0; len];

    if len < 2 {
        df.with_column(Series::new(output_name.into(), &result).into());
    }

    for i in 1..len {
        let o0 = open.get(i - 1).unwrap();
        let h0 = high.get(i - 1).unwrap();
        let l0 = low.get(i - 1).unwrap();
        let c0 = close.get(i - 1).unwrap();

        let o1 = open.get(i).unwrap();
        let c1 = close.get(i).unwrap();

        let body0 = (o0 - c0).abs();
        let range0 = h0 - l0;

        // Día 1: Vela negra larga
        let is_black0 = candle_color(o0, c0) == -1;
        let is_long0 = if range0 > 0.0 {
            body0 > range0 * 0.6
        } else {
            false
        };

        // Día 2: Vela blanca que abre por debajo del mínimo anterior
        let is_white1 = candle_color(o1, c1) == 1;
        let open_below = o1 < l0;

        // Cierre por encima del nivel de penetración (por defecto punto medio) del cuerpo anterior
        let target_level = c0 + body0 * penetration;
        let deep_penetration = c1 > target_level && c1 < o0;

        if is_black0 && is_long0 && is_white1 && open_below && deep_penetration {
            result[i] = PATTERN_BULLISH;
        }
    }

    df.with_column(Series::new(output_name.into(), result).into());
}

/// CDLRICKSHAWMAN - Rickshaw Man
///
/// El Rickshaw Man es un Doji de piernas largas donde la apertura y el cierre
/// están en el centro (o muy cerca) del rango de la vela. Indica extrema indecisión.
///
/// # Parámetros
/// * `df`: DataFrame de Polars con columnas "open", "high", "low", "close".
/// * `output_col`: Nombre opcional para la columna de salida (default: "cdlrickshawman").
///
/// # Retorno
/// * DataFrame con una nueva columna con los valores 0 (nada) o 100 (indecisión/neutral).
pub fn cdlrickshawman(df: &mut DataFrame, output_col: Option<&str>) {
    let output_name = output_col.unwrap_or("cdlrickshawman");
    let (open_s, high_s, low_s, close_s) = get_ohlc(&df).unwrap();
    let open = open_s.f64().unwrap();
    let high = high_s.f64().unwrap();
    let low = low_s.f64().unwrap();
    let close = close_s.f64().unwrap();

    let len = open.len();
    let mut result: Vec<i32> = vec![0; len];

    for i in 0..len {
        let o = open.get(i).unwrap();
        let h = high.get(i).unwrap();
        let l = low.get(i).unwrap();
        let c = close.get(i).unwrap();

        let body = (o - c).abs();
        let range = h - l;

        if range == 0.0 {
            continue;
        }

        // 1. Condición de Doji (cuerpo muy pequeño)
        let is_doji = body <= range * 0.1;

        // 2. Condición de Rickshaw Man: el cuerpo está cerca del centro del rango
        let body_mid = (o + c) / 2.0;
        let range_mid = (h + l) / 2.0;
        let is_centered = (body_mid - range_mid).abs() <= range * 0.1;

        if is_doji && is_centered {
            result[i] = 100; // Usualmente se marca como 100 para indicar presencia del patrón
        }
    }

    df.with_column(Series::new(output_name.into(), result).into());
}

/// CDLRISEFALL3METHODS - Rising/Falling Three Methods
///
/// El patrón de Tres Métodos es un patrón de continuación de 5 velas.
///
/// # Rising Three Methods (Alcista):
/// 1. Una vela blanca larga.
/// 2. Un grupo de velas pequeñas (generalmente 3) que caen pero permanecen dentro del rango de la primera vela.
/// 3. Una vela blanca larga que cierra por encima del cierre de la primera vela.
///
/// # Falling Three Methods (Bajista):
/// 1. Una vela negra larga.
/// 2. Un grupo de velas pequeñas (generalmente 3) que suben pero permanecen dentro del rango de la primera vela.
/// 3. Una vela negra larga que cierra por debajo del cierre de la primera vela.
///
/// # Parámetros
/// * `df`: DataFrame de Polars con columnas "open", "high", "low", "close".
/// * `output_col`: Nombre opcional para la columna de salida (default: "cdlrisefall3methods").
///
/// # Retorno
/// * DataFrame con una nueva columna con los valores 100 (alcista), -100 (bajista) o 0.
pub fn cdlrisefall3methods(df: &mut DataFrame, output_col: Option<&str>) {
    let output_name = output_col.unwrap_or("cdlrisefall3methods");
    let (open_s, high_s, low_s, close_s) = get_ohlc(&df).unwrap();
    let open = open_s.f64().unwrap();
    let high = high_s.f64().unwrap();
    let low = low_s.f64().unwrap();
    let close = close_s.f64().unwrap();

    let len = open.len();
    let mut result: Vec<i32> = vec![0; len];

    if len < 5 {
        df.with_column(Series::new(output_name.into(), &result).into());
    }

    for i in 4..len {
        let o0 = open.get(i - 4).unwrap();
        let h0 = high.get(i - 4).unwrap();
        let l0 = low.get(i - 4).unwrap();
        let c0 = close.get(i - 4).unwrap();

        let o4 = open.get(i).unwrap();
        let c4 = close.get(i).unwrap();

        let body0 = (o0 - c0).abs();
        let range0 = h0 - l0;

        // --- RISING THREE METHODS (Alcista) ---
        let is_white0 = candle_color(o0, c0) == 1;
        let is_long0 = if range0 > 0.0 {
            body0 > range0 * 0.6
        } else {
            false
        };

        if is_white0 && is_long0 {
            let mut all_inside = true;
            let mut rising_continuation = false;

            // Velas 1, 2, 3 (índices i-3, i-2, i-1) deben estar dentro del rango de la vela 0
            for j in 1..4 {
                let hj = high.get(i - j).unwrap();
                let lj = low.get(i - j).unwrap();
                if hj > h0 || lj < l0 {
                    all_inside = false;
                    break;
                }
            }

            // Vela 4 debe ser blanca larga y cerrar sobre c0
            let is_white4 = candle_color(o4, c4) == 1;
            if is_white4 && c4 > c0 && (o4 - c4).abs() > range0 * 0.5 {
                rising_continuation = true;
            }

            if all_inside && rising_continuation {
                result[i] = PATTERN_BULLISH;
                continue;
            }
        }

        // --- FALLING THREE METHODS (Bajista) ---
        let is_black0 = candle_color(o0, c0) == -1;
        if is_black0 && is_long0 {
            let mut all_inside = true;
            let mut falling_continuation = false;

            for j in 1..4 {
                let hj = high.get(i - j).unwrap();
                let lj = low.get(i - j).unwrap();
                if hj > h0 || lj < l0 {
                    all_inside = false;
                    break;
                }
            }

            let is_black4 = candle_color(o4, c4) == -1;
            if is_black4 && c4 < c0 && (o4 - c4).abs() > range0 * 0.5 {
                falling_continuation = true;
            }

            if all_inside && falling_continuation {
                result[i] = PATTERN_BEARISH;
            }
        }
    }

    df.with_column(Series::new(output_name.into(), result).into());
}

/// CDLSEPARATINGLINES - Separating Lines
///
/// El patrón Separating Lines es un patrón de continuación de 2 velas.
///
/// # Bullish Separating Lines (Alcista):
/// 1. Una vela negra.
/// 2. Una vela blanca que abre al mismo nivel que la apertura de la vela negra anterior.
///
/// # Bearish Separating Lines (Bajista):
/// 1. Una vela blanca.
/// 2. Una vela negra que abre al mismo nivel que la apertura de la vela blanca anterior.
///
/// # Parámetros
/// * `df`: DataFrame de Polars con columnas "open", "high", "low", "close".
/// * `output_col`: Nombre opcional para la columna de salida (default: "cdlseparatinglines").
///
/// # Retorno
/// * DataFrame con una nueva columna con los valores 100 (alcista), -100 (bajista) o 0.
pub fn cdlseparatinglines(df: &mut DataFrame, output_col: Option<&str>) {
    let output_name = output_col.unwrap_or("cdlseparatinglines");
    let (open_s, high_s, low_s, close_s) = get_ohlc(&df).unwrap();
    let open = open_s.f64().unwrap();
    let high = high_s.f64().unwrap();
    let low = low_s.f64().unwrap();
    let close = close_s.f64().unwrap();

    let len = open.len();
    let mut result: Vec<i32> = vec![0; len];

    if len < 2 {
        df.with_column(Series::new(output_name.into(), &result).into());
    }

    for i in 1..len {
        let o0 = open.get(i - 1).unwrap();
        let c0 = close.get(i - 1).unwrap();
        let h0 = high.get(i - 1).unwrap();
        let l0 = low.get(i - 1).unwrap();

        let o1 = open.get(i).unwrap();
        let c1 = close.get(i).unwrap();

        let range0 = h0 - l0;
        let tolerance = if range0 > 0.0 { range0 * 0.05 } else { 0.0001 };

        // --- BULLISH SEPARATING LINES ---
        // Vela 0 negra, Vela 1 blanca, Apertura 1 == Apertura 0
        let is_black0 = candle_color(o0, c0) == -1;
        let is_white1 = candle_color(o1, c1) == 1;
        let equal_open = (o1 - o0).abs() <= tolerance;

        if is_black0 && is_white1 && equal_open {
            result[i] = PATTERN_BULLISH;
            continue;
        }

        // --- BEARISH SEPARATING LINES ---
        // Vela 0 blanca, Vela 1 negra, Apertura 1 == Apertura 0
        let is_white0 = candle_color(o0, c0) == 1;
        let is_black1 = candle_color(o1, c1) == -1;

        if is_white0 && is_black1 && equal_open {
            result[i] = PATTERN_BEARISH;
        }
    }

    df.with_column(Series::new(output_name.into(), result).into());
}

/// CDLSHOOTINGSTAR - Shooting Star
///
/// El Shooting Star (Estrella Fugaz) es un patrón de reversión bajista de una vela.
/// Se caracteriza por una vela con un cuerpo pequeño en la parte inferior del rango,
/// una sombra superior muy larga (al menos 2 veces el tamaño del cuerpo)
/// y una sombra inferior muy pequeña o inexistente.
///
/// # Parámetros
/// * `df`: DataFrame de Polars con columnas "open", "high", "low", "close".
/// * `output_col`: Nombre opcional para la columna de salida (default: "cdlshootingstar").
///
/// # Retorno
/// * DataFrame con una nueva columna con los valores 0 o -100 (bajista).
pub fn cdlshootingstar(df: &mut DataFrame, output_col: Option<&str>) {
    let output_name = output_col.unwrap_or("cdlshootingstar");
    let (open_s, high_s, low_s, close_s) = get_ohlc(&df).unwrap();
    let open = open_s.f64().unwrap();
    let high = high_s.f64().unwrap();
    let low = low_s.f64().unwrap();
    let close = close_s.f64().unwrap();

    let len = open.len();
    let mut result: Vec<i32> = vec![0; len];

    if len < 2 {
        df.with_column(Series::new(output_name.into(), &result).into());
    }

    for i in 1..len {
        let o = open.get(i).unwrap();
        let h = high.get(i).unwrap();
        let l = low.get(i).unwrap();
        let c = close.get(i).unwrap();

        // Referencia anterior para tendencia (opcional pero común)
        let c_prev = close.get(i - 1).unwrap();

        let body = (o - c).abs();
        let range = h - l;

        if range == 0.0 {
            continue;
        }

        let upper_shadow = h - o.max(c);
        let lower_shadow = o.min(c) - l;

        // 1. Cuerpo pequeño: cuerpo <= 30% del rango total
        let is_small_body = body <= range * 0.3;

        // 2. Sombra superior larga: sombra superior >= 2 * cuerpo
        let long_upper_shadow = upper_shadow >= body * 2.0;

        // 3. Sombra inferior corta: sombra inferior <= 10% del rango total
        let short_lower_shadow = lower_shadow <= range * 0.1;

        // 4. Trend: Ocurre tras una subida (apertura o máximo sobre cierre anterior)
        let is_uptrend = o > c_prev || h > c_prev;

        if is_small_body && long_upper_shadow && short_lower_shadow && is_uptrend {
            result[i] = PATTERN_BEARISH;
        }
    }

    df.with_column(Series::new(output_name.into(), result).into());
}

/// CDLSHORTLINE - Short Line Candle
///
/// El Short Line Candle es una vela con un cuerpo pequeño y sombras cortas.
/// Indica una falta de impulso o consolidación.
///
/// # Parámetros
/// * `df`: DataFrame de Polars con columnas "open", "high", "low", "close".
/// * `output_col`: Nombre opcional para la columna de salida (default: "cdlshortline").
///
/// # Retorno
/// * DataFrame con una nueva columna con los valores 100 (alcista), -100 (bajista) o 0.
pub fn cdlshortline(df: &mut DataFrame, output_col: Option<&str>) {
    let output_name = output_col.unwrap_or("cdlshortline");
    let (open_s, high_s, low_s, close_s) = get_ohlc(&df).unwrap();
    let open = open_s.f64().unwrap();
    let high = high_s.f64().unwrap();
    let low = low_s.f64().unwrap();
    let close = close_s.f64().unwrap();

    let len = open.len();
    let mut result: Vec<i32> = vec![0; len];

    for i in 0..len {
        let o = open.get(i).unwrap();
        let h = high.get(i).unwrap();
        let l = low.get(i).unwrap();
        let c = close.get(i).unwrap();

        let body = (o - c).abs();
        let range = h - l;

        if range == 0.0 {
            continue;
        }

        // 1. Cuerpo pequeño: entre 20% y 50% del rango total
        // (Si es muy pequeño suele ser Doji o Spinning Top)
        let is_short_body = body > range * 0.1 && body <= range * 0.5;

        // 2. Sombras cortas: ambas sombras deben ser pequeñas
        let upper_shadow = h - o.max(c);
        let lower_shadow = o.min(c) - l;
        let short_shadows = upper_shadow <= range * 0.3 && lower_shadow <= range * 0.3;

        if is_short_body && short_shadows {
            result[i] = if c > o {
                PATTERN_BULLISH
            } else {
                PATTERN_BEARISH
            };
        }
    }

    df.with_column(Series::new(output_name.into(), result).into());
}

/// CDLSPINNINGTOP - Spinning Top
///
/// El Spinning Top (Peonza) es una vela con un cuerpo pequeño y sombras
/// superior e inferior largas que superan el tamaño del cuerpo.
/// Indica indecisión en el mercado.
///
/// # Parámetros
/// * `df`: DataFrame de Polars con columnas "open", "high", "low", "close".
/// * `output_col`: Nombre opcional para la columna de salida (default: "cdlspinningtop").
///
/// # Retorno
/// * DataFrame con una nueva columna con los valores 100 (alcista), -100 (bajista) o 0.
pub fn cdlspinningtop(df: &mut DataFrame, output_col: Option<&str>) {
    let output_name = output_col.unwrap_or("cdlspinningtop");
    let (open_s, high_s, low_s, close_s) = get_ohlc(&df).unwrap();
    let open = open_s.f64().unwrap();
    let high = high_s.f64().unwrap();
    let low = low_s.f64().unwrap();
    let close = close_s.f64().unwrap();

    let len = open.len();
    let mut result: Vec<i32> = vec![0; len];

    for i in 0..len {
        let o = open.get(i).unwrap();
        let h = high.get(i).unwrap();
        let l = low.get(i).unwrap();
        let c = close.get(i).unwrap();

        let body = (o - c).abs();
        let range = h - l;

        if range == 0.0 {
            continue;
        }

        let upper_shadow = h - o.max(c);
        let lower_shadow = o.min(c) - l;

        // 1. Cuerpo pequeño: cuerpo <= 33% del rango total
        let is_small_body = body <= range * 0.33 && body > range * 0.05; // > 0.05 para excluir dojis puros

        // 2. Sombras largas: ambas sombras deben ser mayores que el cuerpo
        let long_shadows = upper_shadow > body && lower_shadow > body;

        if is_small_body && long_shadows {
            result[i] = if c > o {
                PATTERN_BULLISH
            } else {
                PATTERN_BEARISH
            };
        }
    }

    df.with_column(Series::new(output_name.into(), result).into());
}

/// CDLSTALLEDPATTERN - Stalled Pattern (Deliberation)
///
/// El patrón Stalled (Estancamiento) o Deliberation es un patrón de reversión bajista
/// de tres velas que ocurre en una tendencia alcista.
/// 1. Tres velas blancas.
/// 2. La primera y segunda vela son largas y con cierres progresivamente más altos.
/// 3. La tercera vela es pequeña, indicando que el impulso alcista se está agotando.
///
/// # Parámetros
/// * `df`: DataFrame de Polars con columnas "open", "high", "low", "close".
/// * `output_col`: Nombre opcional para la columna de salida (default: "cdlstalledpattern").
///
/// # Retorno
/// * DataFrame con una nueva columna con los valores 0 o -100 (bajista).
pub fn cdlstalledpattern(df: &mut DataFrame, output_col: Option<&str>) {
    let output_name = output_col.unwrap_or("cdlstalledpattern");
    let (open_s, high_s, low_s, close_s) = get_ohlc(&df).unwrap();
    let open = open_s.f64().unwrap();
    let high = high_s.f64().unwrap();
    let low = low_s.f64().unwrap();
    let close = close_s.f64().unwrap();

    let len = open.len();
    let mut result: Vec<i32> = vec![0; len];

    if len < 3 {
        df.with_column(Series::new(output_name.into(), &result).into());
    }

    for i in 2..len {
        let o0 = open.get(i - 2).unwrap();
        let c0 = close.get(i - 2).unwrap();
        let h0 = high.get(i - 2).unwrap();
        let l0 = low.get(i - 2).unwrap();

        let o1 = open.get(i - 1).unwrap();
        let c1 = close.get(i - 1).unwrap();
        let _ = high.get(i - 1).unwrap();

        let o2 = open.get(i).unwrap();
        let c2 = close.get(i).unwrap();

        let body0 = (o0 - c0).abs();
        let body1 = (o1 - c1).abs();
        let body2 = (o2 - c2).abs();
        let range0 = h0 - l0;

        // 1. Tres velas blancas
        let all_white =
            candle_color(o0, c0) == 1 && candle_color(o1, c1) == 1 && candle_color(o2, c2) == 1;

        if !all_white {
            continue;
        }

        // 2. Cierres progresivamente más altos
        let ascending = c1 > c0 && c2 > c1;

        // 3. Vela 1 y 2 son "largas" (cuerpo > 60% del rango de v0)
        let long_v0v1 = body0 > range0 * 0.6 && body1 > range0 * 0.6;

        // 4. Vela 3 es pequeña respecto a la vela 2
        let stalled = body2 < body1 * 0.5;

        // 5. Vela 2 abre cerca del cierre de Vela 1
        let o1_near_c0 = (o1 - c0).abs() <= range0 * 0.2;

        if ascending && long_v0v1 && stalled && o1_near_c0 {
            result[i] = PATTERN_BEARISH;
        }
    }

    df.with_column(Series::new(output_name.into(), result).into());
}

/// CDLSTICKSANDWICH - Stick Sandwich
///
/// El Stick Sandwich es un patrón de reversión alcista de tres velas.
/// 1. Una vela negra.
/// 2. Una vela blanca que abre por encima del cierre anterior.
/// 3. Una vela negra que cierra aproximadamente al mismo nivel que la primera vela negra.
///
/// # Parámetros
/// * `df`: DataFrame de Polars con columnas "open", "high", "low", "close".
/// * `output_col`: Nombre opcional para la columna de salida (default: "cdlsticksandwich").
///
/// # Retorno
/// * DataFrame con una nueva columna con los valores 0 o 100 (alcista).
pub fn cdlsticksandwich(df: &mut DataFrame, output_col: Option<&str>) {
    let output_name = output_col.unwrap_or("cdlsticksandwich");
    let (open_s, high_s, low_s, close_s) = get_ohlc(&df).unwrap();
    let open = open_s.f64().unwrap();
    let high = high_s.f64().unwrap();
    let low = low_s.f64().unwrap();
    let close = close_s.f64().unwrap();

    let len = open.len();
    let mut result: Vec<i32> = vec![0; len];

    if len < 3 {
        df.with_column(Series::new(output_name.into(), &result).into());
    }

    for i in 2..len {
        let o0 = open.get(i - 2).unwrap();
        let c0 = close.get(i - 2).unwrap();
        let h0 = high.get(i - 2).unwrap();
        let l0 = low.get(i - 2).unwrap();

        let o1 = open.get(i - 1).unwrap();
        let c1 = close.get(i - 1).unwrap();

        let o2 = open.get(i).unwrap();
        let c2 = close.get(i).unwrap();

        // 1. Vela 0 negra, Vela 1 blanca, Vela 2 negra
        let pattern_colors =
            candle_color(o0, c0) == -1 && candle_color(o1, c1) == 1 && candle_color(o2, c2) == -1;

        if !pattern_colors {
            continue;
        }

        // 2. Cierre de la Vela 2 es igual al Cierre de la Vela 0
        let range0 = h0 - l0;
        let tolerance = if range0 > 0.0 { range0 * 0.05 } else { 0.0001 };
        let equal_close = (c2 - c0).abs() <= tolerance;

        // 3. La vela blanca (Vela 1) debe tener un cierre mayor que los cierres negros
        let white_higher = c1 > c0 && c1 > c2;

        if equal_close && white_higher {
            result[i] = PATTERN_BULLISH;
        }
    }

    df.with_column(Series::new(output_name.into(), result).into());
}

/// CDLTAKURI - Takuri (Dragonfly Doji with very long lower shadow)
///
/// El Takuri es un patrón de reversión alcista de una sola vela.
/// Se caracteriza por ser un Doji con una sombra inferior extremadamente larga
/// y prácticamente sin sombra superior.
///
/// # Parámetros
/// * `df`: DataFrame de Polars con columnas "open", "high", "low", "close".
/// * `output_col`: Nombre opcional para la columna de salida (default: "cdltakuri").
///
/// # Retorno
/// * DataFrame con una nueva columna con los valores 0 o 100 (alcista).
pub fn cdltakuri(df: &mut DataFrame, output_col: Option<&str>) {
    let output_name = output_col.unwrap_or("cdltakuri");
    let (open_s, high_s, low_s, close_s) = get_ohlc(&df).unwrap();
    let open = open_s.f64().unwrap();
    let high = high_s.f64().unwrap();
    let low = low_s.f64().unwrap();
    let close = close_s.f64().unwrap();

    let len = open.len();
    let mut result: Vec<i32> = vec![0; len];

    for i in 0..len {
        let o = open.get(i).unwrap();
        let h = high.get(i).unwrap();
        let l = low.get(i).unwrap();
        let c = close.get(i).unwrap();

        let body = (o - c).abs();
        let range = h - l;

        if range == 0.0 {
            continue;
        }

        let upper_shadow = h - o.max(c);
        let lower_shadow = o.min(c) - l;

        // 1. Condición de Doji (cuerpo muy pequeño)
        let is_doji = body <= range * 0.1;

        // 2. Sombra inferior muy larga (al menos 3 veces el cuerpo)
        let very_long_lower = lower_shadow >= body * 3.0;

        // 3. Sombra superior muy corta (máximo 10% del rango)
        let very_short_upper = upper_shadow <= range * 0.1;

        if is_doji && very_long_lower && very_short_upper {
            result[i] = PATTERN_BULLISH;
        }
    }

    df.with_column(Series::new(output_name.into(), result).into());
}

/// CDLTASUKIGAP - Tasuki Gap
///
/// El Tasuki Gap es un patrón de continuación de tres velas.
///
/// # Upside Tasuki Gap (Alcista):
/// 1. Dos velas blancas con un gap alcista entre ellas.
/// 2. Una tercera vela negra que abre dentro del cuerpo de la segunda vela
///    y cierra dentro del gap, sin llegar a cerrarlo completamente.
///
/// # Downside Tasuki Gap (Bajista):
/// 1. Dos velas negras con un gap bajista entre ellas.
/// 2. Una tercera vela blanca que abre dentro del cuerpo de la segunda vela
///    y cierra dentro del gap, sin llegar a cerrarlo completamente.
///
/// # Parámetros
/// * `df`: DataFrame de Polars con columnas "open", "high", "low", "close".
/// * `output_col`: Nombre opcional para la columna de salida (default: "cdltasukigap").
///
/// # Retorno
/// * DataFrame con una nueva columna con los valores 100 (alcista), -100 (bajista) o 0.
pub fn cdltasukigap(df: &mut DataFrame, output_col: Option<&str>) {
    let output_name = output_col.unwrap_or("cdltasukigap");
    let (open_s, high_s, low_s, close_s) = get_ohlc(&df).unwrap();
    let open = open_s.f64().unwrap();
    let _ = high_s.f64().unwrap();
    let _ = low_s.f64().unwrap();
    let close = close_s.f64().unwrap();

    let len = open.len();
    let mut result: Vec<i32> = vec![0; len];

    if len < 3 {
        df.with_column(Series::new(output_name.into(), &result).into());
    }

    for i in 2..len {
        let o0 = open.get(i - 2).unwrap();
        let c0 = close.get(i - 2).unwrap();
        let o1 = open.get(i - 1).unwrap();
        let c1 = close.get(i - 1).unwrap();
        let o2 = open.get(i).unwrap();
        let c2 = close.get(i).unwrap();

        // --- UPSIDE TASUKI GAP ---
        let is_white0 = candle_color(o0, c0) == 1;
        let is_white1 = candle_color(o1, c1) == 1;
        let gap_up = c0 < o1;
        let is_black2 = candle_color(o2, c2) == -1;
        let open_within1 = o2 > o1 && o2 < c1;
        let close_in_gap = c2 < o1 && c2 > c0;

        if is_white0 && is_white1 && gap_up && is_black2 && open_within1 && close_in_gap {
            result[i] = PATTERN_BULLISH;
            continue;
        }

        // --- DOWNSIDE TASUKI GAP ---
        let is_black0 = candle_color(o0, c0) == -1;
        let is_black1 = candle_color(o1, c1) == -1;
        let gap_down = c0 > o1;
        let is_white2 = candle_color(o2, c2) == 1;
        let open_within1_down = o2 < o1 && o2 > c1;
        let close_in_gap_down = c2 > o1 && c2 < c0;

        if is_black0 && is_black1 && gap_down && is_white2 && open_within1_down && close_in_gap_down
        {
            result[i] = PATTERN_BEARISH;
        }
    }

    df.with_column(Series::new(output_name.into(), result).into());
}

/// CDLTHRUSTING - Thrusting Pattern
///
/// El patrón Thrusting es un patrón de continuación bajista de dos velas.
/// Consiste en una vela negra larga seguida de una vela blanca que abre por debajo
/// del mínimo anterior y cierra dentro del cuerpo de la primera vela, pero
/// por debajo de su punto medio (a diferencia del Piercing Pattern).
///
/// # Parámetros
/// * `df`: DataFrame de Polars con columnas "open", "high", "low", "close".
/// * `output_col`: Nombre opcional para la columna de salida (default: "cdlthrusting").
///
/// # Retorno
/// * DataFrame con una nueva columna con los valores 0 o -100 (bajista).
pub fn cdlthrusting(df: &mut DataFrame, output_col: Option<&str>) {
    let output_name = output_col.unwrap_or("cdlthrusting");
    let (open_s, high_s, low_s, close_s) = get_ohlc(&df).unwrap();
    let open = open_s.f64().unwrap();
    let high = high_s.f64().unwrap();
    let low = low_s.f64().unwrap();
    let close = close_s.f64().unwrap();

    let len = open.len();
    let mut result: Vec<i32> = vec![0; len];

    if len < 2 {
        df.with_column(Series::new(output_name.into(), &result).into());
    }

    for i in 1..len {
        let o0 = open.get(i - 1).unwrap();
        let h0 = high.get(i - 1).unwrap();
        let l0 = low.get(i - 1).unwrap();
        let c0 = close.get(i - 1).unwrap();

        let o1 = open.get(i).unwrap();
        let c1 = close.get(i).unwrap();

        let body0 = (o0 - c0).abs();
        let range0 = h0 - l0;

        // 1. Vela 0: Negra larga
        let is_black0 = candle_color(o0, c0) == -1;
        let is_long0 = if range0 > 0.0 {
            body0 > range0 * 0.6
        } else {
            false
        };

        // 2. Vela 1: Blanca que abre bajo el mínimo de la vela 0
        let is_white1 = candle_color(o1, c1) == 1;
        let open_below = o1 < l0;

        // 3. Cierre dentro del cuerpo pero BAJO el punto medio
        let mid_point = (o0 + c0) / 2.0;
        let closes_into_body = c1 > c0 && c1 < mid_point;

        if is_black0 && is_long0 && is_white1 && open_below && closes_into_body {
            result[i] = PATTERN_BEARISH;
        }
    }

    df.with_column(Series::new(output_name.into(), result).into());
}

/// CDLTRISTAR - Tristar Pattern
///
/// El patrón Tristar es un patrón de reversión de tres velas compuesto por tres Dojis consecutivos.
/// Es un patrón muy raro pero significativo.
///
/// # Bullish Tristar (Alcista):
/// 1. Tres Dojis consecutivos.
/// 2. El segundo Doji tiene un gap bajista respecto al primero.
///
/// # Bearish Tristar (Bajista):
/// 1. Tres Dojis consecutivos.
/// 2. El segundo Doji tiene un gap alcista respecto al primero.
///
/// # Parámetros
/// * `df`: DataFrame de Polars con columnas "open", "high", "low", "close".
/// * `output_col`: Nombre opcional para la columna de salida (default: "cdltristar").
///
/// # Retorno
/// * DataFrame con una nueva columna con los valores 100 (alcista), -100 (bajista) o 0.
pub fn cdltristar(df: &mut DataFrame, output_col: Option<&str>) {
    let output_name = output_col.unwrap_or("cdltristar");
    let (open_s, high_s, low_s, close_s) = get_ohlc(&df).unwrap();
    let open = open_s.f64().unwrap();
    let high = high_s.f64().unwrap();
    let low = low_s.f64().unwrap();
    let close = close_s.f64().unwrap();

    let len = open.len();
    let mut result: Vec<i32> = vec![0; len];

    if len < 3 {
        df.with_column(Series::new(output_name.into(), &result).into());
    }

    for i in 2..len {
        let o0 = open.get(i - 2).unwrap();
        let h0 = high.get(i - 2).unwrap();
        let l0 = low.get(i - 2).unwrap();
        let c0 = close.get(i - 2).unwrap();

        let o1 = open.get(i - 1).unwrap();
        let h1 = high.get(i - 1).unwrap();
        let l1 = low.get(i - 1).unwrap();
        let c1 = close.get(i - 1).unwrap();

        let o2 = open.get(i).unwrap();
        let h2 = high.get(i).unwrap();
        let l2 = low.get(i).unwrap();
        let c2 = close.get(i).unwrap();

        let body0 = (o0 - c0).abs();
        let range0 = h0 - l0;
        let body1 = (o1 - c1).abs();
        let range1 = h1 - l1;
        let body2 = (o2 - c2).abs();
        let range2 = h2 - l2;

        if range0 == 0.0 || range1 == 0.0 || range2 == 0.0 {
            continue;
        }

        // Condición de Doji para las tres velas
        let is_doji0 = body0 <= range0 * 0.1;
        let is_doji1 = body1 <= range1 * 0.1;
        let is_doji2 = body2 <= range2 * 0.1;

        if is_doji0 && is_doji1 && is_doji2 {
            // --- BULLISH TRISTAR ---
            // Gap bajista del segundo Doji
            if o1 < o0.min(c0) && o1 < o2.min(c2) {
                result[i] = PATTERN_BULLISH;
                continue;
            }

            // --- BEARISH TRISTAR ---
            // Gap alcista del segundo Doji
            if o1 > o0.max(c0) && o1 > o2.max(c2) {
                result[i] = PATTERN_BEARISH;
            }
        }
    }

    df.with_column(Series::new(output_name.into(), result).into());
}

/// CDLUNIQUE3RIVER - Unique 3 River
///
/// El Unique 3 River es un patrón de reversión alcista de tres velas.
/// 1. Una vela negra larga.
/// 2. Una vela negra con un cuerpo tipo harami pero con un nuevo mínimo.
/// 3. Una vela blanca pequeña que está por debajo del cierre de la segunda vela.
///
/// # Parámetros
/// * `df`: DataFrame de Polars con columnas "open", "high", "low", "close".
/// * `output_col`: Nombre opcional para la columna de salida (default: "cdlunique3river").
///
/// # Retorno
/// * DataFrame con una nueva columna con los valores 0 o 100 (alcista).
pub fn cdlunique3river(df: &mut DataFrame, output_col: Option<&str>) {
    let output_name = output_col.unwrap_or("cdlunique3river");
    let (open_s, high_s, low_s, close_s) = get_ohlc(&df).unwrap();
    let open = open_s.f64().unwrap();
    let high = high_s.f64().unwrap();
    let low = low_s.f64().unwrap();
    let close = close_s.f64().unwrap();

    let len = open.len();
    let mut result: Vec<i32> = vec![0; len];

    if len < 3 {
        df.with_column(Series::new(output_name.into(), &result).into());
    }

    for i in 2..len {
        let o0 = open.get(i - 2).unwrap();
        let h0 = high.get(i - 2).unwrap();
        let l0 = low.get(i - 2).unwrap();
        let c0 = close.get(i - 2).unwrap();

        let o1 = open.get(i - 1).unwrap();
        let l1 = low.get(i - 1).unwrap();
        let c1 = close.get(i - 1).unwrap();

        let o2 = open.get(i).unwrap();
        let c2 = close.get(i).unwrap();

        let body0 = (o0 - c0).abs();
        let range0 = h0 - l0;

        // 1. Vela 0: Negra larga
        let is_black0 = candle_color(o0, c0) == -1;
        let is_long0 = if range0 > 0.0 {
            body0 > range0 * 0.6
        } else {
            false
        };

        if !is_black0 || !is_long0 {
            continue;
        }

        // 2. Vela 1: Negra con nuevo mínimo pero cuerpo dentro del anterior (tipo harami)
        let is_black1 = candle_color(o1, c1) == -1;
        let body_inside0 = o1 < o0 && o1 > c0 && c1 < o0 && c1 > c0;
        let new_low1 = l1 < l0;

        // 3. Vela 2: Blanca pequeña y por debajo del cierre anterior (o dentro del rango inferior)
        let is_white2 = candle_color(o2, c2) == 1;
        let is_small2 = (o2 - c2).abs() < body0 * 0.3;
        let below_c1 = c2 < c1; // Simplificación habitual

        if is_black1 && body_inside0 && new_low1 && is_white2 && is_small2 && below_c1 {
            result[i] = PATTERN_BULLISH;
        }
    }

    df.with_column(Series::new(output_name.into(), result).into());
}

/// CDLUPSIDEGAP2CROWS - Upside Gap Two Crows
///
/// El Upside Gap Two Crows es un patrón de reversión bajista de tres velas
/// que ocurre en una tendencia alcista.
/// 1. Una vela blanca larga.
/// 2. Una vela negra pequeña que abre con gap alcista respecto al cierre anterior.
/// 3. Una segunda vela negra que envuelve el cuerpo de la vela anterior
///    pero que aún cierra por encima del cierre de la primera vela blanca.
///
/// # Parámetros
/// * `df`: DataFrame de Polars con columnas "open", "high", "low", "close".
/// * `output_col`: Nombre opcional para la columna de salida (default: "cdlupsidegap2crows").
///
/// # Retorno
/// * DataFrame con una nueva columna con los valores 0 o -100 (bajista).
pub fn cdlupsidegap2crows(df: &mut DataFrame, output_col: Option<&str>) {
    let output_name = output_col.unwrap_or("cdlupsidegap2crows");
    let (open_s, high_s, low_s, close_s) = get_ohlc(&df).unwrap();
    let open = open_s.f64().unwrap();
    let high = high_s.f64().unwrap();
    let low = low_s.f64().unwrap();
    let close = close_s.f64().unwrap();

    let len = open.len();
    let mut result: Vec<i32> = vec![0; len];

    if len < 3 {
        df.with_column(Series::new(output_name.into(), &result).into());
    }

    for i in 2..len {
        let o0 = open.get(i - 2).unwrap();
        let h0 = high.get(i - 2).unwrap();
        let l0 = low.get(i - 2).unwrap();
        let c0 = close.get(i - 2).unwrap();

        let o1 = open.get(i - 1).unwrap();
        let c1 = close.get(i - 1).unwrap();

        let o2 = open.get(i).unwrap();
        let c2 = close.get(i).unwrap();

        let body0 = (o0 - c0).abs();
        let range0 = h0 - l0;

        // 1. Vela 0: Blanca larga
        let is_white0 = candle_color(o0, c0) == 1;
        let is_long0 = if range0 > 0.0 {
            body0 > range0 * 0.6
        } else {
            false
        };

        if !is_white0 || !is_long0 {
            continue;
        }

        // 2. Vela 1: Negra con gap alcista
        let is_black1 = candle_color(o1, c1) == -1;
        let gap_up1 = c1.min(o1) > c0;

        // 3. Vela 2: Negra que envuelve a Vela 1 pero cierra sobre c0
        let is_black2 = candle_color(o2, c2) == -1;
        let engulfs1 = o2 > o1 && c2 < c1;
        let above_c0 = c2 > c0;

        if is_black1 && gap_up1 && is_black2 && engulfs1 && above_c0 {
            result[i] = PATTERN_BEARISH;
        }
    }

    df.with_column(Series::new(output_name.into(), result).into());
}

/// CDLXSIDEGAP3METHODS - Upside/Downside Gap Three Methods
///
/// El patrón Upside/Downside Gap Three Methods es un patrón de continuación de tres velas.
/// Es similar al Tasuki Gap, pero la tercera vela cierra completamente el gap entre la primera y la segunda.
///
/// # Upside Gap Three Methods (Alcista):
/// 1. Dos velas blancas con un gap alcista entre ellas.
/// 2. Una tercera vela negra que abre dentro del cuerpo de la segunda vela y cierra dentro del cuerpo de la primera, cerrando el gap.
///
/// # Downside Gap Three Methods (Bajista):
/// 1. Dos velas negras con un gap bajista entre ellas.
/// 2. Una tercera vela blanca que abre dentro del cuerpo de la segunda vela y cierra dentro del cuerpo de la primera, cerrando el gap.
///
/// # Parámetros
/// * `df`: DataFrame de Polars con columnas "open", "high", "low", "close".
/// * `output_col`: Nombre opcional para la columna de salida (default: "cdlxsidegap3methods").
///
/// # Retorno
/// * DataFrame con una nueva columna con los valores 100 (alcista), -100 (bajista) o 0.
pub fn cdlxsidegap3methods(df: &mut DataFrame, output_col: Option<&str>) {
    let output_name = output_col.unwrap_or("cdlxsidegap3methods");
    let (open_s, high_s, low_s, close_s) = get_ohlc(&df).unwrap();
    let open = open_s.f64().unwrap();
    let _ = high_s.f64().unwrap();
    let _ = low_s.f64().unwrap();
    let close = close_s.f64().unwrap();

    let len = open.len();
    let mut result: Vec<i32> = vec![0; len];

    if len < 3 {
        df.with_column(Series::new(output_name.into(), &result).into());
    }

    for i in 2..len {
        let o0 = open.get(i - 2).unwrap();
        let c0 = close.get(i - 2).unwrap();
        let o1 = open.get(i - 1).unwrap();
        let c1 = close.get(i - 1).unwrap();
        let o2 = open.get(i).unwrap();
        let c2 = close.get(i).unwrap();

        // --- UPSIDE GAP THREE METHODS ---
        let is_white0 = candle_color(o0, c0) == 1;
        let is_white1 = candle_color(o1, c1) == 1;
        let gap_up = c0 < o1;
        let is_black2 = candle_color(o2, c2) == -1;

        // La vela 3 abre dentro del cuerpo de la vela 2 y cierra dentro del cuerpo de la vela 1
        let open_within1 = o2 > o1 && o2 < c1;
        let close_within0 = c2 < o0.max(c0) && c2 > o0.min(c0);

        if is_white0 && is_white1 && gap_up && is_black2 && open_within1 && close_within0 {
            result[i] = PATTERN_BULLISH;
            continue;
        }

        // --- DOWNSIDE GAP THREE METHODS ---
        let is_black0 = candle_color(o0, c0) == -1;
        let is_black1 = candle_color(o1, c1) == -1;
        let gap_down = c0 > o1;
        let is_white2 = candle_color(o2, c2) == 1;

        let open_within1_down = o2 < o1 && o2 > c1;
        let close_within0_down = c2 > o0.min(c0) && c2 < o0.max(c0);

        if is_black0
            && is_black1
            && gap_down
            && is_white2
            && open_within1_down
            && close_within0_down
        {
            result[i] = PATTERN_BEARISH;
        }
    }

    df.with_column(Series::new(output_name.into(), result).into());
}

#[cfg(test)]
mod tests {
    use super::*;

    fn load_data() -> PolarsResult<DataFrame> {
        let df = CsvReadOptions::default()
            .try_into_reader_with_file_path(Some("download/test.csv".into()))
            .unwrap()
            .finish()
            .unwrap();
        Ok(df)
    }

    fn save_data(df_result: &DataFrame, path: &str) -> PolarsResult<()> {
        let mut df: DataFrame = df_result.clone();
        let mut file = std::fs::File::create(path).unwrap();
        CsvWriter::new(&mut file).finish(&mut df).unwrap();
        Ok(())
    }

    #[test]
    fn test_cdl2crows() {
        match load_data() {
            Ok(mut df) => {
                cdl2crows(&mut df, Some("cdl2crows"));
                save_data(&df, "download/test_cdl2crows.csv").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        };
    }

    #[test]
    fn test_cdl3blackcrows() {
        match load_data() {
            Ok(mut df) => {
                cdl3blackcrows(&mut df, Some("cdl3blackcrows"));
                save_data(&df, "download/test_cdl3blackcrows.csv").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        };
    }

    #[test]
    fn test_cdl3inside() {
        match load_data() {
            Ok(mut df) => {
                cdl3inside(&mut df, Some("cdl3inside"));
                save_data(&df, "download/test_cdl3inside.csv").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        };
    }

    #[test]
    fn test_cdl3linestrike() {
        match load_data() {
            Ok(mut df) => {
                cdl3linestrike(&mut df, Some("cdl3linestrike"));
                save_data(&df, "download/test_cdl3linestrike.csv").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        };
    }

    #[test]
    fn test_cdl3outside() {
        match load_data() {
            Ok(mut df) => {
                cdl3outside(&mut df, Some("cdl3outside"));
                save_data(&df, "download/test_cdl3outside.csv").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        };
    }

    #[test]
    fn test_cdl3starsinsouth() {
        match load_data() {
            Ok(mut df) => {
                cdl3starsinsouth(&mut df, Some("cdl3starsinsouth"));
                save_data(&df, "download/test_cdl3starsinsouth.csv").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        };
    }

    #[test]
    fn test_cdl3whitesoldiers() {
        match load_data() {
            Ok(mut df) => {
                cdl3whitesoldiers(&mut df, Some("cdl3whitesoldiers"));
                save_data(&df, "download/test_cdl3whitesoldiers.csv").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        };
    }

    #[test]
    fn test_cdlabandonedbaby() {
        match load_data() {
            Ok(mut df) => {
                cdlabandonedbaby(&mut df, Some("cdlabandonedbaby"));
                save_data(&df, "download/test_cdlabandonedbaby.csv").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        };
    }

    #[test]
    fn test_cdladvanceblock() {
        match load_data() {
            Ok(mut df) => {
                cdladvanceblock(&mut df, Some("cdladvanceblock"));
                save_data(&df, "download/test_cdladvanceblock.csv").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        };
    }

    #[test]
    fn test_cdlbelthold() {
        match load_data() {
            Ok(mut df) => {
                cdlbelthold(&mut df, Some("cdlbelthold"));
                save_data(&df, "download/test_cdlbelthold.csv").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        };
    }

    #[test]
    fn test_cdlbreakaway() {
        match load_data() {
            Ok(mut df) => {
                cdlbreakaway(&mut df, Some("cdlbreakaway"));
                save_data(&df, "download/test_cdlbreakaway.csv").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        };
    }

    #[test]
    fn test_cdlclosingmarubuzo() {
        match load_data() {
            Ok(mut df) => {
                cdlclosingmarubuzo(&mut df, Some("cdlclosingmarubuzo"));
                save_data(&df, "download/test_cdlclosingmarubuzo.csv").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        };
    }

    #[test]
    fn test_cdlconcealbabyswall() {
        match load_data() {
            Ok(mut df) => {
                cdlconcealbabyswall(&mut df, Some("cdlconcealbabyswall"));
                save_data(&df, "download/test_cdlconcealbabyswall.csv").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        };
    }

    #[test]
    fn test_cdlcounterattack() {
        match load_data() {
            Ok(mut df) => {
                cdlcounterattack(&mut df, Some("cdlcounterattack"));
                save_data(&df, "download/test_cdlcounterattack.csv").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        };
    }

    #[test]
    fn test_cdldarkcloudcover() {
        match load_data() {
            Ok(mut df) => {
                cdldarkcloudcover(&mut df, Some("cdldarkcloudcover"));
                save_data(&df, "download/test_cdldarkcloudcover.csv").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        };
    }

    #[test]
    fn test_cdldoji() {
        match load_data() {
            Ok(mut df) => {
                cdldoji(&mut df, Some("cdldoji"));
                save_data(&df, "download/test_cdldoji.csv").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        };
    }

    #[test]
    fn test_cdldojistar() {
        match load_data() {
            Ok(mut df) => {
                cdldojistar(&mut df, Some("cdldojistar"));
                save_data(&df, "download/test_cdldojistar.csv").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        };
    }

    #[test]
    fn test_cdldragonflydoji() {
        match load_data() {
            Ok(mut df) => {
                cdldragonflydoji(&mut df, Some("cdldragonflydoji"));
                save_data(&df, "download/test_cdldragonflydoji.csv").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        };
    }

    #[test]
    fn test_cdlengulfing() {
        match load_data() {
            Ok(mut df) => {
                cdlengulfing(&mut df, Some("cdlengulfing"));
                save_data(&df, "download/test_cdlengulfing.csv").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        };
    }

    #[test]
    fn test_cdleveningdojistar() {
        match load_data() {
            Ok(mut df) => {
                cdleveningdojistar(&mut df, Some("cdleveningdojistar"));
                save_data(&df, "download/test_cdleveningdojistar.csv").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        };
    }

    #[test]
    fn test_cdlmorningdojistar() {
        match load_data() {
            Ok(mut df) => {
                cdlmorningdojistar(&mut df, None, Some("cdlmorningdojistar"));
                save_data(&df, "download/test_cdlmorningdojistar.csv").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        };
    }

    #[test]
    fn test_cdlmorningstar() {
        match load_data() {
            Ok(mut df) => {
                cdlmorningstar(&mut df, None, Some("cdlmorningstar"));
                save_data(&df, "download/test_cdlmorningstar.csv").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        };
    }

    #[test]
    fn test_cdlonneck() {
        match load_data() {
            Ok(mut df) => {
                cdlonneck(&mut df, Some("cdlonneck"));
                save_data(&df, "download/test_cdlonneck.csv").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        };
    }

    #[test]
    fn test_cdlpiercing() {
        match load_data() {
            Ok(mut df) => {
                cdlpiercing(&mut df, None, Some("cdlpiercing"));
                save_data(&df, "download/test_cdlpiercing.csv").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        };
    }

    #[test]
    fn test_cdlrickshawman() {
        match load_data() {
            Ok(mut df) => {
                cdlrickshawman(&mut df, Some("cdlrickshawman"));
                save_data(&df, "download/test_cdlrickshawman.csv").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        };
    }

    #[test]
    fn test_cdlrisefall3methods() {
        match load_data() {
            Ok(mut df) => {
                cdlrisefall3methods(&mut df, Some("cdlrisefall3methods"));
                save_data(&df, "download/test_cdlrisefall3methods.csv").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        };
    }

    #[test]
    fn test_cdlseparatinglines() {
        match load_data() {
            Ok(mut df) => {
                cdlseparatinglines(&mut df, Some("cdlseparatinglines"));
                save_data(&df, "download/test_cdlseparatinglines.csv").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        };
    }

    #[test]
    fn test_cdlshootingstar() {
        match load_data() {
            Ok(mut df) => {
                cdlshootingstar(&mut df, Some("cdlshootingstar"));
                save_data(&df, "download/test_cdlshootingstar.csv").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        };
    }

    #[test]
    fn test_cdlshortline() {
        match load_data() {
            Ok(mut df) => {
                cdlshortline(&mut df, Some("cdlshortline"));
                save_data(&df, "download/test_cdlshortline.csv").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        };
    }

    #[test]
    fn test_cdlspinningtop() {
        match load_data() {
            Ok(mut df) => {
                cdlspinningtop(&mut df, Some("cdlspinningtop"));
                save_data(&df, "download/test_cdlspinningtop.csv").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        };
    }

    #[test]
    fn test_cdlstalledpattern() {
        match load_data() {
            Ok(mut df) => {
                cdlstalledpattern(&mut df, Some("cdlstalledpattern"));
                save_data(&df, "download/test_cdlstalledpattern.csv").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        };
    }

    #[test]
    fn test_cdlsticksandwich() {
        match load_data() {
            Ok(mut df) => {
                cdlsticksandwich(&mut df, Some("cdlsticksandwich"));
                save_data(&df, "download/test_cdlsticksandwich.csv").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        };
    }

    #[test]
    fn test_cdltakuri() {
        match load_data() {
            Ok(mut df) => {
                cdltakuri(&mut df, Some("cdltakuri"));
                save_data(&df, "download/test_cdltakuri.csv").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        };
    }

    #[test]
    fn test_cdltasukigap() {
        match load_data() {
            Ok(mut df) => {
                cdltasukigap(&mut df, Some("cdltasukigap"));
                save_data(&df, "download/test_cdltasukigap.csv").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        };
    }

    #[test]
    fn test_cdlthrusting() {
        match load_data() {
            Ok(mut df) => {
                cdlthrusting(&mut df, Some("cdlthrusting"));
                save_data(&df, "download/test_cdlthrusting.csv").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        };
    }

    #[test]
    fn test_cdltristar() {
        match load_data() {
            Ok(mut df) => {
                cdltristar(&mut df, Some("cdltristar"));
                save_data(&df, "download/test_cdltristar.csv").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        };
    }

    #[test]
    fn test_cdlunique3river() {
        match load_data() {
            Ok(mut df) => {
                cdlunique3river(&mut df, Some("cdlunique3river"));
                save_data(&df, "download/test_cdlunique3river.csv").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        };
    }

    #[test]
    fn test_cdlupsidegap2crows() {
        match load_data() {
            Ok(mut df) => {
                cdlupsidegap2crows(&mut df, Some("cdlupsidegap2crows"));
                save_data(&df, "download/test_cdlupsidegap2crows.csv").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        };
    }

    #[test]
    fn test_cdlxsidegap3methods() {
        match load_data() {
            Ok(mut df) => {
                cdlxsidegap3methods(&mut df, Some("cdlxsidegap3methods"));
                save_data(&df, "download/test_cdlxsidegap3methods.csv").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        };
    }

    #[test]
    fn test_cdleveningstar() {
        match load_data() {
            Ok(mut df) => {
                cdleveningstar(&mut df, Some("cdleveningstar"));
                save_data(&df, "download/test_cdleveningstar.csv").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        };
    }

    #[test]
    fn test_cdlgapsidesidewhite() {
        match load_data() {
            Ok(mut df) => {
                cdlgapsidesidewhite(&mut df, Some("cdlgapsidesidewhite"));
                save_data(&df, "download/test_cdlgapsidesidewhite.csv").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        };
    }

    #[test]
    fn test_cdlgravestonedoji() {
        match load_data() {
            Ok(mut df) => {
                cdlgravestonedoji(&mut df, Some("cdlgravestonedoji"));
                save_data(&df, "download/test_cdlgravestonedoji.csv").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        };
    }

    #[test]
    fn test_cdlhammer() {
        match load_data() {
            Ok(mut df) => {
                cdlhammer(&mut df, Some("cdlhammer"));
                save_data(&df, "download/test_cdlhammer.csv").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        };
    }

    #[test]
    fn test_cdlhangingman() {
        match load_data() {
            Ok(mut df) => {
                cdlhangingman(&mut df, Some("cdlhangingman"));
                save_data(&df, "download/test_cdlhangingman.csv").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        };
    }

    #[test]
    fn test_cdlharami() {
        match load_data() {
            Ok(mut df) => {
                cdlharami(&mut df, Some("cdlharami"));
                save_data(&df, "download/test_cdlharami.csv").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        };
    }

    #[test]
    fn test_cdlharamicross() {
        match load_data() {
            Ok(mut df) => {
                cdlharamicross(&mut df, Some("cdlharamicross"));
                save_data(&df, "download/test_cdlharamicross.csv").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        };
    }

    #[test]
    fn test_cdlhighwave() {
        match load_data() {
            Ok(mut df) => {
                cdlhighwave(&mut df, Some("cdlhighwave"));
                save_data(&df, "download/test_cdlhighwave.csv").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        };
    }

    #[test]
    fn test_cdlhikkake() {
        match load_data() {
            Ok(mut df) => {
                cdlhikkake(&mut df, Some("cdlhikkake"));
                save_data(&df, "download/test_cdlhikkake.csv").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        };
    }

    #[test]
    fn test_cdlhikkakemod() {
        match load_data() {
            Ok(mut df) => {
                cdlhikkakemod(&mut df, Some("cdlhikkakemod"));
                save_data(&df, "download/test_cdlhikkakemod.csv").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        };
    }

    #[test]
    fn test_cdlhomingpigeon() {
        match load_data() {
            Ok(mut df) => {
                cdlhomingpigeon(&mut df, Some("cdlhomingpigeon"));
                save_data(&df, "download/test_cdlhomingpigeon.csv").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        };
    }

    #[test]
    fn test_cdlidentical3crows() {
        match load_data() {
            Ok(mut df) => {
                cdlidentical3crows(&mut df, Some("cdlidentical3crows"));
                save_data(&df, "download/test_cdlidentical3crows.csv").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        };
    }

    #[test]
    fn test_cdlinneck() {
        match load_data() {
            Ok(mut df) => {
                cdlinneck(&mut df, Some("cdlinneck"));
                save_data(&df, "download/test_cdlinneck.csv").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        };
    }

    #[test]
    fn test_cdlinvertedhammer() {
        match load_data() {
            Ok(mut df) => {
                cdlinvertedhammer(&mut df, Some("cdlinvertedhammer"));
                save_data(&df, "download/test_cdlinvertedhammer.csv").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        };
    }

    #[test]
    fn test_cdlkicking() {
        match load_data() {
            Ok(mut df) => {
                cdlkicking(&mut df, Some("cdlkicking"));
                save_data(&df, "download/test_cdlkicking.csv").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        };
    }

    #[test]
    fn test_cdlkickingbylength() {
        match load_data() {
            Ok(mut df) => {
                cdlkickingbylength(&mut df, Some("cdlkickingbylength"));
                save_data(&df, "download/test_cdlkickingbylength.csv").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        };
    }

    #[test]
    fn test_cdladderbottom() {
        match load_data() {
            Ok(mut df) => {
                cdladderbottom(&mut df, Some("cdladderbottom"));
                save_data(&df, "download/test_cdladderbottom.csv").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        };
    }

    #[test]
    fn test_cdllongleggeddoji() {
        match load_data() {
            Ok(mut df) => {
                cdllongleggeddoji(&mut df, Some("cdllongleggeddoji"));
                save_data(&df, "download/test_cdllongleggeddoji.csv").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        };
    }

    #[test]
    fn test_cdllongline() {
        match load_data() {
            Ok(mut df) => {
                cdllongline(&mut df, Some("cdllongline"));
                save_data(&df, "download/test_cdllongline.csv").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        };
    }

    #[test]
    fn test_cdlmarubozu() {
        match load_data() {
            Ok(mut df) => {
                cdlmarubozu(&mut df, Some("cdlmarubozu"));
                save_data(&df, "download/test_cdlmarubozu.csv").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        };
    }

    #[test]
    fn test_cdlmatchinglow() {
        match load_data() {
            Ok(mut df) => {
                cdlmatchinglow(&mut df, Some("cdlmatchinglow"));
                save_data(&df, "download/test_cdlmatchinglow.csv").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        };
    }

    #[test]
    fn test_cdlmathold() {
        match load_data() {
            Ok(mut df) => {
                cdlmathold(&mut df, Some("cdlmathold"));
                save_data(&df, "download/test_cdlmathold.csv").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        };
    }
}
