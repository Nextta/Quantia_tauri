use polars::prelude::*;

/// Lista de indicadores:
/// BBANDS               Bollinger Bands
/// DEMA                 Double Exponential Moving Average
/// EMA                  Exponential Moving Average
/// HT_TRENDLINE         Hilbert Transform - Instantaneous Trendline
/// KAMA                 Kaufman Adaptive Moving Average
/// MA                   Moving average
/// MAMA                 MESA Adaptive Moving Average
/// MAVP                 Moving average with variable period
/// MIDPOINT             MidPoint over period
/// MIDPRICE             Midpoint Price over period
/// SAR                  Parabolic SAR
/// SAREXT               Parabolic SAR - Extended
/// SMA                  Simple Moving Average
/// T3                   Triple Exponential Moving Average (T3)
/// TEMA                 Triple Exponential Moving Average
/// TRIMA                Triangular Moving Average
/// WMA                  Weighted Moving Average

// ============================================================================
// Helper Functions
// ============================================================================

/// Get close column from DataFrame (case insensitive)
fn get_close(df: &DataFrame) -> PolarsResult<Series> {
    let s = df.column("close").or_else(|_| df.column("Close"))?;
    Ok(s.cast(&DataType::Float64)?.take_materialized_series())
}

/// Get high column from DataFrame (case insensitive)
fn get_high(df: &DataFrame) -> PolarsResult<Series> {
    let s = df.column("high").or_else(|_| df.column("High"))?;
    Ok(s.cast(&DataType::Float64)?.take_materialized_series())
}

/// Get low column from DataFrame (case insensitive)
fn get_low(df: &DataFrame) -> PolarsResult<Series> {
    let s = df.column("low").or_else(|_| df.column("Low"))?;
    Ok(s.cast(&DataType::Float64)?.take_materialized_series())
}

/// Simple Moving Average helper
fn calc_sma(values: &[f64], period: usize) -> Vec<f64> {
    let n = values.len();
    let mut result = vec![f64::NAN; n];

    if n < period {
        return result;
    }

    // Initialize - skip NaN values
    let init: f64 = values[..period]
        .iter()
        .filter(|v| !v.is_nan())
        .sum::<f64>();
    let valid_init = values[..period].iter().filter(|v| !v.is_nan()).count();
    
    if valid_init > 0 {
        result[period - 1] = init / valid_init as f64;
    } else {
        return result;
    }

    // Fill remaining values - track valid count for rolling window
    let mut sum = init;
    let mut valid_count = valid_init;
    
    for i in period..n {
        let prev_val = values[i - period];
        let curr_val = values[i];
        
        // Adjust sum and valid_count based on NaN values
        if !prev_val.is_nan() {
            sum -= prev_val;
            valid_count -= 1;
        }
        if !curr_val.is_nan() {
            sum += curr_val;
            valid_count += 1;
        }
        
        if valid_count > 0 {
            result[i] = sum / valid_count as f64;
        }
    }

    result
}

/// Exponential Moving Average helper
fn calc_ema(values: &[f64], period: usize) -> Vec<f64> {
    let n = values.len();
    let mut result = vec![f64::NAN; n];

    if n < period {
        return result;
    }

    let multiplier = 2.0 / (period as f64 + 1.0);

    // Find first valid value to start EMA
    let first_valid = values.iter().position(|v| !v.is_nan());
    if first_valid.is_none() {
        return result;
    }
    let start_idx = first_valid.unwrap();
    
    // Need at least 'period' total valid values
    let valid_count = values[start_idx..].iter().filter(|v| !v.is_nan()).count();
    if valid_count < period {
        return result;
    }

    // Initialize with SMA of first 'period' valid values
    let init_vals: Vec<f64> = values[start_idx..]
        .iter()
        .filter(|v| !v.is_nan())
        .take(period)
        .copied()
        .collect();
    let init_sum: f64 = init_vals.iter().sum();
    result[start_idx + period - 1] = init_sum / period as f64;

    // Fill from start_idx + period onwards
    let mut last_valid = result[start_idx + period - 1];
    for i in (start_idx + period)..n {
        if values[i].is_nan() {
            result[i] = f64::NAN;
        } else {
            result[i] = (values[i] - last_valid) * multiplier + last_valid;
            last_valid = result[i];
        }
    }

    result
}

/// Moving Average Type enum
#[derive(Clone, Copy)]
#[allow(dead_code)]
pub enum MAType {
    Sma = 0,
    Ema = 1,
    Wma = 2,
    Dema = 3,
    Tema = 4,
    Trima = 5,
    Kama = 6,
    Mama = 7,
    T3 = 8,
}

impl MAType {
    fn from_i32(val: i32) -> Self {
        match val {
            0 => MAType::Sma,
            1 => MAType::Ema,
            2 => MAType::Wma,
            3 => MAType::Dema,
            4 => MAType::Tema,
            5 => MAType::Trima,
            6 => MAType::Kama,
            7 => MAType::Mama,
            8 => MAType::T3,
            _ => MAType::Sma,
        }
    }
}

// ============================================================================
// BBANDS - Bollinger Bands
// ============================================================================

/// BBANDS - Bollinger Bands
///
/// Bandas de volatilidad que se sitúan a N desviaciones estándar por encima
/// y por debajo de una media móvil. Las bandas se expanden y contraen según
/// la volatilidad del mercado.
///
/// # Parámetros
/// * `df` - DataFrame con columna: close (case insensitive)
/// * `timeperiod` - Período de cálculo (default: 5)
/// * `nbdevup` - Desviaciones estándar para banda superior (default: 2.0)
/// * `nbdevdn` - Desviaciones estándar para banda inferior (default: 2.0)
/// * `matype` - Tipo de media móvil (0=SMA, default: 0)
/// * `output_col_bb_upper` - Nombre de la columna para la banda superior (default: "bb_upper")
/// * `output_col_bb_middle` - Nombre de la columna para la banda media (default: "bb_middle")
/// * `output_col_bb_lower` - Nombre de la columna para la banda inferior (default: "bb_lower")
///
/// # Retorna
/// DataFrame con columnas "bb_upper", "bb_middle", "bb_lower" añadidas
///
/// # Fórmula
/// Middle = SMA(close, timeperiod)
/// Upper = Middle + (nbdevup * StdDev(close, timeperiod))
/// Lower = Middle - (nbdevdn * StdDev(close, timeperiod))
pub async fn bbands(
    df: DataFrame,
    timeperiod: Option<usize>,
    nbdevup: Option<f64>,
    nbdevdn: Option<f64>,
    matype: Option<i32>,
    output_col_bb_upper: Option<&str>,
    output_col_bb_middle: Option<&str>,
    output_col_bb_lower: Option<&str>,
) -> PolarsResult<DataFrame> {
    let timeperiod = timeperiod.unwrap_or(5);
    let nbdevup = nbdevup.unwrap_or(2.0);
    let nbdevdn = nbdevdn.unwrap_or(2.0);
    let _matype = MAType::from_i32(matype.unwrap_or(0));
    let output_col_bb_upper = output_col_bb_upper.unwrap_or("bb_upper");
    let output_col_bb_middle = output_col_bb_middle.unwrap_or("bb_middle");
    let output_col_bb_lower = output_col_bb_lower.unwrap_or("bb_lower");

    let close = get_close(&df)?;
    let close_ca: ChunkedArray<Float64Type> = close.f64().unwrap().clone();
    let close_vals: Vec<f64> = close_ca.into_no_null_iter().collect();
    let n = close_vals.len();

    let middle = calc_sma(&close_vals, timeperiod);

    let mut upper: Vec<f64> = vec![f64::NAN; n];
    let mut lower: Vec<f64> = vec![f64::NAN; n];

    for i in (timeperiod - 1)..n {
        if !middle[i].is_nan() {
            let window = &close_vals[i - timeperiod + 1..=i];
            let mean = middle[i];
            let variance: f64 =
                window.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / timeperiod as f64;
            let stddev = variance.sqrt();

            upper[i] = mean + nbdevup * stddev;
            lower[i] = mean - nbdevdn * stddev;
        }
    }

    let upper_series = Series::new(output_col_bb_upper.into(), &upper);
    let middle_series = Series::new(output_col_bb_middle.into(), &middle);
    let lower_series = Series::new(output_col_bb_lower.into(), &lower);

    let mut result_df = df;
    result_df
        .with_column(upper_series.into())?
        .with_column(middle_series.into())?
        .with_column(lower_series.into())?;

    Ok(result_df)
}

