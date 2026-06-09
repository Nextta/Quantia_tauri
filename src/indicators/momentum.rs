use polars::prelude::*;
use serde::{Deserialize, Serialize};

/// Lista de indicadores:
/// ADX                  Average Directional Movement Index
/// ADXR                 Average Directional Movement Index Rating
/// APO                  Absolute Price Oscillator
/// AROON                Aroon
/// AROONOSC             Aroon Oscillator
/// BOP                  Balance Of Power
/// CCI                  Commodity Channel Index
/// CMO                  Chande Momentum Oscillator
/// DX                   Directional Movement Index
/// MACD                 Moving Average Convergence/Divergence
/// MACDEXT              MACD with controllable MA type
/// MACDFIX              Moving Average Convergence/Divergence Fix 12/26
/// MFI                  Money Flow Index
/// MINUS_DI             Minus Directional Indicator
/// MINUS_DM             Minus Directional Movement
/// MOM                  Momentum
/// PLUS_DI              Plus Directional Indicator
/// PLUS_DM              Plus Directional Movement
/// PPO                  Percentage Price Oscillator
/// ROC                  Rate of change : ((price/prevPrice)-1)*100
/// ROCP                 Rate of change Percentage: (price-prevPrice)/prevPrice
/// ROCR                 Rate of change ratio: (price/prevPrice)
/// ROCR100              Rate of change ratio 100 scale: (price/prevPrice)*100
/// RSI                  Relative Strength Index
/// STOCH                Stochastic
/// STOCHF               Stochastic Fast
/// STOCHRSI             Stochastic Relative Strength Index
/// TRIX                 1-day Rate-Of-Change (ROC) of a Triple Smooth EMA
/// ULTOSC               Ultimate Oscillator
/// WILLR                Williams' %R

// Helper function: Exponential Moving Average
fn ema_series(values: &Series, period: usize) -> Series {
    let multiplier = 2.0 / (period as f64 + 1.0);
    let ca: ChunkedArray<Float64Type> = values.f64().unwrap().clone();
    let n = ca.len();
    let mut ema_values: Vec<f64> = vec![f64::NAN; n];

    // Find the first `period` valid values to initialize
    let mut valid_count = 0;
    let mut init_sum: f64 = 0.0;
    let mut start_idx = 0;

    for i in 0..n {
        if let Some(val) = ca.get(i) {
            if !val.is_nan() {
                init_sum += val;
                valid_count += 1;

                if valid_count == period {
                    // Initialize EMA with SMA
                    ema_values[i] = init_sum / period as f64;
                    start_idx = i;
                    break;
                }
            }
        }
    }

    // If we couldn't initialize, return all NaN
    if valid_count < period {
        return Series::new("ema".into(), &ema_values);
    }

    // Continue EMA from start_idx + 1
    let mut current_ema = ema_values[start_idx];
    for i in (start_idx + 1)..n {
        if let Some(val) = ca.get(i) {
            if !val.is_nan() {
                current_ema = val * multiplier + current_ema * (1.0 - multiplier);
                ema_values[i] = current_ema;
            }
        }
    }

    Series::new("ema".into(), &ema_values)
}

// Helper function: Simple Moving Average
fn sma_series(values: &Series, period: usize) -> Series {
    let ca: ChunkedArray<Float64Type> = values.f64().unwrap().clone();
    let n = ca.len();
    let mut sma_values: Vec<f64> = vec![f64::NAN; n];

    for i in (period - 1)..n {
        let mut sum = 0.0;
        let mut valid = true;
        let start_j = i + 1 - period;
        for j in start_j..=i {
            if let Some(val) = ca.get(j) {
                if !val.is_nan() {
                    sum += val;
                } else {
                    valid = false;
                    break;
                }
            } else {
                valid = false;
                break;
            }
        }

        if valid {
            sma_values[i] = sum / period as f64;
        }
    }

    Series::new("sma".into(), &sma_values)
}

// Helper function: Wilder's RMA (Running Moving Average)
fn rma_series(values: &Series, period: usize) -> Series {
    let alpha = 1.0 / period as f64;
    let ca: ChunkedArray<Float64Type> = values.f64().unwrap().clone();
    let n = ca.len();
    let mut rma_values: Vec<f64> = vec![f64::NAN; n];

    // Find the first `period` valid values to initialize
    let mut valid_count = 0;
    let mut init_sum: f64 = 0.0;
    let mut start_idx = 0;

    for i in 0..n {
        if let Some(val) = ca.get(i) {
            if !val.is_nan() {
                init_sum += val;
                valid_count += 1;

                if valid_count == period {
                    rma_values[i] = init_sum / period as f64;
                    start_idx = i;
                    break;
                }
            }
        }
    }

    if valid_count < period {
        return Series::new("rma".into(), &rma_values);
    }

    // Continue RMA from start_idx + 1
    let mut current_rma = rma_values[start_idx];
    for i in (start_idx + 1)..n {
        if let Some(val) = ca.get(i) {
            if !val.is_nan() {
                current_rma = val * alpha + current_rma * (1.0 - alpha);
                rma_values[i] = current_rma;
            }
        }
    }

    Series::new("rma".into(), &rma_values)
}

// Helper: Get column as f64 Series
fn get_close(df: &DataFrame) -> PolarsResult<Series> {
    let s = df.column("close").or_else(|_| df.column("Close"))?;
    Ok(s.cast(&DataType::Float64)?.take_materialized_series())
}

fn get_high(df: &DataFrame) -> PolarsResult<Series> {
    let s = df.column("high").or_else(|_| df.column("High"))?;
    Ok(s.cast(&DataType::Float64)?.take_materialized_series())
}

fn get_low(df: &DataFrame) -> PolarsResult<Series> {
    let s = df.column("low").or_else(|_| df.column("Low"))?;
    Ok(s.cast(&DataType::Float64)?.take_materialized_series())
}

fn get_open(df: &DataFrame) -> PolarsResult<Series> {
    let s = df.column("open").or_else(|_| df.column("Open"))?;
    Ok(s.cast(&DataType::Float64)?.take_materialized_series())
}

