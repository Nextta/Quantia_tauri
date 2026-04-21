use polars::prelude::*;

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
        for j in (i - period + 1)..=i {
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
pub async fn adx(
    df: DataFrame,
    timeperiod: Option<usize>,
    output_col: Option<&str>,
) -> PolarsResult<DataFrame> {
    let timeperiod = timeperiod.unwrap_or(14);
    let output_name = output_col.unwrap_or("adx");
    let high = get_high(&df)?;
    let low = get_low(&df)?;
    let close = get_close(&df)?;

    let high_vals = high.f64().unwrap();
    let low_vals = low.f64().unwrap();
    let close_vals = close.f64().unwrap();

    let n = high_vals.len();

    if n < (timeperiod * 2) {
        let mut result_df = df.clone();
        result_df.with_column(Series::new(output_name.into(), vec![f64::NAN; n]).into())?;
        return Ok(result_df);
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

    let mut result_df = df.clone();
    let adx_final = adx_series.with_name(output_name.into());
    result_df.with_column(adx_final.into())?;
    Ok(result_df)
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
pub async fn adxr(
    df: DataFrame,
    timeperiod: Option<usize>,
    output_col: Option<&str>,
) -> PolarsResult<DataFrame> {
    let timeperiod = timeperiod.unwrap_or(14);
    let output_name = output_col.unwrap_or("adxr");

    // Calcular ADX primero
    let adx_df = adx(df.clone(), Some(timeperiod), Some("temp_adx")).await?;
    let adx_col = adx_df.column("temp_adx").unwrap().f64().unwrap();

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
    let mut result_df = df;
    result_df.with_column(adxr_series.into())?;
    Ok(result_df)
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
pub async fn apo(
    df: DataFrame,
    fastperiod: Option<usize>,
    slowperiod: Option<usize>,
    output_col: Option<&str>,
) -> PolarsResult<DataFrame> {
    let fastperiod = fastperiod.unwrap_or(12);
    let slowperiod = slowperiod.unwrap_or(26);
    let output_name = output_col.unwrap_or("apo");

    let close = get_close(&df)?;

    let fast_ema = ema_series(&close, fastperiod);
    let slow_ema = ema_series(&close, slowperiod);

    let fast_ca = fast_ema.f64()?;
    let slow_ca = slow_ema.f64()?;

    let apo_vals: Vec<f64> = fast_ca
        .into_no_null_iter()
        .zip(slow_ca.into_no_null_iter())
        .map(|(f, s)| f - s)
        .collect();

    let apo_series = Series::new(output_name.into(), apo_vals);

    let mut result_df = df;
    result_df.with_column(apo_series.into())?;
    Ok(result_df)
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
pub async fn aroon(
    df: DataFrame,
    timeperiod: Option<usize>,
    output_col_up: Option<&str>,
    output_col_down: Option<&str>,
) -> PolarsResult<DataFrame> {
    let timeperiod = timeperiod.unwrap_or(14);
    let output_col_up = output_col_up.unwrap_or("aroon_up");
    let output_col_down = output_col_down.unwrap_or("aroon_down");

    let high = get_high(&df)?;
    let low = get_low(&df)?;

    let high_ca: ChunkedArray<Float64Type> = high.f64().unwrap().clone();
    let low_ca: ChunkedArray<Float64Type> = low.f64().unwrap().clone();
    let high_vals: Vec<f64> = high_ca.into_no_null_iter().collect();
    let low_vals: Vec<f64> = low_ca.into_no_null_iter().collect();

    let n = high_vals.len();
    let mut aroon_up: Vec<f64> = vec![f64::NAN; n];
    let mut aroon_down: Vec<f64> = vec![f64::NAN; n];

    for i in timeperiod..n {
        let window_high = &high_vals[i - timeperiod + 1..=i];
        let window_low = &low_vals[i - timeperiod + 1..=i];

        let max_idx = window_high
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(idx, _)| idx)
            .unwrap_or(0);
        let min_idx = window_low
            .iter()
            .enumerate()
            .min_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(idx, _)| idx)
            .unwrap_or(0);

        aroon_up[i] =
            ((timeperiod as f64 - (timeperiod - 1 - max_idx) as f64) / timeperiod as f64) * 100.0;
        aroon_down[i] =
            ((timeperiod as f64 - (timeperiod - 1 - min_idx) as f64) / timeperiod as f64) * 100.0;
    }

    let aroon_up_series = Series::new(output_col_up.into(), &aroon_up);
    let aroon_down_series = Series::new(output_col_down.into(), &aroon_down);

    let mut result_df = df;
    result_df
        .with_column(aroon_up_series.into())?
        .with_column(aroon_down_series.into())?;
    Ok(result_df)
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
pub async fn aroonosc(
    df: DataFrame,
    timeperiod: Option<usize>,
    output_col: Option<&str>,
) -> PolarsResult<DataFrame> {
    let timeperiod = timeperiod.unwrap_or(14);
    let output_col = output_col.unwrap_or("aroonosc");
    let aroon_df = aroon(df.clone(), Some(timeperiod), None, None).await?;

    let aroon_up_col = aroon_df.column("aroon_up").unwrap();
    let aroon_down_col = aroon_df.column("aroon_down").unwrap();

    let up_ca: ChunkedArray<Float64Type> = aroon_up_col.f64().unwrap().clone();
    let down_ca: ChunkedArray<Float64Type> = aroon_down_col.f64().unwrap().clone();
    let up_vals: Vec<f64> = up_ca.into_no_null_iter().collect();
    let down_vals: Vec<f64> = down_ca.into_no_null_iter().collect();

    let aroonosc_vals: Vec<f64> = up_vals
        .iter()
        .zip(down_vals.iter())
        .map(|(&u, &d)| {
            if u.is_nan() || d.is_nan() {
                f64::NAN
            } else {
                u - d
            }
        })
        .collect();

    let aroonosc_series = Series::new(output_col.into(), &aroonosc_vals);
    let mut result_df = df;
    result_df.with_column(aroonosc_series.into())?;
    Ok(result_df)
}

/// BOP - Balance Of Power
///
/// Mide la fuerza del precio de cierre en relación con el rango alto-bajo.
///
/// # Parámetros
/// * `df` - DataFrame con columnas: open, high, low, close (case insensitive)
/// * `output_col` - Nombre de la columna de salida (default: "bop")
///
/// # Retorna
/// DataFrame con columna "bop" añadida
///
/// # Fórmula
/// BOP = (close - open) / (high - low)
pub async fn bop(df: DataFrame, output_col: Option<&str>) -> PolarsResult<DataFrame> {
    let open = get_open(&df)?;
    let high = get_high(&df)?;
    let low = get_low(&df)?;
    let close = get_close(&df)?;
    let output_col = output_col.unwrap_or("bop");

    let open_ca: ChunkedArray<Float64Type> = open.f64().unwrap().clone();
    let high_ca: ChunkedArray<Float64Type> = high.f64().unwrap().clone();
    let low_ca: ChunkedArray<Float64Type> = low.f64().unwrap().clone();
    let close_ca: ChunkedArray<Float64Type> = close.f64().unwrap().clone();

    let open_vals: Vec<f64> = open_ca.into_no_null_iter().collect();
    let high_vals: Vec<f64> = high_ca.into_no_null_iter().collect();
    let low_vals: Vec<f64> = low_ca.into_no_null_iter().collect();
    let close_vals: Vec<f64> = close_ca.into_no_null_iter().collect();

    let bop_vals: Vec<f64> = close_vals
        .iter()
        .zip(&open_vals)
        .zip(&high_vals)
        .zip(&low_vals)
        .map(|(((c, o), h), l)| {
            let denominator = h - l;
            if denominator == 0.0 {
                f64::NAN
            } else {
                (c - o) / denominator
            }
        })
        .collect();

    let bop_series = Series::new(output_col.into(), &bop_vals);
    let mut result_df = df;
    result_df.with_column(bop_series.into())?;
    Ok(result_df)
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
pub async fn cci(
    df: DataFrame,
    timeperiod: Option<usize>,
    output_col: Option<&str>,
) -> PolarsResult<DataFrame> {
    let timeperiod = timeperiod.unwrap_or(14);
    let timeperiod = timeperiod.max(2); // Prevent overflow
    let output_col = output_col.unwrap_or("cci");
    let high = get_high(&df)?;
    let low = get_low(&df)?;
    let close = get_close(&df)?;

    let high_ca: ChunkedArray<Float64Type> = high.f64().unwrap().clone();
    let low_ca: ChunkedArray<Float64Type> = low.f64().unwrap().clone();
    let close_ca: ChunkedArray<Float64Type> = close.f64().unwrap().clone();

    let n = df.height();
    let mut typical_price: Vec<f64> = vec![f64::NAN; n];

    for i in 0..n {
        if let (Some(h), Some(l), Some(c)) = (high_ca.get(i), low_ca.get(i), close_ca.get(i)) {
            typical_price[i] = (h + l + c) / 3.0;
        }
    }

    let mut cci_vals: Vec<f64> = vec![f64::NAN; n];

    for i in (timeperiod - 1)..n {
        let window = &typical_price[i - timeperiod + 1..=i];
        let mean: f64 = window.iter().sum::<f64>() / timeperiod as f64;

        let mean_deviation: f64 =
            window.iter().map(|x| (x - mean).abs()).sum::<f64>() / timeperiod as f64;

        if mean_deviation != 0.0 {
            cci_vals[i] = (typical_price[i] - mean) / (0.015 * mean_deviation);
        }
    }

    let cci_series = Series::new(output_col.into(), &cci_vals);
    let mut result_df = df;
    result_df.with_column(cci_series.into())?;
    Ok(result_df)
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
pub async fn cmo(
    df: DataFrame,
    timeperiod: Option<usize>,
    output_col: Option<&str>,
) -> PolarsResult<DataFrame> {
    let timeperiod = timeperiod.unwrap_or(14);
    let output_col = output_col.unwrap_or("cmo");

    let close = get_close(&df)?;

    let close_ca: ChunkedArray<Float64Type> = close.f64().unwrap().clone();
    let close_vals: Vec<f64> = close_ca.into_no_null_iter().collect();

    let n = close_vals.len();
    let mut cmo_vals: Vec<f64> = vec![f64::NAN; n];

    for i in timeperiod..n {
        let mut sum_up = 0.0;
        let mut sum_down = 0.0;

        for j in (i - timeperiod + 1)..=i {
            let change = close_vals[j] - close_vals[j - 1];
            if change > 0.0 {
                sum_up += change;
            } else if change < 0.0 {
                sum_down += change.abs();
            }
        }

        let total = sum_up + sum_down;
        if total != 0.0 {
            cmo_vals[i] = 100.0 * ((sum_up - sum_down) / total);
        }
    }

    let cmo_series = Series::new(output_col.into(), &cmo_vals);
    let mut result_df = df;
    result_df.with_column(cmo_series.into())?;
    Ok(result_df)
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
pub async fn dx(
    df: DataFrame,
    timeperiod: Option<usize>,
    output_col: Option<&str>,
) -> PolarsResult<DataFrame> {
    let timeperiod = timeperiod.unwrap_or(14);
    let output_name = output_col.unwrap_or("dx");
    let high = get_high(&df)?;
    let low = get_low(&df)?;
    let close = get_close(&df)?;

    let high_vals = high.f64().unwrap();
    let low_vals = low.f64().unwrap();
    let close_vals = close.f64().unwrap();

    let n = high_vals.len();
    let mut plus_dm = vec![0.0; n];
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

    let dx_series = Series::new(output_name.into(), dx_vals);
    let mut result_df = df;
    result_df.with_column(dx_series.into())?;
    Ok(result_df)
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
pub async fn macd(
    df: DataFrame,
    fastperiod: Option<usize>,
    slowperiod: Option<usize>,
    signalperiod: Option<usize>,
    output_col: Option<&str>,
    output_col_signal: Option<&str>,
    output_col_hist: Option<&str>,
) -> PolarsResult<DataFrame> {
    let fastperiod = fastperiod.unwrap_or(12);
    let slowperiod = slowperiod.unwrap_or(26);
    let signalperiod = signalperiod.unwrap_or(9);
    let output_col = output_col.unwrap_or("macd");
    let output_col_signal = output_col_signal.unwrap_or("macd_signal");
    let output_col_hist = output_col_hist.unwrap_or("macd_hist");

    let close = get_close(&df)?;
    let fast_ema = ema_series(&close, fastperiod);
    let slow_ema = ema_series(&close, slowperiod);

    // Get values preserving NaN positions (use f64 directly, not into_no_null_iter)
    let fast_ca: ChunkedArray<Float64Type> = fast_ema.f64().unwrap().clone();
    let slow_ca: ChunkedArray<Float64Type> = slow_ema.f64().unwrap().clone();

    // Convert to vec maintaining positions (ChunkedArray already has the right length)
    let n = fast_ema.len();
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

    let signal_ca: ChunkedArray<Float64Type> = signal.f64().unwrap().clone();
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

    let macd_series_final = Series::new(output_col.into(), &macd_vals);
    let signal_series = Series::new(output_col_signal.into(), &signal_vals);
    let hist_series = Series::new(output_col_hist.into(), &hist_vals);

    let mut result_df = df;
    result_df
        .with_column(macd_series_final.into())?
        .with_column(signal_series.into())?
        .with_column(hist_series.into())?;
    Ok(result_df)
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
pub async fn macdext(
    df: DataFrame,
    fastperiod: Option<usize>,
    slowperiod: Option<usize>,
    signalperiod: Option<usize>,
    fastmatype: Option<usize>,
    slowmatype: Option<usize>,
    signalmatype: Option<usize>,
    output_col: Option<&str>,
    output_col_signal: Option<&str>,
    output_col_hist: Option<&str>,
) -> PolarsResult<DataFrame> {
    let fastperiod = fastperiod.unwrap_or(12);
    let slowperiod = slowperiod.unwrap_or(26);
    let signalperiod = signalperiod.unwrap_or(9);
    let fastmatype = fastmatype.unwrap_or(0);
    let slowmatype = slowmatype.unwrap_or(0);
    let signalmatype = signalmatype.unwrap_or(0);
    let output_col = output_col.unwrap_or("macd");
    let output_col_signal = output_col_signal.unwrap_or("macd_signal");
    let output_col_hist = output_col_hist.unwrap_or("macd_hist");

    let close = get_close(&df)?;

    // 0 = EMA, 1 = SMA (simplified)
    let fast_ema = if fastmatype == 0 {
        ema_series(&close, fastperiod)
    } else {
        sma_series(&close, fastperiod)
    };
    let slow_ema = if slowmatype == 0 {
        ema_series(&close, slowperiod)
    } else {
        sma_series(&close, slowperiod)
    };

    let fast_ca: ChunkedArray<Float64Type> = fast_ema.f64().unwrap().clone();
    let slow_ca: ChunkedArray<Float64Type> = slow_ema.f64().unwrap().clone();

    let n = fast_ema.len();
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
    let signal = if signalmatype == 0 {
        ema_series(&macd_series, signalperiod)
    } else {
        sma_series(&macd_series, signalperiod)
    };

    let signal_ca: ChunkedArray<Float64Type> = signal.f64().unwrap().clone();
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

    let macd_series_final = Series::new(output_col.into(), &macd_vals);
    let signal_series = Series::new(output_col_signal.into(), &signal_vals);
    let hist_series = Series::new(output_col_hist.into(), &hist_vals);

    let mut result_df = df;
    result_df
        .with_column(macd_series_final.into())?
        .with_column(signal_series.into())?
        .with_column(hist_series.into())?;
    Ok(result_df)
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
pub async fn macdfix(df: DataFrame, signalperiod: Option<usize>) -> PolarsResult<DataFrame> {
    let signalperiod = signalperiod.unwrap_or(9);
    macd(df, Some(12), Some(26), Some(signalperiod), None, None, None).await
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
pub async fn mfi(
    df: DataFrame,
    timeperiod: Option<usize>,
    output_col: Option<&str>,
) -> PolarsResult<DataFrame> {
    let timeperiod = timeperiod.unwrap_or(14);
    let output_col = output_col.unwrap_or("mfi");
    let high = get_high(&df)?;
    let low = get_low(&df)?;
    let close = get_close(&df)?;
    let volume = get_volume(&df)?;

    let high_ca: ChunkedArray<Float64Type> = high.f64().unwrap().clone();
    let low_ca: ChunkedArray<Float64Type> = low.f64().unwrap().clone();
    let close_ca: ChunkedArray<Float64Type> = close.f64().unwrap().clone();
    let vol_ca: ChunkedArray<Float64Type> = volume.f64().unwrap().clone();

    let high_vals: Vec<f64> = high_ca.into_no_null_iter().collect();
    let low_vals: Vec<f64> = low_ca.into_no_null_iter().collect();
    let close_vals: Vec<f64> = close_ca.into_no_null_iter().collect();
    let vol_vals: Vec<f64> = vol_ca.into_no_null_iter().collect();

    let n = high_vals.len();
    let typical_price: Vec<f64> = high_vals
        .iter()
        .zip(&low_vals)
        .zip(&close_vals)
        .map(|((h, l), c)| (h + l + c) / 3.0)
        .collect();

    let mut money_flow: Vec<f64> = vec![0.0; n];
    for i in 0..n {
        money_flow[i] = typical_price[i] * vol_vals[i];
    }

    let mut mfi_vals: Vec<f64> = vec![f64::NAN; n];

    for i in timeperiod..n {
        let mut positive_flow = 0.0;
        let mut negative_flow = 0.0;

        for j in (i - timeperiod + 1)..=i {
            if typical_price[j] > typical_price[j - 1] {
                positive_flow += money_flow[j];
            } else {
                negative_flow += money_flow[j];
            }
        }

        if negative_flow != 0.0 {
            let money_ratio = positive_flow / negative_flow;
            mfi_vals[i] = 100.0 - (100.0 / (1.0 + money_ratio));
        } else {
            mfi_vals[i] = 100.0;
        }
    }

    let mfi_series = Series::new(output_col.into(), &mfi_vals);
    let mut result_df = df;
    result_df.with_column(mfi_series.into())?;
    Ok(result_df)
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
pub async fn minus_di(
    df: DataFrame,
    timeperiod: Option<usize>,
    output_col: Option<&str>,
) -> PolarsResult<DataFrame> {
    let timeperiod = timeperiod.unwrap_or(14);
    let output_name = output_col.unwrap_or("minus_di");
    let high = get_high(&df)?;
    let low = get_low(&df)?;
    let close = get_close(&df)?;

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
    let mut result_df = df;
    result_df.with_column(minus_di_series.into())?;
    Ok(result_df)
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
pub async fn minus_dm(
    df: DataFrame,
    timeperiod: Option<usize>,
    output_col: Option<&str>,
) -> PolarsResult<DataFrame> {
    let timeperiod = timeperiod.unwrap_or(14);
    let output_name = output_col.unwrap_or("minus_dm");
    let high = get_high(&df)?;
    let low = get_low(&df)?;

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

    let mut result_df = df;
    result_df.with_column(smoothed_minus_dm.into())?;
    Ok(result_df)
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
pub async fn mom(
    df: DataFrame,
    timeperiod: Option<usize>,
    output_col: Option<&str>,
) -> PolarsResult<DataFrame> {
    let timeperiod = timeperiod.unwrap_or(10);
    let output_col = output_col.unwrap_or("mom");
    let close = get_close(&df)?;

    let close_ca: ChunkedArray<Float64Type> = close.f64().unwrap().clone();
    let close_vals: Vec<f64> = close_ca.into_no_null_iter().collect();

    let n = close_vals.len();
    let mut mom_vals: Vec<f64> = vec![f64::NAN; n];

    for i in timeperiod..n {
        mom_vals[i] = close_vals[i] - close_vals[i - timeperiod];
    }

    let mom_series = Series::new(output_col.into(), &mom_vals);
    let mut result_df = df;
    result_df.with_column(mom_series.into())?;
    Ok(result_df)
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
pub async fn plus_di(
    df: DataFrame,
    timeperiod: Option<usize>,
    output_col: Option<&str>,
) -> PolarsResult<DataFrame> {
    let timeperiod = timeperiod.unwrap_or(14);
    let output_name = output_col.unwrap_or("plus_di");
    let high = get_high(&df)?;
    let low = get_low(&df)?;
    let close = get_close(&df)?;

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
    let mut result_df = df;
    result_df.with_column(plus_di_series.into())?;
    Ok(result_df)
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
pub async fn plus_dm(
    df: DataFrame,
    timeperiod: Option<usize>,
    output_col: Option<&str>,
) -> PolarsResult<DataFrame> {
    let timeperiod = timeperiod.unwrap_or(14);
    let output_name = output_col.unwrap_or("plus_dm");
    let high = get_high(&df)?;
    let low = get_low(&df)?;

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

    let mut result_df = df;
    result_df.with_column(smoothed_plus_dm.into())?;
    Ok(result_df)
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
pub async fn ppo(
    df: DataFrame,
    fastperiod: Option<usize>,
    slowperiod: Option<usize>,
    output_col: Option<&str>,
) -> PolarsResult<DataFrame> {
    let fastperiod = fastperiod.unwrap_or(12);
    let slowperiod = slowperiod.unwrap_or(26);
    let output_name = output_col.unwrap_or("ppo");
    let close = get_close(&df)?;

    let fast_ema = ema_series(&close, fastperiod);
    let slow_ema = ema_series(&close, slowperiod);
    let fast_ca = fast_ema.f64()?;
    let slow_ca = slow_ema.f64()?;

    let ppo_vals: Vec<f64> = fast_ca
        .into_no_null_iter()
        .zip(slow_ca.into_no_null_iter())
        .map(|(f, s)| ((f - s) / s) * 100.0)
        .collect();

    let ppo_series = Series::new(output_name.into(), ppo_vals);

    let mut result_df = df;
    result_df.with_column(ppo_series.into())?;
    Ok(result_df)
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
pub async fn roc(
    df: DataFrame,
    timeperiod: Option<usize>,
    output_col: Option<&str>,
) -> PolarsResult<DataFrame> {
    let timeperiod = timeperiod.unwrap_or(10);
    let output_col = output_col.unwrap_or("roc");
    let close = get_close(&df)?;

    let close_ca: ChunkedArray<Float64Type> = close.f64().unwrap().clone();
    let close_vals: Vec<f64> = close_ca.into_no_null_iter().collect();

    let n = close_vals.len();
    let mut roc_vals: Vec<f64> = vec![f64::NAN; n];

    for i in timeperiod..n {
        if close_vals[i - timeperiod] != 0.0 {
            roc_vals[i] = ((close_vals[i] / close_vals[i - timeperiod]) - 1.0) * 100.0;
        }
    }

    let roc_series = Series::new(output_col.into(), &roc_vals);
    let mut result_df = df;
    result_df.with_column(roc_series.into())?;
    Ok(result_df)
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
pub async fn rocp(
    df: DataFrame,
    timeperiod: Option<usize>,
    output_col: Option<&str>,
) -> PolarsResult<DataFrame> {
    let timeperiod = timeperiod.unwrap_or(10);
    let output_col = output_col.unwrap_or("rocp");
    let close = get_close(&df)?;

    let close_ca: ChunkedArray<Float64Type> = close.f64().unwrap().clone();
    let close_vals: Vec<f64> = close_ca.into_no_null_iter().collect();

    let n = close_vals.len();
    let mut rocp_vals: Vec<f64> = vec![f64::NAN; n];

    for i in timeperiod..n {
        if close_vals[i - timeperiod] != 0.0 {
            rocp_vals[i] =
                (close_vals[i] - close_vals[i - timeperiod]) / close_vals[i - timeperiod];
        }
    }

    let rocp_series = Series::new(output_col.into(), &rocp_vals);
    let mut result_df = df;
    result_df.with_column(rocp_series.into())?;
    Ok(result_df)
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
pub async fn rocr(
    df: DataFrame,
    timeperiod: Option<usize>,
    output_col: Option<&str>,
) -> PolarsResult<DataFrame> {
    let timeperiod = timeperiod.unwrap_or(10);
    let output_col = output_col.unwrap_or("rocr");
    let close = get_close(&df)?;

    let close_ca: ChunkedArray<Float64Type> = close.f64().unwrap().clone();
    let close_vals: Vec<f64> = close_ca.into_no_null_iter().collect();

    let n = close_vals.len();
    let mut rocr_vals: Vec<f64> = vec![f64::NAN; n];

    for i in timeperiod..n {
        if close_vals[i - timeperiod] != 0.0 {
            rocr_vals[i] = close_vals[i] / close_vals[i - timeperiod];
        }
    }

    let rocr_series = Series::new(output_col.into(), &rocr_vals);
    let mut result_df = df;
    result_df.with_column(rocr_series.into())?;
    Ok(result_df)
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
pub async fn rocr100(
    df: DataFrame,
    timeperiod: Option<usize>,
    output_col: Option<&str>,
) -> PolarsResult<DataFrame> {
    let timeperiod = timeperiod.unwrap_or(10);
    let output_col = output_col.unwrap_or("rocr100");
    let close = get_close(&df)?;

    let close_ca: ChunkedArray<Float64Type> = close.f64().unwrap().clone();
    let close_vals: Vec<f64> = close_ca.into_no_null_iter().collect();

    let n = close_vals.len();
    let mut rocr100_vals: Vec<f64> = vec![f64::NAN; n];

    for i in timeperiod..n {
        if close_vals[i - timeperiod] != 0.0 {
            rocr100_vals[i] = (close_vals[i] / close_vals[i - timeperiod]) * 100.0;
        }
    }

    let rocr100_series = Series::new(output_col.into(), &rocr100_vals);
    let mut result_df = df;
    result_df.with_column(rocr100_series.into())?;
    Ok(result_df)
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
pub async fn rsi(
    df: DataFrame,
    timeperiod: Option<usize>,
    output_col: Option<&str>,
) -> PolarsResult<DataFrame> {
    let timeperiod = timeperiod.unwrap_or(14);
    let output_col = output_col.unwrap_or("rsi");
    let close = get_close(&df)?;

    let close_ca: ChunkedArray<Float64Type> = close.f64().unwrap().clone();
    let close_vals: Vec<f64> = close_ca.into_no_null_iter().collect();

    let n = close_vals.len();
    let mut gain: Vec<f64> = vec![0.0; n];
    let mut loss: Vec<f64> = vec![0.0; n];

    for i in 1..n {
        let change = close_vals[i] - close_vals[i - 1];
        gain[i] = if change > 0.0 { change } else { 0.0 };
        loss[i] = if change < 0.0 { change.abs() } else { 0.0 };
    }

    let gain_series = Series::new("gain".into(), &gain);
    let loss_series = Series::new("loss".into(), &loss);

    let avg_gain = rma_series(&gain_series, timeperiod);
    let avg_loss = rma_series(&loss_series, timeperiod);

    let avg_gain_ca: ChunkedArray<Float64Type> = avg_gain.f64().unwrap().clone();
    let avg_loss_ca: ChunkedArray<Float64Type> = avg_loss.f64().unwrap().clone();
    let gain_vals: Vec<f64> = avg_gain_ca.into_no_null_iter().collect();
    let loss_vals: Vec<f64> = avg_loss_ca.into_no_null_iter().collect();

    let mut rsi_vals: Vec<f64> = vec![f64::NAN; n];

    for i in 0..n {
        if !gain_vals[i].is_nan() && !loss_vals[i].is_nan() {
            if loss_vals[i] == 0.0 {
                rsi_vals[i] = 100.0;
            } else {
                let rs = gain_vals[i] / loss_vals[i];
                rsi_vals[i] = 100.0 - (100.0 / (1.0 + rs));
            }
        }
    }

    let rsi_series = Series::new(output_col.into(), &rsi_vals);
    let mut result_df = df;
    result_df.with_column(rsi_series.into())?;
    Ok(result_df)
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
pub async fn stoch(
    df: DataFrame,
    fastk_period: Option<usize>,
    slowk_period: Option<usize>,
    slowk_matype: Option<usize>,
    slowd_period: Option<usize>,
    // slowd_matype: Option<usize>,
    output_col_k: Option<&str>,
    output_col_d: Option<&str>,
) -> PolarsResult<DataFrame> {
    let fastk_period = fastk_period.unwrap_or(5).max(2);
    let slowk_period = slowk_period.unwrap_or(3).max(2);
    let slowd_period = slowd_period.unwrap_or(3).max(2);
    let slowk_matype = slowk_matype.unwrap_or(0);
    // let slowd_matype = slowd_matype.unwrap_or(0);
    let output_col_k = output_col_k.unwrap_or("slow_k");
    let output_col_d = output_col_d.unwrap_or("slow_d");

    let high = get_high(&df)?;
    let low = get_low(&df)?;
    let close = get_close(&df)?;

    let high_ca: ChunkedArray<Float64Type> = high.f64().unwrap().clone();
    let low_ca: ChunkedArray<Float64Type> = low.f64().unwrap().clone();
    let close_ca: ChunkedArray<Float64Type> = close.f64().unwrap().clone();

    let n = df.height();
    let mut k_vals: Vec<f64> = vec![f64::NAN; n];

    for i in (fastk_period - 1)..n {
        let mut hh = f64::NEG_INFINITY;
        let mut ll = f64::INFINITY;
        let mut valid = true;

        for j in (i - fastk_period + 1)..=i {
            match (high_ca.get(j), low_ca.get(j), close_ca.get(j)) {
                (Some(h), Some(l), Some(c)) => {
                    if h > hh {
                        hh = h;
                    }
                    if l < ll {
                        ll = l;
                    }
                }
                _ => {
                    valid = false;
                    break;
                }
            }
        }

        if valid && hh != ll {
            if let Some(c) = close_ca.get(i) {
                k_vals[i] = ((c - ll) / (hh - ll)) * 100.0;
            }
        }
    }

    // Apply slowk smoothing
    let k_series = Series::new("k_raw".into(), &k_vals);
    let k_smooth = if slowk_matype == 0 {
        ema_series(&k_series, slowk_period)
    } else {
        sma_series(&k_series, slowk_period)
    };

    // Calculate %D as SMA of %K
    let d_smooth = sma_series(&k_smooth, slowd_period);

    let k_ca: ChunkedArray<Float64Type> = k_smooth.f64().unwrap().clone();
    let d_ca: ChunkedArray<Float64Type> = d_smooth.f64().unwrap().clone();
    let k_final: Vec<f64> = k_ca.into_no_null_iter().collect();
    let d_final: Vec<f64> = d_ca.into_no_null_iter().collect();

    let k_series_final = Series::new(output_col_k.into(), &k_final);
    let d_series_final = Series::new(output_col_d.into(), &d_final);

    let mut result_df = df;
    result_df
        .with_column(k_series_final.into())?
        .with_column(d_series_final.into())?;
    Ok(result_df)
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
pub async fn stochf(
    df: DataFrame,
    fastk_period: Option<usize>,
    fastd_period: Option<usize>,
    fastd_matype: Option<usize>,
    output_col_k: Option<&str>,
    output_col_d: Option<&str>,
) -> PolarsResult<DataFrame> {
    let fastk_period = fastk_period.unwrap_or(5).max(2);
    let fastd_period = fastd_period.unwrap_or(3).max(2);
    let fastd_matype = fastd_matype.unwrap_or(0);
    let output_col_k = output_col_k.unwrap_or("fast_k");
    let output_col_d = output_col_d.unwrap_or("fast_d");

    let high = get_high(&df)?;
    let low = get_low(&df)?;
    let close = get_close(&df)?;

    let high_ca: ChunkedArray<Float64Type> = high.f64().unwrap().clone();
    let low_ca: ChunkedArray<Float64Type> = low.f64().unwrap().clone();
    let close_ca: ChunkedArray<Float64Type> = close.f64().unwrap().clone();

    let n = df.height();
    let mut fastk_vals: Vec<f64> = vec![f64::NAN; n];

    for i in (fastk_period - 1)..n {
        let mut hh = f64::NEG_INFINITY;
        let mut ll = f64::INFINITY;
        let mut valid = true;

        for j in (i - fastk_period + 1)..=i {
            match (high_ca.get(j), low_ca.get(j)) {
                (Some(h), Some(l)) => {
                    if h > hh {
                        hh = h;
                    }
                    if l < ll {
                        ll = l;
                    }
                }
                _ => {
                    valid = false;
                    break;
                }
            }
        }

        if valid && hh != ll {
            if let Some(c) = close_ca.get(i) {
                fastk_vals[i] = ((c - ll) / (hh - ll)) * 100.0;
            }
        }
    }

    let fastk_series = Series::new("fast_k".into(), &fastk_vals);
    let fastd = if fastd_matype == 0 {
        ema_series(&fastk_series, fastd_period)
    } else {
        sma_series(&fastk_series, fastd_period)
    };

    let fastd_ca: ChunkedArray<Float64Type> = fastd.f64().unwrap().clone();
    let fastd_final: Vec<f64> = fastd_ca.into_no_null_iter().collect();

    let fastk_series_final = Series::new(output_col_k.into(), &fastk_vals);
    let fastd_series_final = Series::new(output_col_d.into(), &fastd_final);

    let mut result_df = df;
    result_df
        .with_column(fastk_series_final.into())?
        .with_column(fastd_series_final.into())?;
    Ok(result_df)
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
pub async fn stochrsi(
    df: DataFrame,
    timeperiod: Option<usize>,
    fastk_period: Option<usize>,
    fastd_period: Option<usize>,
    fastd_matype: Option<usize>,
    output_col_k: Option<&str>,
    output_col_d: Option<&str>,
) -> PolarsResult<DataFrame> {
    let timeperiod = timeperiod.unwrap_or(14).max(2);
    let fastk_period = fastk_period.unwrap_or(3).max(2);
    let fastd_period = fastd_period.unwrap_or(3).max(2);
    let fastd_matype = fastd_matype.unwrap_or(0);
    let output_col_k = output_col_k.unwrap_or("stochrsi_k");
    let output_col_d = output_col_d.unwrap_or("stochrsi_d");

    let rsi_df = rsi(df.clone(), Some(timeperiod), None).await?;
    let rsi_col = rsi_df.column("rsi").unwrap();
    let rsi_ca: ChunkedArray<Float64Type> = rsi_col.f64().unwrap().clone();

    let n = df.height();
    let mut stochrsi_vals: Vec<f64> = vec![f64::NAN; n];

    for i in (fastk_period - 1)..n {
        let mut hh = f64::NEG_INFINITY;
        let mut ll = f64::INFINITY;
        let mut valid = true;

        for j in (i - fastk_period + 1)..=i {
            if let Some(rsi_val) = rsi_ca.get(j) {
                if !rsi_val.is_nan() {
                    if rsi_val > hh {
                        hh = rsi_val;
                    }
                    if rsi_val < ll {
                        ll = rsi_val;
                    }
                } else {
                    valid = false;
                    break;
                }
            } else {
                valid = false;
                break;
            }
        }

        if valid && hh != ll {
            if let Some(rsi_val) = rsi_ca.get(i) {
                if !rsi_val.is_nan() {
                    stochrsi_vals[i] = (rsi_val - ll) / (hh - ll);
                }
            }
        }
    }

    let stochrsi_series = Series::new("stochrsi_raw".into(), &stochrsi_vals);
    let fastk = if fastd_matype == 0 {
        ema_series(&stochrsi_series, fastk_period)
    } else {
        sma_series(&stochrsi_series, fastk_period)
    };

    let fastd = sma_series(&fastk, fastd_period);

    let fastk_ca: ChunkedArray<Float64Type> = fastk.f64().unwrap().clone();
    let fastd_ca: ChunkedArray<Float64Type> = fastd.f64().unwrap().clone();
    let k_final: Vec<f64> = fastk_ca.into_no_null_iter().collect();
    let d_final: Vec<f64> = fastd_ca.into_no_null_iter().collect();

    let k_series_final = Series::new(output_col_k.into(), &k_final);
    let d_series_final = Series::new(output_col_d.into(), &d_final);

    let mut result_df = df;
    result_df
        .with_column(k_series_final.into())?
        .with_column(d_series_final.into())?;
    Ok(result_df)
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
pub async fn trix(
    df: DataFrame,
    timeperiod: Option<usize>,
    output_col: Option<&str>,
) -> PolarsResult<DataFrame> {
    let timeperiod = timeperiod.unwrap_or(30);
    let output_col = output_col.unwrap_or("trix");
    let close = get_close(&df)?;

    let ema1 = ema_series(&close, timeperiod);
    let ema2 = ema_series(&ema1, timeperiod);
    let ema3 = ema_series(&ema2, timeperiod);

    let ema3_ca: ChunkedArray<Float64Type> = ema3.f64().unwrap().clone();
    let ema3_vals: Vec<f64> = ema3_ca.into_no_null_iter().collect();

    let n = ema3_vals.len();
    let mut trix_vals: Vec<f64> = vec![f64::NAN; n];

    for i in 1..n {
        if !ema3_vals[i].is_nan() && !ema3_vals[i - 1].is_nan() && ema3_vals[i - 1] != 0.0 {
            trix_vals[i] = ((ema3_vals[i] - ema3_vals[i - 1]) / ema3_vals[i - 1]) * 100.0;
        }
    }

    let trix_series = Series::new(output_col.into(), &trix_vals);
    let mut result_df = df;
    result_df.with_column(trix_series.into())?;
    Ok(result_df)
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
pub async fn ultosc(
    df: DataFrame,
    timeperiod1: Option<usize>,
    timeperiod2: Option<usize>,
    timeperiod3: Option<usize>,
    output_col: Option<&str>,
) -> PolarsResult<DataFrame> {
    let timeperiod1 = timeperiod1.unwrap_or(7).max(2);
    let timeperiod2 = timeperiod2.unwrap_or(14).max(2);
    let timeperiod3 = timeperiod3.unwrap_or(28).max(2);
    let output_col = output_col.unwrap_or("ultosc");

    let high = get_high(&df)?;
    let low = get_low(&df)?;
    let close = get_close(&df)?;

    let high_ca: ChunkedArray<Float64Type> = high.f64().unwrap().clone();
    let low_ca: ChunkedArray<Float64Type> = low.f64().unwrap().clone();
    let close_ca: ChunkedArray<Float64Type> = close.f64().unwrap().clone();

    let n = df.height();
    let mut bp: Vec<f64> = vec![0.0; n];
    let mut tr: Vec<f64> = vec![0.0; n];

    for i in 1..n {
        match (
            close_ca.get(i),
            low_ca.get(i),
            high_ca.get(i),
            close_ca.get(i - 1),
        ) {
            (Some(c), Some(l), Some(h), Some(prev_c)) => {
                bp[i] = c - l.min(prev_c);
                let tr1 = h - l;
                let tr2 = (h - prev_c).abs();
                let tr3 = (l - prev_c).abs();
                tr[i] = tr1.max(tr2).max(tr3);
            }
            _ => {
                bp[i] = 0.0;
                tr[i] = 0.0;
            }
        }
    }

    fn sum_bp_tr(bp: &[f64], tr: &[f64], start: usize, end: usize) -> (f64, f64) {
        if start > end || end >= bp.len() {
            return (0.0, 0.0);
        }
        let sum_bp: f64 = bp[start..=end].iter().sum();
        let sum_tr: f64 = tr[start..=end].iter().sum();
        (sum_bp, sum_tr)
    }

    let mut ultosc_vals: Vec<f64> = vec![f64::NAN; n];

    let min_start = (timeperiod1 + timeperiod2).max(timeperiod3);
    for i in min_start..n {
        let (bp1, tr1_val) = sum_bp_tr(&bp, &tr, i - timeperiod1 + 1, i);
        let (bp2, tr2_val) = sum_bp_tr(&bp, &tr, i - timeperiod2 + 1, i);
        let (bp3, tr3_val) = sum_bp_tr(&bp, &tr, i - timeperiod3 + 1, i);

        let avg1 = if tr1_val != 0.0 { bp1 / tr1_val } else { 0.0 };
        let avg2 = if tr2_val != 0.0 { bp2 / tr2_val } else { 0.0 };
        let avg3 = if tr3_val != 0.0 { bp3 / tr3_val } else { 0.0 };

        ultosc_vals[i] = 100.0 * (4.0 * avg1 + 2.0 * avg2 + avg3) / 7.0;
    }

    let ultosc_series = Series::new(output_col.into(), &ultosc_vals);
    let mut result_df = df;
    result_df.with_column(ultosc_series.into())?;
    Ok(result_df)
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
pub async fn willr(
    df: DataFrame,
    timeperiod: Option<usize>,
    output_col: Option<&str>,
) -> PolarsResult<DataFrame> {
    let timeperiod = timeperiod.unwrap_or(14).max(2);
    let output_col = output_col.unwrap_or("willr");
    let high = get_high(&df)?;
    let low = get_low(&df)?;
    let close = get_close(&df)?;

    let high_ca: ChunkedArray<Float64Type> = high.f64().unwrap().clone();
    let low_ca: ChunkedArray<Float64Type> = low.f64().unwrap().clone();
    let close_ca: ChunkedArray<Float64Type> = close.f64().unwrap().clone();

    let high_vals: Vec<f64> = high_ca.into_no_null_iter().collect();
    let low_vals: Vec<f64> = low_ca.into_no_null_iter().collect();
    let close_vals: Vec<f64> = close_ca.into_no_null_iter().collect();

    let n = high_vals.len();
    let mut willr_vals: Vec<f64> = vec![f64::NAN; n];

    for i in (timeperiod - 1)..n {
        let hh = high_vals[i - timeperiod + 1..=i]
            .iter()
            .fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let ll = low_vals[i - timeperiod + 1..=i]
            .iter()
            .fold(f64::INFINITY, |a, &b| a.min(b));

        if hh != ll {
            willr_vals[i] = ((hh - close_vals[i]) / (hh - ll)) * -100.0;
        }
    }

    let willr_series = Series::new(output_col.into(), &willr_vals);
    let mut result_df = df;
    result_df.with_column(willr_series.into())?;
    Ok(result_df)
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
            Ok(df) => match adx(df, Some(14), None).await {
                Ok(result) => {
                    save_data(&result, "download/test_adx.csv").await.unwrap();
                }
                Err(e) => panic!("Failed to compute ADX: {:?}", e),
            },
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_adxr() {
        match load_data().await {
            Ok(df) => match adxr(df, Some(14), None).await {
                Ok(result) => {
                    save_data(&result, "download/test_adxr.csv").await.unwrap();
                }
                Err(e) => panic!("Failed to compute ADXR: {:?}", e),
            },
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_apo() {
        match load_data().await {
            Ok(df) => match apo(df, Some(12), Some(26), None).await {
                Ok(result) => {
                    save_data(&result, "download/test_apo.csv").await.unwrap();
                }
                Err(e) => panic!("Failed to compute APO: {:?}", e),
            },
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_aroon() {
        match load_data().await {
            Ok(df) => match aroon(df, Some(14), None, None).await {
                Ok(result) => {
                    save_data(&result, "download/test_aroon.csv").await.unwrap();
                }
                Err(e) => panic!("Failed to compute Aroon: {:?}", e),
            },
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_aroonosc() {
        match load_data().await {
            Ok(df) => match aroonosc(df, Some(14), None).await {
                Ok(result) => {
                    save_data(&result, "download/test_aroonosc.csv")
                        .await
                        .unwrap();
                }
                Err(e) => panic!("Failed to compute Aroon Oscillator: {:?}", e),
            },
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_bop() {
        match load_data().await {
            Ok(df) => match bop(df, None).await {
                Ok(result) => {
                    save_data(&result, "download/test_bop.csv").await.unwrap();
                }
                Err(e) => panic!("Failed to compute BOP: {:?}", e),
            },
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_cci() {
        match load_data().await {
            Ok(df) => match cci(df, None, None).await {
                Ok(result) => {
                    save_data(&result, "download/test_cci.csv").await.unwrap();
                }
                Err(e) => panic!("Failed to compute CCI: {:?}", e),
            },
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_cmo() {
        match load_data().await {
            Ok(df) => match cmo(df, None, None).await {
                Ok(result) => {
                    save_data(&result, "download/test_cmo.csv").await.unwrap();
                }
                Err(e) => panic!("Failed to compute CMO: {:?}", e),
            },
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_dx() {
        match load_data().await {
            Ok(df) => match dx(df, None, None).await {
                Ok(result) => {
                    save_data(&result, "download/test_dx.csv").await.unwrap();
                }
                Err(e) => panic!("Failed to compute DX: {:?}", e),
            },
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_macd() {
        match load_data().await {
            Ok(df) => match macd(df, Some(12), Some(26), Some(9), None, None, None).await {
                Ok(result) => {
                    save_data(&result, "download/test_macd.csv").await.unwrap();
                }
                Err(e) => panic!("Failed to compute MACD: {:?}", e),
            },
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_macdext() {
        match load_data().await {
            Ok(df) => match macdext(df, None, None, None, None, None, None, None, None, None).await
            {
                Ok(result) => {
                    save_data(&result, "download/test_macdext.csv")
                        .await
                        .unwrap();
                }
                Err(e) => panic!("Failed to compute macdext: {:?}", e),
            },
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_macdfix() {
        match load_data().await {
            Ok(df) => match macdfix(df, None).await {
                Ok(result) => {
                    save_data(&result, "download/test_macdfix.csv")
                        .await
                        .unwrap();
                }
                Err(e) => panic!("Failed to compute macdfix: {:?}", e),
            },
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_mfi() {
        match load_data().await {
            Ok(df) => match mfi(df, None, None).await {
                Ok(result) => {
                    save_data(&result, "download/test_mfi.csv").await.unwrap();
                }
                Err(e) => panic!("Failed to compute mfi: {:?}", e),
            },
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_minus_di() {
        match load_data().await {
            Ok(df) => match minus_di(df, None, None).await {
                Ok(result) => {
                    save_data(&result, "download/test_minus_di.csv")
                        .await
                        .unwrap();
                }
                Err(e) => panic!("Failed to compute minus_di: {:?}", e),
            },
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_minus_dm() {
        match load_data().await {
            Ok(df) => match minus_dm(df, None, None).await {
                Ok(result) => {
                    save_data(&result, "download/test_minus_dm.csv")
                        .await
                        .unwrap();
                }
                Err(e) => panic!("Failed to compute minus_dm: {:?}", e),
            },
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_mom() {
        match load_data().await {
            Ok(df) => match mom(df, None, None).await {
                Ok(result) => {
                    save_data(&result, "download/test_mom.csv").await.unwrap();
                }
                Err(e) => panic!("Failed to compute mom: {:?}", e),
            },
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_plus_di() {
        match load_data().await {
            Ok(df) => match plus_di(df, None, None).await {
                Ok(result) => {
                    save_data(&result, "download/test_plus_di.csv")
                        .await
                        .unwrap();
                }
                Err(e) => panic!("Failed to compute plus_di: {:?}", e),
            },
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_plus_dm() {
        match load_data().await {
            Ok(df) => match plus_dm(df, None, None).await {
                Ok(result) => {
                    save_data(&result, "download/test_plus_dm.csv")
                        .await
                        .unwrap();
                }
                Err(e) => panic!("Failed to compute plus_dm: {:?}", e),
            },
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_ppo() {
        match load_data().await {
            Ok(df) => match ppo(df, None, None, None).await {
                Ok(result) => {
                    save_data(&result, "download/test_ppo.csv").await.unwrap();
                }
                Err(e) => panic!("Failed to compute ppo: {:?}", e),
            },
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_roc() {
        match load_data().await {
            Ok(df) => match roc(df, None, None).await {
                Ok(result) => {
                    save_data(&result, "download/test_roc.csv").await.unwrap();
                }
                Err(e) => panic!("Failed to compute roc: {:?}", e),
            },
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_rocp() {
        match load_data().await {
            Ok(df) => match rocp(df, None, None).await {
                Ok(result) => {
                    save_data(&result, "download/test_rocp.csv").await.unwrap();
                }
                Err(e) => panic!("Failed to compute rocp: {:?}", e),
            },
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_rocr() {
        match load_data().await {
            Ok(df) => match rocr(df, None, None).await {
                Ok(result) => {
                    save_data(&result, "download/test_rocr.csv").await.unwrap();
                }
                Err(e) => panic!("Failed to compute rocr: {:?}", e),
            },
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_rocr100() {
        match load_data().await {
            Ok(df) => match rocr100(df, None, None).await {
                Ok(result) => {
                    save_data(&result, "download/test_rocr100.csv")
                        .await
                        .unwrap();
                }
                Err(e) => panic!("Failed to compute rocr100: {:?}", e),
            },
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_rsi() {
        match load_data().await {
            Ok(df) => match rsi(df, None, None).await {
                Ok(result) => {
                    save_data(&result, "download/test_rsi.csv").await.unwrap();
                }
                Err(e) => panic!("Failed to compute rsi: {:?}", e),
            },
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_stoch() {
        match load_data().await {
            Ok(df) => match stoch(df, None, None, None, None, None, None).await {
                Ok(result) => {
                    save_data(&result, "download/test_stoch.csv").await.unwrap();
                }
                Err(e) => panic!("Failed to compute stoch: {:?}", e),
            },
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_stochf() {
        match load_data().await {
            Ok(df) => match stochf(df, None, None, None, None, None).await {
                Ok(result) => {
                    save_data(&result, "download/test_stochf.csv")
                        .await
                        .unwrap();
                }
                Err(e) => panic!("Failed to compute stochf: {:?}", e),
            },
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_stochrsi() {
        match load_data().await {
            Ok(df) => match stochrsi(df, None, None, None, None, None, None).await {
                Ok(result) => {
                    save_data(&result, "download/test_stochrsi.csv")
                        .await
                        .unwrap();
                }
                Err(e) => panic!("Failed to compute stochrsi: {:?}", e),
            },
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_trix() {
        match load_data().await {
            Ok(df) => match trix(df, None, None).await {
                Ok(result) => {
                    save_data(&result, "download/test_trix.csv").await.unwrap();
                }
                Err(e) => panic!("Failed to compute trix: {:?}", e),
            },
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_ultosc() {
        match load_data().await {
            Ok(df) => match ultosc(df, None, None, None, None).await {
                Ok(result) => {
                    save_data(&result, "download/test_ultosc.csv")
                        .await
                        .unwrap();
                }
                Err(e) => panic!("Failed to compute ultosc: {:?}", e),
            },
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_willr() {
        match load_data().await {
            Ok(df) => match willr(df, None, None).await {
                Ok(result) => {
                    save_data(&result, "download/test_willr.csv").await.unwrap();
                }
                Err(e) => panic!("Failed to compute willr: {:?}", e),
            },
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }
}