// ============================================================================
// DEMA - Double Exponential Moving Average
// ============================================================================

/// DEMA - Double Exponential Moving Average
///
/// Media móvil exponencial doble que reduce el lag respecto a una EMA tradicional.
/// Combina una EMA de los datos con una EMA de la EMA para lograr mayor sensibilidad.
///
/// # Parámetros
/// * `df` - DataFrame con columna: close (case insensitive)
/// * `timeperiod` - Período de cálculo (default: 30)
/// * `output_col` - Nombre de la columna de salida (default: "dema")
///
/// # Retorna
/// DataFrame con columna "dema" añadida
///
/// # Fórmula
/// DEMA = 2 * EMA(price) - EMA(EMA(price))
pub async fn dema(
    df: DataFrame,
    timeperiod: Option<usize>,
    output_col: Option<&str>,
) -> PolarsResult<DataFrame> {
    let timeperiod = timeperiod.unwrap_or(30);
    let output_col = output_col.unwrap_or("dema");

    let close = get_close(&df)?;
    let close_ca: ChunkedArray<Float64Type> = close.f64().unwrap().clone();
    let close_vals: Vec<f64> = close_ca.into_no_null_iter().collect();

    let ema1 = calc_ema(&close_vals, timeperiod);
    let ema2 = calc_ema(&ema1, timeperiod);

    let dema_vals: Vec<f64> = ema1
        .iter()
        .zip(ema2.iter())
        .map(|(&e1, &e2)| {
            if e1.is_nan() || e2.is_nan() {
                f64::NAN
            } else {
                2.0 * e1 - e2
            }
        })
        .collect();

    let dema_series = Series::new(output_col.into(), &dema_vals);
    let mut result_df = df;
    result_df.with_column(dema_series.into())?;
    Ok(result_df)
}

// ============================================================================
// EMA - Exponential Moving Average
// ============================================================================

/// EMA - Exponential Moving Average
///
/// Media móvil exponencial que da mayor peso a los precios más recientes.
/// Más sensible que la SMA a los cambios recientes de precio.
///
/// # Parámetros
/// * `df` - DataFrame con columna: close (case insensitive)
/// * `timeperiod` - Período de cálculo (default: 30)
/// * `output_col` - Nombre de la columna de salida (default: "ema")
///
/// # Retorna
/// DataFrame con columna "ema" añadida
///
/// # Fórmula
/// EMA(t) = (price(t) - EMA(t-1)) * multiplier + EMA(t-1)
/// multiplier = 2 / (timeperiod + 1)
pub async fn ema(
    df: DataFrame,
    timeperiod: Option<usize>,
    output_col: Option<&str>,
) -> PolarsResult<DataFrame> {
    let timeperiod = timeperiod.unwrap_or(30);
    let output_col = output_col.unwrap_or("ema");

    let close = get_close(&df)?;
    let close_ca: ChunkedArray<Float64Type> = close.f64().unwrap().clone();
    let close_vals: Vec<f64> = close_ca.into_no_null_iter().collect();

    let ema_vals = calc_ema(&close_vals, timeperiod);

    let ema_series = Series::new(output_col.into(), &ema_vals);
    let mut result_df = df;
    result_df.with_column(ema_series.into())?;
    Ok(result_df)
}

// ============================================================================
// HT_TRENDLINE - Hilbert Transform - Instantaneous Trendline
// ============================================================================

/// HT_TRENDLINE - Hilbert Transform - Instantaneous Trendline
///
/// Línea de tendencia instantánea basada en la Transformada de Hilbert.
/// Utiliza el algoritmo de Ehlers para extraer la tendencia dominante
/// del mercado filtrando el ruido cíclico.
///
/// # Parámetros
/// * `df` - DataFrame con columna: close (case insensitive)
/// * `output_col` - Nombre de la columna de salida (default: "ht_trendline")
///
/// # Retorna
/// DataFrame con columna "ht_trendline" añadida
///
/// # Fórmula
/// Aplica filtro WMA(4) → Transformada de Hilbert → Filtro de fase →
/// Trendline = (I1[0] + Q1[0]) / 2 suavizado
pub async fn ht_trendline(df: DataFrame, output_col: Option<&str>) -> PolarsResult<DataFrame> {
    let output_col = output_col.unwrap_or("ht_trendline");

    let close = get_close(&df)?;
    let close_ca: ChunkedArray<Float64Type> = close.f64().unwrap().clone();
    let close_vals: Vec<f64> = close_ca.into_no_null_iter().collect();
    let n = close_vals.len();

    let mut trendline_vals: Vec<f64> = vec![f64::NAN; n];

    // HT Trendline state
    let mut period_wma_sum = 0.0;
    let mut period_wma_sub = 0.0;
    let mut trailing_price = 0.0;
    let mut smoothed_buf = [0.0f64; 6];
    let mut detrender_buf = [0.0f64; 6];
    let mut q1_buf = [0.0f64; 6];
    let mut i1_buf = [0.0f64; 10];
    let mut prev_i2 = 0.0;
    let mut prev_q2 = 0.0;
    let mut i2 = 0.0;
    let mut q2 = 0.0;
    let mut re = 0.0;
    let mut im = 0.0;
    let mut prev_period = 0.0;
    let mut smooth_period = 0.0;
    let mut prev_smooth_period = 0.0;
    let mut trendline = 0.0;
    let mut prev_trendline = 0.0;

    let a = 0.0962;
    let b = 0.5769;

    for (i, &price) in close_vals.iter().enumerate() {
        // Step 1: Price Smoothing (4-period WMA)
        period_wma_sub += price;
        period_wma_sub -= trailing_price;
        period_wma_sum += price * 4.0;
        let smoothed = period_wma_sum * 0.1;
        period_wma_sum -= period_wma_sub;
        trailing_price = price;

        for j in (1..6).rev() {
            smoothed_buf[j] = smoothed_buf[j - 1];
        }
        smoothed_buf[0] = smoothed;

        if i < 5 {
            continue;
        }

        // Step 2: Hilbert Transform - Detrender
        let detrender =
            a * smoothed_buf[0] + b * smoothed_buf[2] - a * smoothed_buf[4] - b * smoothed_buf[5];

        for j in (1..6).rev() {
            detrender_buf[j] = detrender_buf[j - 1];
        }
        detrender_buf[0] = detrender;

        // Q1 computation
        let q1 = a * detrender_buf[0] + b * detrender_buf[2]
            - a * detrender_buf[4]
            - b * detrender_buf[5];

        for j in (1..6).rev() {
            q1_buf[j] = q1_buf[j - 1];
        }
        q1_buf[0] = q1;

        // I1 is detrender delayed 3 bars
        if i >= 8 {
            for j in (1..10).rev() {
                i1_buf[j] = i1_buf[j - 1];
            }
            i1_buf[0] = detrender_buf[0];
        }

        if i < 8 {
            continue;
        }

        // jI and jQ
        let j_i = a * i1_buf[3] + b * i1_buf[5] - a * i1_buf[7] - b * i1_buf[9];
        let j_q = a * q1_buf[0] + b * q1_buf[2] - a * q1_buf[4] - b * q1_buf[5];

        // Step 3: Phasor components (I2, Q2)
        i2 = 0.2 * (i1_buf[0] - j_q) + 0.8 * prev_i2;
        q2 = 0.2 * (q1_buf[0] + j_i) + 0.8 * prev_q2;
        prev_i2 = i2;
        prev_q2 = q2;

        // Step 4: Period computation
        if i >= 9 {
            re = 0.2 * (i2 * prev_i2 + q2 * prev_q2) + 0.8 * re;
            im = 0.2 * (i2 * prev_q2 - q2 * prev_i2) + 0.8 * im;

            if im.abs() < 0.001 {
                im = 0.001;
            }
            if re.abs() < 0.001 {
                re = 0.001;
            }

            let temp_period = 360.0 / (im / re).atan().to_degrees().abs();

            let mut bounded_period = if prev_period > 0.0 {
                let lower = 0.67 * prev_period;
                let upper = 1.5 * prev_period;
                temp_period.max(lower).min(upper)
            } else {
                temp_period
            };

            bounded_period = bounded_period.max(6.0).min(50.0);

            let period_filtered = 0.2 * bounded_period + 0.8 * prev_period;
            smooth_period = 0.33 * period_filtered + 0.67 * prev_smooth_period;

            prev_period = period_filtered;
            prev_smooth_period = smooth_period;

            // Trendline computation
            let temp_trendline = (i1_buf[0] + q1_buf[0]) / 2.0;
            trendline = 0.33 * temp_trendline + 0.67 * prev_trendline;
            prev_trendline = trendline;

            trendline_vals[i] = trendline;
        }
    }

    let trendline_series = Series::new(output_col.into(), &trendline_vals);
    let mut result_df = df;
    result_df.with_column(trendline_series.into())?;
    Ok(result_df)
}

