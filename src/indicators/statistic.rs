use polars::prelude::*;

/// Lista de indicadores:
/// BETA                 Beta
/// CORREL               Pearson's Correlation Coefficient (r)
/// LINEARREG            Linear Regression
/// LINEARREG_ANGLE      Linear Regression Angle
/// LINEARREG_INTERCEPT  Linear Regression Intercept
/// LINEARREG_SLOPE      Linear Regression Slope
/// STDDEV               Standard Deviation
/// TSF                  Time Series Forecast
/// VAR                  Variance

/// Calcula el coeficiente Beta entre dos series de datos con ventana deslizante.
///
/// # Parámetros
/// * `df` - DataFrame que contiene las columnas de datos
/// * `col_real0` - Nombre de la columna de la serie A (ej: precio del activo)
/// * `col_real1` - Nombre de la columna de la serie B (ej: precio del benchmark)
/// * `timeperiod` - Período de la ventana deslizante
/// * `output_col` - Nombre de la columna de salida (por defecto: "beta")
///
/// # Fórmula
/// β = Cov(A, B) / Var(A) = Σ(Ai - Ā)(Bi - B̄) / Σ(Ai - Ā)²
pub async fn beta(
    df: DataFrame,
    col_real0: &str,
    col_real1: &str,
    timeperiod: usize,
    output_col: Option<&str>,
) -> PolarsResult<DataFrame> {
    let output_name = output_col.unwrap_or("beta");
    let series_a = df.column(col_real0)?.f64()?;
    let series_b = df.column(col_real1)?.f64()?;

    let len = series_a.len();
    let mut result: Vec<Option<f64>> = vec![None; len];

    for i in timeperiod..=len {
        let start = i - timeperiod;
        let window_a: Vec<f64> = (start..i).filter_map(|j| series_a.get(j)).collect();
        let window_b: Vec<f64> = (start..i).filter_map(|j| series_b.get(j)).collect();

        if window_a.len() == timeperiod && window_b.len() == timeperiod {
            let mean_a = window_a.iter().sum::<f64>() / timeperiod as f64;
            let mean_b = window_b.iter().sum::<f64>() / timeperiod as f64;

            let cov_ab: f64 = window_a
                .iter()
                .zip(window_b.iter())
                .map(|(a, b)| (a - mean_a) * (b - mean_b))
                .sum();
            let var_a: f64 = window_a.iter().map(|a| (a - mean_a).powi(2)).sum();

            if var_a != 0.0 {
                result[i - 1] = Some(cov_ab / var_a);
            }
        }
    }

    df.lazy()
        .with_column(lit(Series::new(output_name.into(), result.as_slice())))
        .collect()
}

/// Calcula el coeficiente de correlación de Pearson entre dos series con ventana deslizante.
///
/// # Parámetros
/// * `df` - DataFrame que contiene las columnas de datos
/// * `col_real0` - Nombre de la columna de la serie A
/// * `col_real1` - Nombre de la columna de la serie B
/// * `timeperiod` - Período de la ventana deslizante
/// * `output_col` - Nombre de la columna de salida (por defecto: "correl")
///
/// # Fórmula
/// ρ = Cov(A, B) / (σ_A * σ_B)
pub async fn correl(
    df: DataFrame,
    col_real0: &str,
    col_real1: &str,
    timeperiod: usize,
    output_col: Option<&str>,
) -> PolarsResult<DataFrame> {
    let output_name = output_col.unwrap_or("correl");
    let series_a = df.column(col_real0)?.f64()?;
    let series_b = df.column(col_real1)?.f64()?;

    let len = series_a.len();
    let mut result: Vec<Option<f64>> = vec![None; len];

    for i in timeperiod..=len {
        let start = i - timeperiod;
        let window_a: Vec<f64> = (start..i).filter_map(|j| series_a.get(j)).collect();
        let window_b: Vec<f64> = (start..i).filter_map(|j| series_b.get(j)).collect();

        if window_a.len() == timeperiod && window_b.len() == timeperiod {
            let mean_a = window_a.iter().sum::<f64>() / timeperiod as f64;
            let mean_b = window_b.iter().sum::<f64>() / timeperiod as f64;

            let cov_ab: f64 = window_a
                .iter()
                .zip(window_b.iter())
                .map(|(a, b)| (a - mean_a) * (b - mean_b))
                .sum();
            let var_a: f64 = window_a.iter().map(|a| (a - mean_a).powi(2)).sum();
            let var_b: f64 = window_b.iter().map(|b| (b - mean_b).powi(2)).sum();

            let denom = (var_a * var_b).sqrt();
            if denom != 0.0 {
                result[i - 1] = Some(cov_ab / denom);
            }
        }
    }

    df.lazy()
        .with_column(lit(Series::new(output_name.into(), result.as_slice())))
        .collect()
}

