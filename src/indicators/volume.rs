use polars::prelude::*;

/// Lista de indicadores:
/// AD                   Chaikin A/D Line
/// ADOSC                Chaikin A/D Oscillator
/// OBV                  On Balance Volume

// ============================================================================
// Helper Functions
// ============================================================================

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

/// Get close column from DataFrame (case insensitive)
fn get_close(df: &DataFrame) -> PolarsResult<Series> {
    let s = df.column("close").or_else(|_| df.column("Close"))?;
    Ok(s.cast(&DataType::Float64)?.take_materialized_series())
}

/// Get volume column from DataFrame (case insensitive)
fn get_volume(df: &DataFrame) -> PolarsResult<Series> {
    let s = df.column("volume").or_else(|_| df.column("Volume"))?;
    Ok(s.cast(&DataType::Float64)?.take_materialized_series())
}

/// Exponential Moving Average helper
fn calc_ema(values: &[f64], period: usize) -> Vec<f64> {
    let n = values.len();
    let mut result = vec![f64::NAN; n];

    if n < period {
        return result;
    }

    let multiplier = 2.0 / (period as f64 + 1.0);

    // Initialize with SMA
    let init_sum: f64 = values[..period].iter().sum();
    result[period - 1] = init_sum / period as f64;

    for i in period..n {
        result[i] = (values[i] - result[i - 1]) * multiplier + result[i - 1];
    }

    result
}

// ============================================================================
// AD - Chaikin A/D Line
// ============================================================================

/// AD - Chaikin A/D Line (Accumulation/Distribution Line)
///
/// Línea de Acumulación/Distribución desarrollada por Marc Chaikin.
/// Mide el flujo de volumen para identificar si los inversores están
/// acumulando (comprando) o distribuyendo (vendiendo) un activo.
///
/// Este indicador utiliza la posición del cierre dentro del rango
/// alto-bajo para determinar la presión compradora vs vendedora.
///
/// # Parámetros
/// * `df` - DataFrame con columnas: high, low, close, volume (case insensitive)
/// * `output_col` - Nombre de la columna de salida (default: "ad")
///
/// # Retorna
/// DataFrame con columna "ad" añadida
///
/// # Fórmula
/// AD = acumulación de:
///   ((close - low) - (high - close)) / (high - low) * volume
///
/// El multiplicador de volumen varía de -1 a +1:
/// - Si close está cerca de high → acumulacion positiva
/// - Si close está cerca de low → distribucion negativa
/// - Si close está en el medio → flujo neutral
///
/// # Ejemplo
/// ```rust
/// let df_with_ad = ad(df, None).await?;
/// ```
pub async fn ad(df: DataFrame, output_col: Option<&str>) -> PolarsResult<DataFrame> {
    let output_col = output_col.unwrap_or("ad");

    let high = get_high(&df)?;
    let low = get_low(&df)?;
    let close = get_close(&df)?;
    let volume = get_volume(&df)?;

    let high_ca: ChunkedArray<Float64Type> = high.f64().unwrap().clone();
    let low_ca: ChunkedArray<Float64Type> = low.f64().unwrap().clone();
    let close_ca: ChunkedArray<Float64Type> = close.f64().unwrap().clone();
    let volume_ca: ChunkedArray<Float64Type> = volume.f64().unwrap().clone();

    let high_vals: Vec<f64> = high_ca.into_no_null_iter().collect();
    let low_vals: Vec<f64> = low_ca.into_no_null_iter().collect();
    let close_vals: Vec<f64> = close_ca.into_no_null_iter().collect();
    let volume_vals: Vec<f64> = volume_ca.into_no_null_iter().collect();

    let n = high_vals.len();
    let mut ad_vals: Vec<f64> = vec![0.0; n];
    let mut cumulative_ad = 0.0;

    for i in 0..n {
        let hl_range = high_vals[i] - low_vals[i];

        if hl_range != 0.0 {
            // Money Flow Multiplier
            let mfm = ((close_vals[i] - low_vals[i]) - (high_vals[i] - close_vals[i])) / hl_range;

            // Money Flow Volume
            let mfv = mfm * volume_vals[i];

            // Accumulate
            cumulative_ad += mfv;
        }

        ad_vals[i] = cumulative_ad;
    }

    let ad_series = Series::new(output_col.into(), &ad_vals);
    let mut result_df = df;
    result_df.with_column(ad_series.into())?;
    Ok(result_df)
}

// ============================================================================
// ADOSC - Chaikin A/D Oscillator
// ============================================================================