// ============================================================================
// KAMA - Kaufman Adaptive Moving Average
// ============================================================================

/// KAMA - Kaufman Adaptive Moving Average
///
/// Media móvil adaptativa que ajusta su suavizado según la eficiencia del
/// movimiento del precio. Más sensible durante tendencias fuertes y más
/// suave durante mercados laterales.
///
/// # Parámetros
/// * `df` - DataFrame con columna: close (case insensitive)
/// * `timeperiod` - Período de cálculo (default: 30)
/// * `output_col` - Nombre de la columna de salida (default: "kama")
///
/// # Retorna
/// DataFrame con columna "kama" añadida
///
/// # Fórmula
/// ER = Change / Volatility
/// SC = [ER * (2/(fast+1) - 2/(slow+1)) + 2/(slow+1)]^2
/// KAMA = SC * price + (1 - SC) * KAMA_prev
pub async fn kama(
    df: DataFrame,
    timeperiod: Option<usize>,
    output_col: Option<&str>,
) -> PolarsResult<DataFrame> {
    let timeperiod = timeperiod.unwrap_or(30);
    let output_col = output_col.unwrap_or("kama");

    let close = get_close(&df)?;
    let close_ca: ChunkedArray<Float64Type> = close.f64().unwrap().clone();
    let close_vals: Vec<f64> = close_ca.into_no_null_iter().collect();
    let n = close_vals.len();

    let mut kama_vals: Vec<f64> = vec![f64::NAN; n];

    if n >= timeperiod {
        let fast_sc = 2.0 / (3.0); // 2/(2+1)
        let slow_sc = 2.0 / (31.0); // 2/(30+1)

        for i in (timeperiod - 1)..n {
            let change = (close_vals[i] - close_vals[i - timeperiod + 1]).abs();
            let volatility: f64 = (i - timeperiod + 2..=i)
                .map(|j| (close_vals[j] - close_vals[j - 1]).abs())
                .sum();

            if volatility != 0.0 {
                let er = change / volatility;
                let sc = (er * (fast_sc - slow_sc) + slow_sc).powi(2);

                if i == timeperiod - 1 {
                    // Initialize with SMA
                    let sum: f64 = close_vals[..timeperiod].iter().sum();
                    kama_vals[i] = sum / timeperiod as f64;
                } else {
                    kama_vals[i] = sc * close_vals[i] + (1.0 - sc) * kama_vals[i - 1];
                }
            }
        }
    }

    let kama_series = Series::new(output_col.into(), &kama_vals);
    let mut result_df = df;
    result_df.with_column(kama_series.into())?;
    Ok(result_df)
}

// ============================================================================
// MA - Moving Average
// ============================================================================

/// MA - Moving Average
///
/// Media móvil con selección de tipo. Soporta múltiples algoritmos de
/// suavizado: SMA, EMA, WMA, DEMA, TEMA, TRIMA, KAMA, MAMA, T3.
///
/// # Parámetros
/// * `df` - DataFrame con columna: close (case insensitive)
/// * `timeperiod` - Período de cálculo (default: 30)
/// * `matype` - Tipo de media móvil (0=SMA, 1=EMA, 2=WMA, 3=DEMA, 4=TEMA, 5=TRIMA, 6=KAMA, 7=MAMA, 8=T3)
/// * `output_col` - Nombre de la columna de salida (default: "ma")
///
/// # Retorna
/// DataFrame con columna "ma" añadida
pub async fn ma(
    df: DataFrame,
    timeperiod: Option<usize>,
    matype: Option<i32>,
    output_col: Option<&str>,
) -> PolarsResult<DataFrame> {
    let timeperiod = timeperiod.unwrap_or(30);
    let matype = matype.unwrap_or(0);
    let output_col = output_col.unwrap_or("ma");

    let close = get_close(&df)?;
    let close_ca: ChunkedArray<Float64Type> = close.f64().unwrap().clone();
    let close_vals: Vec<f64> = close_ca.into_no_null_iter().collect();

    let ma_vals = match MAType::from_i32(matype) {
        MAType::Sma => calc_sma(&close_vals, timeperiod),
        MAType::Ema => calc_ema(&close_vals, timeperiod),
        MAType::Wma => calc_wma(&close_vals, timeperiod),
        MAType::Dema => {
            let ema1 = calc_ema(&close_vals, timeperiod);
            let ema2 = calc_ema(&ema1, timeperiod);
            ema1.iter()
                .zip(ema2.iter())
                .map(|(&e1, &e2)| {
                    if e1.is_nan() || e2.is_nan() {
                        f64::NAN
                    } else {
                        2.0 * e1 - e2
                    }
                })
                .collect()
        }
        MAType::Tema => {
            let ema1 = calc_ema(&close_vals, timeperiod);
            let ema2 = calc_ema(&ema1, timeperiod);
            let ema3 = calc_ema(&ema2, timeperiod);
            ema1.iter()
                .zip(ema2.iter())
                .zip(ema3.iter())
                .map(|((&e1, &e2), &e3)| {
                    if e1.is_nan() || e2.is_nan() || e3.is_nan() {
                        f64::NAN
                    } else {
                        3.0 * e1 - 3.0 * e2 + e3
                    }
                })
                .collect()
        }
        MAType::Trima => calc_trima(&close_vals, timeperiod),
        MAType::Kama => {
            let mut kama = vec![f64::NAN; close_vals.len()];
            if close_vals.len() >= timeperiod {
                let fast_sc = 2.0 / 3.0;
                let slow_sc = 2.0 / 31.0;
                for i in (timeperiod - 1)..close_vals.len() {
                    let change = (close_vals[i] - close_vals[i - timeperiod + 1]).abs();
                    let volatility: f64 = (i - timeperiod + 2..=i)
                        .map(|j| (close_vals[j] - close_vals[j - 1]).abs())
                        .sum();
                    if volatility != 0.0 {
                        let er = change / volatility;
                        let sc = (er * (fast_sc - slow_sc) + slow_sc).powi(2);
                        if i == timeperiod - 1 {
                            let sum: f64 = close_vals[..timeperiod].iter().sum();
                            kama[i] = sum / timeperiod as f64;
                        } else {
                            kama[i] = sc * close_vals[i] + (1.0 - sc) * kama[i - 1];
                        }
                    }
                }
            }
            kama
        }
        MAType::Mama => {
            let (mama, _) = calc_mama(&close_vals, 0.5, 0.05);
            mama
        }
        MAType::T3 => calc_t3(&close_vals, timeperiod, 0.7),
    };

    let ma_series = Series::new(output_col.into(), &ma_vals);
    let mut result_df = df;
    result_df.with_column(ma_series.into())?;
    Ok(result_df)
}

