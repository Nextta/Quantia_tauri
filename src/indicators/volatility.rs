use polars::prelude::*;
use serde::{Deserialize, Serialize};

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
fn calc_true_range(high: &Series, low: &Series, close: &Series) -> PolarsResult<Series> {
    let n = high.len();
    let high_ca = high.f64()?;
    let low_ca = low.f64()?;
    let close_ca = close.f64()?;

    let mut tr_vals = Vec::with_capacity(n);

    for i in 0..n {
        if i == 0 {
            // First bar: TR = High - Low
            let h = high_ca.get(i);
            let l = low_ca.get(i);
            if let (Some(hv), Some(lv)) = (h, l) {
                tr_vals.push(Some(hv - lv));
            } else {
                tr_vals.push(None);
            }
        } else {
            // TR = max(High - Low, |High - PrevClose|, |Low - PrevClose|)
            let h = high_ca.get(i);
            let l = low_ca.get(i);
            let pc = close_ca.get(i - 1);

            if let (Some(hv), Some(lv), Some(pcv)) = (h, l, pc) {
                let tr1 = hv - lv;
                let tr2 = (hv - pcv).abs();
                let tr3 = (lv - pcv).abs();
                tr_vals.push(Some(tr1.max(tr2).max(tr3)));
            } else {
                tr_vals.push(None);
            }
        }
    }

    Ok(Series::new("trange".into(), tr_vals))
}

/// Wilder's RMA (Running Moving Average) for ATR calculation
fn rma_series(values: &Series, period: usize) -> PolarsResult<Series> {
    let ca = values.f64()?;
    let n = ca.len();
    let mut rma_values = vec![f64::NAN; n];

    if n < period {
        return Ok(Series::new("rma".into(), &rma_values));
    }

    let alpha = 1.0 / period as f64;
    let vals: Vec<f64> = ca.into_iter().map(|v| v.unwrap_or(f64::NAN)).collect();

    // Initialize with SMA
    let mut sum = 0.0;
    let mut count = 0;
    let mut first_valid_idx = None;

    for i in 0..n {
        if !vals[i].is_nan() {
            sum += vals[i];
            count += 1;
            if count == period {
                let initial_sma = sum / period as f64;
                rma_values[i] = initial_sma;
                first_valid_idx = Some(i);
                break;
            }
        }
    }

    if let Some(start_idx) = first_valid_idx {
        let mut current_rma = rma_values[start_idx];
        for i in (start_idx + 1)..n {
            if !vals[i].is_nan() {
                current_rma = vals[i] * alpha + current_rma * (1.0 - alpha);
                rma_values[i] = current_rma;
            } else {
                rma_values[i] = f64::NAN;
            }
        }
    }

    Ok(Series::new("rma".into(), &rma_values))
}

// ============================================================================
// TRANGE - True Range
// ============================================================================

/// TRANGE - True Range
pub fn trange(df: &mut DataFrame, output_col: Option<&str>) {
    let output_col = output_col.unwrap_or("trange");

    let high = get_high(&df).unwrap();
    let low = get_low(&df).unwrap();
    let close = get_close(&df).unwrap();

    let mut tr_series = calc_true_range(&high, &low, &close).unwrap();
    tr_series.rename(output_col.into());

    df.with_column(tr_series.into()).unwrap();
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AtrParams {
    pub timeperiod: usize,
    pub multiplier: f64,
}

// ============================================================================
// ATR - Average True Range
// ============================================================================

/// ATR - Average True Range
pub fn atr(
    df: &mut DataFrame,
    timeperiod: Option<usize>,
    multiplier: Option<f64>,
    output_col: Option<&str>,
) {
    let timeperiod = timeperiod.unwrap_or(14);
    let multiplier = multiplier.unwrap_or(1.0);
    let output_col = output_col.unwrap_or("atr");

    let high = get_high(&df).unwrap();
    let low = get_low(&df).unwrap();
    let close = get_close(&df).unwrap();

    let tr_series = calc_true_range(&high, &low, &close).unwrap();
    let mut atr_series = rma_series(&tr_series, timeperiod).unwrap();
    atr_series = &atr_series * multiplier;
    atr_series.rename(output_col.into());

    df.with_column(atr_series.into()).unwrap();
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NatrParams {
    pub timeperiod: usize,
}

// ============================================================================
// NATR - Normalized Average True Range
// ============================================================================

/// NATR - Normalized Average True Range
pub fn natr(df: &mut DataFrame, timeperiod: Option<usize>, output_col: Option<&str>) {
    let timeperiod = timeperiod.unwrap_or(14);
    let output_col = output_col.unwrap_or("natr");

    let high = get_high(&df).unwrap();
    let low = get_low(&df).unwrap();
    let close = get_close(&df).unwrap();

    let tr_series = calc_true_range(&high, &low, &close).unwrap();
    let atr_series = rma_series(&tr_series, timeperiod).unwrap();

    // (ATR / close) * 100
    let natr_series = (&atr_series / &close).unwrap();
    let mut natr_series = &natr_series * 100.0;
    natr_series.rename(output_col.into());

    df.with_column(natr_series.into()).unwrap();
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
            .try_into_reader_with_file_path(Some("download/test.csv".into()))
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
    fn test_trange() {
        match load_data() {
            Ok(mut df) => {
                trange(&mut df, None);
                // save_data(&df, "download/test_trange.csv").unwrap();
                remove_file("download/test.csv").unwrap();
                remove_file("download/test.parquet").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[test]
    fn test_atr() {
        match load_data() {
            Ok(mut df) => {
                atr(&mut df, Some(14), Some(1.0), None);
                // save_data(&df, "download/test_atr.csv").unwrap();
                remove_file("download/test.csv").unwrap();
                remove_file("download/test.parquet").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[test]
    fn test_natr() {
        match load_data() {
            Ok(mut df) => {
                natr(&mut df, Some(14), None);
                // save_data(&df, "download/test_natr.csv").unwrap();
                remove_file("download/test.csv").unwrap();
                remove_file("download/test.parquet").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }
}
