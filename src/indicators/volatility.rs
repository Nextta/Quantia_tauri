use polars::prelude::*;

/// Lista de indicadores:
/// ATR                  Average True Range
/// NATR                 Normalized Average True Range
/// TRANGE               True Range

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

/// Calculate True Range for each bar
fn calc_true_range(high_vals: &[f64], low_vals: &[f64], close_vals: &[f64]) -> Vec<f64> {
    let n = high_vals.len();
    let mut tr_vals = vec![0.0; n];

    for i in 0..n {
        if i == 0 {
            // First bar: TR = High - Low
            tr_vals[i] = high_vals[i] - low_vals[i];
        } else {
            // TR = max(High - Low, |High - PrevClose|, |Low - PrevClose|)
            let tr1 = high_vals[i] - low_vals[i];
            let tr2 = (high_vals[i] - close_vals[i - 1]).abs();
            let tr3 = (low_vals[i] - close_vals[i - 1]).abs();
            tr_vals[i] = tr1.max(tr2).max(tr3);
        }
    }

    tr_vals
}

/// Wilder's RMA (Running Moving Average) for ATR calculation
fn rma_series(values: &[f64], period: usize) -> Vec<f64> {
    let n = values.len();
    let mut rma_values = vec![f64::NAN; n];

    if n < period {
        return rma_values;
    }

    let alpha = 1.0 / period as f64;

    // Initialize with SMA
    let sum: f64 = values[..period].iter().sum();
    let mut current_rma = sum / period as f64;
    rma_values[period - 1] = current_rma;

    // Continue with Wilder's smoothing
    for i in period..n {
        current_rma = values[i] * alpha + current_rma * (1.0 - alpha);
        rma_values[i] = current_rma;
    }

    rma_values
}

// ============================================================================
// TRANGE - True Range
// ============================================================================

/// TRANGE - True Range
///
/// Mide el rango verdadero de movimiento del precio para cada barra.
/// El True Range es la máxima diferencia entre:
/// - El máximo y el mínimo actual
/// - El máximo y el cierre anterior
/// - El mínimo y el cierre anterior
///
/// Este indicador es la base para calcular el ATR y otras medidas de volatilidad.
///
/// # Parámetros
/// * `df` - DataFrame con columnas: high, low, close (case insensitive)
/// * `output_col` - Nombre de la columna de salida (default: "trange")
///
/// # Retorna
/// DataFrame con columna "trange" añadida
///
/// # Fórmula
/// TR = max(
///     high - low,
///     |high - prev_close|,
///     |low - prev_close|
/// )
///
/// # Ejemplo
/// ```rust
/// let df_with_tr = trange(df, None).await?;
/// ```
pub async fn trange(df: DataFrame, output_col: Option<&str>) -> PolarsResult<DataFrame> {
    let output_col = output_col.unwrap_or("trange");

    let high = get_high(&df)?;
    let low = get_low(&df)?;
    let close = get_close(&df)?;

    let high_ca: ChunkedArray<Float64Type> = high.f64().unwrap().clone();
    let low_ca: ChunkedArray<Float64Type> = low.f64().unwrap().clone();
    let close_ca: ChunkedArray<Float64Type> = close.f64().unwrap().clone();

    let high_vals: Vec<f64> = high_ca.into_no_null_iter().collect();
    let low_vals: Vec<f64> = low_ca.into_no_null_iter().collect();
    let close_vals: Vec<f64> = close_ca.into_no_null_iter().collect();

    let tr_vals = calc_true_range(&high_vals, &low_vals, &close_vals);

    let tr_series = Series::new(output_col.into(), &tr_vals);
    let mut result_df = df;
    result_df.with_column(tr_series.into())?;
    Ok(result_df)
}

// ============================================================================
// ATR - Average True Range
// ============================================================================