// ============================================================================
// MAMA - MESA Adaptive Moving Average
// ============================================================================

/// MAMA - MESA Adaptive Moving Average
///
/// Media móvil adaptativa basada en el algoritmo MESA (Maximum Entropy
/// Spectral Analysis) de Ehlers. Se adapta automáticamente al ciclo
/// dominante del mercado.
///
/// # Parámetros
/// * `df` - DataFrame con columna: close (case insensitive)
/// * `fastlimit` - Límite rápido (default: 0.5)
/// * `slowlimit` - Límite lento (default: 0.05)
/// * `output_col_mama` - Nombre columna MAMA (default: "mama")
/// * `output_col_fama` - Nombre columna FAMA (default: "fama")
///
/// # Retorna
/// DataFrame con columnas "mama" y "fama" añadidas
///
/// # Fórmula
/// Aplica Transformada de Hilbert → Calcula fase → Alpha adaptativo →
/// MAMA = alpha * price + (1 - alpha) * MAMA_prev
/// FAMA = media móvil de MAMA
pub async fn mama(
    df: DataFrame,
    fastlimit: Option<f64>,
    slowlimit: Option<f64>,
    output_col_mama: Option<&str>,
    output_col_fama: Option<&str>,
) -> PolarsResult<DataFrame> {
    let fastlimit = fastlimit.unwrap_or(0.5);
    let slowlimit = slowlimit.unwrap_or(0.05);
    let output_col_mama = output_col_mama.unwrap_or("mama");
    let output_col_fama = output_col_fama.unwrap_or("fama");

    let close = get_close(&df)?;
    let close_ca: ChunkedArray<Float64Type> = close.f64().unwrap().clone();
    let close_vals: Vec<f64> = close_ca.into_no_null_iter().collect();

    let (mama_vals, fama_vals) = calc_mama(&close_vals, fastlimit, slowlimit);

    let mama_series = Series::new(output_col_mama.into(), &mama_vals);
    let fama_series = Series::new(output_col_fama.into(), &fama_vals);

    let mut result_df = df;
    result_df
        .with_column(mama_series.into())?
        .with_column(fama_series.into())?;
    Ok(result_df)
}

fn calc_mama(values: &[f64], fastlimit: f64, slowlimit: f64) -> (Vec<f64>, Vec<f64>) {
    let n = values.len();
    let mut mama_vals = vec![f64::NAN; n];
    let mut fama_vals = vec![f64::NAN; n];

    if n < 9 {
        return (mama_vals, fama_vals);
    }

    let mut period_wma_sum = 0.0;
    let mut period_wma_sub = 0.0;
    let mut trailing_price = 0.0;
    let mut smoothed_buf = [0.0f64; 6];
    let mut detrender_buf = [0.0f64; 6];
    let mut q1_buf = [0.0f64; 6];
    let mut i1_buf = [0.0f64; 10];
    let mut prev_i2 = 0.0;
    let mut prev_q2 = 0.0;
    let mut i2 = 0.0;
    let mut q2 = 0.0;
    let mut re = 0.0;
    let mut im = 0.0;
    let mut prev_period = 0.0;
    let mut smooth_period = 0.0;
    let mut prev_smooth_period = 0.0;
    let mut mama = 0.0;
    let mut fama = 0.0;
    let mut initialized = false;

    let a = 0.0962;
    let b = 0.5769;

    for (i, &price) in values.iter().enumerate() {
        period_wma_sub += price;
        period_wma_sub -= trailing_price;
        period_wma_sum += price * 4.0;
        let smoothed = period_wma_sum * 0.1;
        period_wma_sum -= period_wma_sub;
        trailing_price = price;

        for j in (1..6).rev() {
            smoothed_buf[j] = smoothed_buf[j - 1];
        }
        smoothed_buf[0] = smoothed;

        if i < 5 {
            continue;
        }

        let detrender =
            a * smoothed_buf[0] + b * smoothed_buf[2] - a * smoothed_buf[4] - b * smoothed_buf[5];

        for j in (1..6).rev() {
            detrender_buf[j] = detrender_buf[j - 1];
        }
        detrender_buf[0] = detrender;

        let q1 = a * detrender_buf[0] + b * detrender_buf[2]
            - a * detrender_buf[4]
            - b * detrender_buf[5];

        for j in (1..6).rev() {
            q1_buf[j] = q1_buf[j - 1];
        }
        q1_buf[0] = q1;

        if i >= 8 {
            for j in (1..10).rev() {
                i1_buf[j] = i1_buf[j - 1];
            }
            i1_buf[0] = detrender_buf[0];
        }

        if i < 8 {
            continue;
        }

        let j_i = a * i1_buf[3] + b * i1_buf[5] - a * i1_buf[7] - b * i1_buf[9];
        let j_q = a * q1_buf[0] + b * q1_buf[2] - a * q1_buf[4] - b * q1_buf[5];

        i2 = 0.2 * (i1_buf[0] - j_q) + 0.8 * prev_i2;
        q2 = 0.2 * (q1_buf[0] + j_i) + 0.8 * prev_q2;
        prev_i2 = i2;
        prev_q2 = q2;

        if i >= 9 {
            re = 0.2 * (i2 * prev_i2 + q2 * prev_q2) + 0.8 * re;
            im = 0.2 * (i2 * prev_q2 - q2 * prev_i2) + 0.8 * im;

            if im.abs() < 0.001 {
                im = 0.001;
            }
            if re.abs() < 0.001 {
                re = 0.001;
            }

            let temp_period = 360.0 / (im / re).atan().to_degrees().abs();

            let mut bounded_period = if prev_period > 0.0 {
                temp_period.max(0.67 * prev_period).min(1.5 * prev_period)
            } else {
                temp_period
            };

            bounded_period = bounded_period.max(6.0).min(50.0);

            let period_filtered = 0.2 * bounded_period + 0.8 * prev_period;
            smooth_period = 0.33 * period_filtered + 0.67 * prev_smooth_period;

            prev_period = period_filtered;
            prev_smooth_period = smooth_period;

            // Phase calculation
            let phase = if i1_buf[0].abs() > 0.001 {
                (q1_buf[0] / i1_buf[0]).atan().to_degrees() + 90.0
            } else {
                90.0
            };

            // Alpha adaptativo
            let abs_phase = phase.abs();
            let mut alpha = (fastlimit / abs_phase.max(1.0)).min(slowlimit).max(0.0);
            alpha = alpha.max(0.0).min(fastlimit);

            if !initialized {
                mama = price;
                fama = price;
                initialized = true;
            } else {
                mama = alpha * price + (1.0 - alpha) * mama;
                fama = 0.5 * alpha * mama + (1.0 - 0.5 * alpha) * fama;
            }

            mama_vals[i] = mama;
            fama_vals[i] = fama;
        }
    }

    (mama_vals, fama_vals)
}