/// Calcula la regresión lineal con ventana deslizante.
///
/// # Parámetros
/// * `df` - DataFrame que contiene la columna de datos
/// * `col_real` - Nombre de la columna de entrada (ej: "close")
/// * `timeperiod` - Período de la ventana deslizante
/// * `output_col` - Nombre de la columna de salida (por defecto: "linearreg")
///
/// # Fórmula
/// Valor = m * (N-1) + c (valor en el punto más reciente)
pub async fn linearreg(
    df: DataFrame,
    col_real: &str,
    timeperiod: usize,
    output_col: Option<&str>,
) -> PolarsResult<DataFrame> {
    let output_name = output_col.unwrap_or("linearreg");
    let series = df.column(col_real)?.f64()?;

    let len = series.len();
    let mut result: Vec<Option<f64>> = vec![None; len];

    let sum_t: f64 = (0..timeperiod).map(|t| t as f64).sum();
    let sum_t2: f64 = (0..timeperiod).map(|t| (t as f64).powi(2)).sum();
    let denom = timeperiod as f64 * sum_t2 - sum_t * sum_t;

    for i in timeperiod..=len {
        let start = i - timeperiod;
        let window: Vec<f64> = (start..i).filter_map(|j| series.get(j)).collect();

        if window.len() == timeperiod && denom != 0.0 {
            let sum_y: f64 = window.iter().sum();
            let sum_ty: f64 = window.iter().enumerate().map(|(t, y)| t as f64 * y).sum();

            let m = (timeperiod as f64 * sum_ty - sum_t * sum_y) / denom;
            let c = (sum_y - m * sum_t) / timeperiod as f64;

            // Valor en el último punto (t = timeperiod - 1)
            result[i - 1] = Some(m * (timeperiod as f64 - 1.0) + c);
        }
    }

    df.lazy()
        .with_column(lit(Series::new(output_name.into(), result.as_slice())))
        .collect()
}

/// Calcula el ángulo de la regresión lineal con ventana deslizante.
///
/// # Parámetros
/// * `df` - DataFrame que contiene la columna de datos
/// * `col_real` - Nombre de la columna de entrada
/// * `timeperiod` - Período de la ventana deslizante
/// * `output_col` - Nombre de la columna de salida (por defecto: "linearreg_angle")
///
/// # Fórmula
/// Ángulo = arctan(m) * (180 / π) en grados
pub async fn linearreg_angle(
    df: DataFrame,
    col_real: &str,
    timeperiod: usize,
    output_col: Option<&str>,
) -> PolarsResult<DataFrame> {
    let output_name = output_col.unwrap_or("linearreg_angle");
    let series = df.column(col_real)?.f64()?;

    let len = series.len();
    let mut result: Vec<Option<f64>> = vec![None; len];

    let sum_t: f64 = (0..timeperiod).map(|t| t as f64).sum();
    let sum_t2: f64 = (0..timeperiod).map(|t| (t as f64).powi(2)).sum();
    let denom = timeperiod as f64 * sum_t2 - sum_t * sum_t;

    for i in timeperiod..=len {
        let start = i - timeperiod;
        let window: Vec<f64> = (start..i).filter_map(|j| series.get(j)).collect();

        if window.len() == timeperiod && denom != 0.0 {
            let sum_y: f64 = window.iter().sum();
            let sum_ty: f64 = window.iter().enumerate().map(|(t, y)| t as f64 * y).sum();

            let m = (timeperiod as f64 * sum_ty - sum_t * sum_y) / denom;

            result[i - 1] = Some(m.atan() * 180.0 / std::f64::consts::PI);
        }
    }

    df.lazy()
        .with_column(lit(Series::new(output_name.into(), result.as_slice())))
        .collect()
}