/// ADOSC - Chaikin A/D Oscillator
///
/// Oscilador basado en la línea A/D que mide la divergencia entre
/// dos medias móviles exponenciales de la línea de Acumulación/Distribución.
///
/// El oscilador genera señales cuando cruza por encima o por debajo
/// de cero, indicando cambios en la presión compradora/vendedora.
///
/// Desarrollado por Marc Chaikin para identificar la fuerza detrás
/// de los movimientos de precio.
///
/// # Parámetros
/// * `df` - DataFrame con columnas: high, low, close, volume (case insensitive)
/// * `fastperiod` - Período de EMA rápida (default: 3)
/// * `slowperiod` - Período de EMA lenta (default: 10)
/// * `output_col` - Nombre de la columna de salida (default: "adosc")
///
/// # Retorna
/// DataFrame con columna "adosc" añadida
///
/// # Fórmula
/// AD = Chaikin A/D Line
/// ADOSC = EMA(AD, fastperiod) - EMA(AD, slowperiod)
///
/// # Interpretación
/// - Valores positivos indican presión compradora
/// - Valores negativos indican presión vendedora
/// - Cruces por encima/de debajo de cero generan señales
///
/// # Ejemplo
/// ```rust
/// let df_with_adosc = adosc(df, Some(3), Some(10), None).await?;
/// ```
pub async fn adosc(
    df: DataFrame,
    fastperiod: Option<usize>,
    slowperiod: Option<usize>,
    output_col: Option<&str>,
) -> PolarsResult<DataFrame> {
    let fastperiod = fastperiod.unwrap_or(3);
    let slowperiod = slowperiod.unwrap_or(10);
    let output_col = output_col.unwrap_or("adosc");

    let high = get_high(&df)?;
    let low = get_low(&df)?;
    let close = get_close(&df)?;
    let volume = get_volume(&df)?;

    let high_ca: ChunkedArray<Float64Type> = high.f64().unwrap().clone();
    let low_ca: ChunkedArray<Float64Type> = low.f64().unwrap().clone();
    let close_ca: ChunkedArray<Float64Type> = close.f64().unwrap().clone();
    let volume_ca: ChunkedArray<Float64Type> = volume.f64().unwrap().clone();

    let high_vals: Vec<f64> = high_ca.into_no_null_iter().collect();
    let low_vals: Vec<f64> = low_ca.into_no_null_iter().collect();
    let close_vals: Vec<f64> = close_ca.into_no_null_iter().collect();
    let volume_vals: Vec<f64> = volume_ca.into_no_null_iter().collect();

    let n = high_vals.len();

    // Calculate A/D Line first
    let mut ad_vals: Vec<f64> = vec![0.0; n];
    let mut cumulative_ad = 0.0;

    for i in 0..n {
        let hl_range = high_vals[i] - low_vals[i];

        if hl_range != 0.0 {
            let mfm = ((close_vals[i] - low_vals[i]) - (high_vals[i] - close_vals[i])) / hl_range;
            let mfv = mfm * volume_vals[i];
            cumulative_ad += mfv;
        }

        ad_vals[i] = cumulative_ad;
    }

    // Calculate EMAs of A/D Line
    let ema_fast = calc_ema(&ad_vals, fastperiod);
    let ema_slow = calc_ema(&ad_vals, slowperiod);

    // ADOSC = EMA(fast) - EMA(slow)
    let adosc_vals: Vec<f64> = ema_fast
        .iter()
        .zip(ema_slow.iter())
        .map(|(&fast, &slow)| {
            if fast.is_nan() || slow.is_nan() {
                f64::NAN
            } else {
                fast - slow
            }
        })
        .collect();

    let adosc_series = Series::new(output_col.into(), &adosc_vals);
    let mut result_df = df;
    result_df.with_column(adosc_series.into())?;
    Ok(result_df)
}

// ============================================================================
// OBV - On Balance Volume
// ============================================================================

/// OBV - On Balance Volume
///
/// Indicador de volumen acumulado que relaciona volumen con cambios de precio.
/// Desarrollado por Joe Granville, fue uno de los primeros indicadores
/// de flujo de dinero.
///
/// El OBV suma el volumen en días alcistas y lo resta en días bajistas,
/// creando una línea acumulativa que confirma tendencias de precio o
/// muestra divergencias.
///
/// # Parámetros
/// * `df` - DataFrame con columnas: close, volume (case insensitive)
/// * `output_col` - Nombre de la columna de salida (default: "obv")
///
/// # Retorna
/// DataFrame con columna "obv" añadida
///
/// # Fórmula
/// Si close > prev_close: OBV += volume
/// Si close < prev_close: OBV -= volume
/// Si close == prev_close: OBV unchanged
///
/// # Interpretación
/// - OBV confirmando tendencia de precio → tendencia fuerte
/// - OBV divergiendo del precio → posible cambio de tendencia
/// - Rupturas en la línea de OBV anticipan rupturas de precio
///
/// # Ejemplo
/// ```rust
/// let df_with_obv = obv(df, None).await?;
/// ```
pub async fn obv(df: DataFrame, output_col: Option<&str>) -> PolarsResult<DataFrame> {
    let output_col = output_col.unwrap_or("obv");

    let close = get_close(&df)?;
    let volume = get_volume(&df)?;

    let close_ca: ChunkedArray<Float64Type> = close.f64().unwrap().clone();
    let volume_ca: ChunkedArray<Float64Type> = volume.f64().unwrap().clone();

    let close_vals: Vec<f64> = close_ca.into_no_null_iter().collect();
    let volume_vals: Vec<f64> = volume_ca.into_no_null_iter().collect();

    let n = close_vals.len();
    let mut obv_vals: Vec<f64> = vec![0.0; n];

    if n > 0 {
        obv_vals[0] = volume_vals[0];

        for i in 1..n {
            if close_vals[i] > close_vals[i - 1] {
                obv_vals[i] = obv_vals[i - 1] + volume_vals[i];
            } else if close_vals[i] < close_vals[i - 1] {
                obv_vals[i] = obv_vals[i - 1] - volume_vals[i];
            } else {
                obv_vals[i] = obv_vals[i - 1];
            }
        }
    }

    let obv_series = Series::new(output_col.into(), &obv_vals);
    let mut result_df = df;
    result_df.with_column(obv_series.into())?;
    Ok(result_df)
}
