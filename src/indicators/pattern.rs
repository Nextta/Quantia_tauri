use polars::prelude::*;

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
pub async fn cdl2crows(df: DataFrame, output_col: Option<&str>) -> PolarsResult<DataFrame> {
    let output_name = output_col.unwrap_or("cdl2crows");
    let (open_s, _high_s, _low_s, close_s) = get_ohlc(&df)?;
    let open = open_s.f64()?;
    let close = close_s.f64()?;

    let len = open.len();
    let mut result: Vec<i32> = vec![0; len];

    if len < 3 {
        return df
            .lazy()
            .with_column(lit(Series::new(output_name.into(), result)))
            .collect();
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

    df.lazy()
        .with_column(lit(Series::new(output_name.into(), result)))
        .collect()
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
pub async fn cdl3blackcrows(df: DataFrame, output_col: Option<&str>) -> PolarsResult<DataFrame> {
    let output_name = output_col.unwrap_or("cdl3blackcrows");
    let (open_s, _high_s, _low_s, close_s) = get_ohlc(&df)?;
    let open = open_s.f64()?;
    let close = close_s.f64()?;

    let len = open.len();
    let mut result: Vec<i32> = vec![0; len];

    if len < 3 {
        return df
            .lazy()
            .with_column(lit(Series::new(output_name.into(), result)))
            .collect();
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

    df.lazy()
        .with_column(lit(Series::new(output_name.into(), result)))
        .collect()
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
pub async fn cdl3inside(df: DataFrame, output_col: Option<&str>) -> PolarsResult<DataFrame> {
    let output_name = output_col.unwrap_or("cdl3inside");
    let (open_s, _high_s, _low_s, close_s) = get_ohlc(&df)?;
    let open = open_s.f64()?;
    let close = close_s.f64()?;

    let len = open.len();
    let mut result: Vec<i32> = vec![0; len];

    if len < 3 {
        return df
            .lazy()
            .with_column(lit(Series::new(output_name.into(), result)))
            .collect();
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

    df.lazy()
        .with_column(lit(Series::new(output_name.into(), result)))
        .collect()
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
pub async fn cdl3linestrike(df: DataFrame, output_col: Option<&str>) -> PolarsResult<DataFrame> {
    let output_name = output_col.unwrap_or("cdl3linestrike");
    let (open_s, _high_s, _low_s, close_s) = get_ohlc(&df)?;
    let open = open_s.f64()?;
    let close = close_s.f64()?;

    let len = open.len();
    let mut result: Vec<i32> = vec![0; len];

    if len < 4 {
        return df
            .lazy()
            .with_column(lit(Series::new(output_name.into(), result)))
            .collect();
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

    df.lazy()
        .with_column(lit(Series::new(output_name.into(), result)))
        .collect()
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
pub async fn cdl3outside(df: DataFrame, output_col: Option<&str>) -> PolarsResult<DataFrame> {
    let output_name = output_col.unwrap_or("cdl3outside");
    let (open_s, _high_s, _low_s, close_s) = get_ohlc(&df)?;
    let open = open_s.f64()?;
    let close = close_s.f64()?;

    let len = open.len();
    let mut result: Vec<i32> = vec![0; len];

    if len < 3 {
        return df
            .lazy()
            .with_column(lit(Series::new(output_name.into(), result)))
            .collect();
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

    df.lazy()
        .with_column(lit(Series::new(output_name.into(), result)))
        .collect()
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
pub async fn cdl3starsinsouth(df: DataFrame, output_col: Option<&str>) -> PolarsResult<DataFrame> {
    let output_name = output_col.unwrap_or("cdl3starsinsouth");
    let (open_s, high_s, low_s, close_s) = get_ohlc(&df)?;
    let open = open_s.f64()?;
    let high = high_s.f64()?;
    let low = low_s.f64()?;
    let close = close_s.f64()?;

    let len = open.len();
    let mut result: Vec<i32> = vec![0; len];

    if len < 3 {
        return df
            .lazy()
            .with_column(lit(Series::new(output_name.into(), result)))
            .collect();
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

    df.lazy()
        .with_column(lit(Series::new(output_name.into(), result)))
        .collect()
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
pub async fn cdl3whitesoldiers(df: DataFrame, output_col: Option<&str>) -> PolarsResult<DataFrame> {
    let output_name = output_col.unwrap_or("cdl3whitesoldiers");
    let (open_s, high_s, _low_s, close_s) = get_ohlc(&df)?;
    let open = open_s.f64()?;
    let high = high_s.f64()?;
    let close = close_s.f64()?;

    let len = open.len();
    let mut result: Vec<i32> = vec![0; len];

    if len < 3 {
        return df
            .lazy()
            .with_column(lit(Series::new(output_name.into(), result)))
            .collect();
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

    df.lazy()
        .with_column(lit(Series::new(output_name.into(), result)))
        .collect()
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
pub async fn cdlabandonedbaby(df: DataFrame, output_col: Option<&str>) -> PolarsResult<DataFrame> {
    let output_name = output_col.unwrap_or("cdlabandonedbaby");
    let (open_s, high_s, low_s, close_s) = get_ohlc(&df)?;
    let open = open_s.f64()?;
    let high = high_s.f64()?;
    let low = low_s.f64()?;
    let close = close_s.f64()?;

    let len = open.len();
    let mut result: Vec<i32> = vec![0; len];

    if len < 3 {
        return df
            .lazy()
            .with_column(lit(Series::new(output_name.into(), result)))
            .collect();
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

    df.lazy()
        .with_column(lit(Series::new(output_name.into(), result)))
        .collect()
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
pub async fn cdladvanceblock(df: DataFrame, output_col: Option<&str>) -> PolarsResult<DataFrame> {
    let output_name = output_col.unwrap_or("cdladvanceblock");
    let (open_s, high_s, _low_s, close_s) = get_ohlc(&df)?;
    let open = open_s.f64()?;
    let high = high_s.f64()?;
    let close = close_s.f64()?;

    let len = open.len();
    let mut result: Vec<i32> = vec![0; len];

    if len < 3 {
        return df
            .lazy()
            .with_column(lit(Series::new(output_name.into(), result)))
            .collect();
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

    df.lazy()
        .with_column(lit(Series::new(output_name.into(), result)))
        .collect()
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
pub async fn cdlbelthold(df: DataFrame, output_col: Option<&str>) -> PolarsResult<DataFrame> {
    let output_name = output_col.unwrap_or("cdlbelthold");
    let (open_s, high_s, low_s, close_s) = get_ohlc(&df)?;
    let open = open_s.f64()?;
    let high = high_s.f64()?;
    let low = low_s.f64()?;
    let close = close_s.f64()?;

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

    df.lazy()
        .with_column(lit(Series::new(output_name.into(), result)))
        .collect()
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
pub async fn cdlbreakaway(df: DataFrame, output_col: Option<&str>) -> PolarsResult<DataFrame> {
    let output_name = output_col.unwrap_or("cdlbreakaway");
    let (open_s, _high_s, _low_s, close_s) = get_ohlc(&df)?;
    let open = open_s.f64()?;
    let close = close_s.f64()?;

    let len = open.len();
    let mut result: Vec<i32> = vec![0; len];

    if len < 5 {
        return df
            .lazy()
            .with_column(lit(Series::new(output_name.into(), result)))
            .collect();
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

    df.lazy()
        .with_column(lit(Series::new(output_name.into(), result)))
        .collect()
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
pub async fn cdlclosingmarubuzo(
    df: DataFrame,
    output_col: Option<&str>,
) -> PolarsResult<DataFrame> {
    let output_name = output_col.unwrap_or("cdlclosingmarubozu");
    let (open_s, high_s, low_s, close_s) = get_ohlc(&df)?;
    let open = open_s.f64()?;
    let high = high_s.f64()?;
    let low = low_s.f64()?;
    let close = close_s.f64()?;

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

    df.lazy()
        .with_column(lit(Series::new(output_name.into(), result)))
        .collect()
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
pub async fn cdlconcealbabyswall(
    df: DataFrame,
    output_col: Option<&str>,
) -> PolarsResult<DataFrame> {
    let output_name = output_col.unwrap_or("cdlconcealbabyswall");
    let (open_s, high_s, low_s, close_s) = get_ohlc(&df)?;
    let open = open_s.f64()?;
    let high = high_s.f64()?;
    let low = low_s.f64()?;
    let close = close_s.f64()?;

    let len = open.len();
    let mut result: Vec<i32> = vec![0; len];

    if len < 4 {
        return df
            .lazy()
            .with_column(lit(Series::new(output_name.into(), result)))
            .collect();
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

    df.lazy()
        .with_column(lit(Series::new(output_name.into(), result)))
        .collect()
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
pub async fn cdlcounterattack(df: DataFrame, output_col: Option<&str>) -> PolarsResult<DataFrame> {
    let output_name = output_col.unwrap_or("cdlcounterattack");
    let (open_s, _high_s, _low_s, close_s) = get_ohlc(&df)?;
    let open = open_s.f64()?;
    let close = close_s.f64()?;

    let len = open.len();
    let mut result: Vec<i32> = vec![0; len];

    if len < 2 {
        return df
            .lazy()
            .with_column(lit(Series::new(output_name.into(), result)))
            .collect();
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

    df.lazy()
        .with_column(lit(Series::new(output_name.into(), result)))
        .collect()
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
pub async fn cdldarkcloudcover(df: DataFrame, output_col: Option<&str>) -> PolarsResult<DataFrame> {
    let output_name = output_col.unwrap_or("cdldarkcloudcover");
    let (open_s, _high_s, _low_s, close_s) = get_ohlc(&df)?;
    let open = open_s.f64()?;
    let close = close_s.f64()?;

    let len = open.len();
    let mut result: Vec<i32> = vec![0; len];

    if len < 2 {
        return df
            .lazy()
            .with_column(lit(Series::new(output_name.into(), result)))
            .collect();
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

    df.lazy()
        .with_column(lit(Series::new(output_name.into(), result)))
        .collect()
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
pub async fn cdldoji(df: DataFrame, output_col: Option<&str>) -> PolarsResult<DataFrame> {
    let output_name = output_col.unwrap_or("cdldoji");
    let (open_s, high_s, low_s, close_s) = get_ohlc(&df)?;
    let open = open_s.f64()?;
    let high = high_s.f64()?;
    let low = low_s.f64()?;
    let close = close_s.f64()?;

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

    df.lazy()
        .with_column(lit(Series::new(output_name.into(), result)))
        .collect()
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
pub async fn cdldojistar(df: DataFrame, output_col: Option<&str>) -> PolarsResult<DataFrame> {
    let output_name = output_col.unwrap_or("cdldojistar");
    let (open_s, high_s, low_s, close_s) = get_ohlc(&df)?;
    let open = open_s.f64()?;
    let high = high_s.f64()?;
    let low = low_s.f64()?;
    let close = close_s.f64()?;

    let len = open.len();
    let mut result: Vec<i32> = vec![0; len];

    if len < 2 {
        return df
            .lazy()
            .with_column(lit(Series::new(output_name.into(), result)))
            .collect();
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

    df.lazy()
        .with_column(lit(Series::new(output_name.into(), result)))
        .collect()
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
pub async fn cdldragonflydoji(df: DataFrame, output_col: Option<&str>) -> PolarsResult<DataFrame> {
    let output_name = output_col.unwrap_or("cdldragonflydoji");
    let (open_s, high_s, low_s, close_s) = get_ohlc(&df)?;
    let open = open_s.f64()?;
    let high = high_s.f64()?;
    let low = low_s.f64()?;
    let close = close_s.f64()?;

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

    df.lazy()
        .with_column(lit(Series::new(output_name.into(), result)))
        .collect()
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
pub async fn cdlengulfing(df: DataFrame, output_col: Option<&str>) -> PolarsResult<DataFrame> {
    let output_name = output_col.unwrap_or("cdlengulfing");
    let (open_s, _high_s, _low_s, close_s) = get_ohlc(&df)?;
    let open = open_s.f64()?;
    let close = close_s.f64()?;

    let len = open.len();
    let mut result: Vec<i32> = vec![0; len];

    if len < 2 {
        return df
            .lazy()
            .with_column(lit(Series::new(output_name.into(), result)))
            .collect();
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

    df.lazy()
        .with_column(lit(Series::new(output_name.into(), result)))
        .collect()
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
pub async fn cdleveningdojistar(
    df: DataFrame,
    output_col: Option<&str>,
) -> PolarsResult<DataFrame> {
    let output_name = output_col.unwrap_or("cdleveningdojistar");
    let (open_s, high_s, low_s, close_s) = get_ohlc(&df)?;
    let open = open_s.f64()?;
    let high = high_s.f64()?;
    let low = low_s.f64()?;
    let close = close_s.f64()?;

    let len = open.len();
    let mut result: Vec<i32> = vec![0; len];

    if len < 3 {
        return df
            .lazy()
            .with_column(lit(Series::new(output_name.into(), result)))
            .collect();
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

    df.lazy()
        .with_column(lit(Series::new(output_name.into(), result)))
        .collect()
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
pub async fn cdleveningstar(df: DataFrame, output_col: Option<&str>) -> PolarsResult<DataFrame> {
    let output_name = output_col.unwrap_or("cdleveningstar");
    let (open_s, high_s, low_s, close_s) = get_ohlc(&df)?;
    let open = open_s.f64()?;
    let high = high_s.f64()?;
    let low = low_s.f64()?;
    let close = close_s.f64()?;

    let len = open.len();
    let mut result: Vec<i32> = vec![0; len];

    if len < 3 {
        return df
            .lazy()
            .with_column(lit(Series::new(output_name.into(), result)))
            .collect();
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

    df.lazy()
        .with_column(lit(Series::new(output_name.into(), result)))
        .collect()
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
pub async fn cdlgapsidesidewhite(
    df: DataFrame,
    output_col: Option<&str>,
) -> PolarsResult<DataFrame> {
    let output_name = output_col.unwrap_or("cdlgapsidesidewhite");
    let (open_s, _high_s, _low_s, close_s) = get_ohlc(&df)?;
    let open = open_s.f64()?;
    let close = close_s.f64()?;

    let len = open.len();
    let mut result: Vec<i32> = vec![0; len];

    if len < 3 {
        return df
            .lazy()
            .with_column(lit(Series::new(output_name.into(), result)))
            .collect();
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

    df.lazy()
        .with_column(lit(Series::new(output_name.into(), result)))
        .collect()
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
pub async fn cdlgravestonedoji(df: DataFrame, output_col: Option<&str>) -> PolarsResult<DataFrame> {
    let output_name = output_col.unwrap_or("cdlgravestonedoji");
    let (open_s, high_s, low_s, close_s) = get_ohlc(&df)?;
    let open = open_s.f64()?;
    let high = high_s.f64()?;
    let low = low_s.f64()?;
    let close = close_s.f64()?;

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

    df.lazy()
        .with_column(lit(Series::new(output_name.into(), result)))
        .collect()
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
pub async fn cdlhammer(df: DataFrame, output_col: Option<&str>) -> PolarsResult<DataFrame> {
    let output_name = output_col.unwrap_or("cdlhammer");
    let (open_s, high_s, low_s, close_s) = get_ohlc(&df)?;
    let open = open_s.f64()?;
    let high = high_s.f64()?;
    let low = low_s.f64()?;
    let close = close_s.f64()?;

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

    df.lazy()
        .with_column(lit(Series::new(output_name.into(), result)))
        .collect()
}

pub async fn cdlhangingman(df: DataFrame, _output_col: Option<&str>) -> PolarsResult<DataFrame> {
    Ok(df)
}

pub async fn cdlharami(df: DataFrame, _output_col: Option<&str>) -> PolarsResult<DataFrame> {
    Ok(df)
}

pub async fn cdlharamicross(df: DataFrame, _output_col: Option<&str>) -> PolarsResult<DataFrame> {
    Ok(df)
}

pub async fn cdlhighwave(df: DataFrame, _output_col: Option<&str>) -> PolarsResult<DataFrame> {
    Ok(df)
}

pub async fn cdlhikkake(df: DataFrame, _output_col: Option<&str>) -> PolarsResult<DataFrame> {
    Ok(df)
}

pub async fn cdlhikkakemod(df: DataFrame, _output_col: Option<&str>) -> PolarsResult<DataFrame> {
    Ok(df)
}

pub async fn cdlhomingpigeon(df: DataFrame, _output_col: Option<&str>) -> PolarsResult<DataFrame> {
    Ok(df)
}

pub async fn cdlidentical3crows(
    df: DataFrame,
    _output_col: Option<&str>,
) -> PolarsResult<DataFrame> {
    Ok(df)
}

pub async fn cdlinneck(df: DataFrame, _output_col: Option<&str>) -> PolarsResult<DataFrame> {
    Ok(df)
}

pub async fn cdlinvertedhammer(
    df: DataFrame,
    _output_col: Option<&str>,
) -> PolarsResult<DataFrame> {
    Ok(df)
}

pub async fn cdlkicking(df: DataFrame, _output_col: Option<&str>) -> PolarsResult<DataFrame> {
    Ok(df)
}

pub async fn cdlkickingbylength(
    df: DataFrame,
    _output_col: Option<&str>,
) -> PolarsResult<DataFrame> {
    Ok(df)
}

pub async fn cdladderbottom(df: DataFrame, _output_col: Option<&str>) -> PolarsResult<DataFrame> {
    Ok(df)
}

pub async fn cdllongleggeddoji(
    df: DataFrame,
    _output_col: Option<&str>,
) -> PolarsResult<DataFrame> {
    Ok(df)
}

pub async fn cdllongline(df: DataFrame, _output_col: Option<&str>) -> PolarsResult<DataFrame> {
    Ok(df)
}

pub async fn cdlmarubozu(df: DataFrame, _output_col: Option<&str>) -> PolarsResult<DataFrame> {
    Ok(df)
}

pub async fn cdlmatchinglow(df: DataFrame, _output_col: Option<&str>) -> PolarsResult<DataFrame> {
    Ok(df)
}

pub async fn cdlmathold(df: DataFrame, _output_col: Option<&str>) -> PolarsResult<DataFrame> {
    Ok(df)
}

pub async fn cdlmorningdojistar(
    df: DataFrame,
    _output_col: Option<&str>,
) -> PolarsResult<DataFrame> {
    Ok(df)
}

pub async fn cdlmorningstar(df: DataFrame, _output_col: Option<&str>) -> PolarsResult<DataFrame> {
    Ok(df)
}

pub async fn cdlonneck(df: DataFrame, _output_col: Option<&str>) -> PolarsResult<DataFrame> {
    Ok(df)
}

pub async fn cdlpiercing(df: DataFrame, _output_col: Option<&str>) -> PolarsResult<DataFrame> {
    Ok(df)
}

pub async fn cdlrickshawman(df: DataFrame, _output_col: Option<&str>) -> PolarsResult<DataFrame> {
    Ok(df)
}

pub async fn cdlrisefall3methods(
    df: DataFrame,
    _output_col: Option<&str>,
) -> PolarsResult<DataFrame> {
    Ok(df)
}

pub async fn cdlseparatinglines(
    df: DataFrame,
    _output_col: Option<&str>,
) -> PolarsResult<DataFrame> {
    Ok(df)
}

pub async fn cdlshootingstar(df: DataFrame, _output_col: Option<&str>) -> PolarsResult<DataFrame> {
    Ok(df)
}

pub async fn cdlshortline(df: DataFrame, _output_col: Option<&str>) -> PolarsResult<DataFrame> {
    Ok(df)
}

pub async fn cdlspinningtop(df: DataFrame, _output_col: Option<&str>) -> PolarsResult<DataFrame> {
    Ok(df)
}

pub async fn cdlstalledpattern(
    df: DataFrame,
    _output_col: Option<&str>,
) -> PolarsResult<DataFrame> {
    Ok(df)
}

pub async fn cdlsticksandwich(df: DataFrame, _output_col: Option<&str>) -> PolarsResult<DataFrame> {
    Ok(df)
}

pub async fn cdltakuri(df: DataFrame, _output_col: Option<&str>) -> PolarsResult<DataFrame> {
    Ok(df)
}

pub async fn cdltasukigap(df: DataFrame, _output_col: Option<&str>) -> PolarsResult<DataFrame> {
    Ok(df)
}

pub async fn cdlthrusting(df: DataFrame, _output_col: Option<&str>) -> PolarsResult<DataFrame> {
    Ok(df)
}

pub async fn cdltristar(df: DataFrame, _output_col: Option<&str>) -> PolarsResult<DataFrame> {
    Ok(df)
}

pub async fn cdlunique3river(df: DataFrame, _output_col: Option<&str>) -> PolarsResult<DataFrame> {
    Ok(df)
}

pub async fn cdlupsidegap2crows(
    df: DataFrame,
    _output_col: Option<&str>,
) -> PolarsResult<DataFrame> {
    Ok(df)
}

pub async fn cdlxsidegap3methods(
    df: DataFrame,
    _output_col: Option<&str>,
) -> PolarsResult<DataFrame> {
    Ok(df)
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn load_data() -> PolarsResult<DataFrame> {
        let df = CsvReadOptions::default()
            .try_into_reader_with_file_path(Some("download/test.csv".into()))
            .unwrap()
            .finish()
            .unwrap();
        Ok(df)
    }

    async fn save_data(df_result: &DataFrame, path: &str) -> PolarsResult<()> {
        let mut df: DataFrame = df_result.clone();
        let mut file = std::fs::File::create(path).unwrap();
        CsvWriter::new(&mut file).finish(&mut df).unwrap();
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_cdl2crows() {
        match load_data().await {
            Ok(df) => {
                match cdl2crows(df, Some("cdl2crows")).await {
                    Ok(result) => {
                        save_data(&result, "download/test_cdl2crows.csv")
                            .await
                            .unwrap();
                        println!("CDL2CROWS calculated and saved to download/test_cdl2crows.csv");
                    }
                    Err(e) => panic!("Failed to calculate cdl2crows: {:?}", e),
                };
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        };
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_cdl3blackcrows() {
        match load_data().await {
            Ok(df) => {
                match cdl3blackcrows(df, Some("cdl3blackcrows")).await {
                    Ok(result) => {
                        save_data(&result, "download/test_cdl3blackcrows.csv")
                            .await
                            .unwrap();
                        println!("CDL3BLACKCROWS calculated and saved to download/test_cdl3blackcrows.csv");
                    }
                    Err(e) => panic!("Failed to calculate cdl3blackcrows: {:?}", e),
                };
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        };
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_cdl3inside() {
        match load_data().await {
            Ok(df) => {
                match cdl3inside(df, Some("cdl3inside")).await {
                    Ok(result) => {
                        save_data(&result, "download/test_cdl3inside.csv")
                            .await
                            .unwrap();
                        println!("CDL3INSIDE calculated and saved to download/test_cdl3inside.csv");
                    }
                    Err(e) => panic!("Failed to calculate cdl3inside: {:?}", e),
                };
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        };
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_cdl3linestrike() {
        match load_data().await {
            Ok(df) => {
                match cdl3linestrike(df, Some("cdl3linestrike")).await {
                    Ok(result) => {
                        save_data(&result, "download/test_cdl3linestrike.csv")
                            .await
                            .unwrap();
                        println!("CDL3LINESTRIKE calculated and saved to download/test_cdl3linestrike.csv");
                    }
                    Err(e) => panic!("Failed to calculate cdl3linestrike: {:?}", e),
                };
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        };
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_cdl3outside() {
        match load_data().await {
            Ok(df) => {
                match cdl3outside(df, Some("cdl3outside")).await {
                    Ok(result) => {
                        save_data(&result, "download/test_cdl3outside.csv")
                            .await
                            .unwrap();
                        println!(
                            "CDL3OUTSIDE calculated and saved to download/test_cdl3outside.csv"
                        );
                    }
                    Err(e) => panic!("Failed to calculate cdl3outside: {:?}", e),
                };
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        };
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_cdl3starsinsouth() {
        match load_data().await {
            Ok(df) => {
                match cdl3starsinsouth(df, Some("cdl3starsinsouth")).await {
                    Ok(result) => {
                        save_data(&result, "download/test_cdl3starsinsouth.csv")
                            .await
                            .unwrap();
                        println!("CDL3STARSINSOUTH calculated and saved to download/test_cdl3starsinsouth.csv");
                    }
                    Err(e) => panic!("Failed to calculate cdl3starsinsouth: {:?}", e),
                };
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        };
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_cdl3whitesoldiers() {
        match load_data().await {
            Ok(df) => {
                match cdl3whitesoldiers(df, Some("cdl3whitesoldiers")).await {
                    Ok(result) => {
                        save_data(&result, "download/test_cdl3whitesoldiers.csv")
                            .await
                            .unwrap();
                        println!("CDL3WHITESOLDIERS calculated and saved to download/test_cdl3whitesoldiers.csv");
                    }
                    Err(e) => panic!("Failed to calculate cdl3whitesoldiers: {:?}", e),
                };
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        };
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_cdlabandonedbaby() {
        match load_data().await {
            Ok(df) => {
                match cdlabandonedbaby(df, Some("cdlabandonedbaby")).await {
                    Ok(result) => {
                        save_data(&result, "download/test_cdlabandonedbaby.csv")
                            .await
                            .unwrap();
                        println!("CDLABANDONEDBABY calculated and saved to download/test_cdlabandonedbaby.csv");
                    }
                    Err(e) => panic!("Failed to calculate cdlabandonedbaby: {:?}", e),
                };
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        };
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_cdladvanceblock() {
        match load_data().await {
            Ok(df) => {
                match cdladvanceblock(df, Some("cdladvanceblock")).await {
                    Ok(result) => {
                        save_data(&result, "download/test_cdladvanceblock.csv")
                            .await
                            .unwrap();
                        println!("CDLADVANCEBLOCK calculated and saved to download/test_cdladvanceblock.csv");
                    }
                    Err(e) => panic!("Failed to calculate cdladvanceblock: {:?}", e),
                };
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        };
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_cdlbelthold() {
        match load_data().await {
            Ok(df) => {
                match cdlbelthold(df, Some("cdlbelthold")).await {
                    Ok(result) => {
                        save_data(&result, "download/test_cdlbelthold.csv")
                            .await
                            .unwrap();
                        println!(
                            "CDLBELTHOLD calculated and saved to download/test_cdlbelthold.csv"
                        );
                    }
                    Err(e) => panic!("Failed to calculate cdlbelthold: {:?}", e),
                };
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        };
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_cdlbreakaway() {
        match load_data().await {
            Ok(df) => {
                match cdlbreakaway(df, Some("cdlbreakaway")).await {
                    Ok(result) => {
                        save_data(&result, "download/test_cdlbreakaway.csv")
                            .await
                            .unwrap();
                        println!(
                            "CDLBREAKAWAY calculated and saved to download/test_cdlbreakaway.csv"
                        );
                    }
                    Err(e) => panic!("Failed to calculate cdlbreakaway: {:?}", e),
                };
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        };
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_cdlclosingmarubuzo() {
        match load_data().await {
            Ok(df) => {
                match cdlclosingmarubuzo(df, Some("cdlclosingmarubuzo")).await {
                    Ok(result) => {
                        save_data(&result, "download/test_cdlclosingmarubuzo.csv")
                            .await
                            .unwrap();
                        println!("CDLCLOSINGMARUBOZU calculated and saved to download/test_cdlclosingmarubuzo.csv");
                    }
                    Err(e) => panic!("Failed to calculate cdlclosingmarubuzo: {:?}", e),
                };
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        };
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_cdlconcealbabyswall() {
        match load_data().await {
            Ok(df) => {
                match cdlconcealbabyswall(df, Some("cdlconcealbabyswall")).await {
                    Ok(result) => {
                        save_data(&result, "download/test_cdlconcealbabyswall.csv")
                            .await
                            .unwrap();
                        println!("CDLCONCEALBABYSWALL calculated and saved to download/test_cdlconcealbabyswall.csv");
                    }
                    Err(e) => panic!("Failed to calculate cdlconcealbabyswall: {:?}", e),
                };
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        };
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_cdlcounterattack() {
        match load_data().await {
            Ok(df) => {
                match cdlcounterattack(df, Some("cdlcounterattack")).await {
                    Ok(result) => {
                        save_data(&result, "download/test_cdlcounterattack.csv")
                            .await
                            .unwrap();
                        println!("CDLCOUNTERATTACK calculated and saved to download/test_cdlcounterattack.csv");
                    }
                    Err(e) => panic!("Failed to calculate cdlcounterattack: {:?}", e),
                };
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        };
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_cdldarkcloudcover() {
        match load_data().await {
            Ok(df) => {
                match cdldarkcloudcover(df, Some("cdldarkcloudcover")).await {
                    Ok(result) => {
                        save_data(&result, "download/test_cdldarkcloudcover.csv")
                            .await
                            .unwrap();
                        println!("CDLDARKCLOUDCOVER calculated and saved to download/test_cdldarkcloudcover.csv");
                    }
                    Err(e) => panic!("Failed to calculate cdldarkcloudcover: {:?}", e),
                };
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        };
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_cdldoji() {
        match load_data().await {
            Ok(df) => {
                match cdldoji(df, Some("cdldoji")).await {
                    Ok(result) => {
                        save_data(&result, "download/test_cdldoji.csv")
                            .await
                            .unwrap();
                        println!("CDLDOJI calculated and saved to download/test_cdldoji.csv");
                    }
                    Err(e) => panic!("Failed to calculate cdldoji: {:?}", e),
                };
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        };
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_cdldojistar() {
        match load_data().await {
            Ok(df) => {
                match cdldojistar(df, Some("cdldojistar")).await {
                    Ok(result) => {
                        save_data(&result, "download/test_cdldojistar.csv")
                            .await
                            .unwrap();
                        println!(
                            "CDLDOJISTAR calculated and saved to download/test_cdldojistar.csv"
                        );
                    }
                    Err(e) => panic!("Failed to calculate cdldojistar: {:?}", e),
                };
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        };
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_cdldragonflydoji() {
        match load_data().await {
            Ok(df) => {
                match cdldragonflydoji(df, Some("cdldragonflydoji")).await {
                    Ok(result) => {
                        save_data(&result, "download/test_cdldragonflydoji.csv")
                            .await
                            .unwrap();
                        println!("CDLDRAGONFLYDOJI calculated and saved to download/test_cdldragonflydoji.csv");
                    }
                    Err(e) => panic!("Failed to calculate cdldragonflydoji: {:?}", e),
                };
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        };
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_cdlengulfing() {
        match load_data().await {
            Ok(df) => {
                match cdlengulfing(df, Some("cdlengulfing")).await {
                    Ok(result) => {
                        save_data(&result, "download/test_cdlengulfing.csv")
                            .await
                            .unwrap();
                        println!(
                            "CDLENGULFING calculated and saved to download/test_cdlengulfing.csv"
                        );
                    }
                    Err(e) => panic!("Failed to calculate cdlengulfing: {:?}", e),
                };
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        };
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_cdleveningdojistar() {
        match load_data().await {
            Ok(df) => {
                match cdleveningdojistar(df, Some("cdleveningdojistar")).await {
                    Ok(result) => {
                        save_data(&result, "download/test_cdleveningdojistar.csv")
                            .await
                            .unwrap();
                        println!("CDLEVENINGDOJISTAR calculated and saved to download/test_cdleveningdojistar.csv");
                    }
                    Err(e) => panic!("Failed to calculate cdleveningdojistar: {:?}", e),
                };
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        };
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_cdleveningstar() {
        match load_data().await {
            Ok(df) => {
                match cdleveningstar(df, Some("cdleveningstar")).await {
                    Ok(result) => {
                        save_data(&result, "download/test_cdleveningstar.csv")
                            .await
                            .unwrap();
                        println!("CDLEVENINGSTAR calculated and saved to download/test_cdleveningstar.csv");
                    }
                    Err(e) => panic!("Failed to calculate cdleveningstar: {:?}", e),
                };
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        };
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_cdlgapsidesidewhite() {
        match load_data().await {
            Ok(df) => {
                match cdlgapsidesidewhite(df, Some("cdlgapsidesidewhite")).await {
                    Ok(result) => {
                        save_data(&result, "download/test_cdlgapsidesidewhite.csv")
                            .await
                            .unwrap();
                        println!("CDLGAPSIDESIDEWHITE calculated and saved to download/test_cdlgapsidesidewhite.csv");
                    }
                    Err(e) => panic!("Failed to calculate cdlgapsidesidewhite: {:?}", e),
                };
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        };
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_cdlgravestonedoji() {
        match load_data().await {
            Ok(df) => {
                match cdlgravestonedoji(df, Some("cdlgravestonedoji")).await {
                    Ok(result) => {
                        save_data(&result, "download/test_cdlgravestonedoji.csv")
                            .await
                            .unwrap();
                        println!("CDLGRAVESTONEDOJI calculated and saved to download/test_cdlgravestonedoji.csv");
                    }
                    Err(e) => panic!("Failed to calculate cdlgravestonedoji: {:?}", e),
                };
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        };
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_cdlhammer() {
        match load_data().await {
            Ok(df) => {
                match cdlhammer(df, Some("cdlhammer")).await {
                    Ok(result) => {
                        save_data(&result, "download/test_cdlhammer.csv")
                            .await
                            .unwrap();
                        println!("CDLHAMMER calculated and saved to download/test_cdlhammer.csv");
                    }
                    Err(e) => panic!("Failed to calculate cdlhammer: {:?}", e),
                };
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        };
    }
}
