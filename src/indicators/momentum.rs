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
    let mut ema_values: Vec<f64> = Vec::with_capacity(values.len());
    let mut current_ema: f64 = 0.0;
    let mut initialized = false;

    // Get f64 values from series
    let ca: &ChunkedArray<Float64Type> = values.f64().unwrap();
    let values_vec: Vec<f64> = ca.into_no_null_iter().collect();

    // Initialize with SMA
    if values_vec.len() >= period {
        let sum: f64 = values_vec[..period].iter().sum();
        current_ema = sum / period as f64;
        initialized = true;
    }

    for (i, &val) in values_vec.iter().enumerate() {
        if i < period - 1 {
            ema_values.push(f64::NAN);
        } else if i == period - 1 {
            ema_values.push(current_ema);
        } else {
            current_ema = val * multiplier + current_ema * (1.0 - multiplier);
            ema_values.push(current_ema);
        }
    }

    Series::new("ema".into(), &ema_values)
}

// Helper function: Simple Moving Average
fn sma_series(values: &Series, period: usize) -> Series {
    let ca: &ChunkedArray<Float64Type> = values.f64().unwrap();
    let values_vec: Vec<f64> = ca.into_no_null_iter().collect();
    let mut sma_values: Vec<f64> = Vec::with_capacity(values_vec.len());

    for i in 0..values_vec.len() {
        if i < period - 1 {
            sma_values.push(f64::NAN);
        } else {
            let sum: f64 = values_vec[i - period + 1..=i].iter().sum();
            sma_values.push(sum / period as f64);
        }
    }

    Series::new("sma".into(), &sma_values)
}

