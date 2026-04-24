use polars::prelude::*;

/// Lista de indicadores:
/// AD                   Chaikin A/D Line
/// ADOSC                Chaikin A/D Oscillator
/// OBV                  On Balance Volume

// ============================================================================
// Helper Functions
// ============================================================================

/// Obtiene la columna 'high' del DataFrame (insensible a mayúsculas/minúsculas).
fn get_high(df: &DataFrame) -> PolarsResult<Series> {
    let s = df.column("high").or_else(|_| df.column("High"))?;
    Ok(s.cast(&DataType::Float64)?.take_materialized_series())
}

/// Obtiene la columna 'low' del DataFrame (insensible a mayúsculas/minúsculas).
fn get_low(df: &DataFrame) -> PolarsResult<Series> {
    let s = df.column("low").or_else(|_| df.column("Low"))?;
    Ok(s.cast(&DataType::Float64)?.take_materialized_series())
}

/// Obtiene la columna 'close' del DataFrame (insensible a mayúsculas/minúsculas).
fn get_close(df: &DataFrame) -> PolarsResult<Series> {
    let s = df.column("close").or_else(|_| df.column("Close"))?;
    Ok(s.cast(&DataType::Float64)?.take_materialized_series())
}

/// Obtiene la columna 'volume' del DataFrame (insensible a mayúsculas/minúsculas).
fn get_volume(df: &DataFrame) -> PolarsResult<Series> {
    let s = df.column("volume").or_else(|_| df.column("Volume"))?;
    Ok(s.cast(&DataType::Float64)?.take_materialized_series())
}

/// Calcula la Media Móvil Exponencial (EMA) sobre un slice de f64.
///
/// Utiliza el método estándar de inicialización con el promedio simple (SMA)
/// de los primeros `period` valores válidos.
fn calc_ema(values: &[f64], period: usize) -> Vec<f64> {
    let n = values.len();
    let mut result = vec![f64::NAN; n];

    if n < period {
        return result;
    }

    let multiplier = 2.0 / (period as f64 + 1.0);

    // Initialize with SMA
    let mut init_sum = 0.0;
    let mut valid_count = 0;
    let mut start_idx = 0;

    for i in 0..n {
        if !values[i].is_nan() {
            init_sum += values[i];
            valid_count += 1;
            if valid_count == period {
                result[i] = init_sum / period as f64;
                start_idx = i;
                break;
            }
        }
    }

    if valid_count < period {
        return result;
    }

    for i in (start_idx + 1)..n {
        if !values[i].is_nan() {
            result[i] = (values[i] - result[i - 1]) * multiplier + result[i - 1];
        } else {
            result[i] = result[i - 1];
        }
    }

    result
}

// ============================================================================
// AD - Chaikin A/D Line
// ============================================================================

/// AD - Chaikin A/D Line (Línea de Acumulación/Distribución)
///
/// Este indicador mide el flujo acumulado de dinero hacia adentro o hacia afuera de un activo.
/// Utiliza la relación entre el precio de cierre y el rango alto-bajo para determinar
/// la presión de compra o venta.
///
/// # Parámetros
/// * `df` - DataFrame con columnas: high, low, close, volume (insensible a mayúsculas).
/// * `output_col` - Nombre opcional para la columna de salida (default: "ad").
///
/// # Retorna
/// Un `PolarsResult` con el DataFrame original más la columna del indicador.
///
/// # Fórmula
/// 1. Money Flow Multiplier (MFM) = ((Close - Low) - (High - Close)) / (High - Low)
/// 2. Money Flow Volume (MFV) = MFM * Volume
/// 3. AD = AD previo + MFV actual
///
/// # Ejemplo
/// ```rust
/// let df_with_ad = ad(df, Some("mi_ad")).await?;
/// ```
pub async fn ad(mut df: DataFrame, output_col: Option<&str>) -> PolarsResult<DataFrame> {
    let output_col = output_col.unwrap_or("ad");

    let high = get_high(&df)?;
    let low = get_low(&df)?;
    let close = get_close(&df)?;
    let volume = get_volume(&df)?;

    let hl_range = (&high - &low)?;
    let mut mfm = (&close + &close)?;
    mfm = (&mfm - &high)?;
    mfm = (&mfm - &low)?;

    // safe division manually for robustness if traits are missing
    let mfm_final: Vec<f64> = mfm
        .f64()?
        .into_iter()
        .zip(hl_range.f64()?.into_iter())
        .zip(volume.f64()?.into_iter())
        .map(|((m, r), v)| match (m, r, v) {
            (Some(mv), Some(rv), Some(vv)) if rv > 0.0 => (mv / rv) * vv,
            _ => 0.0,
        })
        .collect();

    let mut current_ad = 0.0;
    let ad_vals: Vec<f64> = mfm_final
        .into_iter()
        .map(|mfv| {
            current_ad += mfv;
            current_ad
        })
        .collect();

    let ad_series = Series::new(output_col.into(), ad_vals);
    df.with_column(ad_series.into())?;
    Ok(df)
}

// ============================================================================
// ADOSC - Chaikin A/D Oscillator
// ============================================================================

