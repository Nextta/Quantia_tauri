use polars::prelude::*;
use serde::{Deserialize, Serialize};

// Lista de indicadores:
// BBANDS               Bollinger Bands
// DEMA                 Double Exponential Moving Average
// EMA                  Exponential Moving Average
// KAMA                 Kaufman Adaptive Moving Average
// MA                   Moving average
// MAMA                 MESA Adaptive Moving Average
// MIDPOINT             MidPoint over period
// MIDPRICE             Midpoint Price over period
// SAR                  Parabolic SAR
// SAREXT               Parabolic SAR - Extended
// SMA                  Simple Moving Average
// T3                   Triple Exponential Moving Average (T3)
// TEMA                 Triple Exponential Moving Average
// TRIMA                Triangular Moving Average
// WMA                  Weighted Moving Average
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

    if n < period || period == 0 {
        return result;
    }

    // Find the first window of 'period' consecutive finite values
    let mut first_valid_idx = None;
    for i in 0..=(n - period) {
        let window = &values[i..i + period];
        if window.iter().all(|v| v.is_finite()) {
            first_valid_idx = Some(i);
            break;
        }
    }

    if let Some(start) = first_valid_idx {
        let mut sum: f64 = values[start..start + period].iter().sum();
        result[start + period - 1] = sum / period as f64;

        for i in (start + period)..n {
            let val = values[i];
            let oldest = values[i - period];

            if val.is_finite() && oldest.is_finite() {
                sum += val - oldest;
                result[i] = sum / period as f64;
            } else {
                // If we encounter a NaN, we need to re-sync or propagate
                // Standard SMA typically propagates NaN
                sum = f64::NAN;
                result[i] = f64::NAN;

                // Try to find the next valid window if we want to be resilient,
                // but standard TA-Lib behavior is to propagate or handle NaNs differently.
                // For now, let's keep it simple and standard (propagate).
            }
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

/// Kaufman Adaptive Moving Average helper
fn calc_kama(values: &[f64], period: usize) -> Vec<f64> {
    let n = values.len();
    let mut kama_vals = vec![f64::NAN; n];

    if n < period {
        return kama_vals;
    }

    let fast_sc = 2.0 / 3.0;
    let slow_sc = 2.0 / 31.0;

    for i in 0..n {
        let start_idx = i.saturating_sub(period - 1);
        if start_idx == 0 && i < period - 1 {
            continue;
        }

        let change = (values[i] - values[start_idx]).abs();
        let mut volatility = 0.0;
        for j in start_idx..i {
            volatility += (values[j + 1] - values[j]).abs();
        }

        if volatility != 0.0 {
            let er = change / volatility;
            let sc = (er * (fast_sc - slow_sc) + slow_sc).powi(2);

            if i == period - 1 {
                let sum: f64 = values[..period].iter().sum();
                kama_vals[i] = sum / period as f64;
            } else if i > period - 1 {
                kama_vals[i] = sc * values[i] + (1.0 - sc) * kama_vals[i - 1];
            }
        } else if i > 0 {
            kama_vals[i] = kama_vals[i - 1];
        }
    }

    kama_vals
}

/// Generic Moving Average helper
fn calc_ma(values: &[f64], period: usize, matype: MAType) -> Vec<f64> {
    match matype {
        MAType::Sma => calc_sma(values, period),
        MAType::Ema => calc_ema(values, period),
        MAType::Wma => calc_wma(values, period),
        MAType::Dema => {
            let ema1 = calc_ema(values, period);
            let ema2 = calc_ema(&ema1, period);
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
            let ema1 = calc_ema(values, period);
            let ema2 = calc_ema(&ema1, period);
            let ema3 = calc_ema(&ema2, period);
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
        MAType::Trima => calc_trima(values, period),
        MAType::Kama => calc_kama(values, period),
        MAType::Mama => {
            let (mama, _) = calc_mama(values, 0.5, 0.05);
            mama
        }
        MAType::T3 => calc_t3(values, period, 0.7),
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct BbandsParams {
    pub timeperiod: usize,
    pub nbdevup: f64,
    pub nbdevdn: f64,
    pub matype: i32,
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
/// Middle = MA(close, timeperiod, matype)
/// Upper = Middle + (nbdevup * StdDev(close, timeperiod))
/// Lower = Middle - (nbdevdn * StdDev(close, timeperiod))
// Convención TA-Lib: parámetros opcionales por indicador.
#[allow(clippy::too_many_arguments)]
pub fn bbands(
    df: &mut DataFrame,
    timeperiod: Option<usize>,
    nbdevup: Option<f64>,
    nbdevdn: Option<f64>,
    matype: Option<i32>,
    output_col_bb_upper: Option<&str>,
    output_col_bb_middle: Option<&str>,
    output_col_bb_lower: Option<&str>,
) {
    let timeperiod = timeperiod.unwrap_or(5);
    let nbdevup = nbdevup.unwrap_or(2.0);
    let nbdevdn = nbdevdn.unwrap_or(2.0);
    let matype = MAType::from_i32(matype.unwrap_or(0));
    let output_col_bb_upper = output_col_bb_upper.unwrap_or("bb_upper");
    let output_col_bb_middle = output_col_bb_middle.unwrap_or("bb_middle");
    let output_col_bb_lower = output_col_bb_lower.unwrap_or("bb_lower");

    let close = get_close(df).unwrap();
    let close_ca: ChunkedArray<Float64Type> = close.f64().unwrap().clone();
    let close_vals: Vec<f64> = close_ca.into_no_null_iter().collect();
    let n = close_vals.len();

    // Middle band uses the selected MAType
    let middle = calc_ma(&close_vals, timeperiod, matype);

    // Standard deviation is ALWAYS calculated using SMA as the mean (classical Bollinger definition)
    let sma_for_stddev = calc_sma(&close_vals, timeperiod);

    let mut upper: Vec<f64> = vec![f64::NAN; n];
    let mut lower: Vec<f64> = vec![f64::NAN; n];

    for i in 0..n {
        if i >= timeperiod - 1 && !middle[i].is_nan() && !sma_for_stddev[i].is_nan() {
            let start = i.saturating_sub(timeperiod - 1);
            let window = &close_vals[start..=i];
            let mean = sma_for_stddev[i];
            let variance: f64 =
                window.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / timeperiod as f64;
            let stddev = variance.sqrt();

            upper[i] = middle[i] + nbdevup * stddev;
            lower[i] = middle[i] - nbdevdn * stddev;
        }
    }

    let upper_series = Series::new(output_col_bb_upper.into(), &upper);
    let middle_series = Series::new(output_col_bb_middle.into(), &middle);
    let lower_series = Series::new(output_col_bb_lower.into(), &lower);

    df.with_column(upper_series.into())
        .unwrap()
        .with_column(middle_series.into())
        .unwrap()
        .with_column(lower_series.into())
        .unwrap();
}

#[derive(Deserialize, Serialize, Debug)]
pub struct DemaParams {
    pub timeperiod: usize,
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
pub fn dema(df: &mut DataFrame, timeperiod: Option<usize>, output_col: Option<&str>) {
    let timeperiod = timeperiod.unwrap_or(30);
    let output_col = output_col.unwrap_or("dema");

    let close = get_close(df).unwrap();
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
    df.with_column(dema_series.into()).unwrap();
}

#[derive(Deserialize, Serialize, Debug)]
pub struct EmaParams {
    pub timeperiod: usize,
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
pub fn ema(df: &mut DataFrame, timeperiod: Option<usize>, output_col: Option<&str>) {
    let timeperiod = timeperiod.unwrap_or(30);
    let output_col = output_col.unwrap_or("ema");

    let close = get_close(df).unwrap();
    let close_ca: ChunkedArray<Float64Type> = close.f64().unwrap().clone();
    let close_vals: Vec<f64> = close_ca.into_no_null_iter().collect();

    let ema_vals = calc_ema(&close_vals, timeperiod);

    let ema_series = Series::new(output_col.into(), &ema_vals);
    df.with_column(ema_series.into()).unwrap();
}

#[derive(Deserialize, Serialize, Debug)]
pub struct KamaParams {
    pub timeperiod: usize,
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
pub fn kama(df: &mut DataFrame, timeperiod: Option<usize>, output_col: Option<&str>) {
    let timeperiod = timeperiod.unwrap_or(30);
    let output_col = output_col.unwrap_or("kama");

    let close = get_close(df).unwrap();
    let close_ca: ChunkedArray<Float64Type> = close.f64().unwrap().clone();
    let close_vals: Vec<f64> = close_ca.into_no_null_iter().collect();

    let kama_vals = calc_kama(&close_vals, timeperiod);

    let kama_series = Series::new(output_col.into(), &kama_vals);
    df.with_column(kama_series.into()).unwrap();
}

#[derive(Deserialize, Serialize, Debug)]
pub struct MaParams {
    pub timeperiod: usize,
    pub matype: i32,
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
pub fn ma(
    df: &mut DataFrame,
    timeperiod: Option<usize>,
    matype: Option<i32>,
    output_col: Option<&str>,
) {
    let timeperiod = timeperiod.unwrap_or(30);
    let matype = matype.unwrap_or(0);
    let output_col = output_col.unwrap_or("ma");

    let close = get_close(df).unwrap();
    let close_ca: ChunkedArray<Float64Type> = close.f64().unwrap().clone();
    let close_vals: Vec<f64> = close_ca.into_no_null_iter().collect();

    let ma_vals = calc_ma(&close_vals, timeperiod, MAType::from_i32(matype));

    let ma_series = Series::new(output_col.into(), &ma_vals);
    df.with_column(ma_series.into()).unwrap();
}

#[derive(Deserialize, Serialize, Debug)]
pub struct MamaParams {
    pub fastlimit: f64,
    pub slowlimit: f64,
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
pub fn mama(
    df: &mut DataFrame,
    fastlimit: Option<f64>,
    slowlimit: Option<f64>,
    output_col_mama: Option<&str>,
    output_col_fama: Option<&str>,
) {
    let fastlimit = fastlimit.unwrap_or(0.5);
    let slowlimit = slowlimit.unwrap_or(0.05);
    let output_col_mama = output_col_mama.unwrap_or("mama");
    let output_col_fama = output_col_fama.unwrap_or("fama");

    let close = get_close(df).unwrap();
    let close_ca: ChunkedArray<Float64Type> = close.f64().unwrap().clone();
    let close_vals: Vec<f64> = close_ca.into_no_null_iter().collect();

    let (mama_vals, fama_vals) = calc_mama(&close_vals, fastlimit, slowlimit);

    let mama_series = Series::new(output_col_mama.into(), &mama_vals);
    let fama_series = Series::new(output_col_fama.into(), &fama_vals);

    df.with_column(mama_series.into())
        .unwrap()
        .with_column(fama_series.into())
        .unwrap();
}

fn calc_mama(values: &[f64], fastlimit: f64, slowlimit: f64) -> (Vec<f64>, Vec<f64>) {
    let n = values.len();
    let mut mama_vals = vec![f64::NAN; n];
    let mut fama_vals = vec![f64::NAN; n];

    if n < 30 {
        return (mama_vals, fama_vals);
    }

    let mut price_buf = [0.0f64; 4];
    let mut smoothed_buf = [0.0f64; 7];
    let mut detrender_buf = [0.0f64; 7];
    let mut q1_buf = [0.0f64; 7];
    let mut i1_buf = [0.0f64; 7];

    let mut prev_i2 = 0.0;
    let mut prev_q2 = 0.0;
    let mut re = 0.0;
    let mut im = 0.0;
    let mut prev_period = 0.0;
    let mut prev_smooth_period = 0.0;
    let mut prev_phase = 0.0;
    let mut mama = 0.0;
    let mut fama = 0.0;
    let mut initialized = false;

    let a = 0.0962;
    let b = 0.5769;

    for (i, &price) in values.iter().enumerate() {
        // Shift buffers
        for j in (1..4).rev() {
            price_buf[j] = price_buf[j - 1];
        }
        price_buf[0] = price;

        let smoothed =
            (price_buf[0] + 2.0 * price_buf[1] + 2.0 * price_buf[2] + price_buf[3]) / 6.0;

        for j in (1..7).rev() {
            smoothed_buf[j] = smoothed_buf[j - 1];
        }
        smoothed_buf[0] = smoothed;

        if i < 6 {
            continue;
        }

        let mult = 0.075 * prev_period + 0.54;
        let detrender =
            (a * smoothed_buf[0] + b * smoothed_buf[2] - b * smoothed_buf[4] - a * smoothed_buf[6])
                * mult;

        for j in (1..7).rev() {
            detrender_buf[j] = detrender_buf[j - 1];
        }
        detrender_buf[0] = detrender;

        let q1 = (a * detrender_buf[0] + b * detrender_buf[2]
            - b * detrender_buf[4]
            - a * detrender_buf[6])
            * mult;

        for j in (1..7).rev() {
            q1_buf[j] = q1_buf[j - 1];
        }
        q1_buf[0] = q1;

        let i1 = detrender_buf[3];
        for j in (1..7).rev() {
            i1_buf[j] = i1_buf[j - 1];
        }
        i1_buf[0] = i1;

        let j_i = (a * i1_buf[0] + b * i1_buf[2] - b * i1_buf[4] - a * i1_buf[6]) * mult;
        let j_q = (a * q1_buf[0] + b * q1_buf[2] - b * q1_buf[4] - a * q1_buf[6]) * mult;

        let i2 = i1 - j_q;
        let q2 = q1 + j_i;

        let smoothed_i2 = 0.2 * i2 + 0.8 * prev_i2;
        let smoothed_q2 = 0.2 * q2 + 0.8 * prev_q2;

        re = 0.2 * (smoothed_i2 * prev_i2 + smoothed_q2 * prev_q2) + 0.8 * re;
        im = 0.2 * (smoothed_i2 * prev_q2 - smoothed_q2 * prev_i2) + 0.8 * im;

        prev_i2 = smoothed_i2;
        prev_q2 = smoothed_q2;

        if i >= 12 {
            let temp_period = if im != 0.0 && re != 0.0 {
                360.0 / (im / re).atan().to_degrees()
            } else {
                prev_period
            };

            let mut bounded_period = if prev_period > 0.0 {
                temp_period.clamp(0.67 * prev_period, 1.5 * prev_period)
            } else {
                temp_period
            };
            bounded_period = bounded_period.clamp(6.0, 50.0);

            let period_filtered = 0.2 * bounded_period + 0.8 * prev_period;
            let smooth_period = 0.33 * period_filtered + 0.67 * prev_smooth_period;

            prev_period = period_filtered;
            prev_smooth_period = smooth_period;

            // Phase calculation
            let phase = if i1.abs() > 0.0 {
                (q1 / i1).atan().to_degrees()
            } else {
                0.0
            };

            let mut delta_phase = prev_phase - phase;
            if prev_phase < phase {
                delta_phase = 360.0 + prev_phase - phase;
            }
            if delta_phase < 1.0 {
                delta_phase = 1.0;
            }

            let mut alpha = fastlimit / delta_phase;
            if alpha < slowlimit {
                alpha = slowlimit;
            }
            if alpha > fastlimit {
                alpha = fastlimit;
            }

            if !initialized {
                mama = price;
                fama = price;
                initialized = true;
            } else {
                mama = alpha * price + (1.0 - alpha) * mama;
                fama = 0.5 * alpha * mama + (1.0 - 0.5 * alpha) * fama;
            }

            prev_phase = phase;

            if i >= 30 {
                mama_vals[i] = mama;
                fama_vals[i] = fama;
            }
        }
    }

    (mama_vals, fama_vals)
}

#[derive(Deserialize, Serialize, Debug)]
pub struct MidpointParams {
    pub timeperiod: usize,
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
pub fn midpoint(df: &mut DataFrame, timeperiod: Option<usize>, output_col: Option<&str>) {
    let timeperiod = timeperiod.unwrap_or(14);
    let output_col = output_col.unwrap_or("midpoint");

    let close = get_close(df).unwrap();
    let close_ca = close.f64().unwrap();
    // Maintain original length by including nulls as NaN
    let close_vals: Vec<f64> = close_ca
        .into_iter()
        .map(|opt| opt.unwrap_or(f64::NAN))
        .collect();
    let n = close_vals.len();

    let mut midpoint_vals: Vec<f64> = vec![f64::NAN; n];

    if n >= timeperiod {
        for i in (timeperiod - 1)..n {
            let start = i + 1 - timeperiod;
            let window = &close_vals[start..=i];

            let mut max_val = f64::NEG_INFINITY;
            let mut min_val = f64::INFINITY;
            let mut has_valid = false;

            for &val in window {
                if val.is_finite() {
                    if val > max_val {
                        max_val = val;
                    }
                    if val < min_val {
                        min_val = val;
                    }
                    has_valid = true;
                }
            }

            if has_valid {
                midpoint_vals[i] = (max_val + min_val) / 2.0;
            }
        }
    }

    let midpoint_series = Series::new(output_col.into(), &midpoint_vals);

    df.with_column(midpoint_series.into()).unwrap();
}

#[derive(Deserialize, Serialize, Debug)]
pub struct MidpriceParams {
    pub timeperiod: usize,
}

// ============================================================================
// MIDPRICE - Midprice over period
// ============================================================================

/// MIDPRICE - Midprice over period
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
pub fn midprice(df: &mut DataFrame, timeperiod: Option<usize>, output_col: Option<&str>) {
    let timeperiod = timeperiod.unwrap_or(14);
    let output_col = output_col.unwrap_or("midprice");

    let high = get_high(df).unwrap();
    let low = get_low(df).unwrap();

    let high_ca = high.f64().unwrap();
    let low_ca = low.f64().unwrap();

    // Maintain original length by including nulls as NaN and ensuring alignment
    let high_vals: Vec<f64> = high_ca
        .into_iter()
        .map(|opt| opt.unwrap_or(f64::NAN))
        .collect();
    let low_vals: Vec<f64> = low_ca
        .into_iter()
        .map(|opt| opt.unwrap_or(f64::NAN))
        .collect();

    let n = high_vals.len();
    let mut midprice_vals: Vec<f64> = vec![f64::NAN; n];

    if n >= timeperiod {
        for i in (timeperiod - 1)..n {
            let start = i + 1 - timeperiod;
            let high_window = &high_vals[start..=i];
            let low_window = &low_vals[start..=i];

            let mut max_high = f64::NEG_INFINITY;
            let mut min_low = f64::INFINITY;
            let mut has_valid = false;

            for j in 0..timeperiod {
                let h = high_window[j];
                let l = low_window[j];

                if h.is_finite() {
                    if h > max_high {
                        max_high = h;
                    }
                    has_valid = true;
                }
                if l.is_finite() {
                    if l < min_low {
                        min_low = l;
                    }
                    has_valid = true;
                }
            }

            if has_valid {
                midprice_vals[i] = (max_high + min_low) / 2.0;
            }
        }
    }

    let midprice_series = Series::new(output_col.into(), &midprice_vals);
    df.with_column(midprice_series.into()).unwrap();
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SarParams {
    pub acceleration: f64,
    pub maximum: f64,
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
pub fn sar(
    df: &mut DataFrame,
    acceleration: Option<f64>,
    maximum: Option<f64>,
    output_col: Option<&str>,
) {
    let acceleration = acceleration.unwrap_or(0.02);
    let maximum = maximum.unwrap_or(0.2);
    let output_col = output_col.unwrap_or("sar");

    let high = get_high(df).unwrap();
    let low = get_low(df).unwrap();

    let high_ca: ChunkedArray<Float64Type> = high.f64().unwrap().clone();
    let low_ca: ChunkedArray<Float64Type> = low.f64().unwrap().clone();
    let high_vals: Vec<f64> = high_ca.into_no_null_iter().collect();
    let low_vals: Vec<f64> = low_ca.into_no_null_iter().collect();
    let n = high_vals.len();

    let mut sar_vals: Vec<f64> = vec![f64::NAN; n];

    if n < 2 {
        let sar_series = Series::new(output_col.into(), &sar_vals);
        df.with_column(sar_series.into()).unwrap();
        return;
    }

    let mut af = acceleration;
    let mut is_long = high_vals[1] > high_vals[0];
    let mut sar = if is_long { low_vals[0] } else { high_vals[0] };
    let mut ep = if is_long { high_vals[1] } else { low_vals[1] };

    // Initial values
    sar_vals[0] = sar;
    sar_vals[1] = sar;

    for i in 2..n {
        // 1. Calculate next SAR (tentative)
        let mut next_sar = sar + af * (ep - sar);

        // 2. Apply constraints: SAR cannot be higher than low of last two bars (long)
        // or lower than high of last two bars (short)
        if is_long {
            next_sar = next_sar.min(low_vals[i - 1]).min(low_vals[i - 2]);
        } else {
            next_sar = next_sar.max(high_vals[i - 1]).max(high_vals[i - 2]);
        }

        // 3. Check for reversal
        if is_long {
            if low_vals[i] < next_sar {
                // Reversal: Long to Short
                is_long = false;
                sar = ep; // New SAR is the highest high of the previous trend
                ep = low_vals[i];
                af = acceleration;
            } else {
                // Continue Long
                sar = next_sar;
                if high_vals[i] > ep {
                    ep = high_vals[i];
                    af = (af + acceleration).min(maximum);
                }
            }
        } else {
            if high_vals[i] > next_sar {
                // Reversal: Short to Long
                is_long = true;
                sar = ep; // New SAR is the lowest low of the previous trend
                ep = high_vals[i];
                af = acceleration;
            } else {
                // Continue Short
                sar = next_sar;
                if low_vals[i] < ep {
                    ep = low_vals[i];
                    af = (af + acceleration).min(maximum);
                }
            }
        }

        sar_vals[i] = sar;
    }

    let sar_series = Series::new(output_col.into(), &sar_vals);

    df.with_column(sar_series.into()).unwrap();
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SarextParams {
    pub startvalue: f64,
    pub offsetonlong: f64,
    pub offsetonshort: f64,
    pub blockonlong: f64,
    pub blockonshort: f64,
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
pub fn sarext(
    df: &mut DataFrame,
    startvalue: Option<f64>,
    offsetonlong: Option<f64>,
    offsetonshort: Option<f64>,
    blockonlong: Option<f64>,
    blockonshort: Option<f64>,
    output_col: Option<&str>,
) {
    let startvalue = startvalue.unwrap_or(0.02);
    let offsetonlong = offsetonlong.unwrap_or(0.0);
    let offsetonshort = offsetonshort.unwrap_or(0.0);
    let _blockonlong = blockonlong.unwrap_or(0.0);
    let _blockonshort = blockonshort.unwrap_or(0.0);
    let output_col = output_col.unwrap_or("sarext");

    let high = get_high(df).unwrap();
    let low = get_low(df).unwrap();

    let high_ca: ChunkedArray<Float64Type> = high.f64().unwrap().clone();
    let low_ca: ChunkedArray<Float64Type> = low.f64().unwrap().clone();
    let high_vals: Vec<f64> = high_ca.into_no_null_iter().collect();
    let low_vals: Vec<f64> = low_ca.into_no_null_iter().collect();
    let n = high_vals.len();

    let mut sarext_vals: Vec<f64> = vec![f64::NAN; n];

    if n < 2 {
        let sarext_series = Series::new(output_col.into(), &sarext_vals);
        df.with_column(sarext_series.into()).unwrap();
        return;
    }

    let mut af = startvalue;
    let mut is_long = high_vals[1] > high_vals[0];
    let mut sar = if is_long { low_vals[0] } else { high_vals[0] };
    let mut ep = if is_long { high_vals[1] } else { low_vals[1] };

    // Initial values
    sarext_vals[0] = sar;
    sarext_vals[1] = sar;

    for i in 2..n {
        // 1. Calculate next SAR
        let mut next_sar = sar + af * (ep - sar);

        // 2. Apply constraints and offsets
        if is_long {
            next_sar = next_sar.min(low_vals[i - 1]).min(low_vals[i - 2]);
            next_sar -= offsetonlong;
        } else {
            next_sar = next_sar.max(high_vals[i - 1]).max(high_vals[i - 2]);
            next_sar += offsetonshort;
        }

        // 3. Check for reversal
        if is_long {
            if low_vals[i] < next_sar {
                // Reversal: Long to Short
                is_long = false;
                sar = ep + offsetonshort;
                ep = low_vals[i];
                af = startvalue;
            } else {
                // Continue Long
                sar = next_sar;
                if high_vals[i] > ep {
                    ep = high_vals[i];
                    af = (af + startvalue).min(0.2);
                }
            }
        } else {
            if high_vals[i] > next_sar {
                // Reversal: Short to Long
                is_long = true;
                sar = ep - offsetonlong;
                ep = high_vals[i];
                af = startvalue;
            } else {
                // Continue Short
                sar = next_sar;
                if low_vals[i] < ep {
                    ep = low_vals[i];
                    af = (af + startvalue).min(0.2);
                }
            }
        }

        sarext_vals[i] = sar;
    }

    let sarext_series = Series::new(output_col.into(), &sarext_vals);

    df.with_column(sarext_series.into()).unwrap();
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SmaParams {
    pub timeperiod: usize,
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
pub fn sma(df: &mut DataFrame, timeperiod: Option<usize>, output_col: Option<&str>) {
    let timeperiod = timeperiod.unwrap_or(30);
    let output_col = output_col.unwrap_or("sma");

    let close = get_close(df).unwrap();
    let close_ca: ChunkedArray<Float64Type> = close.f64().unwrap().clone();
    let close_vals: Vec<f64> = close_ca.into_no_null_iter().collect();

    let sma_vals = calc_sma(&close_vals, timeperiod);

    let sma_series = Series::new(output_col.into(), &sma_vals);

    df.with_column(sma_series.into()).unwrap();
}

#[derive(Deserialize, Serialize, Debug)]
pub struct T3Params {
    pub timeperiod: usize,
    pub vfactor: f64,
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
/// e1 = EMA(price, period)
/// e2 = EMA(e1, period)
/// e3 = EMA(e2, period)
/// e4 = EMA(e3, period)
/// e5 = EMA(e4, period)
/// e6 = EMA(e5, period)
/// T3 = c1*e6 + c2*e5 + c3*e4 + c4*e3
/// donde:
/// a = vfactor
/// c1 = -a^3
/// c2 = 3a^2 + 3a^3
/// c3 = -6a^2 - 3a - 3a^3
/// c4 = 1 + 3a + 3a^2 + a^3
pub fn t3(
    df: &mut DataFrame,
    timeperiod: Option<usize>,
    vfactor: Option<f64>,
    output_col: Option<&str>,
) {
    let timeperiod = timeperiod.unwrap_or(5);
    let vfactor = vfactor.unwrap_or(0.7);
    let output_col = output_col.unwrap_or("t3");

    let close = get_close(df).unwrap();
    let close_ca: ChunkedArray<Float64Type> = close.f64().unwrap().clone();
    let close_vals: Vec<f64> = close_ca.into_no_null_iter().collect();

    let t3_vals = calc_t3(&close_vals, timeperiod, vfactor);

    let t3_series = Series::new(output_col.into(), &t3_vals);
    df.with_column(t3_series.into()).unwrap();
}

fn calc_t3(values: &[f64], period: usize, vfactor: f64) -> Vec<f64> {
    let n = values.len();
    let mut result = vec![f64::NAN; n];

    if n < period || period == 0 {
        return result;
    }

    let k = 2.0 / (period as f64 + 1.0);
    let a = vfactor;
    let c1 = -(a.powi(3));
    let c2 = 3.0 * a.powi(2) + 3.0 * a.powi(3);
    let c3 = -6.0 * a.powi(2) - 3.0 * a - 3.0 * a.powi(3);
    let c4 = 1.0 + 3.0 * a + a.powi(3) + 3.0 * a.powi(2);

    // Find first valid window
    let mut first_valid_idx = None;
    for i in 0..=(n - period) {
        if values[i..i + period].iter().all(|v| v.is_finite()) {
            first_valid_idx = Some(i);
            break;
        }
    }

    if let Some(start) = first_valid_idx {
        // Initialize with SMA
        let sma: f64 = values[start..start + period].iter().sum::<f64>() / period as f64;
        let mut e1 = sma;
        let mut e2 = sma;
        let mut e3 = sma;
        let mut e4 = sma;
        let mut e5 = sma;
        let mut e6 = sma;

        // The first T3 value is at the end of the first window
        result[start + period - 1] = c1 * e6 + c2 * e5 + c3 * e4 + c4 * e3;

        // Loop through the rest
        for i in (start + period)..n {
            if values[i].is_finite() {
                e1 = e1 + k * (values[i] - e1);
                e2 = e2 + k * (e1 - e2);
                e3 = e3 + k * (e2 - e3);
                e4 = e4 + k * (e3 - e4);
                e5 = e5 + k * (e4 - e5);
                e6 = e6 + k * (e5 - e6);
                result[i] = c1 * e6 + c2 * e5 + c3 * e4 + c4 * e3;
            } else {
                result[i] = f64::NAN;
            }
        }
    }

    result
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct TemaParams {
    pub timeperiod: usize,
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
pub fn tema(df: &mut DataFrame, timeperiod: Option<usize>, output_col: Option<&str>) {
    let timeperiod = timeperiod.unwrap_or(30);
    let output_col = output_col.unwrap_or("tema");

    let close = get_close(df).unwrap();
    let close_ca: ChunkedArray<Float64Type> = close.f64().unwrap().clone();
    let close_vals: Vec<f64> = close_ca.into_no_null_iter().collect();

    let n = close_vals.len();
    let mut result = vec![f64::NAN; n];

    if n < timeperiod || timeperiod == 0 {
        let tema_series = Series::new(output_col.into(), &result);
        df.with_column(tema_series.into()).unwrap();
        return;
    }

    let k = 2.0 / (timeperiod as f64 + 1.0);

    // Find first valid window
    let mut first_valid_idx = None;
    for i in 0..=(n - timeperiod) {
        if close_vals[i..i + timeperiod].iter().all(|v| v.is_finite()) {
            first_valid_idx = Some(i);
            break;
        }
    }

    if let Some(start) = first_valid_idx {
        // Initialize with SMA
        let sma: f64 =
            close_vals[start..start + timeperiod].iter().sum::<f64>() / timeperiod as f64;
        let mut e1 = sma;
        let mut e2 = sma;
        let mut e3 = sma;

        // The first TEMA value is at the end of the first window
        result[start + timeperiod - 1] = 3.0 * e1 - 3.0 * e2 + e3;

        // Loop through the rest
        for i in (start + timeperiod)..n {
            if close_vals[i].is_finite() {
                e1 = e1 + k * (close_vals[i] - e1);
                e2 = e2 + k * (e1 - e2);
                e3 = e3 + k * (e2 - e3);
                result[i] = 3.0 * e1 - 3.0 * e2 + e3;
            } else {
                result[i] = f64::NAN;
            }
        }
    }

    let tema_series = Series::new(output_col.into(), &result);

    df.with_column(tema_series.into()).unwrap();
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct TrimaParams {
    pub timeperiod: usize,
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
pub fn trima(df: &mut DataFrame, timeperiod: Option<usize>, output_col: Option<&str>) {
    let timeperiod = timeperiod.unwrap_or(30);
    let output_col = output_col.unwrap_or("trima");

    let close = get_close(df).unwrap();
    let close_ca: ChunkedArray<Float64Type> = close.f64().unwrap().clone();
    let close_vals: Vec<f64> = close_ca.into_no_null_iter().collect();

    let trima_vals = calc_trima(&close_vals, timeperiod);

    let trima_series = Series::new(output_col.into(), &trima_vals);
    df.with_column(trima_series.into()).unwrap();
}

fn calc_trima(values: &[f64], period: usize) -> Vec<f64> {
    let n = values.len();
    if n < period {
        return vec![f64::NAN; n];
    }

    let half1 = period.div_ceil(2);
    let half2 = period / 2 + 1;

    let sma1 = calc_sma(values, half1);

    calc_sma(&sma1, half2)
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct WmaParams {
    pub timeperiod: usize,
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
pub fn wma(df: &mut DataFrame, timeperiod: Option<usize>, output_col: Option<&str>) {
    let timeperiod = timeperiod.unwrap_or(30);
    let output_col = output_col.unwrap_or("wma");

    let close = get_close(df).unwrap();
    let close_ca: ChunkedArray<Float64Type> = close.f64().unwrap().clone();
    let close_vals: Vec<f64> = close_ca.into_no_null_iter().collect();

    if close_vals.len() < timeperiod {
        let wma_series = Series::new(output_col.into(), &vec![f64::NAN; close_vals.len()]);
        df.with_column(wma_series.into()).unwrap();
        return;
    }

    let wma_vals = calc_wma(&close_vals, timeperiod);

    let wma_series = Series::new(output_col.into(), &wma_vals);

    df.with_column(wma_series.into()).unwrap();
}

fn calc_wma(values: &[f64], period: usize) -> Vec<f64> {
    let n = values.len();
    let mut result = vec![f64::NAN; n];

    if n < period || period == 0 {
        return result;
    }

    let weight_sum = (period * (period + 1)) as f64 / 2.0;

    // Find the first window of 'period' consecutive finite values
    let mut first_valid_idx = None;
    for i in 0..=(n - period) {
        if values[i..i + period].iter().all(|v| v.is_finite()) {
            first_valid_idx = Some(i);
            break;
        }
    }

    if let Some(start) = first_valid_idx {
        let mut current_weighted_sum = 0.0;
        let mut current_sum = 0.0;

        for j in 0..period {
            let val = values[start + j];
            current_weighted_sum += val * (j + 1) as f64;
            current_sum += val;
        }

        result[start + period - 1] = current_weighted_sum / weight_sum;

        for i in (start + period)..n {
            let val = values[i];
            let oldest = values[i - period];

            if val.is_finite() && oldest.is_finite() {
                // WMA(t+1) = WMA(t) + Price(t+1)*period - Sum(Price(t-period+1) to Price(t))
                // The current_sum we have is Sum(Price(start) to Price(start+period-1))
                // To move to next bar, we use the formula:
                // NewWeightedSum = OldWeightedSum - OldSum + NewPrice * period
                current_weighted_sum = current_weighted_sum - current_sum + val * period as f64;
                current_sum = current_sum - oldest + val;
                result[i] = current_weighted_sum / weight_sum;
            } else {
                result[i] = f64::NAN;
                // If we hit a NaN, we'd need to re-initialize or propagate.
                // Standard behavior is propagation.
                current_sum = f64::NAN;
                current_weighted_sum = f64::NAN;
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::data_format::DataFormatSymbol;
    use crate::utils::data_test::create_test_data;
    use std::fs::remove_file;

    //Para los test crear una carpeta llamada data en la raiz de este proyecto
    // y llamar a los datos test.csv
    fn load_data() -> PolarsResult<DataFrame> {
        create_test_data(&DataFormatSymbol::Csv);

        let df = CsvReadOptions::default()
            .try_into_reader_with_file_path(Some("data/test.csv".into()))
            .unwrap()
            .finish()
            .unwrap();
        Ok(df)
    }

    // fn save_data(df_result: &DataFrame, path: &str) -> PolarsResult<()> {
    //     let mut df: DataFrame = df_result.clone();
    //     let mut file = std::fs::File::create(path).unwrap();
    //     CsvWriter::new(&mut file).finish(&mut df).unwrap();
    //     Ok(())
    // }

    #[test]
    fn test_bbands() {
        let _guardia = crate::utils::data_test::BLOQUEO_RECURSOS.blocking_lock();
        match load_data() {
            Ok(mut df) => {
                bbands(&mut df, None, None, None, None, None, None, None);
                // save_data(&df, "data/test_bbands.csv").unwrap();
                // remove_file("data/test_bbands.csv").unwrap();
                remove_file("data/test.csv").unwrap();
                remove_file("data/test.parquet").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[test]
    fn test_dema() {
        let _guardia = crate::utils::data_test::BLOQUEO_RECURSOS.blocking_lock();
        match load_data() {
            Ok(mut df) => {
                dema(&mut df, None, None);
                // save_data(&df, "data/test_dema.csv").unwrap();
                // remove_file("data/test_dema.csv").unwrap();
                remove_file("data/test.csv").unwrap();
                remove_file("data/test.parquet").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[test]
    fn test_ema() {
        let _guardia = crate::utils::data_test::BLOQUEO_RECURSOS.blocking_lock();
        match load_data() {
            Ok(mut df) => {
                ema(&mut df, None, None);
                // save_data(&df, "data/test_ema.csv").unwrap();
                // remove_file("data/test_ema.csv").unwrap();
                remove_file("data/test.csv").unwrap();
                remove_file("data/test.parquet").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[test]
    fn test_kama() {
        let _guardia = crate::utils::data_test::BLOQUEO_RECURSOS.blocking_lock();
        match load_data() {
            Ok(mut df) => {
                kama(&mut df, None, None);
                // save_data(&df, "data/test_kama.csv").unwrap();
                // remove_file("data/test_kama.csv").unwrap();
                remove_file("data/test.csv").unwrap();
                remove_file("data/test.parquet").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[test]
    fn test_ma() {
        let _guardia = crate::utils::data_test::BLOQUEO_RECURSOS.blocking_lock();
        match load_data() {
            Ok(mut df) => {
                ma(&mut df, None, None, None);
                // save_data(&df, "data/test_ma.csv").unwrap();
                // remove_file("data/test_ma.csv").unwrap();
                remove_file("data/test.csv").unwrap();
                remove_file("data/test.parquet").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[test]
    fn test_mama() {
        let _guardia = crate::utils::data_test::BLOQUEO_RECURSOS.blocking_lock();
        match load_data() {
            Ok(mut df) => {
                mama(&mut df, None, None, None, None);
                // save_data(&df, "data/test_mama.csv").unwrap();
                // remove_file("data/test_mama.csv").unwrap();
                remove_file("data/test.csv").unwrap();
                remove_file("data/test.parquet").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[test]
    fn test_midpoint() {
        let _guardia = crate::utils::data_test::BLOQUEO_RECURSOS.blocking_lock();
        match load_data() {
            Ok(mut df) => {
                midpoint(&mut df, None, None);
                // save_data(&df, "data/test_midpoint.csv").unwrap();
                // remove_file("data/test_midpoint.csv").unwrap();
                remove_file("data/test.csv").unwrap();
                remove_file("data/test.parquet").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[test]
    fn test_midprice() {
        let _guardia = crate::utils::data_test::BLOQUEO_RECURSOS.blocking_lock();
        match load_data() {
            Ok(mut df) => {
                midprice(&mut df, None, None);
                // save_data(&df, "data/test_midprice.csv").unwrap();
                // remove_file("data/test_midprice.csv").unwrap();
                remove_file("data/test.csv").unwrap();
                remove_file("data/test.parquet").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[test]
    fn test_sar() {
        let _guardia = crate::utils::data_test::BLOQUEO_RECURSOS.blocking_lock();
        match load_data() {
            Ok(mut df) => {
                sar(&mut df, None, None, None);
                // save_data(&df, "data/test_sar.csv").unwrap();
                // remove_file("data/test_sar.csv").unwrap();
                remove_file("data/test.csv").unwrap();
                remove_file("data/test.parquet").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[test]
    fn test_sarext() {
        let _guardia = crate::utils::data_test::BLOQUEO_RECURSOS.blocking_lock();
        match load_data() {
            Ok(mut df) => {
                sarext(&mut df, None, None, None, None, None, None);
                // save_data(&df, "data/test_sarext.csv").unwrap();
                // remove_file("data/test_sarext.csv").unwrap();
                remove_file("data/test.csv").unwrap();
                remove_file("data/test.parquet").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[test]
    fn test_sma() {
        let _guardia = crate::utils::data_test::BLOQUEO_RECURSOS.blocking_lock();
        match load_data() {
            Ok(mut df) => {
                sma(&mut df, None, None);
                // save_data(&df, "data/test_sma.csv").unwrap();
                // remove_file("data/test_sma.csv").unwrap();
                remove_file("data/test.csv").unwrap();
                remove_file("data/test.parquet").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[test]
    fn test_t3() {
        let _guardia = crate::utils::data_test::BLOQUEO_RECURSOS.blocking_lock();
        match load_data() {
            Ok(mut df) => {
                t3(&mut df, None, None, None);
                // save_data(&df, "data/test_t3.csv").unwrap();
                // remove_file("data/test_t3.csv").unwrap();
                remove_file("data/test.csv").unwrap();
                remove_file("data/test.parquet").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[test]
    fn test_tema() {
        let _guardia = crate::utils::data_test::BLOQUEO_RECURSOS.blocking_lock();
        match load_data() {
            Ok(mut df) => {
                tema(&mut df, None, None);
                // save_data(&df, "data/test_tema.csv").unwrap();
                // remove_file("data/test_tema.csv").unwrap();
                remove_file("data/test.csv").unwrap();
                remove_file("data/test.parquet").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[test]
    fn test_trima() {
        let _guardia = crate::utils::data_test::BLOQUEO_RECURSOS.blocking_lock();
        match load_data() {
            Ok(mut df) => {
                trima(&mut df, None, None);
                // save_data(&df, "data/test_trima.csv").unwrap();
                // remove_file("data/test_trima.csv").unwrap();
                remove_file("data/test.csv").unwrap();
                remove_file("data/test.parquet").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[test]
    fn test_wma() {
        let _guardia = crate::utils::data_test::BLOQUEO_RECURSOS.blocking_lock();
        match load_data() {
            Ok(mut df) => {
                wma(&mut df, None, None);
                // save_data(&df, "data/test_wma.csv").unwrap();
                // remove_file("data/test_wma.csv").unwrap();
                remove_file("data/test.csv").unwrap();
                remove_file("data/test.parquet").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }
}