// ============================================================================
// MAVP - Moving Average with Variable Period
// ============================================================================

/// MAVP - Moving Average with Variable Period
///
/// Media móvil con período variable que cambia en cada barra según una
/// serie de entrada. Permite adaptar dinámicamente la sensibilidad
/// de la media móvil.
///
/// # Parámetros
/// * `df` - DataFrame con columnas: close, period (case insensitive)
/// * `minperiod` - Período mínimo (default: 2)
/// * `maxperiod` - Período máximo (default: 30)
/// * `matype` - Tipo de media móvil (default: 0=SMA)
/// * `output_col` - Nombre de la columna de salida (default: "mavp")
///
/// # Retorna
/// DataFrame con columna "mavp" añadida
pub async fn mavp(
    df: DataFrame,
    minperiod: Option<usize>,
    maxperiod: Option<usize>,
    matype: Option<i32>,
    output_col: Option<&str>,
) -> PolarsResult<DataFrame> {
    let minperiod = minperiod.unwrap_or(2);
    let maxperiod = maxperiod.unwrap_or(30);
    let matype = matype.unwrap_or(0);
    let output_col = output_col.unwrap_or("mavp");

    let close = get_close(&df)?;
    let close_ca: ChunkedArray<Float64Type> = close.f64().unwrap().clone();
    let close_vals: Vec<f64> = close_ca.into_no_null_iter().collect();

    // Try to get period column
    let period_col = df.column("period").or_else(|_| df.column("Period"))?;
    let period_ca: ChunkedArray<Float64Type> = period_col.cast(&DataType::Float64)?.f64()?.clone();
    let period_vals: Vec<f64> = period_ca.into_no_null_iter().collect();

    let n = close_vals.len();
    let mut mavp_vals: Vec<f64> = vec![f64::NAN; n];

    for i in 0..n {
        if !period_vals[i].is_nan() && period_vals[i] > 0.0 {
            let period = period_vals[i].round() as usize;
            let period = period.max(minperiod).min(maxperiod);

            if i >= period - 1 {
                let window = &close_vals[i - period + 1..=i];
                mavp_vals[i] = match MAType::from_i32(matype) {
                    MAType::Sma => window.iter().sum::<f64>() / period as f64,
                    MAType::Ema => {
                        // Simplified: use last value of EMA
                        let ema = calc_ema(window, period);
                        ema.last().copied().unwrap_or(f64::NAN)
                    }
                    _ => window.iter().sum::<f64>() / period as f64, // Default to SMA
                };
            }
        }
    }

    let mavp_series = Series::new(output_col.into(), &mavp_vals);
    let mut result_df = df;
    result_df.with_column(mavp_series.into())?;
    Ok(result_df)
}

// ============================================================================
// MIDPOINT - MidPoint over period
// ============================================================================

/// MIDPOINT - MidPoint over period
///
/// Punto medio entre el valor más alto y más bajo en un período.
/// Representa el nivel medio de precio durante la ventana.
///
/// # Parámetros
/// * `df` - DataFrame con columna: close (case insensitive)
/// * `timeperiod` - Período de cálculo (default: 14)
/// * `output_col` - Nombre de la columna de salida (default: "midpoint")
///
/// # Retorna
/// DataFrame con columna "midpoint" añadida
///
/// # Fórmula
/// MIDPOINT = (max(close, timeperiod) + min(close, timeperiod)) / 2
pub async fn midpoint(
    df: DataFrame,
    timeperiod: Option<usize>,
    output_col: Option<&str>,
) -> PolarsResult<DataFrame> {
    let timeperiod = timeperiod.unwrap_or(14);
    let output_col = output_col.unwrap_or("midpoint");

    let close = get_close(&df)?;
    let close_ca: ChunkedArray<Float64Type> = close.f64().unwrap().clone();
    let close_vals: Vec<f64> = close_ca.into_no_null_iter().collect();
    let n = close_vals.len();

    let mut midpoint_vals: Vec<f64> = vec![f64::NAN; n];

    for i in (timeperiod - 1)..n {
        let window = &close_vals[i - timeperiod + 1..=i];
        let max_val = window.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let min_val = window.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        midpoint_vals[i] = (max_val + min_val) / 2.0;
    }

    let midpoint_series = Series::new(output_col.into(), &midpoint_vals);
    let mut result_df = df;
    result_df.with_column(midpoint_series.into())?;
    Ok(result_df)
}

// ============================================================================
// MIDPRICE - Midpoint Price over period
// ============================================================================

/// MIDPRICE - Midpoint Price over period
///
/// Punto medio entre el precio más alto y más bajo en un período.
/// Similar a MIDPOINT pero usa high y low en lugar de close.
///
/// # Parámetros
/// * `df` - DataFrame con columnas: high, low (case insensitive)
/// * `timeperiod` - Período de cálculo (default: 14)
/// * `output_col` - Nombre de la columna de salida (default: "midprice")
///
/// # Retorna
/// DataFrame con columna "midprice" añadida
///
/// # Fórmula
/// MIDPRICE = (max(high, timeperiod) + min(low, timeperiod)) / 2
pub async fn midprice(
    df: DataFrame,
    timeperiod: Option<usize>,
    output_col: Option<&str>,
) -> PolarsResult<DataFrame> {
    let timeperiod = timeperiod.unwrap_or(14);
    let output_col = output_col.unwrap_or("midprice");

    let high = get_high(&df)?;
    let low = get_low(&df)?;

    let high_ca: ChunkedArray<Float64Type> = high.f64().unwrap().clone();
    let low_ca: ChunkedArray<Float64Type> = low.f64().unwrap().clone();
    let high_vals: Vec<f64> = high_ca.into_no_null_iter().collect();
    let low_vals: Vec<f64> = low_ca.into_no_null_iter().collect();
    let n = high_vals.len();

    let mut midprice_vals: Vec<f64> = vec![f64::NAN; n];

    for i in (timeperiod - 1)..n {
        let high_window = &high_vals[i - timeperiod + 1..=i];
        let low_window = &low_vals[i - timeperiod + 1..=i];
        let max_high = high_window.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let min_low = low_window.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        midprice_vals[i] = (max_high + min_low) / 2.0;
    }

    let midprice_series = Series::new(output_col.into(), &midprice_vals);
    let mut result_df = df;
    result_df.with_column(midprice_series.into())?;
    Ok(result_df)
}

// ============================================================================
// SAR - Parabolic SAR
// ============================================================================