// Helper function: Wilder's RMA (Running Moving Average)
fn rma_series(values: &Series, period: usize) -> Series {
    let alpha = 1.0 / period as f64;
    let ca: &ChunkedArray<Float64Type> = values.f64().unwrap();
    let values_vec: Vec<f64> = ca.into_no_null_iter().collect();
    let mut rma_values: Vec<f64> = Vec::with_capacity(values_vec.len());
    let mut current_rma: f64 = 0.0;

    for i in 0..values_vec.len() {
        if i < period - 1 {
            rma_values.push(f64::NAN);
        } else if i == period - 1 {
            let sum: f64 = values_vec[..period].iter().sum();
            current_rma = sum / period as f64;
            rma_values.push(current_rma);
        } else {
            current_rma = values_vec[i] * alpha + current_rma * (1.0 - alpha);
            rma_values.push(current_rma);
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
/// Default: timeperiod=14
pub async fn adx(df: DataFrame, timeperiod: usize) -> PolarsResult<DataFrame> {
    let timeperiod = if timeperiod == 0 { 14 } else { timeperiod };
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
    let mut plus_dm: Vec<f64> = vec![0.0; n];
    let mut minus_dm: Vec<f64> = vec![0.0; n];
    let mut tr: Vec<f64> = vec![0.0; n];

    for i in 1..n {
        let high_diff = high_vals[i] - high_vals[i - 1];
        let low_diff = low_vals[i - 1] - low_vals[i];

        plus_dm[i] = if high_diff > low_diff && high_diff > 0.0 {
            high_diff
        } else {
            0.0
        };
        minus_dm[i] = if low_diff > high_diff && low_diff > 0.0 {
            low_diff
        } else {
            0.0
        };

        let tr1 = high_vals[i] - low_vals[i];
        let tr2 = (high_vals[i] - close_vals[i - 1]).abs();
        let tr3 = (low_vals[i] - close_vals[i - 1]).abs();
        tr[i] = tr1.max(tr2).max(tr3);
    }

    let plus_dm_series = Series::new("plus_dm".into(), &plus_dm);
    let minus_dm_series = Series::new("minus_dm".into(), &minus_dm);
    let tr_series = Series::new("tr".into(), &tr);

    let smoothed_plus_dm = rma_series(&plus_dm_series, timeperiod);
    let smoothed_minus_dm = rma_series(&minus_dm_series, timeperiod);
    let smoothed_tr = rma_series(&tr_series, timeperiod);

    let plus_di_ca: ChunkedArray<Float64Type> = smoothed_plus_dm.f64().unwrap().clone();
    let minus_di_ca: ChunkedArray<Float64Type> = smoothed_minus_dm.f64().unwrap().clone();
    let tr_ca: ChunkedArray<Float64Type> = smoothed_tr.f64().unwrap().clone();

    let plus_di_vals: Vec<f64> = plus_di_ca.into_no_null_iter().collect();
    let minus_di_vals: Vec<f64> = minus_di_ca.into_no_null_iter().collect();
    let tr_vals: Vec<f64> = tr_ca.into_no_null_iter().collect();

    let mut dx_vals: Vec<f64> = vec![f64::NAN; n];
    let mut adx_vals: Vec<f64> = vec![f64::NAN; n];

    for i in 0..n {
        if !plus_di_vals[i].is_nan()
            && !minus_di_vals[i].is_nan()
            && !tr_vals[i].is_nan()
            && tr_vals[i] != 0.0
        {
            let plus_di = (plus_di_vals[i] / tr_vals[i]) * 100.0;
            let minus_di = (minus_di_vals[i] / tr_vals[i]) * 100.0;
            let di_sum = plus_di + minus_di;
            if di_sum != 0.0 {
                dx_vals[i] = ((plus_di - minus_di).abs() / di_sum) * 100.0;
            }
        }
    }

    let dx_series = Series::new("dx".into(), &dx_vals);
    let dx_ca: ChunkedArray<Float64Type> = dx_series.f64().unwrap().clone();
    let dx_vec: Vec<f64> = dx_ca.into_no_null_iter().collect();

    // ADX is EMA of DX
    let mut sum_dx = 0.0;
    let mut count = 0;
    for i in 0..n {
        if !dx_vec[i].is_nan() {
            sum_dx += dx_vec[i];
            count += 1;
            if count == timeperiod {
                adx_vals[i] = sum_dx / timeperiod as f64;
                break;
            }
        }
    }

    // Continue with EMA
    for i in (timeperiod)..n {
        if !dx_vec[i].is_nan() {
            let prev = if adx_vals[i - 1].is_nan() {
                adx_vals
                    .iter()
                    .rev()
                    .find(|&&x| !x.is_nan())
                    .copied()
                    .unwrap_or(0.0)
            } else {
                adx_vals[i - 1]
            };
            if prev != 0.0 || !adx_vals[i - 1].is_nan() {
                adx_vals[i] = (prev * (timeperiod as f64 - 1.0) + dx_vec[i]) / timeperiod as f64;
            }
        }
    }

    let adx_series = Series::new("adx".into(), &adx_vals);
    let mut result_df = df.clone();
    result_df.with_column(adx_series.into())?;
    Ok(result_df)
}

/// ADXR - Average Directional Movement Index Rating
/// Default: timeperiod=14
pub async fn adxr(df: DataFrame, timeperiod: usize) -> PolarsResult<DataFrame> {
    let timeperiod = if timeperiod == 0 { 14 } else { timeperiod };
    let adx_df = adx(df.clone(), timeperiod).await?;
    let adx_col = adx_df.column("adx").unwrap();
    let adx_ca: ChunkedArray<Float64Type> = adx_col.f64().unwrap().clone();
    let adx_vals: Vec<f64> = adx_ca.into_no_null_iter().collect();

    let n = adx_vals.len();
    let mut adxr_vals: Vec<f64> = vec![f64::NAN; n];

    for i in (timeperiod - 1)..n {
        if !adx_vals[i].is_nan() && !adx_vals[i - timeperiod + 1].is_nan() {
            adxr_vals[i] = (adx_vals[i] + adx_vals[i - timeperiod + 1]) / 2.0;
        }
    }

    let adxr_series = Series::new("adxr".into(), &adxr_vals);
    let mut result_df = df;
    result_df.with_column(adxr_series.into())?;
    Ok(result_df)
}

/// APO - Absolute Price Oscillator
/// Default: fastperiod=12, slowperiod=26, matype=0 (0=EMA)
pub async fn apo(df: DataFrame, fastperiod: usize, slowperiod: usize) -> PolarsResult<DataFrame> {
    let fastperiod = if fastperiod == 0 { 12 } else { fastperiod };
    let slowperiod = if slowperiod == 0 { 26 } else { slowperiod };
    let close = get_close(&df)?;

    let fast_ema = ema_series(&close, fastperiod);
    let slow_ema = ema_series(&close, slowperiod);

    let fast_ca: ChunkedArray<Float64Type> = fast_ema.f64().unwrap().clone();
    let slow_ca: ChunkedArray<Float64Type> = slow_ema.f64().unwrap().clone();
    let fast_vals: Vec<f64> = fast_ca.into_no_null_iter().collect();
    let slow_vals: Vec<f64> = slow_ca.into_no_null_iter().collect();

    let apo_vals: Vec<f64> = fast_vals
        .iter()
        .zip(slow_vals.iter())
        .map(|(&f, &s)| {
            if f.is_nan() || s.is_nan() {
                f64::NAN
            } else {
                f - s
            }
        })
        .collect();

    let apo_series = Series::new("apo".into(), &apo_vals);
    let mut result_df = df;
    result_df.with_column(apo_series.into())?;
    Ok(result_df)
}

/// AROON - Aroon Indicator
/// Default: timeperiod=14
pub async fn aroon(df: DataFrame, timeperiod: usize) -> PolarsResult<DataFrame> {
    let timeperiod = if timeperiod == 0 { 14 } else { timeperiod };
    let high = get_high(&df)?;
    let low = get_low(&df)?;

    let high_ca: ChunkedArray<Float64Type> = high.f64().unwrap().clone();
    let low_ca: ChunkedArray<Float64Type> = low.f64().unwrap().clone();
    let high_vals: Vec<f64> = high_ca.into_no_null_iter().collect();
    let low_vals: Vec<f64> = low_ca.into_no_null_iter().collect();

    let n = high_vals.len();
    let mut aroon_up: Vec<f64> = vec![f64::NAN; n];
    let mut aroon_down: Vec<f64> = vec![f64::NAN; n];

    for i in (timeperiod - 1)..n {
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

    let aroon_up_series = Series::new("aroon_up".into(), &aroon_up);
    let aroon_down_series = Series::new("aroon_down".into(), &aroon_down);

    let mut result_df = df;
    result_df
        .with_column(aroon_up_series.into())?
        .with_column(aroon_down_series.into())?;
    Ok(result_df)
}

/// AROONOSC - Aroon Oscillator
/// Default: timeperiod=14
pub async fn aroonosc(df: DataFrame, timeperiod: usize) -> PolarsResult<DataFrame> {
    let timeperiod = if timeperiod == 0 { 14 } else { timeperiod };
    let aroon_df = aroon(df.clone(), timeperiod).await?;

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

    let aroonosc_series = Series::new("aroonosc".into(), &aroonosc_vals);
    let mut result_df = df;
    result_df.with_column(aroonosc_series.into())?;
    Ok(result_df)
}

/// BOP - Balance Of Power
pub async fn bop(df: DataFrame) -> PolarsResult<DataFrame> {
    let open = get_open(&df)?;
    let high = get_high(&df)?;
    let low = get_low(&df)?;
    let close = get_close(&df)?;

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

    let bop_series = Series::new("bop".into(), &bop_vals);
    let mut result_df = df;
    result_df.with_column(bop_series.into())?;
    Ok(result_df)
}

/// CCI - Commodity Channel Index
/// Default: timeperiod=14
pub async fn cci(df: DataFrame, timeperiod: usize) -> PolarsResult<DataFrame> {
    let timeperiod = if timeperiod == 0 { 14 } else { timeperiod };
    let high = get_high(&df)?;
    let low = get_low(&df)?;
    let close = get_close(&df)?;

    let high_ca: ChunkedArray<Float64Type> = high.f64().unwrap().clone();
    let low_ca: ChunkedArray<Float64Type> = low.f64().unwrap().clone();
    let close_ca: ChunkedArray<Float64Type> = close.f64().unwrap().clone();

    let high_vals: Vec<f64> = high_ca.into_no_null_iter().collect();
    let low_vals: Vec<f64> = low_ca.into_no_null_iter().collect();
    let close_vals: Vec<f64> = close_ca.into_no_null_iter().collect();

    let typical_price: Vec<f64> = high_vals
        .iter()
        .zip(&low_vals)
        .zip(&close_vals)
        .map(|((h, l), c)| (h + l + c) / 3.0)
        .collect();

    let n = typical_price.len();
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

    let cci_series = Series::new("cci".into(), &cci_vals);
    let mut result_df = df;
    result_df.with_column(cci_series.into())?;
    Ok(result_df)
}

/// CMO - Chande Momentum Oscillator
/// Default: timeperiod=14
pub async fn cmo(df: DataFrame, timeperiod: usize) -> PolarsResult<DataFrame> {
    let timeperiod = if timeperiod == 0 { 14 } else { timeperiod };
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

    let cmo_series = Series::new("cmo".into(), &cmo_vals);
    let mut result_df = df;
    result_df.with_column(cmo_series.into())?;
    Ok(result_df)
}

/// DX - Directional Movement Index
/// Default: timeperiod=14
pub async fn dx(df: DataFrame, timeperiod: usize) -> PolarsResult<DataFrame> {
    let timeperiod = if timeperiod == 0 { 14 } else { timeperiod };
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
    let mut plus_dm: Vec<f64> = vec![0.0; n];
    let mut minus_dm: Vec<f64> = vec![0.0; n];
    let mut tr: Vec<f64> = vec![0.0; n];

    for i in 1..n {
        let high_diff = high_vals[i] - high_vals[i - 1];
        let low_diff = low_vals[i - 1] - low_vals[i];

        plus_dm[i] = if high_diff > low_diff && high_diff > 0.0 {
            high_diff
        } else {
            0.0
        };
        minus_dm[i] = if low_diff > high_diff && low_diff > 0.0 {
            low_diff
        } else {
            0.0
        };

        let tr1 = high_vals[i] - low_vals[i];
        let tr2 = (high_vals[i] - close_vals[i - 1]).abs();
        let tr3 = (low_vals[i] - close_vals[i - 1]).abs();
        tr[i] = tr1.max(tr2).max(tr3);
    }

    let plus_dm_series = Series::new("plus_dm".into(), &plus_dm);
    let minus_dm_series = Series::new("minus_dm".into(), &minus_dm);
    let tr_series = Series::new("tr".into(), &tr);

    let smoothed_plus_dm = rma_series(&plus_dm_series, timeperiod);
    let smoothed_minus_dm = rma_series(&minus_dm_series, timeperiod);
    let smoothed_tr = rma_series(&tr_series, timeperiod);

    let smoothed_plus_dm_ca: ChunkedArray<Float64Type> = smoothed_plus_dm.f64().unwrap().clone();
    let smoothed_minus_dm_ca: ChunkedArray<Float64Type> = smoothed_minus_dm.f64().unwrap().clone();
    let smoothed_tr_ca: ChunkedArray<Float64Type> = smoothed_tr.f64().unwrap().clone();

    let smoothed_plus_dm_vals: Vec<f64> = smoothed_plus_dm_ca.into_no_null_iter().collect();
    let smoothed_minus_dm_vals: Vec<f64> = smoothed_minus_dm_ca.into_no_null_iter().collect();
    let smoothed_tr_vals: Vec<f64> = smoothed_tr_ca.into_no_null_iter().collect();

    let mut dx_vals: Vec<f64> = vec![f64::NAN; n];

    for i in 0..n {
        if !smoothed_plus_dm_vals[i].is_nan()
            && !smoothed_minus_dm_vals[i].is_nan()
            && !smoothed_tr_vals[i].is_nan()
            && smoothed_tr_vals[i] != 0.0
        {
            let plus_di = (smoothed_plus_dm_vals[i] / smoothed_tr_vals[i]) * 100.0;
            let minus_di = (smoothed_minus_dm_vals[i] / smoothed_tr_vals[i]) * 100.0;
            let di_sum = plus_di + minus_di;
            if di_sum != 0.0 {
                dx_vals[i] = ((plus_di - minus_di).abs() / di_sum) * 100.0;
            }
        }
    }

    let dx_series = Series::new("dx".into(), &dx_vals);
    let mut result_df = df;
    result_df.with_column(dx_series.into())?;
    Ok(result_df)
}

/// MACD - Moving Average Convergence/Divergence
/// Default: fastperiod=12, slowperiod=26, signalperiod=9
pub async fn macd(
    df: DataFrame,
    fastperiod: usize,
    slowperiod: usize,
    signalperiod: usize,
) -> PolarsResult<DataFrame> {
    let fastperiod = if fastperiod == 0 { 12 } else { fastperiod };
    let slowperiod = if slowperiod == 0 { 26 } else { slowperiod };
    let signalperiod = if signalperiod == 0 { 9 } else { signalperiod };

    let close = get_close(&df)?;
    let fast_ema = ema_series(&close, fastperiod);
    let slow_ema = ema_series(&close, slowperiod);

    let fast_ca: ChunkedArray<Float64Type> = fast_ema.f64().unwrap().clone();
    let slow_ca: ChunkedArray<Float64Type> = slow_ema.f64().unwrap().clone();
    let fast_vals: Vec<f64> = fast_ca.into_no_null_iter().collect();
    let slow_vals: Vec<f64> = slow_ca.into_no_null_iter().collect();

    let macd_vals: Vec<f64> = fast_vals
        .iter()
        .zip(&slow_vals)
        .map(|(&f, &s)| {
            if f.is_nan() || s.is_nan() {
                f64::NAN
            } else {
                f - s
            }
        })
        .collect();

    let macd_series = Series::new("macd".into(), &macd_vals);
    let signal = ema_series(&macd_series, signalperiod);

    let signal_ca: ChunkedArray<Float64Type> = signal.f64().unwrap().clone();
    let signal_vals: Vec<f64> = signal_ca.into_no_null_iter().collect();

    let hist_vals: Vec<f64> = macd_vals
        .iter()
        .zip(&signal_vals)
        .map(|(&m, &s)| {
            if m.is_nan() || s.is_nan() {
                f64::NAN
            } else {
                m - s
            }
        })
        .collect();

    let macd_series_final = Series::new("macd".into(), &macd_vals);
    let signal_series = Series::new("macd_signal".into(), &signal_vals);
    let hist_series = Series::new("macd_hist".into(), &hist_vals);

    let mut result_df = df;
    result_df
        .with_column(macd_series_final.into())?
        .with_column(signal_series.into())?
        .with_column(hist_series.into())?;
    Ok(result_df)
}

/// MACDEXT - MACD with controllable MA type
/// Default: fastperiod=12, slowperiod=26, signalperiod=9, fastmatype=0, slowmatype=0, signalmatype=0
pub async fn macdext(
    df: DataFrame,
    fastperiod: usize,
    slowperiod: usize,
    signalperiod: usize,
    fastmatype: usize,
    slowmatype: usize,
    signalmatype: usize,
) -> PolarsResult<DataFrame> {
    let fastperiod = if fastperiod == 0 { 12 } else { fastperiod };
    let slowperiod = if slowperiod == 0 { 26 } else { slowperiod };
    let signalperiod = if signalperiod == 0 { 9 } else { signalperiod };

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
    let fast_vals: Vec<f64> = fast_ca.into_no_null_iter().collect();
    let slow_vals: Vec<f64> = slow_ca.into_no_null_iter().collect();

    let macd_vals: Vec<f64> = fast_vals
        .iter()
        .zip(&slow_vals)
        .map(|(&f, &s)| {
            if f.is_nan() || s.is_nan() {
                f64::NAN
            } else {
                f - s
            }
        })
        .collect();

    let macd_series = Series::new("macd".into(), &macd_vals);
    let signal = if signalmatype == 0 {
        ema_series(&macd_series, signalperiod)
    } else {
        sma_series(&macd_series, signalperiod)
    };

    let signal_ca: ChunkedArray<Float64Type> = signal.f64().unwrap().clone();
    let signal_vals: Vec<f64> = signal_ca.into_no_null_iter().collect();

    let hist_vals: Vec<f64> = macd_vals
        .iter()
        .zip(&signal_vals)
        .map(|(&m, &s)| {
            if m.is_nan() || s.is_nan() {
                f64::NAN
            } else {
                m - s
            }
        })
        .collect();

    let macd_series_final = Series::new("macd".into(), &macd_vals);
    let signal_series = Series::new("macd_signal".into(), &signal_vals);
    let hist_series = Series::new("macd_hist".into(), &hist_vals);

    let mut result_df = df;
    result_df
        .with_column(macd_series_final.into())?
        .with_column(signal_series.into())?
        .with_column(hist_series.into())?;
    Ok(result_df)
}

/// MACDFIX - Moving Average Convergence/Divergence Fix 12/26
/// Default: signalperiod=9
pub async fn macdfix(df: DataFrame, signalperiod: usize) -> PolarsResult<DataFrame> {
    macd(df, 12, 26, signalperiod).await
}

/// MFI - Money Flow Index
/// Default: timeperiod=14
pub async fn mfi(df: DataFrame, timeperiod: usize) -> PolarsResult<DataFrame> {
    let timeperiod = if timeperiod == 0 { 14 } else { timeperiod };
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

    let mfi_series = Series::new("mfi".into(), &mfi_vals);
    let mut result_df = df;
    result_df.with_column(mfi_series.into())?;
    Ok(result_df)
}

/// MINUS_DI - Minus Directional Indicator
/// Default: timeperiod=14
pub async fn minus_di(df: DataFrame, timeperiod: usize) -> PolarsResult<DataFrame> {
    let timeperiod = if timeperiod == 0 { 14 } else { timeperiod };
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
    let mut minus_dm: Vec<f64> = vec![0.0; n];
    let mut tr: Vec<f64> = vec![0.0; n];

    for i in 1..n {
        let high_diff = high_vals[i] - high_vals[i - 1];
        let low_diff = low_vals[i - 1] - low_vals[i];

        minus_dm[i] = if low_diff > high_diff && low_diff > 0.0 {
            low_diff
        } else {
            0.0
        };

        let tr1 = high_vals[i] - low_vals[i];
        let tr2 = (high_vals[i] - close_vals[i - 1]).abs();
        let tr3 = (low_vals[i] - close_vals[i - 1]).abs();
        tr[i] = tr1.max(tr2).max(tr3);
    }

    let minus_dm_series = Series::new("minus_dm".into(), &minus_dm);
    let tr_series = Series::new("tr".into(), &tr);

    let smoothed_minus_dm = rma_series(&minus_dm_series, timeperiod);
    let smoothed_tr = rma_series(&tr_series, timeperiod);

    let smoothed_minus_dm_ca: ChunkedArray<Float64Type> = smoothed_minus_dm.f64().unwrap().clone();
    let smoothed_tr_ca: ChunkedArray<Float64Type> = smoothed_tr.f64().unwrap().clone();
    let minus_dm_vals: Vec<f64> = smoothed_minus_dm_ca.into_no_null_iter().collect();
    let tr_vals: Vec<f64> = smoothed_tr_ca.into_no_null_iter().collect();

    let minus_di_vals: Vec<f64> = minus_dm_vals
        .iter()
        .zip(&tr_vals)
        .map(|(&dm, &tr)| {
            if tr == 0.0 || dm.is_nan() || tr.is_nan() {
                f64::NAN
            } else {
                (dm / tr) * 100.0
            }
        })
        .collect();

    let minus_di_series = Series::new("minus_di".into(), &minus_di_vals);
    let mut result_df = df;
    result_df.with_column(minus_di_series.into())?;
    Ok(result_df)
}

/// MINUS_DM - Minus Directional Movement
/// Default: timeperiod=14
pub async fn minus_dm(df: DataFrame, timeperiod: usize) -> PolarsResult<DataFrame> {
    let timeperiod = if timeperiod == 0 { 14 } else { timeperiod };
    let high = get_high(&df)?;
    let low = get_low(&df)?;

    let high_ca: ChunkedArray<Float64Type> = high.f64().unwrap().clone();
    let low_ca: ChunkedArray<Float64Type> = low.f64().unwrap().clone();

    let high_vals: Vec<f64> = high_ca.into_no_null_iter().collect();
    let low_vals: Vec<f64> = low_ca.into_no_null_iter().collect();

    let n = high_vals.len();
    let mut minus_dm: Vec<f64> = vec![0.0; n];

    for i in 1..n {
        let high_diff = high_vals[i] - high_vals[i - 1];
        let low_diff = low_vals[i - 1] - low_vals[i];

        minus_dm[i] = if low_diff > high_diff && low_diff > 0.0 {
            low_diff
        } else {
            0.0
        };
    }

    let minus_dm_series = Series::new("minus_dm".into(), &minus_dm);
    let smoothed_minus_dm = rma_series(&minus_dm_series, timeperiod);

    let mut result_df = df;
    let mut renamed = smoothed_minus_dm;
    renamed.rename("minus_dm".into());
    result_df.with_column(renamed.into())?;
    Ok(result_df)
}

/// MOM - Momentum
/// Default: timeperiod=10
pub async fn mom(df: DataFrame, timeperiod: usize) -> PolarsResult<DataFrame> {
    let timeperiod = if timeperiod == 0 { 10 } else { timeperiod };
    let close = get_close(&df)?;

    let close_ca: ChunkedArray<Float64Type> = close.f64().unwrap().clone();
    let close_vals: Vec<f64> = close_ca.into_no_null_iter().collect();

    let n = close_vals.len();
    let mut mom_vals: Vec<f64> = vec![f64::NAN; n];

    for i in timeperiod..n {
        mom_vals[i] = close_vals[i] - close_vals[i - timeperiod];
    }

    let mom_series = Series::new("mom".into(), &mom_vals);
    let mut result_df = df;
    result_df.with_column(mom_series.into())?;
    Ok(result_df)
}

/// PLUS_DI - Plus Directional Indicator
/// Default: timeperiod=14
pub async fn plus_di(df: DataFrame, timeperiod: usize) -> PolarsResult<DataFrame> {
    let timeperiod = if timeperiod == 0 { 14 } else { timeperiod };
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
    let mut plus_dm: Vec<f64> = vec![0.0; n];
    let mut tr: Vec<f64> = vec![0.0; n];

    for i in 1..n {
        let high_diff = high_vals[i] - high_vals[i - 1];
        let low_diff = low_vals[i - 1] - low_vals[i];

        plus_dm[i] = if high_diff > low_diff && high_diff > 0.0 {
            high_diff
        } else {
            0.0
        };

        let tr1 = high_vals[i] - low_vals[i];
        let tr2 = (high_vals[i] - close_vals[i - 1]).abs();
        let tr3 = (low_vals[i] - close_vals[i - 1]).abs();
        tr[i] = tr1.max(tr2).max(tr3);
    }

    let plus_dm_series = Series::new("plus_dm".into(), &plus_dm);
    let tr_series = Series::new("tr".into(), &tr);

    let smoothed_plus_dm = rma_series(&plus_dm_series, timeperiod);
    let smoothed_tr = rma_series(&tr_series, timeperiod);

    let smoothed_plus_dm_ca: ChunkedArray<Float64Type> = smoothed_plus_dm.f64().unwrap().clone();
    let smoothed_tr_ca: ChunkedArray<Float64Type> = smoothed_tr.f64().unwrap().clone();
    let plus_dm_vals: Vec<f64> = smoothed_plus_dm_ca.into_no_null_iter().collect();
    let tr_vals: Vec<f64> = smoothed_tr_ca.into_no_null_iter().collect();

    let plus_di_vals: Vec<f64> = plus_dm_vals
        .iter()
        .zip(&tr_vals)
        .map(|(&dm, &tr)| {
            if tr == 0.0 || dm.is_nan() || tr.is_nan() {
                f64::NAN
            } else {
                (dm / tr) * 100.0
            }
        })
        .collect();

    let plus_di_series = Series::new("plus_di".into(), &plus_di_vals);
    let mut result_df = df;
    result_df.with_column(plus_di_series.into())?;
    Ok(result_df)
}

/// PLUS_DM - Plus Directional Movement
/// Default: timeperiod=14
pub async fn plus_dm(df: DataFrame, timeperiod: usize) -> PolarsResult<DataFrame> {
    let timeperiod = if timeperiod == 0 { 14 } else { timeperiod };
    let high = get_high(&df)?;
    let low = get_low(&df)?;

    let high_ca: ChunkedArray<Float64Type> = high.f64().unwrap().clone();
    let low_ca: ChunkedArray<Float64Type> = low.f64().unwrap().clone();

    let high_vals: Vec<f64> = high_ca.into_no_null_iter().collect();
    let low_vals: Vec<f64> = low_ca.into_no_null_iter().collect();

    let n = high_vals.len();
    let mut plus_dm: Vec<f64> = vec![0.0; n];

    for i in 1..n {
        let high_diff = high_vals[i] - high_vals[i - 1];
        let low_diff = low_vals[i - 1] - low_vals[i];

        plus_dm[i] = if high_diff > low_diff && high_diff > 0.0 {
            high_diff
        } else {
            0.0
        };
    }

    let plus_dm_series = Series::new("plus_dm".into(), &plus_dm);
    let smoothed_plus_dm = rma_series(&plus_dm_series, timeperiod);

    let mut result_df = df;
    let mut renamed = smoothed_plus_dm;
    renamed.rename("plus_dm".into());
    result_df.with_column(renamed.into())?;
    Ok(result_df)
}

/// PPO - Percentage Price Oscillator
/// Default: fastperiod=12, slowperiod=26, matype=0 (0=EMA)
pub async fn ppo(df: DataFrame, fastperiod: usize, slowperiod: usize) -> PolarsResult<DataFrame> {
    let fastperiod = if fastperiod == 0 { 12 } else { fastperiod };
    let slowperiod = if slowperiod == 0 { 26 } else { slowperiod };
    let close = get_close(&df)?;

    let fast_ema = ema_series(&close, fastperiod);
    let slow_ema = ema_series(&close, slowperiod);

    let fast_ca: ChunkedArray<Float64Type> = fast_ema.f64().unwrap().clone();
    let slow_ca: ChunkedArray<Float64Type> = slow_ema.f64().unwrap().clone();
    let fast_vals: Vec<f64> = fast_ca.into_no_null_iter().collect();
    let slow_vals: Vec<f64> = slow_ca.into_no_null_iter().collect();

    let ppo_vals: Vec<f64> = fast_vals
        .iter()
        .zip(&slow_vals)
        .map(|(&f, &s)| {
            if s.is_nan() || s == 0.0 || f.is_nan() {
                f64::NAN
            } else {
                ((f - s) / s) * 100.0
            }
        })
        .collect();

    let ppo_series = Series::new("ppo".into(), &ppo_vals);
    let mut result_df = df;
    result_df.with_column(ppo_series.into())?;
    Ok(result_df)
}

/// ROC - Rate of change : ((price/prevPrice)-1)*100
/// Default: timeperiod=10
pub async fn roc(df: DataFrame, timeperiod: usize) -> PolarsResult<DataFrame> {
    let timeperiod = if timeperiod == 0 { 10 } else { timeperiod };
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

    let roc_series = Series::new("roc".into(), &roc_vals);
    let mut result_df = df;
    result_df.with_column(roc_series.into())?;
    Ok(result_df)
}

/// ROCP - Rate of change Percentage: (price-prevPrice)/prevPrice
/// Default: timeperiod=10
pub async fn rocp(df: DataFrame, timeperiod: usize) -> PolarsResult<DataFrame> {
    let timeperiod = if timeperiod == 0 { 10 } else { timeperiod };
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

    let rocp_series = Series::new("rocp".into(), &rocp_vals);
    let mut result_df = df;
    result_df.with_column(rocp_series.into())?;
    Ok(result_df)
}

/// ROCR - Rate of change ratio: (price/prevPrice)
/// Default: timeperiod=10
pub async fn rocr(df: DataFrame, timeperiod: usize) -> PolarsResult<DataFrame> {
    let timeperiod = if timeperiod == 0 { 10 } else { timeperiod };
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

    let rocr_series = Series::new("rocr".into(), &rocr_vals);
    let mut result_df = df;
    result_df.with_column(rocr_series.into())?;
    Ok(result_df)
}

/// ROCR100 - Rate of change ratio 100 scale: (price/prevPrice)*100
/// Default: timeperiod=10
pub async fn rocr100(df: DataFrame, timeperiod: usize) -> PolarsResult<DataFrame> {
    let timeperiod = if timeperiod == 0 { 10 } else { timeperiod };
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

    let rocr100_series = Series::new("rocr100".into(), &rocr100_vals);
    let mut result_df = df;
    result_df.with_column(rocr100_series.into())?;
    Ok(result_df)
}

/// RSI - Relative Strength Index
/// Default: timeperiod=14
pub async fn rsi(df: DataFrame, timeperiod: usize) -> PolarsResult<DataFrame> {
    let timeperiod = if timeperiod == 0 { 14 } else { timeperiod };
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

    let rsi_series = Series::new("rsi".into(), &rsi_vals);
    let mut result_df = df;
    result_df.with_column(rsi_series.into())?;
    Ok(result_df)
}

/// STOCH - Stochastic
/// Default: fastk_period=5, slowk_period=3, slowk_matype=0, slowd_period=3, slowd_matype=0
pub async fn stoch(
    df: DataFrame,
    fastk_period: usize,
    slowk_period: usize,
    slowk_matype: usize,
    slowd_period: usize,
    slowd_matype: usize,
) -> PolarsResult<DataFrame> {
    let fastk_period = if fastk_period == 0 { 5 } else { fastk_period };
    let slowk_period = if slowk_period == 0 { 3 } else { slowk_period };
    let slowd_period = if slowd_period == 0 { 3 } else { slowd_period };

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
    let mut k_vals: Vec<f64> = vec![f64::NAN; n];

    for i in (fastk_period - 1)..n {
        let hh = high_vals[i - fastk_period + 1..=i]
            .iter()
            .fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let ll = low_vals[i - fastk_period + 1..=i]
            .iter()
            .fold(f64::INFINITY, |a, &b| a.min(b));

        if hh != ll {
            k_vals[i] = ((close_vals[i] - ll) / (hh - ll)) * 100.0;
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

    let k_series_final = Series::new("slow_k".into(), &k_final);
    let d_series_final = Series::new("slow_d".into(), &d_final);

    let mut result_df = df;
    result_df
        .with_column(k_series_final.into())?
        .with_column(d_series_final.into())?;
    Ok(result_df)
}

/// STOCHF - Stochastic Fast
/// Default: fastk_period=5, fastd_period=3, fastd_matype=0
pub async fn stochf(
    df: DataFrame,
    fastk_period: usize,
    fastd_period: usize,
    fastd_matype: usize,
) -> PolarsResult<DataFrame> {
    let fastk_period = if fastk_period == 0 { 5 } else { fastk_period };
    let fastd_period = if fastd_period == 0 { 3 } else { fastd_period };

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
    let mut fastk_vals: Vec<f64> = vec![f64::NAN; n];

    for i in (fastk_period - 1)..n {
        let hh = high_vals[i - fastk_period + 1..=i]
            .iter()
            .fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let ll = low_vals[i - fastk_period + 1..=i]
            .iter()
            .fold(f64::INFINITY, |a, &b| a.min(b));

        if hh != ll {
            fastk_vals[i] = ((close_vals[i] - ll) / (hh - ll)) * 100.0;
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

    let fastk_series_final = Series::new("fast_k".into(), &fastk_vals);
    let fastd_series_final = Series::new("fast_d".into(), &fastd_final);

    let mut result_df = df;
    result_df
        .with_column(fastk_series_final.into())?
        .with_column(fastd_series_final.into())?;
    Ok(result_df)
}

/// STOCHRSI - Stochastic Relative Strength Index
/// Default: timeperiod=14, fastk_period=3, fastd_period=3, fastd_matype=0
pub async fn stochrsi(
    df: DataFrame,
    timeperiod: usize,
    fastk_period: usize,
    fastd_period: usize,
    fastd_matype: usize,
) -> PolarsResult<DataFrame> {
    let timeperiod = if timeperiod == 0 { 14 } else { timeperiod };
    let fastk_period = if fastk_period == 0 { 3 } else { fastk_period };
    let fastd_period = if fastd_period == 0 { 3 } else { fastd_period };

    let rsi_df = rsi(df.clone(), timeperiod).await?;
    let rsi_col = rsi_df.column("rsi").unwrap();
    let rsi_ca: ChunkedArray<Float64Type> = rsi_col.f64().unwrap().clone();
    let rsi_vals: Vec<f64> = rsi_ca.into_no_null_iter().collect();

    let n = rsi_vals.len();
    let mut stochrsi_vals: Vec<f64> = vec![f64::NAN; n];

    for i in (timeperiod - 1)..n {
        let window = &rsi_vals[i - timeperiod + 1..=i];
        let hh = window.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let ll = window.iter().fold(f64::INFINITY, |a, &b| a.min(b));

        if hh != ll {
            stochrsi_vals[i] = (rsi_vals[i] - ll) / (hh - ll);
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

    let k_series_final = Series::new("stochrsi_k".into(), &k_final);
    let d_series_final = Series::new("stochrsi_d".into(), &d_final);

    let mut result_df = df;
    result_df
        .with_column(k_series_final.into())?
        .with_column(d_series_final.into())?;
    Ok(result_df)
}

/// TRIX - 1-day Rate-Of-Change (ROC) of a Triple Smooth EMA
/// Default: timeperiod=30
pub async fn trix(df: DataFrame, timeperiod: usize) -> PolarsResult<DataFrame> {
    let timeperiod = if timeperiod == 0 { 30 } else { timeperiod };
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

    let trix_series = Series::new("trix".into(), &trix_vals);
    let mut result_df = df;
    result_df.with_column(trix_series.into())?;
    Ok(result_df)
}

/// ULTOSC - Ultimate Oscillator
/// Default: timeperiod1=7, timeperiod2=14, timeperiod3=28
pub async fn ultosc(
    df: DataFrame,
    timeperiod1: usize,
    timeperiod2: usize,
    timeperiod3: usize,
) -> PolarsResult<DataFrame> {
    let timeperiod1 = if timeperiod1 == 0 { 7 } else { timeperiod1 };
    let timeperiod2 = if timeperiod2 == 0 { 14 } else { timeperiod2 };
    let timeperiod3 = if timeperiod3 == 0 { 28 } else { timeperiod3 };

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
    let mut bp: Vec<f64> = vec![0.0; n]; // Buying Pressure
    let mut tr: Vec<f64> = vec![0.0; n]; // True Range

    for i in 1..n {
        bp[i] = close_vals[i] - min(low_vals[i], close_vals[i - 1]);
        let tr1 = high_vals[i] - low_vals[i];
        let tr2 = (high_vals[i] - close_vals[i - 1]).abs();
        let tr3 = (low_vals[i] - close_vals[i - 1]).abs();
        tr[i] = tr1.max(tr2).max(tr3);
    }

    fn sum_bp_tr(bp: &[f64], tr: &[f64], start: usize, end: usize) -> (f64, f64) {
        let sum_bp: f64 = bp[start..=end].iter().sum();
        let sum_tr: f64 = tr[start..=end].iter().sum();
        (sum_bp, sum_tr)
    }

    fn min(a: f64, b: f64) -> f64 {
        if a < b {
            a
        } else {
            b
        }
    }

    let mut ultosc_vals: Vec<f64> = vec![f64::NAN; n];

    for i in (timeperiod1 + timeperiod2)..n {
        let (bp1, tr1) = sum_bp_tr(&bp, &tr, i - timeperiod1 + 1, i);
        let (bp2, tr2) = sum_bp_tr(&bp, &tr, i - timeperiod2 + 1, i);
        let (bp3, tr3) = sum_bp_tr(&bp, &tr, i - timeperiod3 + 1, i);

        let avg1 = if tr1 != 0.0 { bp1 / tr1 } else { 0.0 };
        let avg2 = if tr2 != 0.0 { bp2 / tr2 } else { 0.0 };
        let avg3 = if tr3 != 0.0 { bp3 / tr3 } else { 0.0 };

        ultosc_vals[i] = 100.0 * (4.0 * avg1 + 2.0 * avg2 + avg3) / 7.0;
    }

    let ultosc_series = Series::new("ultosc".into(), &ultosc_vals);
    let mut result_df = df;
    result_df.with_column(ultosc_series.into())?;
    Ok(result_df)
}

/// WILLR - Williams' %R
/// Default: timeperiod=14
pub async fn willr(df: DataFrame, timeperiod: usize) -> PolarsResult<DataFrame> {
    let timeperiod = if timeperiod == 0 { 14 } else { timeperiod };
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

    let willr_series = Series::new("willr".into(), &willr_vals);
    let mut result_df = df;
    result_df.with_column(willr_series.into())?;
    Ok(result_df)
}