fn get_volume(df: &DataFrame) -> PolarsResult<Series> {
    let s = df.column("volume").or_else(|_| df.column("Volume"))?;
    Ok(s.cast(&DataType::Float64)?.take_materialized_series())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdxParams {
    pub timeperiod: usize,
}

/// ADX - Average Directional Movement Index
///
/// Mide la fuerza de la tendencia actual, independientemente de su dirección.
///
/// # Parámetros
/// * `df` - DataFrame con columnas: high, low, close (case insensitive)
/// * `timeperiod` - Período de cálculo (default: 14)
/// * `output_col` - Nombre de la columna de salida (default: "adx")
///
/// # Retorna
/// DataFrame con columna "adx" añadida
///
/// # Fórmula
/// ADX = EMA(DX), donde DX = ((+|DI| - |DI|) / (+|DI| + |DI|)) * 100
pub fn adx(df: &mut DataFrame, timeperiod: Option<usize>, output_col: Option<&str>) {
    let timeperiod = timeperiod.unwrap_or(14);
    let output_name = output_col.unwrap_or("adx");
    let high = get_high(&df).unwrap();
    let low = get_low(&df).unwrap();
    let close = get_close(&df).unwrap();

    let high_vals = high.f64().unwrap();
    let low_vals = low.f64().unwrap();
    let close_vals = close.f64().unwrap();

    let n = high_vals.len();

    if n < (timeperiod * 2) {
        df.with_column(Series::new(output_name.into(), vec![f64::NAN; n]).into())
            .unwrap();
        return;
    }

    let mut plus_dm = vec![0.0; n];
    let mut minus_dm = vec![0.0; n];
    let mut tr = vec![0.0; n];

    // First bar TR calculation
    if n > 0 {
        tr[0] = high_vals.get(0).unwrap() - low_vals.get(0).unwrap();
    }

    for i in 1..n {
        let h_curr = high_vals.get(i).unwrap();
        let h_prev = high_vals.get(i - 1).unwrap();
        let l_curr = low_vals.get(i).unwrap();
        let l_prev = low_vals.get(i - 1).unwrap();
        let c_prev = close_vals.get(i - 1).unwrap();

        let high_diff = h_curr - h_prev;
        let low_diff = l_prev - l_curr;

        if high_diff > low_diff && high_diff > 0.0 {
            plus_dm[i] = high_diff;
        } else {
            plus_dm[i] = 0.0;
        }

        if low_diff > high_diff && low_diff > 0.0 {
            minus_dm[i] = low_diff;
        } else {
            minus_dm[i] = 0.0;
        }

        let tr1 = h_curr - l_curr;
        let tr2 = (h_curr - c_prev).abs();
        let tr3 = (l_curr - c_prev).abs();
        tr[i] = tr1.max(tr2).max(tr3);
    }

    let tr_series = Series::new("tr".into(), tr);
    let plus_dm_series = Series::new("plus_dm".into(), plus_dm);
    let minus_dm_series = Series::new("minus_dm".into(), minus_dm);

    let smoothed_tr = rma_series(&tr_series, timeperiod);
    let smoothed_plus_dm = rma_series(&plus_dm_series, timeperiod);
    let smoothed_minus_dm = rma_series(&minus_dm_series, timeperiod);

    let tr_v = smoothed_tr.f64().unwrap();
    let pdm_v = smoothed_plus_dm.f64().unwrap();
    let mdm_v = smoothed_minus_dm.f64().unwrap();

    let mut dx_vals = vec![f64::NAN; n];

    for i in 0..n {
        let tr_val = tr_v.get(i).unwrap_or(f64::NAN);
        let pdm_val = pdm_v.get(i).unwrap_or(f64::NAN);
        let mdm_val = mdm_v.get(i).unwrap_or(f64::NAN);

        if !tr_val.is_nan() && tr_val != 0.0 {
            let plus_di = (pdm_val / tr_val) * 100.0;
            let minus_di = (mdm_val / tr_val) * 100.0;
            let di_sum = plus_di + minus_di;
            if di_sum != 0.0 {
                dx_vals[i] = ((plus_di - minus_di).abs() / di_sum) * 100.0;
            } else {
                dx_vals[i] = 0.0;
            }
        }
    }

    let dx_series = Series::new("dx".into(), dx_vals);
    let adx_series = rma_series(&dx_series, timeperiod);

    let adx_final = adx_series.with_name(output_name.into());
    df.with_column(adx_final.into()).unwrap();
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdxrParams {
    pub timeperiod: usize,
}

/// ADXR - Average Directional Movement Index Rating
///
/// Variante del ADX que es menos sensible y produce menos señales falsas.
///
/// # Parámetros
/// * `df` - DataFrame con columnas: high, low, close (case insensitive)
/// * `timeperiod` - Período de cálculo (default: 14)
///
/// # Retorna
/// DataFrame con columna "adxr" añadida
///
/// # Fórmula
/// ADXR = (ADX + ADX[timeperiod]) / 2
pub fn adxr(df: &mut DataFrame, timeperiod: Option<usize>, output_col: Option<&str>) {
    let timeperiod = timeperiod.unwrap_or(14);
    let output_name = output_col.unwrap_or("adxr");

    // Calcular ADX primero
    adx(df, Some(timeperiod), Some("temp_adx"));
    let adx_col = df.column("temp_adx").unwrap().f64().unwrap();

    let n = adx_col.len();
    let mut adxr_vals: Vec<f64> = vec![f64::NAN; n];

    // ADXR = (ADX[i] + ADX[i - (timeperiod - 1)]) / 2
    let lookback = timeperiod - 1;
    for i in lookback..n {
        let current_adx = adx_col.get(i).unwrap_or(f64::NAN);
        let past_adx = adx_col.get(i - lookback).unwrap_or(f64::NAN);

        if !current_adx.is_nan() && !past_adx.is_nan() {
            adxr_vals[i] = (current_adx + past_adx) / 2.0;
        }
    }

    let adxr_series = Series::new(output_name.into(), adxr_vals);
    df.with_column(adxr_series.into()).unwrap();
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApoParams {
    pub fastperiod: usize,
    pub slowperiod: usize,
}

/// APO - Absolute Price Oscillator
///
/// Diferencia absoluta entre dos medias móviles (EMA rápida y lenta).
///
/// # Parámetros
/// * `df` - DataFrame con columna: close (case insensitive)
/// * `fastperiod` - Período de EMA rápida (default: 12)
/// * `slowperiod` - Período de EMA lenta (default: 26)
/// * `output_col` - Nombre de la columna de salida (default: "apo")
///
/// # Retorna
/// DataFrame con columna "apo" añadida
///
/// # Fórmula
/// APO = EMA(fast) - EMA(slow)
pub fn apo(
    df: &mut DataFrame,
    fastperiod: Option<usize>,
    slowperiod: Option<usize>,
    output_col: Option<&str>,
) {
    let fastperiod = fastperiod.unwrap_or(12);
    let slowperiod = slowperiod.unwrap_or(26);
    let output_name = output_col.unwrap_or("apo");

    let close = get_close(&df).unwrap();

    let fast_ema = ema_series(&close, fastperiod);
    let slow_ema = ema_series(&close, slowperiod);

    let fast_ca = fast_ema.f64().unwrap();
    let slow_ca = slow_ema.f64().unwrap();

    let apo_vals: Vec<f64> = fast_ca
        .into_no_null_iter()
        .zip(slow_ca.into_no_null_iter())
        .map(|(f, s)| f - s)
        .collect();

    let apo_series = Series::new(output_name.into(), apo_vals);

    df.with_column(apo_series.into()).unwrap();
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AroonParams {
    pub timeperiod: usize,
}

/// AROON - Aroon Indicator
///
/// Identifica cambios de tendencia y la fortaleza de una tendencia.
/// Devuelve dos líneas: Aroon Up y Aroon Down.
///
/// # Parámetros
/// * `df` - DataFrame con columnas: high, low (case insensitive)
/// * `timeperiod` - Período de cálculo (default: 14)
/// * `output_col_up` - Nombre de la columna para Aroon Up (default: "aroon_up")
/// * `output_col_down` - Nombre de la columna para Aroon Down (default: "aroon_down")
///
/// # Retorna
/// DataFrame con columnas "aroon_up" y "aroon_down" añadidas
///
/// # Fórmula
/// Aroon Up = ((timeperiod - períodos desde máximo) / timeperiod) * 100
/// Aroon Down = ((timeperiod - períodos desde mínimo) / timeperiod) * 100
pub fn aroon(
    df: &mut DataFrame,
    timeperiod: Option<usize>,
    output_col_up: Option<&str>,
    output_col_down: Option<&str>,
) {
    let timeperiod = timeperiod.unwrap_or(14);
    let output_col_up = output_col_up.unwrap_or("aroon_up");
    let output_col_down = output_col_down.unwrap_or("aroon_down");

    let high = get_high(&df).unwrap();
    let low = get_low(&df).unwrap();

    let high_ca = high.f64().unwrap();
    let low_ca = low.f64().unwrap();

    // Convertir a Vec<f64> manteniendo alineación con NaNs para nulos
    let high_vals: Vec<f64> = high_ca.into_iter().map(|v| v.unwrap_or(f64::NAN)).collect();
    let low_vals: Vec<f64> = low_ca.into_iter().map(|v| v.unwrap_or(f64::NAN)).collect();

    let n = high_vals.len();
    let mut aroon_up: Vec<f64> = vec![f64::NAN; n];
    let mut aroon_down: Vec<f64> = vec![f64::NAN; n];

    for i in timeperiod..n {
        let window_high = &high_vals[i - timeperiod..=i];
        let window_low = &low_vals[i - timeperiod..=i];

        let (max_idx, _) = window_high.iter().enumerate().fold(
            (0, f64::MIN),
            |(max_idx, max_val), (idx, &val)| {
                if val >= max_val {
                    (idx, val)
                } else {
                    (max_idx, max_val)
                }
            },
        );

        let (min_idx, _) =
            window_low
                .iter()
                .enumerate()
                .fold((0, f64::MAX), |(min_idx, min_val), (idx, &val)| {
                    if val <= min_val {
                        (idx, val)
                    } else {
                        (min_idx, min_val)
                    }
                });

        let days_since_high = (timeperiod - max_idx) as f64;
        let days_since_low = (timeperiod - min_idx) as f64;

        aroon_up[i] = ((timeperiod as f64 - days_since_high) / timeperiod as f64) * 100.0;
        aroon_down[i] = ((timeperiod as f64 - days_since_low) / timeperiod as f64) * 100.0;
    }

    let aroon_up_series = Series::new(output_col_up.into(), &aroon_up);
    let aroon_down_series = Series::new(output_col_down.into(), &aroon_down);

    df.with_column(aroon_up_series.into())
        .unwrap()
        .with_column(aroon_down_series.into())
        .unwrap();
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AroonoscParams {
    pub timeperiod: usize,
}

/// AROONOSC - Aroon Oscillator
///
/// Oscilador derivado de Aroon que mide la diferencia entre Aroon Up y Aroon Down.
///
/// # Parámetros
/// * `df` - DataFrame con columnas: high, low (case insensitive)
/// * `timeperiod` - Período de cálculo (default: 14)
/// * `output_col` - Nombre de la columna de salida (default: "aroonosc")
///
/// # Retorna
/// DataFrame con columna "aroonosc" añadida
///
/// # Fórmula
/// AROONOSC = Aroon Up - Aroon Down
pub fn aroonosc(df: &mut DataFrame, timeperiod: Option<usize>, output_col: Option<&str>) {
    let timeperiod = timeperiod.unwrap_or(14);
    let output_name = output_col.unwrap_or("aroonosc");

    // Calcular Aroon Up y Down usando nombres temporales
    let temp_up = "temp_aroon_up";
    let temp_down = "temp_aroon_down";

    aroon(df, Some(timeperiod), Some(temp_up), Some(temp_down));

    let up_col = df.column(temp_up).unwrap();
    let down_col = df.column(temp_down).unwrap();

    // AROONOSC = Aroon Up - Aroon Down
    let aroonosc_series = (up_col - down_col).unwrap().with_name(output_name.into());

    df.with_column(aroonosc_series.into()).unwrap();
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BopParams {
    pub timeperiod: usize,
}

/// BOP - Balance Of Power
///
/// Mide la fuerza del precio de cierre en relación con el rango alto-bajo.
///
/// # Parámetros
/// * `df` - DataFrame con columnas: open, high, low, close (case insensitive)
/// * `timeperiod` - Período de suavizado SMA (default: 14)
/// * `output_col` - Nombre de la columna de salida (default: "bop")
///
/// # Retorna
/// DataFrame con columna "bop" añadida
///
/// # Fórmula
/// BOP = SMA((close - open) / (high - low), timeperiod)
pub fn bop(df: &mut DataFrame, timeperiod: Option<usize>, output_col: Option<&str>) {
    let timeperiod = timeperiod.unwrap_or(14);
    let output_name = output_col.unwrap_or("bop");

    let open = get_open(&df).unwrap();
    let high = get_high(&df).unwrap();
    let low = get_low(&df).unwrap();
    let close = get_close(&df).unwrap();

    // Calcular BOP raw usando expresiones de Polars para eficiencia y alineación
    let denominator = (&high - &low).unwrap();

    // Evitar división por cero: donde denominator == 0, el resultado es NaN
    let raw_bop = ((&close - &open).unwrap() / denominator).unwrap();

    // Aplicar suavizado SMA (Estándar de la industria)
    let bop_smoothed = sma_series(&raw_bop, timeperiod).with_name(output_name.into());

    df.with_column(bop_smoothed.into()).unwrap();
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CciParams {
    pub timeperiod: usize,
}

/// CCI - Commodity Channel Index
///
/// Identifica retrocesos cíclicos y sobrecompra/sobreventa.
///
/// # Parámetros
/// * `df` - DataFrame con columnas: high, low, close (case insensitive)
/// * `timeperiod` - Período de cálculo (default: 14)
/// * `output_col` - Nombre de la columna de salida (default: "cci")
///
/// # Retorna
/// DataFrame con columna "cci" añadida
///
/// # Fórmula
/// CCI = (Typical Price - SMA(Typical Price)) / (0.015 * Mean Deviation)
/// Typical Price = (high + low + close) / 3
pub fn cci(df: &mut DataFrame, timeperiod: Option<usize>, output_col: Option<&str>) {
    let timeperiod = timeperiod.unwrap_or(14);
    let timeperiod = timeperiod.max(2);
    let output_name = output_col.unwrap_or("cci");

    let high = get_high(&df).unwrap();
    let low = get_low(&df).unwrap();
    let close = get_close(&df).unwrap();

    let high_vals = high
        .f64()
        .unwrap()
        .into_iter()
        .map(|v| v.unwrap_or(f64::NAN))
        .collect::<Vec<f64>>();
    let low_vals = low
        .f64()
        .unwrap()
        .into_iter()
        .map(|v| v.unwrap_or(f64::NAN))
        .collect::<Vec<f64>>();
    let close_vals = close
        .f64()
        .unwrap()
        .into_iter()
        .map(|v| v.unwrap_or(f64::NAN))
        .collect::<Vec<f64>>();

    // Sumar high + low (simple operación de vectores)
    let sum_high_low: Vec<f64> = high_vals
        .iter()
        .zip(low_vals.iter())
        .map(|(a, b)| a + b)
        .collect();

    // Calcular Typical Price = (high + low + close) / 3 (también operación de vectores)
    let typical_price_series: Vec<f64> = sum_high_low
        .iter()
        .zip(close_vals.iter())
        .map(|(a, b)| (a + b) / 3.0)
        .collect();

    let tp_vals: Vec<f64> = typical_price_series.into_iter().map(|v| v).collect();

    let n = tp_vals.len();
    let mut cci_vals: Vec<f64> = vec![f64::NAN; n];

    // El cálculo del CCI requiere una ventana completa de 'timeperiod'
    for i in (timeperiod - 1)..n {
        let start_idx = i + 1 - timeperiod;
        let window = &tp_vals[start_idx..=i];

        // Calcular media de la ventana
        let sum: f64 = window.iter().sum();
        let mean = sum / timeperiod as f64;

        // Calcular Desviación Media (Mean Deviation)
        let sum_abs_diff: f64 = window.iter().map(|x| (x - mean).abs()).sum();
        let mean_deviation = sum_abs_diff / timeperiod as f64;

        if mean_deviation != 0.0 {
            cci_vals[i] = (tp_vals[i] - mean) / (0.015 * mean_deviation);
        } else {
            cci_vals[i] = 0.0; // O NaN, según la convención deseada si no hay desviación
        }
    }

    let cci_series = Series::new(output_name.into(), &cci_vals);
    df.with_column(cci_series.into()).unwrap();
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CmoParams {
    pub timeperiod: usize,
}

/// CMO - Chande Momentum Oscillator
///
/// Oscilador de momentum que mide la fuerza de los movimientos alcistas vs bajistas.
///
/// # Parámetros
/// * `df` - DataFrame con columna: close (case insensitive)
/// * `timeperiod` - Período de cálculo (default: 14)
/// * `output_col` - Nombre de la columna de salida (default: "cmo")
///
/// # Retorna
/// DataFrame con columna "cmo" añadida
///
/// # Fórmula
/// CMO = 100 * ((sum_up - sum_down) / (sum_up + sum_down))
/// sum_up = suma de precios que subieron
/// sum_down = suma de precios que bajaron
pub fn cmo(df: &mut DataFrame, timeperiod: Option<usize>, output_col: Option<&str>) {
    let timeperiod = timeperiod.unwrap_or(14);
    let output_col = output_col.unwrap_or("cmo");

    let close = get_close(&df).unwrap();
    let close_ca = close.f64().unwrap();
    let close_vals: Vec<f64> = close_ca.into_no_null_iter().collect();

    let n = close_vals.len();
    let mut cmo_vals: Vec<f64> = vec![f64::NAN; n];

    if n > timeperiod {
        // Pre-calculamos los cambios absolutos
        let mut abs_changes = vec![0.0; n];
        for i in 1..n {
            abs_changes[i] = (close_vals[i] - close_vals[i - 1]).abs();
        }

        // Suma inicial de cambios absolutos para la primera ventana completa
        let mut sum_abs: f64 = abs_changes[1..=timeperiod].iter().sum();

        // Primer cálculo en el índice 'timeperiod'
        let diff = close_vals[timeperiod] - close_vals[0];
        if sum_abs != 0.0 {
            cmo_vals[timeperiod] = 100.0 * (diff / sum_abs);
        } else {
            cmo_vals[timeperiod] = 0.0;
        }

        // Cálculo rodante para el resto de los datos (O(n))
        for i in (timeperiod + 1)..n {
            // Actualizamos la suma restando el valor que sale de la ventana y sumando el que entra
            sum_abs = sum_abs - abs_changes[i - timeperiod] + abs_changes[i];

            let diff = close_vals[i] - close_vals[i - timeperiod];
            if sum_abs != 0.0 {
                cmo_vals[i] = 100.0 * (diff / sum_abs);
            } else {
                cmo_vals[i] = 0.0;
            }
        }
    }

    let cmo_series = Series::new(output_col.into(), &cmo_vals);
    df.with_column(cmo_series.into()).unwrap();
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DxParams {
    pub timeperiod: usize,
}

/// DX - Directional Movement Index
///
/// Mide la fuerza de la diferencia entre +DI y -DI, ignorando la dirección.
///
/// # Parámetros
/// * `df` - DataFrame con columnas: high, low, close (case insensitive)
/// * `timeperiod` - Período de cálculo (default: 14)
/// * `output_col` - Nombre de la columna de salida (default: "dx")
///
/// # Retorna
/// DataFrame con columna "dx" añadida
///
/// # Fórmula
/// DX = (|+DI - -DI| / (+DI + -DI)) * 100
pub fn dx(df: &mut DataFrame, timeperiod: Option<usize>, output_col: Option<&str>) {
    let timeperiod = timeperiod.unwrap_or(14);
    let output_name = output_col.unwrap_or("dx");
    let high = get_high(&df).unwrap();
    let low = get_low(&df).unwrap();
    let close = get_close(&df).unwrap();

    let high_vals = high.f64().unwrap();
    let low_vals = low.f64().unwrap();
    let close_vals = close.f64().unwrap();

    let n = high_vals.len();
    let mut plus_dm = vec![0.0; n];
    let mut minus_dm = vec![0.0; n];
    let mut tr = vec![0.0; n];

    // Convertimos a vectores de f64 de forma segura
    let h_v: Vec<f64> = high_vals.into_no_null_iter().collect();
    let l_v: Vec<f64> = low_vals.into_no_null_iter().collect();
    let c_v: Vec<f64> = close_vals.into_no_null_iter().collect();

    if n > 0 {
        tr[0] = h_v[0] - l_v[0];
    }

    for i in 1..n {
        let h_curr = h_v[i];
        let h_prev = h_v[i - 1];
        let l_curr = l_v[i];
        let l_prev = l_v[i - 1];
        let c_prev = c_v[i - 1];

        let high_diff = h_curr - h_prev;
        let low_diff = l_prev - l_curr;

        if high_diff > low_diff && high_diff > 0.0 {
            plus_dm[i] = high_diff;
        } else {
            plus_dm[i] = 0.0;
        }

        if low_diff > high_diff && low_diff > 0.0 {
            minus_dm[i] = low_diff;
        } else {
            minus_dm[i] = 0.0;
        }

        let tr1 = h_curr - l_curr;
        let tr2 = (h_curr - c_prev).abs();
        let tr3 = (l_curr - c_prev).abs();
        tr[i] = tr1.max(tr2).max(tr3);
    }

    let tr_series = Series::new("tr".into(), tr);
    let plus_dm_series = Series::new("plus_dm".into(), plus_dm);
    let minus_dm_series = Series::new("minus_dm".into(), minus_dm);

    let _ = rma_series(&tr_series, timeperiod);
    let smoothed_plus_dm = rma_series(&plus_dm_series, timeperiod);
    let smoothed_minus_dm = rma_series(&minus_dm_series, timeperiod);

    let pdm_v: Vec<f64> = smoothed_plus_dm
        .f64()
        .unwrap()
        .into_no_null_iter()
        .collect();
    let mdm_v: Vec<f64> = smoothed_minus_dm
        .f64()
        .unwrap()
        .into_no_null_iter()
        .collect();

    let mut dx_vals = vec![f64::NAN; n];

    for i in 0..n {
        let pdm_val = pdm_v[i];
        let mdm_val = mdm_v[i];
        let di_sum = pdm_val + mdm_val;

        if !di_sum.is_nan() && di_sum != 0.0 {
            dx_vals[i] = ((pdm_val - mdm_val).abs() / di_sum) * 100.0;
        } else if di_sum == 0.0 {
            dx_vals[i] = 0.0;
        }
    }

    let dx_series = Series::new(output_name.into(), dx_vals);
    df.with_column(dx_series.into()).unwrap();
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MacdParams {
    pub timeperiod: usize,
    pub slowperiod: usize,
    pub signalperiod: usize,
}

/// MACD - Moving Average Convergence/Divergence
///
/// Indicador de tendencia que muestra la relación entre dos EMAs.
/// Devuelve: MACD line, Signal line, y Histogram.
///
/// # Parámetros
/// * `df` - DataFrame con columna: close (case insensitive)
/// * `fastperiod` - Período de EMA rápida (default: 12)
/// * `slowperiod` - Período de EMA lenta (default: 26)
/// * `signalperiod` - Período de EMA de señal (default: 9)
/// * `output_col` - Nombre de la columna de salida (default: "macd")
/// * `output_col_signal` - Nombre de la columna de señal (default: "macd_signal")
/// * `output_col_hist` - Nombre de la columna de histograma (default: "macd_hist")
///
/// # Retorna
/// DataFrame con columnas "macd", "macd_signal" y "macd_hist" añadidas
///
/// # Fórmula
/// MACD = EMA(fast) - EMA(slow)
/// Signal = EMA(MACD, signalperiod)
/// Histogram = MACD - Signal
pub fn macd(
    df: &mut DataFrame,
    fastperiod: Option<usize>,
    slowperiod: Option<usize>,
    signalperiod: Option<usize>,
    output_col: Option<&str>,
    output_col_signal: Option<&str>,
    output_col_hist: Option<&str>,
) {
    let fastperiod = fastperiod.unwrap_or(12);
    let slowperiod = slowperiod.unwrap_or(26);
    let signalperiod = signalperiod.unwrap_or(9);
    let output_col = output_col.unwrap_or("macd");
    let output_col_signal = output_col_signal.unwrap_or("macd_signal");
    let output_col_hist = output_col_hist.unwrap_or("macd_hist");

    let close = get_close(&df).unwrap();
    let fast_ema = ema_series(&close, fastperiod);
    let slow_ema = ema_series(&close, slowperiod);

    let n = fast_ema.len();
    let fast_ca = fast_ema.f64().unwrap();
    let slow_ca = slow_ema.f64().unwrap();

    let mut macd_vals: Vec<f64> = vec![f64::NAN; n];

    for i in 0..n {
        let f = fast_ca.get(i);
        let s = slow_ca.get(i);

        if let (Some(f_val), Some(s_val)) = (f, s) {
            if !f_val.is_nan() && !s_val.is_nan() {
                macd_vals[i] = f_val - s_val;
            }
        }
    }

    let macd_series = Series::new("macd".into(), &macd_vals);
    let signal = ema_series(&macd_series, signalperiod);

    let signal_ca = signal.f64().unwrap();
    let signal_vals: Vec<f64> = (0..n)
        .map(|i| signal_ca.get(i).unwrap_or(f64::NAN))
        .collect();

    let mut hist_vals: Vec<f64> = vec![f64::NAN; n];
    for i in 0..n {
        let m = macd_vals[i];
        let s = signal_vals[i];

        if !m.is_nan() && !s.is_nan() {
            hist_vals[i] = m - s;
        }
    }

    df.with_column(Series::new(output_col.into(), macd_vals).into())
        .unwrap();
    df.with_column(Series::new(output_col_signal.into(), signal_vals).into())
        .unwrap();
    df.with_column(Series::new(output_col_hist.into(), hist_vals).into())
        .unwrap();
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MacdextParams {
    pub fastperiod: usize,
    pub slowperiod: usize,
    pub signalperiod: usize,
    pub fastmatype: usize,
    pub slowmatype: usize,
    pub signalmatype: usize,
}

/// MACDEXT - MACD with controllable MA type
///
/// MACD con tipo de media móvil configurable para todas las componentes.
///
/// # Parámetros
/// * `df` - DataFrame con columna: close (case insensitive)
/// * `fastperiod` - Período de EMA rápida (default: 12)
/// * `slowperiod` - Período de EMA lenta (default: 26)
/// * `signalperiod` - Período de EMA de señal (default: 9)
/// * `fastmatype` - Tipo de media para EMA rápida: 0=EMA, 1=SMA (default: 0)
/// * `slowmatype` - Tipo de media para EMA lenta: 0=EMA, 1=SMA (default: 0)
/// * `signalmatype` - Tipo de media para señal: 0=EMA, 1=SMA (default: 0)
/// * `output_col` - Nombre de la columna de salida para MACD (default: "macd")
/// * `output_col_signal` - Nombre de la columna de salida para señal (default: "macd_signal")
/// * `output_col_hist` - Nombre de la columna de salida para histograma (default: "macd_hist")
///
/// # Retorna
/// DataFrame con columnas "macd", "macd_signal" y "macd_hist" añadidas
pub fn macdext(
    df: &mut DataFrame,
    fastperiod: Option<usize>,
    slowperiod: Option<usize>,
    signalperiod: Option<usize>,
    fastmatype: Option<usize>,
    slowmatype: Option<usize>,
    signalmatype: Option<usize>,
    output_col: Option<&str>,
    output_col_signal: Option<&str>,
    output_col_hist: Option<&str>,
) {
    let fastperiod = fastperiod.unwrap_or(12);
    let slowperiod = slowperiod.unwrap_or(26);
    let signalperiod = signalperiod.unwrap_or(9);

    // MAType: 0=SMA, 1=EMA, 2=RMA (Wilder's)
    let fastmatype = fastmatype.unwrap_or(0);
    let slowmatype = slowmatype.unwrap_or(0);
    let signalmatype = signalmatype.unwrap_or(0);

    let output_col = output_col.unwrap_or("macd");
    let output_col_signal = output_col_signal.unwrap_or("macd_signal");
    let output_col_hist = output_col_hist.unwrap_or("macd_hist");

    let close = get_close(&df).unwrap();

    // Helper para despachar tipos de MA
    fn get_ma(series: &Series, period: usize, ma_type: usize) -> Series {
        match ma_type {
            1 => ema_series(series, period),
            2 => rma_series(series, period),
            _ => sma_series(series, period), // Default a SMA (0)
        }
    }

    let fast_ma = get_ma(&close, fastperiod, fastmatype);
    let slow_ma = get_ma(&close, slowperiod, slowmatype);

    let n = fast_ma.len();
    let fast_ca = fast_ma.f64().unwrap();
    let slow_ca = slow_ma.f64().unwrap();

    let mut macd_vals: Vec<f64> = vec![f64::NAN; n];

    for i in 0..n {
        let f = fast_ca.get(i);
        let s = slow_ca.get(i);

        if let (Some(f_val), Some(s_val)) = (f, s) {
            if !f_val.is_nan() && !s_val.is_nan() {
                macd_vals[i] = f_val - s_val;
            }
        }
    }

    let macd_series = Series::new("macd".into(), &macd_vals);
    let signal_series_raw = get_ma(&macd_series, signalperiod, signalmatype);

    let signal_ca = signal_series_raw.f64().unwrap();
    let signal_vals: Vec<f64> = (0..n)
        .map(|i| signal_ca.get(i).unwrap_or(f64::NAN))
        .collect();

    let mut hist_vals: Vec<f64> = vec![f64::NAN; n];
    for i in 0..n {
        let m = macd_vals[i];
        let s = signal_vals[i];

        if !m.is_nan() && !s.is_nan() {
            hist_vals[i] = m - s;
        }
    }

    df.with_column(Series::new(output_col.into(), macd_vals).into())
        .unwrap();
    df.with_column(Series::new(output_col_signal.into(), signal_vals).into())
        .unwrap();
    df.with_column(Series::new(output_col_hist.into(), hist_vals).into())
        .unwrap();
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MacdfixParams {
    pub signalperiod: usize,
}

/// MACDFIX - Moving Average Convergence/Divergence Fix 12/26
///
/// Variante del MACD con períodos fijos en 12 y 26.
///
/// # Parámetros
/// * `df` - DataFrame con columna: close (case insensitive)
/// * `signalperiod` - Período de EMA de señal (default: 9)
///
/// # Retorna
/// DataFrame con columnas "macd", "macd_signal" y "macd_hist" añadidas
pub fn macdfix(
    df: &mut DataFrame,
    signalperiod: Option<usize>,
    output_col: Option<&str>,
    output_col_signal: Option<&str>,
    output_col_hist: Option<&str>,
) {
    let signalperiod = signalperiod.unwrap_or(9);
    let output_col = output_col.unwrap_or("macd");
    let output_col_signal = output_col_signal.unwrap_or("macd_signal");
    let output_col_hist = output_col_hist.unwrap_or("macd_hist");
    macd(
        df,
        Some(12),
        Some(26),
        Some(signalperiod),
        Some(output_col),
        Some(output_col_signal),
        Some(output_col_hist),
    );
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MfiParams {
    pub timeperiod: usize,
}

/// MFI - Money Flow Index
///
/// Indicador de volumen que mide la fuerza del flujo de dinero.
/// Combina precio y volumen para identificar sobrecompra/sobreventa.
///
/// # Parámetros
/// * `df` - DataFrame con columnas: high, low, close, volume (case insensitive)
/// * `timeperiod` - Período de cálculo (default: 14)
/// * `output_col` - Nombre de la columna de salida (default: "mfi")
///
/// # Retorna
/// DataFrame con columna "mfi" añadida
///
/// # Fórmula
/// Typical Price = (high + low + close) / 3
/// Money Flow = Typical Price * Volume
/// Money Ratio = Positive Flow / Negative Flow
/// MFI = 100 - (100 / (1 + Money Ratio))
pub fn mfi(df: &mut DataFrame, timeperiod: Option<usize>, output_col: Option<&str>) {
    let timeperiod = timeperiod.unwrap_or(14);
    let output_col = output_col.unwrap_or("mfi");
    let high = get_high(&df).unwrap();
    let low = get_low(&df).unwrap();
    let close = get_close(&df).unwrap();
    let volume = get_volume(&df).unwrap();

    let high_ca = high.f64().unwrap();
    let low_ca = low.f64().unwrap();
    let close_ca = close.f64().unwrap();
    let vol_ca = volume.f64().unwrap();

    let n = high_ca.len();
    if n <= timeperiod {
        df.with_column(Series::new(output_col.into(), vec![f64::NAN; n]).into())
            .unwrap();
        return;
    }

    // Convertir a vectores de f64 para acceso rápido
    let high_vals: Vec<f64> = high_ca.into_no_null_iter().collect();
    let low_vals: Vec<f64> = low_ca.into_no_null_iter().collect();
    let close_vals: Vec<f64> = close_ca.into_no_null_iter().collect();
    let vol_vals: Vec<f64> = vol_ca.into_no_null_iter().collect();

    let typical_price: Vec<f64> = high_vals
        .iter()
        .zip(&low_vals)
        .zip(&close_vals)
        .map(|((h, l), c)| (h + l + c) / 3.0)
        .collect();

    let mut pos_flow = vec![0.0; n];
    let mut neg_flow = vec![0.0; n];

    for i in 1..n {
        let raw_money_flow = typical_price[i] * vol_vals[i];
        if typical_price[i] > typical_price[i - 1] {
            pos_flow[i] = raw_money_flow;
        } else if typical_price[i] < typical_price[i - 1] {
            neg_flow[i] = raw_money_flow;
        }
        // Si son iguales, ambos se quedan en 0.0
    }

    let mut mfi_vals = vec![f64::NAN; n];
    let mut current_pos_sum: f64 = pos_flow[1..=timeperiod].iter().sum();
    let mut current_neg_sum: f64 = neg_flow[1..=timeperiod].iter().sum();

    // Primer cálculo en el índice 'timeperiod'
    if current_pos_sum + current_neg_sum != 0.0 {
        mfi_vals[timeperiod] = 100.0 * current_pos_sum / (current_pos_sum + current_neg_sum);
    } else {
        mfi_vals[timeperiod] = 50.0; // Valor neutral si no hay flujo
    }

    // Cálculo rodante (O(n))
    for i in (timeperiod + 1)..n {
        current_pos_sum = current_pos_sum - pos_flow[i - timeperiod] + pos_flow[i];
        current_neg_sum = current_neg_sum - neg_flow[i - timeperiod] + neg_flow[i];

        let total_flow = current_pos_sum + current_neg_sum;
        if total_flow != 0.0 {
            mfi_vals[i] = 100.0 * current_pos_sum / total_flow;
        } else {
            mfi_vals[i] = 50.0;
        }
    }

    let mfi_series = Series::new(output_col.into(), mfi_vals);
    df.with_column(mfi_series.into()).unwrap();
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MinusDiParams {
    pub timeperiod: usize,
}

/// MINUS_DI - Minus Directional Indicator
///
/// Indica la fuerza de la tendencia bajista.
///
/// # Parámetros
/// * `df` - DataFrame con columnas: high, low, close (case insensitive)
/// * `timeperiod` - Período de cálculo (default: 14)
/// * `output_col` - Nombre de la columna de salida (default: "minus_di")
///
/// # Retorna
/// DataFrame con columna "minus_di" añadida
///
/// # Fórmula
/// -DI = (Smoothed -DM / Smoothed TR) * 100
pub fn minus_di(df: &mut DataFrame, timeperiod: Option<usize>, output_col: Option<&str>) {
    let timeperiod = timeperiod.unwrap_or(14);
    let output_name = output_col.unwrap_or("minus_di");
    let high = get_high(&df).unwrap();
    let low = get_low(&df).unwrap();
    let close = get_close(&df).unwrap();

    let high_vals = high.f64().unwrap();
    let low_vals = low.f64().unwrap();
    let close_vals = close.f64().unwrap();

    let n = high_vals.len();
    let mut minus_dm = vec![0.0; n];
    let mut tr = vec![0.0; n];

    if n > 0 {
        tr[0] = high_vals.get(0).unwrap() - low_vals.get(0).unwrap();
    }

    for i in 1..n {
        let h_curr = high_vals.get(i).unwrap();
        let h_prev = high_vals.get(i - 1).unwrap();
        let l_curr = low_vals.get(i).unwrap();
        let l_prev = low_vals.get(i - 1).unwrap();
        let c_prev = close_vals.get(i - 1).unwrap();

        let high_diff = h_curr - h_prev;
        let low_diff = l_prev - l_curr;

        if low_diff > high_diff && low_diff > 0.0 {
            minus_dm[i] = low_diff;
        } else {
            minus_dm[i] = 0.0;
        }

        let tr1 = h_curr - l_curr;
        let tr2 = (h_curr - c_prev).abs();
        let tr3 = (l_curr - c_prev).abs();
        tr[i] = tr1.max(tr2).max(tr3);
    }

    let minus_dm_series = Series::new("minus_dm".into(), minus_dm);
    let tr_series = Series::new("tr".into(), tr);

    let smoothed_minus_dm = rma_series(&minus_dm_series, timeperiod);
    let smoothed_tr = rma_series(&tr_series, timeperiod);

    let mdm_v = smoothed_minus_dm.f64().unwrap();
    let tr_v = smoothed_tr.f64().unwrap();

    let minus_di_vals: Vec<f64> = mdm_v
        .into_no_null_iter()
        .zip(tr_v.into_no_null_iter())
        .map(|(dm, tr)| {
            if tr == 0.0 {
                f64::NAN
            } else {
                (dm / tr) * 100.0
            }
        })
        .collect();

    let minus_di_series = Series::new(output_name.into(), minus_di_vals);
    df.with_column(minus_di_series.into()).unwrap();
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MinusDmParams {
    pub timeperiod: usize,
}

/// MINUS_DM - Minus Directional Movement
///
/// Movimiento direccional negativo suavizado (RMA).
///
/// # Parámetros
/// * `df` - DataFrame con columnas: high, low (case insensitive)
/// * `timeperiod` - Período de cálculo (default: 14)
/// * `output_col` - Nombre de la columna de salida (default: "minus_dm")
///
/// # Retorna
/// DataFrame con columna "minus_dm" añadida
///
/// # Fórmula
/// -DM = RMA(Max(high - low, high - prev_close, prev_close - low))
pub fn minus_dm(df: &mut DataFrame, timeperiod: Option<usize>, output_col: Option<&str>) {
    let timeperiod = timeperiod.unwrap_or(14);
    let output_name = output_col.unwrap_or("minus_dm");
    let high = get_high(&df).unwrap();
    let low = get_low(&df).unwrap();

    let high_vals = high.f64().unwrap();
    let low_vals = low.f64().unwrap();

    let n = high_vals.len();
    let mut minus_dm = vec![0.0; n];

    for i in 1..n {
        let h_curr = high_vals.get(i).unwrap();
        let h_prev = high_vals.get(i - 1).unwrap();
        let l_curr = low_vals.get(i).unwrap();
        let l_prev = low_vals.get(i - 1).unwrap();

        let high_diff = h_curr - h_prev;
        let low_diff = l_prev - l_curr;

        if low_diff > high_diff && low_diff > 0.0 {
            minus_dm[i] = low_diff;
        } else {
            minus_dm[i] = 0.0;
        }
    }

    let minus_dm_series = Series::new(output_name.into(), minus_dm);
    let smoothed_minus_dm = rma_series(&minus_dm_series, timeperiod);

    df.with_column(smoothed_minus_dm.into()).unwrap();
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MomParams {
    pub timeperiod: usize,
}

/// MOM - Momentum
///
/// Mide la tasa de cambio del precio en un período.
///
/// # Parámetros
/// * `df` - DataFrame con columna: close (case insensitive)
/// * `timeperiod` - Período de cálculo (default: 10)
/// * `output_col` - Nombre de la columna de salida (default: "mom")
///
/// # Retorna
/// DataFrame con columna "mom" añadida
///
/// # Fórmula
/// MOM = close[i] - close[i - timeperiod]
pub fn mom(df: &mut DataFrame, timeperiod: Option<usize>, output_col: Option<&str>) {
    let timeperiod = timeperiod.unwrap_or(10);
    let output_col = output_col.unwrap_or("mom");
    let close = get_close(&df).unwrap();

    let close_ca = close.f64().unwrap();
    let n = close_ca.len();
    let mut mom_vals = vec![f64::NAN; n];

    if n > timeperiod {
        // Convertimos a vector manejando nulos para evitar desalineación
        let close_vals: Vec<f64> = close_ca
            .into_iter()
            .map(|v| v.unwrap_or(f64::NAN))
            .collect();

        for i in timeperiod..n {
            let curr = close_vals[i];
            let prev = close_vals[i - timeperiod];

            if !curr.is_nan() && !prev.is_nan() {
                mom_vals[i] = curr - prev;
            }
        }
    }

    df.with_column(Series::new(output_col.into(), mom_vals).into())
        .unwrap();
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlusDiParams {
    pub timeperiod: usize,
}

/// PLUS_DI - Plus Directional Indicator
///
/// Indica la fuerza de la tendencia alcista.
///
/// # Parámetros
/// * `df` - DataFrame con columnas: high, low, close (case insensitive)
/// * `timeperiod` - Período de cálculo (default: 14)
/// * `output_col` - Nombre de la columna de salida (default: "plus_di")
///
/// # Retorna
/// DataFrame con columna "plus_di" añadida
///
/// # Fórmula
/// +DI = (Smoothed +DM / Smoothed TR) * 100
pub fn plus_di(df: &mut DataFrame, timeperiod: Option<usize>, output_col: Option<&str>) {
    let timeperiod = timeperiod.unwrap_or(14);
    let output_name = output_col.unwrap_or("plus_di");
    let high = get_high(&df).unwrap();
    let low = get_low(&df).unwrap();
    let close = get_close(&df).unwrap();

    let high_vals = high.f64().unwrap();
    let low_vals = low.f64().unwrap();
    let close_vals = close.f64().unwrap();

    let n = high_vals.len();
    let mut plus_dm = vec![0.0; n];
    let mut tr = vec![0.0; n];

    if n > 0 {
        tr[0] = high_vals.get(0).unwrap() - low_vals.get(0).unwrap();
    }

    for i in 1..n {
        let h_curr = high_vals.get(i).unwrap();
        let h_prev = high_vals.get(i - 1).unwrap();
        let l_curr = low_vals.get(i).unwrap();
        let l_prev = low_vals.get(i - 1).unwrap();
        let c_prev = close_vals.get(i - 1).unwrap();

        let high_diff = h_curr - h_prev;
        let low_diff = l_prev - l_curr;

        if high_diff > low_diff && high_diff > 0.0 {
            plus_dm[i] = high_diff;
        } else {
            plus_dm[i] = 0.0;
        }

        let tr1 = h_curr - l_curr;
        let tr2 = (h_curr - c_prev).abs();
        let tr3 = (l_curr - c_prev).abs();
        tr[i] = tr1.max(tr2).max(tr3);
    }

    let plus_dm_series = Series::new("plus_dm".into(), plus_dm);
    let tr_series = Series::new("tr".into(), tr);

    let smoothed_plus_dm = rma_series(&plus_dm_series, timeperiod);
    let smoothed_tr = rma_series(&tr_series, timeperiod);

    let pdm_v = smoothed_plus_dm.f64().unwrap();
    let tr_v = smoothed_tr.f64().unwrap();

    let plus_di_vals: Vec<f64> = pdm_v
        .into_no_null_iter()
        .zip(tr_v.into_no_null_iter())
        .map(|(dm, tr)| {
            if tr == 0.0 {
                f64::NAN
            } else {
                (dm / tr) * 100.0
            }
        })
        .collect();

    let plus_di_series = Series::new(output_name.into(), plus_di_vals);
    df.with_column(plus_di_series.into()).unwrap();
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlusDmParams {
    pub timeperiod: usize,
}

/// PLUS_DM - Plus Directional Movement
///
/// Movimiento direccional positivo suavizado (RMA).
///
/// # Parámetros
/// * `df` - DataFrame con columnas: high, low (case insensitive)
/// * `timeperiod` - Período de cálculo (default: 14)
/// * `output_col` - Nombre de la columna de salida (default: "plus_dm")
///
/// # Retorna
/// DataFrame con columna "plus_dm" añadida
///
/// # Fórmula
/// +DM = RMA(Max(high - low, high - prev_close, prev_close - low))
pub fn plus_dm(df: &mut DataFrame, timeperiod: Option<usize>, output_col: Option<&str>) {
    let timeperiod = timeperiod.unwrap_or(14);
    let output_name = output_col.unwrap_or("plus_dm");
    let high = get_high(&df).unwrap();
    let low = get_low(&df).unwrap();

    let high_vals = high.f64().unwrap();
    let low_vals = low.f64().unwrap();

    let n = high_vals.len();
    let mut plus_dm = vec![0.0; n];

    for i in 1..n {
        let h_curr = high_vals.get(i).unwrap();
        let h_prev = high_vals.get(i - 1).unwrap();
        let l_curr = low_vals.get(i).unwrap();
        let l_prev = low_vals.get(i - 1).unwrap();

        let high_diff = h_curr - h_prev;
        let low_diff = l_prev - l_curr;

        if high_diff > low_diff && high_diff > 0.0 {
            plus_dm[i] = high_diff;
        } else {
            plus_dm[i] = 0.0;
        }
    }

    let plus_dm_series = Series::new(output_name.into(), plus_dm);
    let smoothed_plus_dm = rma_series(&plus_dm_series, timeperiod);

    df.with_column(smoothed_plus_dm.into()).unwrap();
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PpoParams {
    pub fastperiod: usize,
    pub slowperiod: usize,
}

/// PPO - Percentage Price Oscillator
///
/// Oscilador de precio porcentual que muestra la diferencia entre dos EMAs.
///
/// # Parámetros
/// * `df` - DataFrame con columna: close (case insensitive)
/// * `fastperiod` - Período de EMA rápida (default: 12)
/// * `slowperiod` - Período de EMA lenta (default: 26)
/// * `output_col` - Nombre de la columna de salida (default: "ppo")
///
/// # Retorna
/// DataFrame con columna "ppo" añadida
///
/// # Fórmula
/// PPO = ((EMA(fast) - EMA(slow)) / EMA(slow)) * 100
pub fn ppo(
    df: &mut DataFrame,
    fastperiod: Option<usize>,
    slowperiod: Option<usize>,
    output_col: Option<&str>,
) {
    let fastperiod = fastperiod.unwrap_or(12);
    let slowperiod = slowperiod.unwrap_or(26);
    let output_name = output_col.unwrap_or("ppo");
    let close = get_close(&df).unwrap();

    let fast_ema = ema_series(&close, fastperiod);
    let slow_ema = ema_series(&close, slowperiod);
    let fast_ca = fast_ema.f64().unwrap();
    let slow_ca = slow_ema.f64().unwrap();

    let ppo_vals: Vec<f64> = fast_ca
        .into_no_null_iter()
        .zip(slow_ca.into_no_null_iter())
        .map(|(f, s)| ((f - s) / s) * 100.0)
        .collect();

    let ppo_series = Series::new(output_name.into(), ppo_vals);

    df.with_column(ppo_series.into()).unwrap();
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RocParams {
    pub timeperiod: usize,
}

/// ROC - Rate of change
///
/// Mide el cambio porcentual del precio desde hace timeperiod períodos.
///
/// # Parámetros
/// * `df` - DataFrame con columna: close (case insensitive)
/// * `timeperiod` - Período de cálculo (default: 10)
/// * `output_col` - Nombre de la columna de salida (default: "roc")
///
/// # Retorna
/// DataFrame con columna "roc" añadida
///
/// # Fórmula
/// ROC = ((close[i] / close[i - timeperiod]) - 1) * 100
pub fn roc(df: &mut DataFrame, timeperiod: Option<usize>, output_col: Option<&str>) {
    let timeperiod = timeperiod.unwrap_or(10);
    let output_col = output_col.unwrap_or("roc");
    let close = get_close(&df).unwrap();

    let close_ca = close.f64().unwrap();
    let n = close_ca.len();
    let mut roc_vals: Vec<f64> = vec![f64::NAN; n];

    if n > timeperiod {
        let close_vals: Vec<f64> = close_ca
            .into_iter()
            .map(|v| v.unwrap_or(f64::NAN))
            .collect();

        for i in timeperiod..n {
            let curr = close_vals[i];
            let prev = close_vals[i - timeperiod];

            if !curr.is_nan() && !prev.is_nan() && prev != 0.0 {
                roc_vals[i] = ((curr / prev) - 1.0) * 100.0;
            }
        }
    }

    df.with_column(Series::new(output_col.into(), roc_vals).into())
        .unwrap();
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RocpParams {
    pub timeperiod: usize,
}

/// ROCP - Rate of change Percentage
///
/// Variante del ROC que devuelve el cambio decimal en lugar de porcentual.
///
/// # Parámetros
/// * `df` - DataFrame con columna: close (case insensitive)
/// * `timeperiod` - Período de cálculo (default: 10)
/// * `output_col` - Nombre de la columna de salida (default: "rocp")
///
/// # Retorna
/// DataFrame con columna "rocp" añadida
///
/// # Fórmula
/// ROCP = (close[i] - close[i - timeperiod]) / close[i - timeperiod]
pub fn rocp(df: &mut DataFrame, timeperiod: Option<usize>, output_col: Option<&str>) {
    let timeperiod = timeperiod.unwrap_or(10);
    let output_col = output_col.unwrap_or("rocp");
    let close = get_close(&df).unwrap();

    let close_ca = close.f64().unwrap();
    let n = close_ca.len();
    let mut rocp_vals: Vec<f64> = vec![f64::NAN; n];

    if n > timeperiod {
        let close_vals: Vec<f64> = close_ca
            .into_iter()
            .map(|v| v.unwrap_or(f64::NAN))
            .collect();

        for i in timeperiod..n {
            let curr = close_vals[i];
            let prev = close_vals[i - timeperiod];

            if !curr.is_nan() && !prev.is_nan() && prev != 0.0 {
                rocp_vals[i] = (curr - prev) / prev;
            }
        }
    }

    df.with_column(Series::new(output_col.into(), rocp_vals).into())
        .unwrap();
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RocrParams {
    pub timeperiod: usize,
}

/// ROCR - Rate of change ratio
///
/// Ratio de cambio de precio entre el precio actual y el de hace timeperiod períodos.
///
/// # Parámetros
/// * `df` - DataFrame con columna: close (case insensitive)
/// * `timeperiod` - Período de cálculo (default: 10)
/// * `output_col` - Nombre de la columna de salida (default: "rocr")
///
/// # Retorna
/// DataFrame con columna "rocr" añadida
///
/// # Fórmula
/// ROCR = close[i] / close[i - timeperiod]
pub fn rocr(df: &mut DataFrame, timeperiod: Option<usize>, output_col: Option<&str>) {
    let timeperiod = timeperiod.unwrap_or(10);
    let output_col = output_col.unwrap_or("rocr");
    let close = get_close(&df).unwrap();

    let close_ca = close.f64().unwrap();
    let n = close_ca.len();
    let mut rocr_vals: Vec<f64> = vec![f64::NAN; n];

    if n > timeperiod {
        let close_vals: Vec<f64> = close_ca
            .into_iter()
            .map(|v| v.unwrap_or(f64::NAN))
            .collect();

        for i in timeperiod..n {
            let curr = close_vals[i];
            let prev = close_vals[i - timeperiod];

            if !curr.is_nan() && !prev.is_nan() && prev != 0.0 {
                rocr_vals[i] = curr / prev;
            }
        }
    }

    df.with_column(Series::new(output_col.into(), rocr_vals).into())
        .unwrap();
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Roc100Params {
    pub timeperiod: usize,
}

/// ROCR100 - Rate of change ratio 100 scale
///
/// ROCR multiplicado por 100 para dar una escala más legible.
///
/// # Parámetros
/// * `df` - DataFrame con columna: close (case insensitive)
/// * `timeperiod` - Período de cálculo (default: 10)
/// * `output_col` - Nombre de la columna de salida (default: "rocr100")
///
/// # Retorna
/// DataFrame con columna "rocr100" añadida
///
/// # Fórmula
/// ROCR100 = (close[i] / close[i - timeperiod]) * 100
pub fn rocr100(df: &mut DataFrame, timeperiod: Option<usize>, output_col: Option<&str>) {
    let timeperiod = timeperiod.unwrap_or(10);
    let output_col = output_col.unwrap_or("rocr100");
    let close = get_close(&df).unwrap();

    let close_ca = close.f64().unwrap();
    let n = close_ca.len();
    let mut rocr100_vals: Vec<f64> = vec![f64::NAN; n];

    if n > timeperiod {
        let close_vals: Vec<f64> = close_ca
            .into_iter()
            .map(|v| v.unwrap_or(f64::NAN))
            .collect();

        for i in timeperiod..n {
            let curr = close_vals[i];
            let prev = close_vals[i - timeperiod];

            if !curr.is_nan() && !prev.is_nan() && prev != 0.0 {
                rocr100_vals[i] = (curr / prev) * 100.0;
            }
        }
    }

    df.with_column(Series::new(output_col.into(), rocr100_vals).into())
        .unwrap();
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RsiParams {
    pub timeperiod: usize,
}

/// RSI - Relative Strength Index
///
/// Oscilador de momentum que mide la velocidad y magnitud de los cambios de precio.
/// Escala de 0-100: >70 sobrecompra, <70 sobreventa.
///
/// # Parámetros
/// * `df` - DataFrame con columna: close (case insensitive)
/// * `timeperiod` - Período de cálculo (default: 14)
/// * `output_col` - Nombre de la columna de salida (default: "rsi")
///
/// # Retorna
/// DataFrame con columna "rsi" añadida
///
/// # Fórmula
/// RSI = 100 - (100 / (1 + RS))
/// RS = Average Gain / Average Loss
pub fn rsi(df: &mut DataFrame, timeperiod: Option<usize>, output_col: Option<&str>) {
    let timeperiod = timeperiod.unwrap_or(14);
    let output_col = output_col.unwrap_or("rsi");
    let close = get_close(&df).unwrap();

    let close_ca = close.f64().unwrap();
    let n = close_ca.len();

    if n <= 1 {
        df.with_column(Series::new(output_col.into(), vec![f64::NAN; n]).into())
            .unwrap();
        return;
    }

    // Convertimos a vector manejando nulos para evitar desalineación
    let close_vals: Vec<f64> = close_ca
        .into_iter()
        .map(|v| v.unwrap_or(f64::NAN))
        .collect();

    let mut gain = vec![0.0; n];
    let mut loss = vec![0.0; n];

    for i in 1..n {
        let curr = close_vals[i];
        let prev = close_vals[i - 1];

        if !curr.is_nan() && !prev.is_nan() {
            let change = curr - prev;
            if change > 0.0 {
                gain[i] = change;
            } else if change < 0.0 {
                loss[i] = change.abs();
            }
        }
    }

    // Usamos rma_series sobre las ganancias y pérdidas
    let gain_series = Series::new("gain".into(), gain);
    let loss_series = Series::new("loss".into(), loss);

    let avg_gain_series = rma_series(&gain_series, timeperiod);
    let avg_loss_series = rma_series(&loss_series, timeperiod);

    let avg_gain_ca = avg_gain_series.f64().unwrap();
    let avg_loss_ca = avg_loss_series.f64().unwrap();

    let mut rsi_vals = vec![f64::NAN; n];

    for i in 0..n {
        let g = avg_gain_ca.get(i);
        let l = avg_loss_ca.get(i);

        if let (Some(g_val), Some(l_val)) = (g, l) {
            if !g_val.is_nan() && !l_val.is_nan() {
                if l_val == 0.0 {
                    rsi_vals[i] = 100.0;
                } else if g_val == 0.0 {
                    rsi_vals[i] = 0.0;
                } else {
                    let rs = g_val / l_val;
                    rsi_vals[i] = 100.0 - (100.0 / (1.0 + rs));
                }
            }
        }
    }

    df.with_column(Series::new(output_col.into(), rsi_vals).into())
        .unwrap();
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StochParams {
    pub fastk_period: usize,
    pub slowk_period: usize,
    pub slowk_matype: usize,
    pub slowd_period: usize,
    pub slowd_matype: usize,
}

/// STOCH - Stochastic
///
/// Oscilador que compara el precio de cierre con el rango alto-bajo en un período.
/// Devuelve %K (lenta) y %D.
///
/// # Parámetros
/// * `df` - DataFrame con columnas: high, low, close (case insensitive)
/// * `fastk_period` - Período para cálculo de %K rápido (default: 5)
/// * `slowk_period` - Período de suavizado de %K (default: 3)
/// * `slowk_matype` - Tipo de media para %K: 0=EMA, 1=SMA (default: 0)
/// * `slowd_period` - Período de cálculo de %D (default: 3)
/// * `slowd_matype` - Tipo de media para %D: 0=EMA, 1=SMA (default: 0)
/// * `output_col_k` - Nombre de la columna de salida para %K (default: "slow_k")
/// * `output_col_d` - Nombre de la columna de salida para %D (default: "slow_d")
///
/// # Retorna
/// DataFrame con columnas "slow_k" y "slow_d" añadidas
///
/// # Fórmula
/// %K = ((close - lowest_low) / (highest_high - lowest_low)) * 100
/// %D = SMA(%K, slowd_period)
pub fn stoch(
    df: &mut DataFrame,
    fastk_period: Option<usize>,
    slowk_period: Option<usize>,
    slowk_matype: Option<usize>,
    slowd_period: Option<usize>,
    output_col_k: Option<&str>,
    output_col_d: Option<&str>,
) {
    let fastk_period = fastk_period.unwrap_or(5).max(2);
    let slowk_period = slowk_period.unwrap_or(3).max(2);
    let slowd_period = slowd_period.unwrap_or(3).max(2);

    // MAType: 0=SMA, 1=EMA (para consistencia con otros indicadores)
    let slowk_matype = slowk_matype.unwrap_or(0);

    let output_col_k = output_col_k.unwrap_or("slow_k");
    let output_col_d = output_col_d.unwrap_or("slow_d");

    let high = get_high(&df).unwrap();
    let low = get_low(&df).unwrap();
    let close = get_close(&df).unwrap();

    let high_ca = high.f64().unwrap();
    let low_ca = low.f64().unwrap();
    let close_ca = close.f64().unwrap();

    let n = df.height();
    let mut k_vals: Vec<f64> = vec![f64::NAN; n];

    // Helper para obtener valores manejando nulos
    let h_v: Vec<f64> = high_ca.into_iter().map(|v| v.unwrap_or(f64::NAN)).collect();
    let l_v: Vec<f64> = low_ca.into_iter().map(|v| v.unwrap_or(f64::NAN)).collect();
    let c_v: Vec<f64> = close_ca
        .into_iter()
        .map(|v| v.unwrap_or(f64::NAN))
        .collect();

    for i in (fastk_period - 1)..n {
        let mut hh = f64::NEG_INFINITY;
        let mut ll = f64::INFINITY;
        let mut valid = true;

        let start_j = i + 1 - fastk_period;
        for j in start_j..=i {
            let h = h_v[j];
            let l = l_v[j];
            if h.is_nan() || l.is_nan() {
                valid = false;
                break;
            }
            if h > hh {
                hh = h;
            }
            if l < ll {
                ll = l;
            }
        }

        if valid && hh != ll {
            let curr_c = c_v[i];
            if !curr_c.is_nan() {
                k_vals[i] = ((curr_c - ll) / (hh - ll)) * 100.0;
            }
        }
    }

    // Apply slowk smoothing
    let k_series_raw = Series::new("k_raw".into(), &k_vals);
    let mut k_smooth_series = if slowk_matype == 1 {
        ema_series(&k_series_raw, slowk_period)
    } else {
        sma_series(&k_series_raw, slowk_period)
    };

    // Calculate %D as SMA of %K
    let mut d_smooth_series = sma_series(&k_smooth_series, slowd_period);

    k_smooth_series.rename(output_col_k.into());
    d_smooth_series.rename(output_col_d.into());

    df.with_column(k_smooth_series.into()).unwrap();
    df.with_column(d_smooth_series.into()).unwrap();
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StochfParams {
    pub fastk_period: usize,
    pub fastd_period: usize,
    pub fastd_matype: usize,
}

/// STOCHF - Stochastic Fast
///
/// Versión rápida del Stochastic que no suaviza %K.
///
/// # Parámetros
/// * `df` - DataFrame con columnas: high, low, close (case insensitive)
/// * `fastk_period` - Período para cálculo de %K rápido (default: 5)
/// * `fastd_period` - Período de cálculo de %D (default: 3)
/// * `fastd_matype` - Tipo de media para %D: 0=EMA, 1=SMA (default: 0)
/// * `output_col_k` - Nombre de la columna de salida para %K (default: "fast_k")
/// * `output_col_d` - Nombre de la columna de salida para %D (default: "fast_d")
///
/// # Retorna
/// DataFrame con columnas "fast_k" y "fast_d" añadidas
///
/// # Fórmula
/// %K = ((close - lowest_low) / (highest_high - lowest_low)) * 100
/// %D = EMA(%K, fastd_period)
pub fn stochf(
    df: &mut DataFrame,
    fastk_period: Option<usize>,
    fastd_period: Option<usize>,
    fastd_matype: Option<usize>,
    output_col_k: Option<&str>,
    output_col_d: Option<&str>,
) {
    let fastk_period = fastk_period.unwrap_or(5).max(2);
    let fastd_period = fastd_period.unwrap_or(3).max(2);

    // MAType: 0=SMA, 1=EMA (para consistencia)
    let fastd_matype = fastd_matype.unwrap_or(0);

    let output_col_k = output_col_k.unwrap_or("fast_k");
    let output_col_d = output_col_d.unwrap_or("fast_d");

    let high = get_high(&df).unwrap();
    let low = get_low(&df).unwrap();
    let close = get_close(&df).unwrap();

    let high_ca = high.f64().unwrap();
    let low_ca = low.f64().unwrap();
    let close_ca = close.f64().unwrap();

    let n = df.height();
    let mut fastk_vals: Vec<f64> = vec![f64::NAN; n];

    // Helper para obtener valores manejando nulos
    let h_v: Vec<f64> = high_ca.into_iter().map(|v| v.unwrap_or(f64::NAN)).collect();
    let l_v: Vec<f64> = low_ca.into_iter().map(|v| v.unwrap_or(f64::NAN)).collect();
    let c_v: Vec<f64> = close_ca
        .into_iter()
        .map(|v| v.unwrap_or(f64::NAN))
        .collect();

    for i in (fastk_period - 1)..n {
        let mut hh = f64::NEG_INFINITY;
        let mut ll = f64::INFINITY;
        let mut valid = true;

        let start_j = i + 1 - fastk_period;
        for j in start_j..=i {
            let h = h_v[j];
            let l = l_v[j];
            if h.is_nan() || l.is_nan() {
                valid = false;
                break;
            }
            if h > hh {
                hh = h;
            }
            if l < ll {
                ll = l;
            }
        }

        if valid && hh != ll {
            let curr_c = c_v[i];
            if !curr_c.is_nan() {
                fastk_vals[i] = ((curr_c - ll) / (hh - ll)) * 100.0;
            }
        }
    }

    let fastk_series = Series::new(output_col_k.into(), &fastk_vals);
    let mut fastd_series = if fastd_matype == 1 {
        ema_series(&fastk_series, fastd_period)
    } else {
        sma_series(&fastk_series, fastd_period)
    };

    fastd_series.rename(output_col_d.into());

    df.with_column(fastk_series.into()).unwrap();
    df.with_column(fastd_series.into()).unwrap();
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StochRsiParams {
    pub timeperiod: usize,
    pub fastk_period: usize,
    pub fastd_period: usize,
    pub fastd_matype: usize,
}

/// STOCHRSI - Stochastic Relative Strength Index
///
/// Aplica el análisis Stochastic al RSI en lugar de al precio.
///
/// # Parámetros
/// * `df` - DataFrame con columna: close (case insensitive)
/// * `timeperiod` - Período de cálculo del RSI (default: 14)
/// * `fastk_period` - Período para cálculo de %K rápido (default: 3)
/// * `fastd_period` - Período de cálculo de %D (default: 3)
/// * `fastd_matype` - Tipo de media para %D: 0=EMA, 1=SMA (default: 0)
/// * `output_col_k` - Nombre de la columna para %K (default: "stochrsi_k")
/// * `output_col_d` - Nombre de la columna para %D (default: "stochrsi_d")
///
/// # Retorna
/// DataFrame con columnas "stochrsi_k" y "stochrsi_d" añadidas
///
/// # Fórmula
/// %K = (RSI - lowest_RSI) / (highest_RSI - lowest_RSI)
/// %D = SMA(%K, fastd_period)
pub fn stochrsi(
    df: &mut DataFrame,
    timeperiod: Option<usize>,
    fastk_period: Option<usize>,
    fastd_period: Option<usize>,
    fastd_matype: Option<usize>,
    output_col_k: Option<&str>,
    output_col_d: Option<&str>,
) {
    let timeperiod = timeperiod.unwrap_or(14).max(2);
    let fastk_period = fastk_period.unwrap_or(3).max(2);
    let fastd_period = fastd_period.unwrap_or(3).max(2);

    // MAType: 0=SMA, 1=EMA
    let fastd_matype = fastd_matype.unwrap_or(0);

    let output_col_k = output_col_k.unwrap_or("stochrsi_k");
    let output_col_d = output_col_d.unwrap_or("stochrsi_d");

    // Calculamos el RSI (usando una copia para no alterar el original antes de tiempo)
    rsi(df, Some(timeperiod), Some("temp_rsi"));
    let rsi_col = df.column("temp_rsi").unwrap();
    let rsi_ca = rsi_col.f64().unwrap();

    let n = df.height();
    let mut stochrsi_vals: Vec<f64> = vec![f64::NAN; n];

    // Helper para obtener valores manejando nulos
    let rsi_v: Vec<f64> = rsi_ca.into_iter().map(|v| v.unwrap_or(f64::NAN)).collect();

    for i in (fastk_period - 1)..n {
        let mut hh = f64::NEG_INFINITY;
        let mut ll = f64::INFINITY;
        let mut valid = true;

        let start_j = i + 1 - fastk_period;
        for j in start_j..=i {
            let rsi_val = rsi_v[j];
            if rsi_val.is_nan() {
                valid = false;
                break;
            }
            if rsi_val > hh {
                hh = rsi_val;
            }
            if rsi_val < ll {
                ll = rsi_val;
            }
        }

        if valid && hh != ll {
            let curr_rsi = rsi_v[i];
            if !curr_rsi.is_nan() {
                stochrsi_vals[i] = (curr_rsi - ll) / (hh - ll);
            }
        } else if valid && hh == ll {
            stochrsi_vals[i] = 0.5; // Valor neutral si no hay rango
        }
    }

    let k_series_raw = Series::new("stochrsi_raw".into(), &stochrsi_vals);
    let mut k_smooth_series = if fastd_matype == 1 {
        ema_series(&k_series_raw, fastk_period)
    } else {
        sma_series(&k_series_raw, fastk_period)
    };

    let mut d_smooth_series = sma_series(&k_smooth_series, fastd_period);

    k_smooth_series.rename(output_col_k.into());
    d_smooth_series.rename(output_col_d.into());

    df.with_column(k_smooth_series.into()).unwrap();
    df.with_column(d_smooth_series.into()).unwrap();
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TrixParams {
    pub timeperiod: usize,
}

/// TRIX - Triple EMA Rate of Change
///
/// Oscilador de momento que muestra el ritmo de cambio de una triple EMA.
/// Fija tendencias a largo plazo y filtra fluctuaciones de precio menores.
///
/// # Parámetros
/// * `df` - DataFrame con columna: close (case insensitive)
/// * `timeperiod` - Período de cálculo (default: 30)
/// * `output_col` - Nombre de la columna de salida (default: "trix")
///
/// # Retorna
/// DataFrame con columna "trix" añadida
///
/// # Fórmula
/// TRIX = ((EMA3[t] - EMA3[t-1]) / EMA3[t-1]) * 100
/// donde EMA3 = EMA(EMA(EMA(close)))
pub fn trix(df: &mut DataFrame, timeperiod: Option<usize>, output_col: Option<&str>) {
    let timeperiod = timeperiod.unwrap_or(30);
    let output_col = output_col.unwrap_or("trix");
    let close = get_close(&df).unwrap();

    let ema1 = ema_series(&close, timeperiod);
    let ema2 = ema_series(&ema1, timeperiod);
    let ema3 = ema_series(&ema2, timeperiod);

    let n = ema3.len();
    let ema3_ca = ema3.f64().unwrap();

    // Convertimos a vector manejando nulos para evitar desalineación
    let ema3_vals: Vec<f64> = ema3_ca.into_iter().map(|v| v.unwrap_or(f64::NAN)).collect();

    let mut trix_vals: Vec<f64> = vec![f64::NAN; n];

    for i in 1..n {
        let curr = ema3_vals[i];
        let prev = ema3_vals[i - 1];

        if !curr.is_nan() && !prev.is_nan() && prev != 0.0 {
            trix_vals[i] = ((curr - prev) / prev) * 100.0;
        }
    }

    df.with_column(Series::new(output_col.into(), trix_vals).into())
        .unwrap();
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UltoscParams {
    pub timeperiod1: usize,
    pub timeperiod2: usize,
    pub timeperiod3: usize,
}

/// ULTOSC - Ultimate Oscillator
///
/// Oscilador multi-tiempo que reduce señales falsas usando tres períodos diferentes.
///
/// # Parámetros
/// * `df` - DataFrame con columnas: high, low, close (case insensitive)
/// * `timeperiod1` - Período corto (default: 7)
/// * `timeperiod2` - Período medio (default: 14)
/// * `timeperiod3` - Período largo (default: 28)
/// * `output_col` - Nombre de la columna de salida (default: "ultosc")
///
/// # Retorna
/// DataFrame con columna "ultosc" añadida
///
/// # Fórmula
/// BP = close - min(low, prev_close)
/// TR = max(high, prev_close) - min(low, prev_close)
/// Avg = sum(BP) / sum(TR)
/// ULTOSC = 100 * (4*Avg1 + 2*Avg2 + Avg3) / 7
pub fn ultosc(
    df: &mut DataFrame,
    timeperiod1: Option<usize>,
    timeperiod2: Option<usize>,
    timeperiod3: Option<usize>,
    output_col: Option<&str>,
) {
    let t1 = timeperiod1.unwrap_or(7);
    let t2 = timeperiod2.unwrap_or(14);
    let t3 = timeperiod3.unwrap_or(28);
    let output_col = output_col.unwrap_or("ultosc");

    let high = get_high(&df).unwrap();
    let low = get_low(&df).unwrap();
    let close = get_close(&df).unwrap();

    let h_ca = high.f64().unwrap();
    let l_ca = low.f64().unwrap();
    let c_ca = close.f64().unwrap();
    let n = df.height();

    let mut bp = vec![0.0; n];
    let mut tr = vec![0.0; n];

    // Acceso rápido a valores manejando nulos
    let h_v: Vec<f64> = h_ca.into_iter().map(|v| v.unwrap_or(f64::NAN)).collect();
    let l_v: Vec<f64> = l_ca.into_iter().map(|v| v.unwrap_or(f64::NAN)).collect();
    let c_v: Vec<f64> = c_ca.into_iter().map(|v| v.unwrap_or(f64::NAN)).collect();

    for i in 1..n {
        let curr_c = c_v[i];
        let curr_h = h_v[i];
        let curr_l = l_v[i];
        let prev_c = c_v[i - 1];

        if !curr_c.is_nan() && !curr_h.is_nan() && !curr_l.is_nan() && !prev_c.is_nan() {
            let min_l_pc = curr_l.min(prev_c);
            let max_h_pc = curr_h.max(prev_c);

            bp[i] = curr_c - min_l_pc;
            tr[i] = max_h_pc - min_l_pc;
        }
    }

    let mut ultosc_vals = vec![f64::NAN; n];
    let max_period = t1.max(t2).max(t3);

    if n > max_period {
        // Sumas rodantes para cada periodo
        let mut sum_bp1: f64 = bp[1..=t1].iter().sum();
        let mut sum_tr1: f64 = tr[1..=t1].iter().sum();
        let mut sum_bp2: f64 = bp[1..=t2].iter().sum();
        let mut sum_tr2: f64 = tr[1..=t2].iter().sum();
        let mut sum_bp3: f64 = bp[1..=t3].iter().sum();
        let mut sum_tr3: f64 = tr[1..=t3].iter().sum();

        for i in max_period..n {
            // Actualizamos sumas rodantes si i > period
            if i > t1 {
                sum_bp1 = sum_bp1 - bp[i - t1] + bp[i];
                sum_tr1 = sum_tr1 - tr[i - t1] + tr[i];
            }
            if i > t2 {
                sum_bp2 = sum_bp2 - bp[i - t2] + bp[i];
                sum_tr2 = sum_tr2 - tr[i - t2] + tr[i];
            }
            if i > t3 {
                sum_bp3 = sum_bp3 - bp[i - t3] + bp[i];
                sum_tr3 = sum_tr3 - tr[i - t3] + tr[i];
            }

            let avg1 = if sum_tr1 != 0.0 {
                sum_bp1 / sum_tr1
            } else {
                0.0
            };
            let avg2 = if sum_tr2 != 0.0 {
                sum_bp2 / sum_tr2
            } else {
                0.0
            };
            let avg3 = if sum_tr3 != 0.0 {
                sum_bp3 / sum_tr3
            } else {
                0.0
            };

            ultosc_vals[i] = 100.0 * (4.0 * avg1 + 2.0 * avg2 + avg3) / 7.0;
        }
    }

    let ultosc_series = Series::new(output_col.into(), ultosc_vals);

    df.with_column(ultosc_series.into()).unwrap();
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WillrParams {
    pub timeperiod: usize,
}

/// WILLR - Williams' %R
///
/// Oscilador de momento que mide el nivel de cierre respecto al máximo-mínimo.
/// Escala invertida: -100 = sobreventa, 0 = sobrecompra.
///
/// # Parámetros
/// * `df` - DataFrame con columnas: high, low, close (case insensitive)
/// * `timeperiod` - Período de cálculo (default: 14)
/// * `output_col` - Nombre de la columna de salida (default: "willr")
///
/// # Retorna
/// DataFrame con columna "willr" añadida
///
/// # Fórmula
/// %R = ((highest_high - close) / (highest_high - lowest_low)) * -100
pub fn willr(df: &mut DataFrame, timeperiod: Option<usize>, output_col: Option<&str>) {
    let timeperiod = timeperiod.unwrap_or(14).max(2);
    let output_col = output_col.unwrap_or("willr");
    let high = get_high(&df).unwrap();
    let low = get_low(&df).unwrap();
    let close = get_close(&df).unwrap();

    let high_ca = high.f64().unwrap();
    let low_ca = low.f64().unwrap();
    let close_ca = close.f64().unwrap();

    let n = high_ca.len();
    let mut willr_vals: Vec<f64> = vec![f64::NAN; n];

    // Convertimos a vectores manejando nulos para asegurar la alineación
    let h_v: Vec<f64> = high_ca.into_iter().map(|v| v.unwrap_or(f64::NAN)).collect();
    let l_v: Vec<f64> = low_ca.into_iter().map(|v| v.unwrap_or(f64::NAN)).collect();
    let c_v: Vec<f64> = close_ca
        .into_iter()
        .map(|v| v.unwrap_or(f64::NAN))
        .collect();

    for i in (timeperiod - 1)..n {
        let start_j = i + 1 - timeperiod;
        let window_h = &h_v[start_j..=i];
        let window_l = &l_v[start_j..=i];

        let mut hh = f64::NEG_INFINITY;
        let mut ll = f64::INFINITY;
        let mut valid = true;

        for &val in window_h {
            if val.is_nan() {
                valid = false;
                break;
            }
            if val > hh {
                hh = val;
            }
        }
        if !valid {
            continue;
        }

        for &val in window_l {
            if val.is_nan() {
                valid = false;
                break;
            }
            if val < ll {
                ll = val;
            }
        }

        if valid && hh != ll {
            let curr_c = c_v[i];
            if !curr_c.is_nan() {
                willr_vals[i] = ((hh - curr_c) / (hh - ll)) * -100.0;
            }
        }
    }

    df.with_column(Series::new(output_col.into(), willr_vals).into())
        .unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    //Para los test crear una carpeta llamada download en la raiz de este proyecto
    // y llamar a los datos test.csv
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
    fn test_adx() {
        match load_data() {
            Ok(mut df) => {
                adx(&mut df, Some(14), None);
                save_data(&df, "download/test_adx.csv").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[test]
    fn test_adxr() {
        match load_data() {
            Ok(mut df) => {
                adxr(&mut df, Some(14), None);
                save_data(&df, "download/test_adxr.csv").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[test]
    fn test_apo() {
        match load_data() {
            Ok(mut df) => {
                apo(&mut df, Some(12), Some(26), None);
                save_data(&df, "download/test_apo.csv").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[test]
    fn test_aroon() {
        match load_data() {
            Ok(mut df) => {
                aroon(&mut df, Some(14), None, None);
                save_data(&df, "download/test_aroon.csv").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[test]
    fn test_aroonosc() {
        match load_data() {
            Ok(mut df) => {
                aroonosc(&mut df, Some(14), None);
                save_data(&df, "download/test_aroonosc.csv").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[test]
    fn test_bop() {
        match load_data() {
            Ok(mut df) => {
                bop(&mut df, None, None);
                save_data(&df, "download/test_bop.csv").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[test]
    fn test_cci() {
        match load_data() {
            Ok(mut df) => {
                cci(&mut df, None, None);
                save_data(&df, "download/test_cci.csv").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[test]
    fn test_cmo() {
        match load_data() {
            Ok(mut df) => {
                cmo(&mut df, None, None);
                save_data(&df, "download/test_cmo.csv").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[test]
    fn test_dx() {
        match load_data() {
            Ok(mut df) => {
                dx(&mut df, None, None);
                save_data(&df, "download/test_dx.csv").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[test]
    fn test_macd() {
        match load_data() {
            Ok(mut df) => {
                macd(&mut df, Some(12), Some(26), Some(9), None, None, None);
                save_data(&df, "download/test_macd.csv").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[test]
    fn test_macdext() {
        match load_data() {
            Ok(mut df) => {
                macdext(
                    &mut df, None, None, None, None, None, None, None, None, None,
                );
                save_data(&df, "download/test_macdext.csv").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[test]
    fn test_macdfix() {
        match load_data() {
            Ok(mut df) => {
                macdfix(&mut df, None, None, None, None);
                save_data(&df, "download/test_macdfix.csv").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[test]
    fn test_mfi() {
        match load_data() {
            Ok(mut df) => {
                mfi(&mut df, None, None);
                save_data(&df, "download/test_mfi.csv").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[test]
    fn test_minus_di() {
        match load_data() {
            Ok(mut df) => {
                minus_di(&mut df, None, None);
                save_data(&df, "download/test_minus_di.csv").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[test]
    fn test_minus_dm() {
        match load_data() {
            Ok(mut df) => {
                minus_dm(&mut df, None, None);
                save_data(&df, "download/test_minus_dm.csv").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[test]
    fn test_mom() {
        match load_data() {
            Ok(mut df) => {
                mom(&mut df, None, None);
                save_data(&df, "download/test_mom.csv").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[test]
    fn test_plus_di() {
        match load_data() {
            Ok(mut df) => {
                plus_di(&mut df, None, None);
                save_data(&df, "download/test_plus_di.csv").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[test]
    fn test_plus_dm() {
        match load_data() {
            Ok(mut df) => {
                plus_dm(&mut df, None, None);
                save_data(&df, "download/test_plus_dm.csv").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[test]
    fn test_ppo() {
        match load_data() {
            Ok(mut df) => {
                ppo(&mut df, None, None, None);
                save_data(&df, "download/test_ppo.csv").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[test]
    fn test_roc() {
        match load_data() {
            Ok(mut df) => {
                roc(&mut df, None, None);
                save_data(&df, "download/test_roc.csv").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[test]
    fn test_rocp() {
        match load_data() {
            Ok(mut df) => {
                rocp(&mut df, None, None);
                save_data(&df, "download/test_rocp.csv").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[test]
    fn test_rocr() {
        match load_data() {
            Ok(mut df) => {
                rocr(&mut df, None, None);
                save_data(&df, "download/test_rocr.csv").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[test]
    fn test_rocr100() {
        match load_data() {
            Ok(mut df) => {
                rocr100(&mut df, None, None);
                save_data(&df, "download/test_rocr100.csv").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[test]
    fn test_rsi() {
        match load_data() {
            Ok(mut df) => {
                rsi(&mut df, None, None);
                save_data(&df, "download/test_rsi.csv").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[test]
    fn test_stoch() {
        match load_data() {
            Ok(mut df) => {
                stoch(&mut df, None, None, None, None, None, None);
                save_data(&df, "download/test_stoch.csv").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[test]
    fn test_stochf() {
        match load_data() {
            Ok(mut df) => {
                stochf(&mut df, None, None, None, None, None);
                save_data(&df, "download/test_stochf.csv").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[test]
    fn test_stochrsi() {
        match load_data() {
            Ok(mut df) => {
                stochrsi(&mut df, None, None, None, None, None, None);
                save_data(&df, "download/test_stochrsi.csv").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[test]
    fn test_trix() {
        match load_data() {
            Ok(mut df) => {
                trix(&mut df, None, None);
                save_data(&df, "download/test_trix.csv").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[test]
    fn test_ultosc() {
        match load_data() {
            Ok(mut df) => {
                ultosc(&mut df, None, None, None, None);
                save_data(&df, "download/test_ultosc.csv").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[test]
    fn test_willr() {
        match load_data() {
            Ok(mut df) => {
                willr(&mut df, None, None);
                save_data(&df, "download/test_willr.csv").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }
}