/// SAR - Parabolic SAR (Stop And Reverse)
///
/// Sistema de stop parabólico que se ajusta automáticamente a la tendencia.
/// Los puntos SAR se sitúan por debajo del precio en tendencias alcistas
/// y por encima en tendencias bajistas.
///
/// # Parámetros
/// * `df` - DataFrame con columnas: high, low (case insensitive)
/// * `acceleration` - Factor de aceleración inicial (default: 0.02)
/// * `maximum` - Factor de aceleración máximo (default: 0.2)
/// * `output_col` - Nombre de la columna de salida (default: "sar")
///
/// # Retorna
/// DataFrame con columna "sar" añadida
pub async fn sar(
    df: DataFrame,
    acceleration: Option<f64>,
    maximum: Option<f64>,
    output_col: Option<&str>,
) -> PolarsResult<DataFrame> {
    let acceleration = acceleration.unwrap_or(0.02);
    let maximum = maximum.unwrap_or(0.2);
    let output_col = output_col.unwrap_or("sar");

    let high = get_high(&df)?;
    let low = get_low(&df)?;

    let high_ca: ChunkedArray<Float64Type> = high.f64().unwrap().clone();
    let low_ca: ChunkedArray<Float64Type> = low.f64().unwrap().clone();
    let high_vals: Vec<f64> = high_ca.into_no_null_iter().collect();
    let low_vals: Vec<f64> = low_ca.into_no_null_iter().collect();
    let n = high_vals.len();

    let mut sar_vals: Vec<f64> = vec![f64::NAN; n];

    if n < 2 {
        let sar_series = Series::new(output_col.into(), &sar_vals);
        let mut result_df = df;
        result_df.with_column(sar_series.into())?;
        return Ok(result_df);
    }

    let mut af = acceleration; // Acceleration Factor
    let mut ep = high_vals[0]; // Extreme Point
    let mut is_long = high_vals[1] > high_vals[0]; // Initial trend direction

    // Initialize SAR
    sar_vals[0] = low_vals[0];
    sar_vals[1] = if is_long { low_vals[0] } else { high_vals[0] };

    for i in 2..n {
        let prev_sar = sar_vals[i - 1];
        let prev_prev_sar = if i >= 2 { sar_vals[i - 2] } else { prev_sar };

        // Calculate current SAR
        let mut current_sar = prev_sar + af * (ep - prev_sar);

        // SAR should not cross price
        if is_long {
            current_sar = current_sar
                .min(low_vals[i - 1])
                .min(low_vals[i - 2].min(prev_sar));
        } else {
            current_sar = current_sar
                .max(high_vals[i - 1])
                .max(high_vals[i - 2].max(prev_sar));
        }

        // Check for SAR crossover (trend reversal)
        let mut reversed = false;
        if is_long && current_sar > low_vals[i] {
            reversed = true;
        } else if !is_long && current_sar < high_vals[i] {
            reversed = true;
        }

        if reversed {
            // Reverse position - set SAR to previous extreme point
            is_long = !is_long;
            af = acceleration;
            ep = if is_long { high_vals[i] } else { low_vals[i] };
            // On reversal, SAR equals prior extreme point
            sar_vals[i] = if is_long {
                high_vals[i - 1].max(high_vals[i - 2])
            } else {
                low_vals[i - 1].min(low_vals[i - 2])
            };
            // Reset for next iteration
            let next_sar = sar_vals[i] + af * (ep - sar_vals[i]);
            sar_vals[i] = next_sar;
        } else {
            // Update extreme point and acceleration factor
            if is_long && high_vals[i] > ep {
                ep = high_vals[i];
                af = (af + acceleration).min(maximum);
            } else if !is_long && low_vals[i] < ep {
                ep = low_vals[i];
                af = (af + acceleration).min(maximum);
            }

            sar_vals[i] = current_sar;
        }
    }

    let sar_series = Series::new(output_col.into(), &sar_vals);
    let mut result_df = df;
    result_df.with_column(sar_series.into())?;
    Ok(result_df)
}

// ============================================================================
// SAREXT - Parabolic SAR - Extended
// ============================================================================

/// SAREXT - Parabolic SAR - Extended
///
/// Versión extendida del SAR parabólico con parámetros adicionales para
/// personalizar el comportamiento de aceleración y posicionamiento.
/// Permite ajustar offsets y factores de aceleración de forma independiente.
///
/// # Parámetros
/// * `df` - DataFrame con columnas: high, low (case insensitive)
/// * `startvalue` - Valor inicial del AF (default: 0.02)
/// * `offsetonlong` - Offset para posiciones largas (default: 0.0)
/// * `offsetonshort` - Offset para posiciones cortas (default: 0.0)
/// * `blockonlong` - Bloqueo para posiciones largas (default: 0.0)
/// * `blockonshort` - Bloqueo para posiciones cortas (default: 0.0)
/// * `output_col` - Nombre de la columna de salida (default: "sarext")
///
/// # Retorna
/// DataFrame con columna "sarext" añadida
pub async fn sarext(
    df: DataFrame,
    startvalue: Option<f64>,
    offsetonlong: Option<f64>,
    offsetonshort: Option<f64>,
    blockonlong: Option<f64>,
    blockonshort: Option<f64>,
    output_col: Option<&str>,
) -> PolarsResult<DataFrame> {
    let startvalue = startvalue.unwrap_or(0.02);
    let offsetonlong = offsetonlong.unwrap_or(0.0);
    let offsetonshort = offsetonshort.unwrap_or(0.0);
    let _blockonlong = blockonlong.unwrap_or(0.0);
    let _blockonshort = blockonshort.unwrap_or(0.0);
    let output_col = output_col.unwrap_or("sarext");

    let high = get_high(&df)?;
    let low = get_low(&df)?;

    let high_ca: ChunkedArray<Float64Type> = high.f64().unwrap().clone();
    let low_ca: ChunkedArray<Float64Type> = low.f64().unwrap().clone();
    let high_vals: Vec<f64> = high_ca.into_no_null_iter().collect();
    let low_vals: Vec<f64> = low_ca.into_no_null_iter().collect();
    let n = high_vals.len();

    let mut sarext_vals: Vec<f64> = vec![f64::NAN; n];

    if n < 2 {
        let sarext_series = Series::new(output_col.into(), &sarext_vals);
        let mut result_df = df;
        result_df.with_column(sarext_series.into())?;
        return Ok(result_df);
    }

    // Extended SAR with configurable parameters
    let mut af = startvalue;
    let mut ep = high_vals[0];
    let mut is_long = high_vals[1] > high_vals[0];

    sarext_vals[0] = low_vals[0];
    sarext_vals[1] = if is_long { low_vals[0] } else { high_vals[0] };

    for i in 2..n {
        let prev_sar = sarext_vals[i - 1];

        let mut current_sar = prev_sar + af * (ep - prev_sar);

        // Apply offsets
        if is_long {
            current_sar = current_sar.min(low_vals[i - 1]) - offsetonlong;
        } else {
            current_sar = current_sar.max(high_vals[i - 1]) + offsetonshort;
        }

        // Check for reversal
        let mut reversed = false;
        if is_long && current_sar > low_vals[i] {
            reversed = true;
        } else if !is_long && current_sar < high_vals[i] {
            reversed = true;
        }

        if reversed {
            is_long = !is_long;
            af = startvalue;
            ep = if is_long { high_vals[i] } else { low_vals[i] };
            sarext_vals[i] = ep;
        } else {
            if is_long && high_vals[i] > ep {
                ep = high_vals[i];
                af = (af + startvalue).min(0.2);
            } else if !is_long && low_vals[i] < ep {
                ep = low_vals[i];
                af = (af + startvalue).min(0.2);
            }

            sarext_vals[i] = current_sar;
        }
    }

    let sarext_series = Series::new(output_col.into(), &sarext_vals);
    let mut result_df = df;
    result_df.with_column(sarext_series.into())?;
    Ok(result_df)
}

// ============================================================================
// SMA - Simple Moving Average
// ============================================================================