/// ADOSC - Chaikin A/D Oscillator (Oscilador Chaikin)
///
/// El Oscilador Chaikin mide el momentum de la Línea de Acumulación/Distribución (AD).
/// Se calcula como la diferencia entre una EMA rápida y una EMA lenta de la línea AD.
///
/// # Parámetros
/// * `df` - DataFrame con columnas: high, low, close, volume (insensible a mayúsculas).
/// * `fastperiod` - Período para la EMA rápida (default: 3).
/// * `slowperiod` - Período para la EMA lenta (default: 10).
/// * `output_col` - Nombre opcional para la columna de salida (default: "adosc").
///
/// # Retorna
/// Un `PolarsResult` con el DataFrame original más la columna del indicador.
///
/// # Fórmula
/// ADOSC = EMA(AD, fastperiod) - EMA(AD, slowperiod)
///
/// # Ejemplo
/// ```rust
/// let df_with_adosc = adosc(df, Some(3), Some(10), None).await?;
/// ```
pub async fn adosc(
    mut df: DataFrame,
    fastperiod: Option<usize>,
    slowperiod: Option<usize>,
    output_col: Option<&str>,
) -> PolarsResult<DataFrame> {
    let fastperiod = fastperiod.unwrap_or(3);
    let slowperiod = slowperiod.unwrap_or(10);
    let output_col = output_col.unwrap_or("adosc");

    let ad_df = ad(df.clone(), Some("temp_ad")).await?;
    let ad_series = ad_df.column("temp_ad")?;

    let ad_vals: Vec<f64> = ad_series
        .f64()?
        .into_iter()
        .map(|v| v.unwrap_or(0.0))
        .collect();

    let ema_fast = calc_ema(&ad_vals, fastperiod);
    let ema_slow = calc_ema(&ad_vals, slowperiod);

    let adosc_vals: Vec<f64> = ema_fast
        .iter()
        .zip(ema_slow.iter())
        .map(|(&f, &s)| {
            if f.is_nan() || s.is_nan() {
                f64::NAN
            } else {
                f - s
            }
        })
        .collect();

    let adosc_series = Series::new(output_col.into(), adosc_vals);
    df.with_column(adosc_series.into())?;
    Ok(df)
}

// ============================================================================
// OBV - On Balance Volume
// ============================================================================

/// OBV - On Balance Volume
///
/// El On-Balance Volume es un indicador de momentum acumulado que relaciona el volumen
/// con los cambios de precio para predecir movimientos futuros.
///
/// # Parámetros
/// * `df` - DataFrame con columnas: close, volume (insensible a mayúsculas).
/// * `output_col` - Nombre opcional para la columna de salida (default: "obv").
///
/// # Retorna
/// Un `PolarsResult` con el DataFrame original más la columna del indicador.
///
/// # Fórmula
/// * Si Close > Close_prev: OBV = OBV_prev + Volume
/// * Si Close < Close_prev: OBV = OBV_prev - Volume
/// * Si Close == Close_prev: OBV = OBV_prev
///
/// # Ejemplo
/// ```rust
/// let df_with_obv = obv(df, None).await?;
/// ```
pub async fn obv(mut df: DataFrame, output_col: Option<&str>) -> PolarsResult<DataFrame> {
    let output_col = output_col.unwrap_or("obv");

    let close = get_close(&df)?;
    let volume = get_volume(&df)?;

    let close_vals: Vec<f64> = close
        .f64()?
        .into_iter()
        .map(|v| v.unwrap_or(f64::NAN))
        .collect();
    let volume_vals: Vec<f64> = volume
        .f64()?
        .into_iter()
        .map(|v| v.unwrap_or(0.0))
        .collect();
    let n = close_vals.len();

    if n == 0 {
        df.with_column(Series::new(output_col.into(), Vec::<f64>::new()).into())?;
        return Ok(df);
    }

    let mut obv_vals = Vec::with_capacity(n);
    let mut current_obv = volume_vals[0];
    obv_vals.push(current_obv);

    for i in 1..n {
        let c = close_vals[i];
        let p = close_vals[i - 1];
        let v = volume_vals[i];

        if c.is_nan() || p.is_nan() {
            // keep previous
        } else if c > p {
            current_obv += v;
        } else if c < p {
            current_obv -= v;
        }
        obv_vals.push(current_obv);
    }

    let obv_series = Series::new(output_col.into(), obv_vals);
    df.with_column(obv_series.into())?;
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
    async fn test_ad() {
        if let Ok(df) = load_data().await {
            if let Ok(result) = ad(df, None).await {
                let _ = save_data(&result, "download/test_ad.csv").await;
            }
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_adosc() {
        if let Ok(df) = load_data().await {
            if let Ok(result) = adosc(df, None, None, None).await {
                let _ = save_data(&result, "download/test_adosc.csv").await;
            }
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_obv() {
        if let Ok(df) = load_data().await {
            if let Ok(result) = obv(df, None).await {
                let _ = save_data(&result, "download/test_obv.csv").await;
            }
        }
    }
}