/// Calcula la intersección de la regresión lineal con ventana deslizante.
///
/// # Parámetros
/// * `df` - DataFrame que contiene la columna de datos
/// * `col_real` - Nombre de la columna de entrada
/// * `timeperiod` - Período de la ventana deslizante
/// * `output_col` - Nombre de la columna de salida (por defecto: "linearreg_intercept")
///
/// # Fórmula
/// c = Ȳ - m * t̄
pub async fn linearreg_intercept(
    df: DataFrame,
    col_real: &str,
    timeperiod: usize,
    output_col: Option<&str>,
) -> PolarsResult<DataFrame> {
    let output_name = output_col.unwrap_or("linearreg_intercept");
    let series = df.column(col_real)?.f64()?;

    let len = series.len();
    let mut result: Vec<Option<f64>> = vec![None; len];

    let sum_t: f64 = (0..timeperiod).map(|t| t as f64).sum();
    let sum_t2: f64 = (0..timeperiod).map(|t| (t as f64).powi(2)).sum();
    let denom = timeperiod as f64 * sum_t2 - sum_t * sum_t;

    for i in timeperiod..=len {
        let start = i - timeperiod;
        let window: Vec<f64> = (start..i).filter_map(|j| series.get(j)).collect();

        if window.len() == timeperiod && denom != 0.0 {
            let sum_y: f64 = window.iter().sum();
            let sum_ty: f64 = window.iter().enumerate().map(|(t, y)| t as f64 * y).sum();

            let m = (timeperiod as f64 * sum_ty - sum_t * sum_y) / denom;
            let c = (sum_y - m * sum_t) / timeperiod as f64;

            result[i - 1] = Some(c);
        }
    }

    df.lazy()
        .with_column(lit(Series::new(output_name.into(), result.as_slice())))
        .collect()
}

/// Calcula la pendiente (slope) de la regresión lineal con ventana deslizante.
///
/// # Parámetros
/// * `df` - DataFrame que contiene la columna de datos
/// * `col_real` - Nombre de la columna de entrada
/// * `timeperiod` - Período de la ventana deslizante
/// * `output_col` - Nombre de la columna de salida (por defecto: "linearreg_slope")
///
/// # Fórmula
/// m = (N * Σ(t * Pt) - Σt * ΣPt) / (N * Σt² - (Σt)²)
pub async fn linearreg_slope(
    df: DataFrame,
    col_real: &str,
    timeperiod: usize,
    output_col: Option<&str>,
) -> PolarsResult<DataFrame> {
    let output_name = output_col.unwrap_or("linearreg_slope");
    let series = df.column(col_real)?.f64()?;

    let len = series.len();
    let mut result: Vec<Option<f64>> = vec![None; len];

    let sum_t: f64 = (0..timeperiod).map(|t| t as f64).sum();
    let sum_t2: f64 = (0..timeperiod).map(|t| (t as f64).powi(2)).sum();
    let denom = timeperiod as f64 * sum_t2 - sum_t * sum_t;

    for i in timeperiod..=len {
        let start = i - timeperiod;
        let window: Vec<f64> = (start..i).filter_map(|j| series.get(j)).collect();

        if window.len() == timeperiod && denom != 0.0 {
            let sum_y: f64 = window.iter().sum();
            let sum_ty: f64 = window.iter().enumerate().map(|(t, y)| t as f64 * y).sum();

            let m = (timeperiod as f64 * sum_ty - sum_t * sum_y) / denom;

            result[i - 1] = Some(m);
        }
    }

    df.lazy()
        .with_column(lit(Series::new(output_name.into(), result.as_slice())))
        .collect()
}