/// SMA - Simple Moving Average
///
/// Media móvil simple que calcula el promedio aritmético de los precios
/// en un período determinado. Es el indicador más básico y ampliamente
/// utilizado.
///
/// # Parámetros
/// * `df` - DataFrame con columna: close (case insensitive)
/// * `timeperiod` - Período de cálculo (default: 30)
/// * `output_col` - Nombre de la columna de salida (default: "sma")
///
/// # Retorna
/// DataFrame con columna "sma" añadida
///
/// # Fórmula
/// SMA = sum(close, timeperiod) / timeperiod
pub async fn sma(
    df: DataFrame,
    timeperiod: Option<usize>,
    output_col: Option<&str>,
) -> PolarsResult<DataFrame> {
    let timeperiod = timeperiod.unwrap_or(30);
    let output_col = output_col.unwrap_or("sma");

    let close = get_close(&df)?;
    let close_ca: ChunkedArray<Float64Type> = close.f64().unwrap().clone();
    let close_vals: Vec<f64> = close_ca.into_no_null_iter().collect();

    let sma_vals = calc_sma(&close_vals, timeperiod);

    let sma_series = Series::new(output_col.into(), &sma_vals);
    let mut result_df = df;
    result_df.with_column(sma_series.into())?;
    Ok(result_df)
}

// ============================================================================
// T3 - Triple Exponential Moving Average (T3)
// ============================================================================

/// T3 - Triple Exponential Moving Average (T3)
///
/// Media móvil exponencial triple desarrollada por Tim Tillson. Utiliza
/// un factor de volumen para optimizar el suavizado y reducir el lag.
/// Más suave que TEMA y con mejor comportamiento en ruido.
///
/// # Parámetros
/// * `df` - DataFrame con columna: close (case insensitive)
/// * `timeperiod` - Período de cálculo (default: 5)
/// * `vfactor` - Factor de volumen (default: 0.7)
/// * `output_col` - Nombre de la columna de salida (default: "t3")
///
/// # Retorna
/// DataFrame con columna "t3" añadida
///
/// # Fórmula
/// e1 = EMA(price)
/// e2 = EMA(e1)
/// e3 = EMA(e2)
/// e4 = EMA(e3)
/// e5 = EMA(e4)
/// e6 = EMA(e5)
/// T3 = c1*e1 + c2*e2 + c3*e3 + c4*e4 + c5*e5 + c6*e6
/// donde c1..c6 son coeficientes basados en vfactor
pub async fn t3(
    df: DataFrame,
    timeperiod: Option<usize>,
    vfactor: Option<f64>,
    output_col: Option<&str>,
) -> PolarsResult<DataFrame> {
    let timeperiod = timeperiod.unwrap_or(5);
    let vfactor = vfactor.unwrap_or(0.7);
    let output_col = output_col.unwrap_or("t3");

    let close = get_close(&df)?;
    let close_ca: ChunkedArray<Float64Type> = close.f64().unwrap().clone();
    let close_vals: Vec<f64> = close_ca.into_no_null_iter().collect();

    let t3_vals = calc_t3(&close_vals, timeperiod, vfactor);

    let t3_series = Series::new(output_col.into(), &t3_vals);
    let mut result_df = df;
    result_df.with_column(t3_series.into())?;
    Ok(result_df)
}

fn calc_t3(values: &[f64], period: usize, vfactor: f64) -> Vec<f64> {
    let n = values.len();
    let mut result = vec![f64::NAN; n];

    if n < period {
        return result;
    }

    let c1 = -(vfactor.powi(3));
    let c2 = 3.0 * vfactor.powi(2) + 3.0 * vfactor.powi(3);
    let c3 = -6.0 * vfactor.powi(2) - 3.0 * vfactor - 3.0 * vfactor.powi(3);
    let c4 = 1.0 + 3.0 * vfactor + vfactor.powi(3) + 3.0 * vfactor.powi(2);

    let e1 = calc_ema(values, period);
    let e2 = calc_ema(&e1, period);
    let e3 = calc_ema(&e2, period);
    let e4 = calc_ema(&e3, period);
    let e5 = calc_ema(&e4, period);
    let e6 = calc_ema(&e5, period);

    for i in 0..n {
        if !e1[i].is_nan()
            && !e2[i].is_nan()
            && !e3[i].is_nan()
            && !e4[i].is_nan()
            && !e5[i].is_nan()
            && !e6[i].is_nan()
        {
            result[i] = c1 * e6[i] + c2 * e5[i] + c3 * e4[i] + c4 * e3[i];
        }
    }

    result
}

// ============================================================================
// TEMA - Triple Exponential Moving Average
// ============================================================================

/// TEMA - Triple Exponential Moving Average
///
/// Media móvil exponencial triple que reduce significativamente el lag
/// comparado con EMA tradicional. Combina EMA, EMA de EMA y EMA de
/// EMA de EMA para lograr mayor sensibilidad.
///
/// # Parámetros
/// * `df` - DataFrame con columna: close (case insensitive)
/// * `timeperiod` - Período de cálculo (default: 30)
/// * `output_col` - Nombre de la columna de salida (default: "tema")
///
/// # Retorna
/// DataFrame con columna "tema" añadida
///
/// # Fórmula
/// TEMA = 3 * EMA - 3 * EMA(EMA) + EMA(EMA(EMA))
pub async fn tema(
    df: DataFrame,
    timeperiod: Option<usize>,
    output_col: Option<&str>,
) -> PolarsResult<DataFrame> {
    let timeperiod = timeperiod.unwrap_or(30);
    let output_col = output_col.unwrap_or("tema");

    let close = get_close(&df)?;
    let close_ca: ChunkedArray<Float64Type> = close.f64().unwrap().clone();
    let close_vals: Vec<f64> = close_ca.into_no_null_iter().collect();

    let ema1 = calc_ema(&close_vals, timeperiod);
    let ema2 = calc_ema(&ema1, timeperiod);
    let ema3 = calc_ema(&ema2, timeperiod);

    let tema_vals: Vec<f64> = ema1
        .iter()
        .zip(ema2.iter())
        .zip(ema3.iter())
        .map(|((&e1, &e2), &e3)| {
            if e1.is_nan() || e2.is_nan() || e3.is_nan() {
                f64::NAN
            } else {
                3.0 * e1 - 3.0 * e2 + e3
            }
        })
        .collect();

    let tema_series = Series::new(output_col.into(), &tema_vals);
    let mut result_df = df;
    result_df.with_column(tema_series.into())?;
    Ok(result_df)
}

// ============================================================================
// TRIMA - Triangular Moving Average
// ============================================================================

/// TRIMA - Triangular Moving Average
///
/// Media móvil triangular que aplica doble suavizado lineal. Da más peso
/// a los valores centrales del período y menos a los extremos.
///
/// # Parámetros
/// * `df` - DataFrame con columna: close (case insensitive)
/// * `timeperiod` - Período de cálculo (default: 30)
/// * `output_col` - Nombre de la columna de salida (default: "trima")
///
/// # Retorna
/// DataFrame con columna "trima" añadida
///
/// # Fórmula
/// TRIMA = SMA(SMA(price, ceil(period/2)), floor(period/2)+1)
pub async fn trima(
    df: DataFrame,
    timeperiod: Option<usize>,
    output_col: Option<&str>,
) -> PolarsResult<DataFrame> {
    let timeperiod = timeperiod.unwrap_or(30);
    let output_col = output_col.unwrap_or("trima");

    let close = get_close(&df)?;
    let close_ca: ChunkedArray<Float64Type> = close.f64().unwrap().clone();
    let close_vals: Vec<f64> = close_ca.into_no_null_iter().collect();

    let trima_vals = calc_trima(&close_vals, timeperiod);

    let trima_series = Series::new(output_col.into(), &trima_vals);
    let mut result_df = df;
    result_df.with_column(trima_series.into())?;
    Ok(result_df)
}