/// ATR - Average True Range
///
/// Promedio del rango verdadero de movimiento del precio durante un período.
/// Mide la volatilidad del mercado: valores altos indican alta volatilidad,
/// valores bajos indican baja volatilidad.
///
/// Desarrollado por J. Welles Wilder Jr., utiliza su método de suavizado RMA
/// (Running Moving Average) que es similar a una EMA pero con diferente factor.
///
/// # Parámetros
/// * `df` - DataFrame con columnas: high, low, close (case insensitive)
/// * `timeperiod` - Período de cálculo (default: 14)
/// * `output_col` - Nombre de la columna de salida (default: "atr")
///
/// # Retorna
/// DataFrame con columna "atr" añadida
///
/// # Fórmula
/// TR = max(high - low, |high - prev_close|, |low - prev_close|)
/// ATR = RMA(TR, timeperiod)
/// donde RMA es el Running Moving Average de Wilder
///
/// # Ejemplo
/// ```rust
/// let df_with_atr = atr(df, Some(14), None).await?;
/// ```
pub async fn atr(
    df: DataFrame,
    timeperiod: Option<usize>,
    output_col: Option<&str>,
) -> PolarsResult<DataFrame> {
    let timeperiod = timeperiod.unwrap_or(14);
    let output_col = output_col.unwrap_or("atr");

    let high = get_high(&df)?;
    let low = get_low(&df)?;
    let close = get_close(&df)?;

    let high_ca: ChunkedArray<Float64Type> = high.f64().unwrap().clone();
    let low_ca: ChunkedArray<Float64Type> = low.f64().unwrap().clone();
    let close_ca: ChunkedArray<Float64Type> = close.f64().unwrap().clone();

    let high_vals: Vec<f64> = high_ca.into_no_null_iter().collect();
    let low_vals: Vec<f64> = low_ca.into_no_null_iter().collect();
    let close_vals: Vec<f64> = close_ca.into_no_null_iter().collect();

    // Calculate True Range
    let tr_vals = calc_true_range(&high_vals, &low_vals, &close_vals);

    // Calculate ATR using Wilder's RMA
    let atr_vals = rma_series(&tr_vals, timeperiod);

    let atr_series = Series::new(output_col.into(), &atr_vals);
    let mut result_df = df;
    result_df.with_column(atr_series.into())?;
    Ok(result_df)
}

// ============================================================================
// NATR - Normalized Average True Range
// ============================================================================

/// NATR - Normalized Average True Range
///
/// Versión normalizada del ATR que expresa la volatilidad como porcentaje
/// del precio de cierre. Esto permite comparar la volatilidad entre
/// diferentes activos independientemente de su precio absoluto.
///
/// Útil para:
/// - Comparar volatilidad entre activos con precios diferentes
/// - Ajustar stops y posición sizing según volatilidad relativa
/// - Identificar cambios en el régimen de volatilidad
///
/// # Parámetros
/// * `df` - DataFrame con columnas: high, low, close (case insensitive)
/// * `timeperiod` - Período de cálculo (default: 14)
/// * `output_col` - Nombre de la columna de salida (default: "natr")
///
/// # Retorna
/// DataFrame con columna "natr" añadida
///
/// # Fórmula
/// TR = max(high - low, |high - prev_close|, |low - prev_close|)
/// ATR = RMA(TR, timeperiod)
/// NATR = (ATR / close) * 100
///
/// # Ejemplo
/// ```rust
/// let df_with_natr = natr(df, Some(14), None).await?;
/// ```
pub async fn natr(
    df: DataFrame,
    timeperiod: Option<usize>,
    output_col: Option<&str>,
) -> PolarsResult<DataFrame> {
    let timeperiod = timeperiod.unwrap_or(14);
    let output_col = output_col.unwrap_or("natr");

    let high = get_high(&df)?;
    let low = get_low(&df)?;
    let close = get_close(&df)?;

    let high_ca: ChunkedArray<Float64Type> = high.f64().unwrap().clone();
    let low_ca: ChunkedArray<Float64Type> = low.f64().unwrap().clone();
    let close_ca: ChunkedArray<Float64Type> = close.f64().unwrap().clone();

    let high_vals: Vec<f64> = high_ca.into_no_null_iter().collect();
    let low_vals: Vec<f64> = low_ca.into_no_null_iter().collect();
    let close_vals: Vec<f64> = close_ca.into_no_null_iter().collect();

    // Calculate True Range
    let tr_vals = calc_true_range(&high_vals, &low_vals, &close_vals);

    // Calculate ATR using Wilder's RMA
    let atr_vals = rma_series(&tr_vals, timeperiod);

    // Normalize by close price
    let natr_vals: Vec<f64> = atr_vals
        .iter()
        .zip(close_vals.iter())
        .map(|(&atr, &close)| {
            if atr.is_nan() || close == 0.0 {
                f64::NAN
            } else {
                (atr / close) * 100.0
            }
        })
        .collect();

    let natr_series = Series::new(output_col.into(), &natr_vals);
    let mut result_df = df;
    result_df.with_column(natr_series.into())?;
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
    async fn test_trange() {
        match load_data().await {
            Ok(df) => match trange(df, None).await {
                Ok(result) => {
                    save_data(&result, "download/test_trange.csv")
                        .await
                        .unwrap();
                }
                Err(e) => panic!("Failed to compute trange: {:?}", e),
            },
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_atr() {
        match load_data().await {
            Ok(df) => match atr(df, Some(14), None).await {
                Ok(result) => {
                    save_data(&result, "download/test_atr.csv").await.unwrap();
                }
                Err(e) => panic!("Failed to compute ATR: {:?}", e),
            },
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_natr() {
        match load_data().await {
            Ok(df) => match natr(df, Some(14), None).await {
                Ok(result) => {
                    save_data(&result, "download/test_natr.csv").await.unwrap();
                }
                Err(e) => panic!("Failed to compute NATR: {:?}", e),
            },
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }
}