/// Calcula la desviación estándar con ventana deslizante.
///
/// # Parámetros
/// * `df` - DataFrame que contiene la columna de datos
/// * `col_real` - Nombre de la columna de entrada
/// * `timeperiod` - Período de la ventana deslizante
/// * `nbdev` - Número de desviaciones (por defecto: 1.0)
/// * `output_col` - Nombre de la columna de salida (por defecto: "stddev")
///
/// # Fórmula
/// stddev = nbdev * sqrt(Σ(Pt - P̄)² / N)
pub async fn stddev(
    df: DataFrame,
    col_real: &str,
    timeperiod: usize,
    nbdev: Option<f64>,
    output_col: Option<&str>,
) -> PolarsResult<DataFrame> {
    let output_name = output_col.unwrap_or("stddev");
    let nbdev = nbdev.unwrap_or(1.0);
    let series = df.column(col_real)?.f64()?;

    let len = series.len();
    let mut result: Vec<Option<f64>> = vec![None; len];

    for i in timeperiod..=len {
        let start = i - timeperiod;
        let window: Vec<f64> = (start..i).filter_map(|j| series.get(j)).collect();

        if window.len() == timeperiod {
            let mean = window.iter().sum::<f64>() / timeperiod as f64;
            let variance: f64 =
                window.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / timeperiod as f64;

            result[i - 1] = Some(nbdev * variance.sqrt());
        }
    }

    df.lazy()
        .with_column(lit(Series::new(output_name.into(), result.as_slice())))
        .collect()
}

/// Calcula la previsión de serie temporal (Time Series Forecast) con ventana deslizante.
///
/// # Parámetros
/// * `df` - DataFrame que contiene la columna de datos
/// * `col_real` - Nombre de la columna de entrada
/// * `timeperiod` - Período de la ventana deslizante
/// * `output_col` - Nombre de la columna de salida (por defecto: "tsf")
///
/// # Fórmula
/// TSF = m * N + c (valor extrapolado para el próximo período)
pub async fn tsf(
    df: DataFrame,
    col_real: &str,
    timeperiod: usize,
    output_col: Option<&str>,
) -> PolarsResult<DataFrame> {
    let output_name = output_col.unwrap_or("tsf");
    let series = df.column(col_real)?.f64()?;

    let len = series.len();
    let mut result: Vec<Option<f64>> = vec![None; len];

    let sum_t: f64 = (0..timeperiod).map(|t| t as f64).sum();
    let sum_t2: f64 = (0..timeperiod).map(|t| (t as f64).powi(2)).sum();
    let denom = timeperiod as f64 * sum_t2 - sum_t * sum_t;

    for i in timeperiod..=len {
        let start = i - timeperiod;
        let window: Vec<f64> = (start..i).filter_map(|j| series.get(j)).collect();

        if window.len() == timeperiod && denom != 0.0 {
            let sum_y: f64 = window.iter().sum();
            let sum_ty: f64 = window.iter().enumerate().map(|(t, y)| t as f64 * y).sum();

            let m = (timeperiod as f64 * sum_ty - sum_t * sum_y) / denom;
            let c = (sum_y - m * sum_t) / timeperiod as f64;

            // Extrapolación al siguiente período (t = timeperiod)
            result[i - 1] = Some(m * timeperiod as f64 + c);
        }
    }

    df.lazy()
        .with_column(lit(Series::new(output_name.into(), result.as_slice())))
        .collect()
}

/// Calcula la varianza con ventana deslizante.
///
/// # Parámetros
/// * `df` - DataFrame que contiene la columna de datos
/// * `col_real` - Nombre de la columna de entrada
/// * `timeperiod` - Período de la ventana deslizante
/// * `nbdev` - Número de desviaciones (por defecto: 1.0)
/// * `output_col` - Nombre de la columna de salida (por defecto: "var")
///
/// # Fórmula
/// var = nbdev * (Σ(Pt - P̄)² / N)
pub async fn var(
    df: DataFrame,
    col_real: &str,
    timeperiod: usize,
    nbdev: Option<f64>,
    output_col: Option<&str>,
) -> PolarsResult<DataFrame> {
    let output_name = output_col.unwrap_or("var");
    let nbdev = nbdev.unwrap_or(1.0);
    let series = df.column(col_real)?.f64()?;

    let len = series.len();
    let mut result: Vec<Option<f64>> = vec![None; len];

    for i in timeperiod..=len {
        let start = i - timeperiod;
        let window: Vec<f64> = (start..i).filter_map(|j| series.get(j)).collect();

        if window.len() == timeperiod {
            let mean = window.iter().sum::<f64>() / timeperiod as f64;
            let variance: f64 =
                window.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / timeperiod as f64;

            result[i - 1] = Some(nbdev * variance);
        }
    }

    df.lazy()
        .with_column(lit(Series::new(output_name.into(), result.as_slice())))
        .collect()
}