fn calc_trima(values: &[f64], period: usize) -> Vec<f64> {
    let n = values.len();
    if n < period {
        return vec![f64::NAN; n];
    }

    let half1 = (period + 1) / 2;
    let half2 = period / 2 + 1;

    let sma1 = calc_sma(values, half1);
    let trima = calc_sma(&sma1, half2);

    trima
}

// ============================================================================
// WMA - Weighted Moving Average
// ============================================================================

/// WMA - Weighted Moving Average
///
/// Media móvil ponderada que asigna pesos linealmente decrecientes a los
/// datos históricos. Los precios más recientes tienen mayor peso.
///
/// # Parámetros
/// * `df` - DataFrame con columna: close (case insensitive)
/// * `timeperiod` - Período de cálculo (default: 30)
/// * `output_col` - Nombre de la columna de salida (default: "wma")
///
/// # Retorna
/// DataFrame con columna "wma" añadida
///
/// # Fórmula
/// WMA = sum(price[i] * (i+1)) / sum(1..period) para i en 0..period
pub async fn wma(
    df: DataFrame,
    timeperiod: Option<usize>,
    output_col: Option<&str>,
) -> PolarsResult<DataFrame> {
    let timeperiod = timeperiod.unwrap_or(30);
    let output_col = output_col.unwrap_or("wma");

    let close = get_close(&df)?;
    let close_ca: ChunkedArray<Float64Type> = close.f64().unwrap().clone();
    let close_vals: Vec<f64> = close_ca.into_no_null_iter().collect();

    let wma_vals = calc_wma(&close_vals, timeperiod);

    let wma_series = Series::new(output_col.into(), &wma_vals);
    let mut result_df = df;
    result_df.with_column(wma_series.into())?;
    Ok(result_df)
}

fn calc_wma(values: &[f64], period: usize) -> Vec<f64> {
    let n = values.len();
    let mut result = vec![f64::NAN; n];

    if n < period {
        return result;
    }

    let weight_sum: f64 = (1..=period).map(|w| w as f64).sum();

    for i in (period - 1)..n {
        let mut weighted_sum = 0.0;
        for j in 0..period {
            let weight = (j + 1) as f64;
            weighted_sum += values[i - period + 1 + j] * weight;
        }
        result[i] = weighted_sum / weight_sum;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    //Para los test crear una carpeta llamada download en la raiz de este proyecto
    // y llamar a los datos test.csv
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
    async fn test_adx() {
        match load_data().await {
            Ok(df) => match bbands(df, None, None, None, None, None, None, None).await {
                Ok(result) => {
                    save_data(&result, "download/test_bbands.csv")
                        .await
                        .unwrap();
                }
                Err(e) => panic!("Failed to compute bbands: {:?}", e),
            },
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_dema() {
        match load_data().await {
            Ok(df) => match dema(df, None, None).await {
                Ok(result) => {
                    save_data(&result, "download/test_dema.csv").await.unwrap();
                }
                Err(e) => panic!("Failed to compute dema: {:?}", e),
            },
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_ema() {
        match load_data().await {
            Ok(df) => match ema(df, None, None).await {
                Ok(result) => {
                    save_data(&result, "download/test_ema.csv").await.unwrap();
                }
                Err(e) => panic!("Failed to compute ema: {:?}", e),
            },
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_ht_trendline() {
        match load_data().await {
            Ok(df) => match ht_trendline(df, None).await {
                Ok(result) => {
                    save_data(&result, "download/test_ht_trendline.csv")
                        .await
                        .unwrap();
                }
                Err(e) => panic!("Failed to compute ht_trendline: {:?}", e),
            },
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_kama() {
        match load_data().await {
            Ok(df) => match kama(df, None, None).await {
                Ok(result) => {
                    save_data(&result, "download/test_kama.csv").await.unwrap();
                }
                Err(e) => panic!("Failed to compute kama: {:?}", e),
            },
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_ma() {
        match load_data().await {
            Ok(df) => match ma(df, None, None, None).await {
                Ok(result) => {
                    save_data(&result, "download/test_ma.csv").await.unwrap();
                }
                Err(e) => panic!("Failed to compute ma: {:?}", e),
            },
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_mama() {
        match load_data().await {
            Ok(df) => match mama(df, None, None, None, None).await {
                Ok(result) => {
                    save_data(&result, "download/test_mama.csv").await.unwrap();
                }
                Err(e) => panic!("Failed to compute mama: {:?}", e),
            },
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_mavp() {
        match load_data().await {
            Ok(df) => match mavp(df, None, None, None, None).await {
                Ok(result) => {
                    save_data(&result, "download/test_mavp.csv").await.unwrap();
                }
                Err(e) => panic!("Failed to compute mavp: {:?}", e),
            },
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_midpoint() {
        match load_data().await {
            Ok(df) => match midpoint(df, None, None).await {
                Ok(result) => {
                    save_data(&result, "download/test_midpoint.csv")
                        .await
                        .unwrap();
                }
                Err(e) => panic!("Failed to compute midpoint: {:?}", e),
            },
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_midprice() {
        match load_data().await {
            Ok(df) => match midprice(df, None, None).await {
                Ok(result) => {
                    save_data(&result, "download/test_midprice.csv")
                        .await
                        .unwrap();
                }
                Err(e) => panic!("Failed to compute midprice: {:?}", e),
            },
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_sar() {
        match load_data().await {
            Ok(df) => match sar(df, None, None, None).await {
                Ok(result) => {
                    save_data(&result, "download/test_sar.csv").await.unwrap();
                }
                Err(e) => panic!("Failed to compute sar: {:?}", e),
            },
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_sarext() {
        match load_data().await {
            Ok(df) => match sarext(df, None, None, None, None, None, None).await {
                Ok(result) => {
                    save_data(&result, "download/test_sarext.csv")
                        .await
                        .unwrap();
                }
                Err(e) => panic!("Failed to compute sarext: {:?}", e),
            },
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_sma() {
        match load_data().await {
            Ok(df) => match sma(df, None, None).await {
                Ok(result) => {
                    save_data(&result, "download/test_sma.csv").await.unwrap();
                }
                Err(e) => panic!("Failed to compute sma: {:?}", e),
            },
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_t3() {
        match load_data().await {
            Ok(df) => match t3(df, None, None, None).await {
                Ok(result) => {
                    save_data(&result, "download/test_t3.csv").await.unwrap();
                }
                Err(e) => panic!("Failed to compute t3: {:?}", e),
            },
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_tema() {
        match load_data().await {
            Ok(df) => match tema(df, None, None).await {
                Ok(result) => {
                    save_data(&result, "download/test_tema.csv").await.unwrap();
                }
                Err(e) => panic!("Failed to compute tema: {:?}", e),
            },
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_trima() {
        match load_data().await {
            Ok(df) => match trima(df, None, None).await {
                Ok(result) => {
                    save_data(&result, "download/test_trima.csv").await.unwrap();
                }
                Err(e) => panic!("Failed to compute trima: {:?}", e),
            },
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_wma() {
        match load_data().await {
            Ok(df) => match wma(df, None, None).await {
                Ok(result) => {
                    save_data(&result, "download/test_wma.csv").await.unwrap();
                }
                Err(e) => panic!("Failed to compute wma: {:?}", e),
            },
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }
}
