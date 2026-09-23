use polars::prelude::*;
use serde::{Deserialize, Serialize};

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

#[derive(Serialize, Deserialize, Debug)]
pub struct BetaParams {
    pub col_real0: String,
    pub col_real1: String,
    pub timeperiod: usize,
}

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
/// β = Cov(Ri, Rm) / Var(Rm)
/// donde Ri son los retornos del activo y Rm los retornos del mercado.
pub fn beta(
    df: &mut DataFrame,
    col_real0: &str,
    col_real1: &str,
    timeperiod: Option<usize>,
    output_col: Option<&str>,
) {
    let timeperiod = timeperiod.unwrap_or(20);
    let output_name = output_col.unwrap_or("beta");

    // Obtenemos los precios
    let series_a = df.column(col_real0).unwrap().f64().unwrap();
    let series_b = df.column(col_real1).unwrap().f64().unwrap();

    let len = series_a.len();
    let mut result: Vec<Option<f64>> = vec![None; len];

    // Necesitamos al menos timeperiod + 1 datos para calcular retornos y luego la ventana
    if len <= timeperiod {
        df.with_column(Series::new(output_name.into(), result).into())
            .unwrap();
        return;
    }

    // Calculamos los retornos: r_t = (p_t / p_{t-1}) - 1
    let mut returns_a: Vec<f64> = Vec::with_capacity(len - 1);
    let mut returns_b: Vec<f64> = Vec::with_capacity(len - 1);

    for i in 1..len {
        let a_t = series_a.get(i);
        let a_t1 = series_a.get(i - 1);
        let b_t = series_b.get(i);
        let b_t1 = series_b.get(i - 1);

        if let (Some(at), Some(at1), Some(bt), Some(bt1)) = (a_t, a_t1, b_t, b_t1) {
            if at1 != 0.0 && bt1 != 0.0 {
                returns_a.push((at / at1) - 1.0);
                returns_b.push((bt / bt1) - 1.0);
            } else {
                returns_a.push(0.0);
                returns_b.push(0.0);
            }
        }
    }

    // El cálculo de Beta sobre los retornos
    // i empieza en timeperiod porque necesitamos 'timeperiod' retornos.
    // Como los retornos están desplazados 1 índice respecto a los precios originales,
    // el retorno en el índice j corresponde al precio en el índice j+1.
    for i in timeperiod..=returns_a.len() {
        let start = i - timeperiod;
        let window_a = &returns_a[start..i];
        let window_b = &returns_b[start..i];

        let mean_a = window_a.iter().sum::<f64>() / timeperiod as f64;
        let mean_b = window_b.iter().sum::<f64>() / timeperiod as f64;

        let mut cov_ab = 0.0;
        let mut var_b = 0.0;

        for j in 0..timeperiod {
            let diff_a = window_a[j] - mean_a;
            let diff_b = window_b[j] - mean_b;
            cov_ab += diff_a * diff_b;
            var_b += diff_b.powi(2);
        }

        if var_b != 0.0 {
            // El resultado se asigna al índice i en el array original de precios
            // (que es el final de la ventana de retornos que termina en el precio i)
            result[i] = Some(cov_ab / var_b);
        }
    }

    df.with_column(Series::new(output_name.into(), result).into())
        .unwrap();
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CorrelParams {
    pub col_real0: String,
    pub col_real1: String,
    pub timeperiod: usize,
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
pub fn correl(
    df: &mut DataFrame,
    col_real0: &str,
    col_real1: &str,
    timeperiod: Option<usize>,
    output_col: Option<&str>,
) {
    let timeperiod = timeperiod.unwrap_or(20);
    let output_name = output_col.unwrap_or("correl");
    let series_a = df.column(col_real0).unwrap().f64().unwrap();
    let series_b = df.column(col_real1).unwrap().f64().unwrap();

    let len = series_a.len();
    let mut result: Vec<Option<f64>> = vec![None; len];

    if len < timeperiod {
        df.with_column(Series::new(output_name.into(), result).into())
            .unwrap();
        return;
    }

    for i in timeperiod..=len {
        let start = i - timeperiod;
        let mut window_a = Vec::with_capacity(timeperiod);
        let mut window_b = Vec::with_capacity(timeperiod);

        for j in start..i {
            if let (Some(a), Some(b)) = (series_a.get(j), series_b.get(j)) {
                if !a.is_nan() && !b.is_nan() {
                    window_a.push(a);
                    window_b.push(b);
                }
            }
        }

        if window_a.len() == timeperiod && window_b.len() == timeperiod {
            let sum_a: f64 = window_a.iter().sum();
            let sum_b: f64 = window_b.iter().sum();
            let mean_a = sum_a / timeperiod as f64;
            let mean_b = sum_b / timeperiod as f64;

            let mut cov_ab = 0.0;
            let mut var_a = 0.0;
            let mut var_b = 0.0;

            for j in 0..timeperiod {
                let diff_a = window_a[j] - mean_a;
                let diff_b = window_b[j] - mean_b;
                cov_ab += diff_a * diff_b;
                var_a += diff_a.powi(2);
                var_b += diff_b.powi(2);
            }

            let denom = (var_a * var_b).sqrt();
            if denom != 0.0 {
                result[i - 1] = Some(cov_ab / denom);
            }
        }
    }

    df.with_column(Series::new(output_name.into(), result).into())
        .unwrap();
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LinearRegParams {
    pub col_real: String,
    pub timeperiod: usize,
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
pub fn linearreg(
    df: &mut DataFrame,
    col_real: &str,
    timeperiod: Option<usize>,
    output_col: Option<&str>,
) {
    let timeperiod = timeperiod.unwrap_or(20);
    let output_name = output_col.unwrap_or("linearreg");
    let series = df.column(col_real).unwrap().f64().unwrap();

    let len = series.len();
    let mut result: Vec<Option<f64>> = vec![None; len];

    if len < timeperiod {
        df.with_column(Series::new(output_name.into(), result).into())
            .unwrap();
        return;
    }

    let sum_t: f64 = (0..timeperiod).map(|t| t as f64).sum();
    let sum_t2: f64 = (0..timeperiod).map(|t| (t as f64).powi(2)).sum();
    let denom = timeperiod as f64 * sum_t2 - sum_t * sum_t;

    for i in timeperiod..=len {
        let start = i - timeperiod;
        let window = series.slice(start as i64, timeperiod);

        if window.null_count() == 0 && denom != 0.0 {
            let sum_y: f64 = window.sum().unwrap();
            let sum_ty: f64 = window
                .into_no_null_iter()
                .enumerate()
                .map(|(t, y)| t as f64 * y)
                .sum();

            let m = (timeperiod as f64 * sum_ty - sum_t * sum_y) / denom;
            let c = (sum_y - m * sum_t) / timeperiod as f64;

            result[i - 1] = Some(m * (timeperiod as f64 - 1.0) + c);
        }
    }

    df.with_column(Series::new(output_name.into(), result).into())
        .unwrap();
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LinearRegAngleParams {
    pub col_real: String,
    pub timeperiod: usize,
}

/// Calcula el ángulo de la regresión lineal con ventana deslizante.
pub fn linearreg_angle(
    df: &mut DataFrame,
    col_real: &str,
    timeperiod: Option<usize>,
    output_col: Option<&str>,
) {
    let timeperiod = timeperiod.unwrap_or(20);
    let output_name = output_col.unwrap_or("linearreg_angle");
    let series = df.column(col_real).unwrap().f64().unwrap();

    let len = series.len();
    let mut result: Vec<Option<f64>> = vec![None; len];

    if len < timeperiod {
        df.with_column(Series::new(output_name.into(), result).into())
            .unwrap();
        return;
    }

    let sum_t: f64 = (0..timeperiod).map(|t| t as f64).sum();
    let sum_t2: f64 = (0..timeperiod).map(|t| (t as f64).powi(2)).sum();
    let denom = timeperiod as f64 * sum_t2 - sum_t * sum_t;

    for i in timeperiod..=len {
        let start = i - timeperiod;
        let window = series.slice(start as i64, timeperiod);

        if window.null_count() == 0 && denom != 0.0 {
            let sum_y: f64 = window.sum().unwrap();
            let sum_ty: f64 = window
                .into_no_null_iter()
                .enumerate()
                .map(|(t, y)| t as f64 * y)
                .sum();

            let m = (timeperiod as f64 * sum_ty - sum_t * sum_y) / denom;
            result[i - 1] = Some(m.atan() * 180.0 / std::f64::consts::PI);
        }
    }

    df.with_column(Series::new(output_name.into(), result).into())
        .unwrap();
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LinearRegInterceptParams {
    pub col_real: String,
    pub timeperiod: usize,
}

/// Calcula la intersección de la regresión lineal con ventana deslizante.
pub fn linearreg_intercept(
    df: &mut DataFrame,
    col_real: &str,
    timeperiod: Option<usize>,
    output_col: Option<&str>,
) {
    let timeperiod = timeperiod.unwrap_or(20);
    let output_name = output_col.unwrap_or("linearreg_intercept");
    let series = df.column(col_real).unwrap().f64().unwrap();

    let len = series.len();
    let mut result: Vec<Option<f64>> = vec![None; len];

    if len < timeperiod {
        df.with_column(Series::new(output_name.into(), result).into())
            .unwrap();
        return;
    }

    let sum_t: f64 = (0..timeperiod).map(|t| t as f64).sum();
    let sum_t2: f64 = (0..timeperiod).map(|t| (t as f64).powi(2)).sum();
    let denom = timeperiod as f64 * sum_t2 - sum_t * sum_t;

    for i in timeperiod..=len {
        let start = i - timeperiod;
        let window = series.slice(start as i64, timeperiod);

        if window.null_count() == 0 && denom != 0.0 {
            let sum_y: f64 = window.sum().unwrap();
            let sum_ty: f64 = window
                .into_no_null_iter()
                .enumerate()
                .map(|(t, y)| t as f64 * y)
                .sum();

            let m = (timeperiod as f64 * sum_ty - sum_t * sum_y) / denom;
            let c = (sum_y - m * sum_t) / timeperiod as f64;
            result[i - 1] = Some(c);
        }
    }

    df.with_column(Series::new(output_name.into(), result).into())
        .unwrap();
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LinearRegSlopeParams {
    pub col_real: String,
    pub timeperiod: usize,
}

/// Calcula la pendiente (slope) de la regresión lineal con ventana deslizante.
pub fn linearreg_slope(
    df: &mut DataFrame,
    col_real: &str,
    timeperiod: Option<usize>,
    output_col: Option<&str>,
) {
    let timeperiod = timeperiod.unwrap_or(20);
    let output_name = output_col.unwrap_or("linearreg_slope");
    let series = df.column(col_real).unwrap().f64().unwrap();

    let len = series.len();
    let mut result: Vec<Option<f64>> = vec![None; len];

    if len < timeperiod {
        df.with_column(Series::new(output_name.into(), result).into())
            .unwrap();
        return;
    }

    let sum_t: f64 = (0..timeperiod).map(|t| t as f64).sum();
    let sum_t2: f64 = (0..timeperiod).map(|t| (t as f64).powi(2)).sum();
    let denom = timeperiod as f64 * sum_t2 - sum_t * sum_t;

    for i in timeperiod..=len {
        let start = i - timeperiod;
        let window = series.slice(start as i64, timeperiod);

        if window.null_count() == 0 && denom != 0.0 {
            let sum_y: f64 = window.sum().unwrap();
            let sum_ty: f64 = window
                .into_no_null_iter()
                .enumerate()
                .map(|(t, y)| t as f64 * y)
                .sum();

            let m = (timeperiod as f64 * sum_ty - sum_t * sum_y) / denom;
            result[i - 1] = Some(m);
        }
    }

    df.with_column(Series::new(output_name.into(), result).into())
        .unwrap();
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StddevParams {
    pub col_real: String,
    pub timeperiod: usize,
    pub nbdev: f64,
}

/// Calcula la desviación estándar con ventana deslizante.
pub fn stddev(
    df: &mut DataFrame,
    col_real: &str,
    timeperiod: Option<usize>,
    nbdev: Option<f64>,
    output_col: Option<&str>,
) {
    let timeperiod = timeperiod.unwrap_or(1);
    let output_name = output_col.unwrap_or("stddev");
    let nbdev = nbdev.unwrap_or(1.0);
    let series = df.column(col_real).unwrap().f64().unwrap();

    let len = series.len();
    let mut result: Vec<Option<f64>> = vec![None; len];

    if len < timeperiod {
        df.with_column(Series::new(output_name.into(), result).into())
            .unwrap();
        return;
    }

    for i in timeperiod..=len {
        let start = i - timeperiod;
        let window = series.slice(start as i64, timeperiod);

        if window.null_count() == 0 {
            let mean = window.sum().unwrap() / timeperiod as f64;
            let variance: f64 = window
                .into_no_null_iter()
                .map(|v| (v - mean).powi(2))
                .sum::<f64>()
                / timeperiod as f64;

            result[i - 1] = Some(nbdev * variance.sqrt());
        }
    }

    df.with_column(Series::new(output_name.into(), result).into())
        .unwrap();
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TsfParams {
    pub col_real: String,
    pub timeperiod: usize,
}

/// Calcula la previsión de serie temporal (Time Series Forecast) con ventana deslizante.
pub fn tsf(
    df: &mut DataFrame,
    col_real: &str,
    timeperiod: Option<usize>,
    output_col: Option<&str>,
) {
    let timeperiod = timeperiod.unwrap_or(20);
    let output_name = output_col.unwrap_or("tsf");
    let series = df.column(col_real).unwrap().f64().unwrap();

    let len = series.len();
    let mut result: Vec<Option<f64>> = vec![None; len];

    if len < timeperiod {
        df.with_column(Series::new(output_name.into(), result).into())
            .unwrap();
        return;
    }

    let sum_t: f64 = (0..timeperiod).map(|t| t as f64).sum();
    let sum_t2: f64 = (0..timeperiod).map(|t| (t as f64).powi(2)).sum();
    let denom = timeperiod as f64 * sum_t2 - sum_t * sum_t;

    for i in timeperiod..=len {
        let start = i - timeperiod;
        let window = series.slice(start as i64, timeperiod);

        if window.null_count() == 0 && denom != 0.0 {
            let sum_y: f64 = window.sum().unwrap();
            let sum_ty: f64 = window
                .into_no_null_iter()
                .enumerate()
                .map(|(t, y)| t as f64 * y)
                .sum();

            let m = (timeperiod as f64 * sum_ty - sum_t * sum_y) / denom;
            let c = (sum_y - m * sum_t) / timeperiod as f64;

            // Previsión para el próximo período (x = timeperiod)
            result[i - 1] = Some(m * timeperiod as f64 + c);
        }
    }

    df.with_column(Series::new(output_name.into(), result).into())
        .unwrap();
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VarParams {
    pub col_real: String,
    pub timeperiod: usize,
    pub nbdev: f64,
}

/// Calcula la varianza con ventana deslizante.
pub fn var(
    df: &mut DataFrame,
    col_real: &str,
    timeperiod: Option<usize>,
    nbdev: Option<f64>,
    output_col: Option<&str>,
) {
    let timeperiod = timeperiod.unwrap_or(20);
    let output_name = output_col.unwrap_or("var");
    let nbdev = nbdev.unwrap_or(1.0);
    let series = df.column(col_real).unwrap().f64().unwrap();

    let len = series.len();
    let mut result: Vec<Option<f64>> = vec![None; len];

    if len < timeperiod {
        df.with_column(Series::new(output_name.into(), result).into())
            .unwrap();
        return;
    }

    for i in timeperiod..=len {
        let start = i - timeperiod;
        let window = series.slice(start as i64, timeperiod);

        if window.null_count() == 0 {
            let mean = window.sum().unwrap() / timeperiod as f64;
            let variance: f64 = window
                .into_no_null_iter()
                .map(|v| (v - mean).powi(2))
                .sum::<f64>()
                / timeperiod as f64;

            result[i - 1] = Some(nbdev * variance);
        }
    }

    df.with_column(Series::new(output_name.into(), result).into())
        .unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::data_format::DataFormatSymbol;
    use crate::utils::data_test::create_test_data;
    use std::fs::remove_file;

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
    fn test_beta() {
        match load_data() {
            Ok(mut df) => {
                beta(&mut df, "close", "open", None, Some("beta"));
                // save_data(&df, "data/test_beta.csv").unwrap();
                remove_file("data/test.csv").unwrap();
                remove_file("data/test.parquet").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        };
    }

    #[test]
    fn test_correl() {
        match load_data() {
            Ok(mut df) => {
                correl(&mut df, "close", "open", None, None);
                // save_data(&df, "data/test_correl.csv").unwrap();
                remove_file("data/test.csv").unwrap();
                remove_file("data/test.parquet").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        };
    }

    #[test]
    fn test_linearreg() {
        match load_data() {
            Ok(mut df) => {
                linearreg(&mut df, "close", None, None);
                // save_data(&df, "data/test_linearreg.csv").unwrap();
                remove_file("data/test.csv").unwrap();
                remove_file("data/test.parquet").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        };
    }

    #[test]
    fn test_stddev() {
        match load_data() {
            Ok(mut df) => {
                stddev(&mut df, "close", None, None, None);
                // save_data(&df, "data/test_stddev.csv").unwrap();
                remove_file("data/test.csv").unwrap();
                remove_file("data/test.parquet").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        };
    }

    #[test]
    fn test_linearreg_angle() {
        match load_data() {
            Ok(mut df) => {
                linearreg_angle(&mut df, "close", None, None);
                // save_data(&df, "data/test_linearreg_angle.csv").unwrap();
                remove_file("data/test.csv").unwrap();
                remove_file("data/test.parquet").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        };
    }

    #[test]
    fn test_linearreg_intercept() {
        match load_data() {
            Ok(mut df) => {
                linearreg_intercept(&mut df, "close", None, None);
                // save_data(&df, "data/test_linearreg_intercept.csv").unwrap();
                remove_file("data/test.csv").unwrap();
                remove_file("data/test.parquet").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        };
    }

    #[test]
    fn test_linearreg_slope() {
        match load_data() {
            Ok(mut df) => {
                linearreg_slope(&mut df, "close", None, None);
                // save_data(&df, "data/test_linearreg_slope.csv").unwrap();
                remove_file("data/test.csv").unwrap();
                remove_file("data/test.parquet").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        };
    }

    #[test]
    fn test_tsf() {
        match load_data() {
            Ok(mut df) => {
                tsf(&mut df, "close", None, None);
                // save_data(&df, "data/test_tsf.csv").unwrap();
                remove_file("data/test.csv").unwrap();
                remove_file("data/test.parquet").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        };
    }

    #[test]
    fn test_var() {
        match load_data() {
            Ok(mut df) => {
                var(&mut df, "close", None, None, None);
                // save_data(&df, "data/test_var.csv").unwrap();
                remove_file("data/test.csv").unwrap();
                remove_file("data/test.parquet").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        };
    }
}
