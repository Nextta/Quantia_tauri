use polars::prelude::*;

/// Lista de indicadores:
/// AVGPRICE             Average Price
/// MEDPRICE             Median Price
/// TYPPRICE             Typical Price
/// WCLPRICE             Weighted Close Price

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

/// Get open column from DataFrame (case insensitive)
fn get_open(df: &DataFrame) -> PolarsResult<Series> {
    let s = df.column("open").or_else(|_| df.column("Open"))?;
    Ok(s.cast(&DataType::Float64)?.take_materialized_series())
}

// ============================================================================
// AVGPRICE - Average Price
// ============================================================================

/// AVGPRICE - Average Price
///
/// Calcula el precio promedio de la barra usando los cuatro precios principales:
/// open, high, low y close. Proporciona un valor de precio único que representa
/// toda la actividad de la barra.
///
/// Este indicador es útil para simplificar el análisis cuando se necesita
/// un solo valor de precio en lugar de los cuatro componentes separados.
///
/// # Parámetros
/// * `df` - DataFrame con columnas: open, high, low, close (case insensitive)
/// * `output_col` - Nombre de la columna de salida (default: "avgprice")
///
/// # Retorna
/// DataFrame con columna "avgprice" añadida
///
/// # Fórmula
/// AVGPRICE = (open + high + low + close) / 4
///
/// # Ejemplo
/// ```rust
/// let df_with_avg = avgprice(df, None).await?;
/// ```
pub async fn avgprice(df: DataFrame, output_col: Option<&str>) -> PolarsResult<DataFrame> {
    let output_col = output_col.unwrap_or("avgprice");

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

    let avgprice_vals: Vec<f64> = open_vals
        .iter()
        .zip(&high_vals)
        .zip(&low_vals)
        .zip(&close_vals)
        .map(|(((o, h), l), c)| (o + h + l + c) / 4.0)
        .collect();

    let avgprice_series = Series::new(output_col.into(), &avgprice_vals);
    let mut result_df = df;
    result_df.with_column(avgprice_series.into())?;
    Ok(result_df)
}

// ============================================================================
// MEDPRICE - Median Price
// ============================================================================

/// MEDPRICE - Median Price
///
/// Calcula el precio medio de la barra usando únicamente el high y el low.
/// Representa el punto medio del rango de negociación de la barra.
///
/// Este indicador es simple y efectivo para obtener un valor representativo
/// del precio cuando no se dispone de open/close o no son relevantes.
///
/// # Parámetros
/// * `df` - DataFrame con columnas: high, low (case insensitive)
/// * `output_col` - Nombre de la columna de salida (default: "medprice")
///
/// # Retorna
/// DataFrame con columna "medprice" añadida
///
/// # Fórmula
/// MEDPRICE = (high + low) / 2
///
/// # Ejemplo
/// ```rust
/// let df_with_med = medprice(df, None).await?;
/// ```
pub async fn medprice(df: DataFrame, output_col: Option<&str>) -> PolarsResult<DataFrame> {
    let output_col = output_col.unwrap_or("medprice");

    let high = get_high(&df)?;
    let low = get_low(&df)?;

    let high_ca: ChunkedArray<Float64Type> = high.f64().unwrap().clone();
    let low_ca: ChunkedArray<Float64Type> = low.f64().unwrap().clone();

    let high_vals: Vec<f64> = high_ca.into_no_null_iter().collect();
    let low_vals: Vec<f64> = low_ca.into_no_null_iter().collect();

    let medprice_vals: Vec<f64> = high_vals
        .iter()
        .zip(&low_vals)
        .map(|(h, l)| (h + l) / 2.0)
        .collect();

    let medprice_series = Series::new(output_col.into(), &medprice_vals);
    let mut result_df = df;
    result_df.with_column(medprice_series.into())?;
    Ok(result_df)
}

// ============================================================================
// TYPPRICE - Typical Price
// ============================================================================

/// TYPPRICE - Typical Price
///
/// Calcula el precio típico como el promedio de high, low y close.
/// Es uno de los precios compuestos más utilizados en análisis técnico.
///
/// El Typical Price es la base para muchos indicadores como el CCI
/// (Commodity Channel Index) y el Money Flow Index (MFI).
///
/// # Parámetros
/// * `df` - DataFrame con columnas: high, low, close (case insensitive)
/// * `output_col` - Nombre de la columna de salida (default: "typprice")
///
/// # Retorna
/// DataFrame con columna "typprice" añadida
///
/// # Fórmula
/// TYPPRICE = (high + low + close) / 3
///
/// # Ejemplo
/// ```rust
/// let df_with_typ = typprice(df, None).await?;
/// ```
pub async fn typprice(df: DataFrame, output_col: Option<&str>) -> PolarsResult<DataFrame> {
    let output_col = output_col.unwrap_or("typprice");

    let high = get_high(&df)?;
    let low = get_low(&df)?;
    let close = get_close(&df)?;

    let high_ca: ChunkedArray<Float64Type> = high.f64().unwrap().clone();
    let low_ca: ChunkedArray<Float64Type> = low.f64().unwrap().clone();
    let close_ca: ChunkedArray<Float64Type> = close.f64().unwrap().clone();

    let high_vals: Vec<f64> = high_ca.into_no_null_iter().collect();
    let low_vals: Vec<f64> = low_ca.into_no_null_iter().collect();
    let close_vals: Vec<f64> = close_ca.into_no_null_iter().collect();

    let typprice_vals: Vec<f64> = high_vals
        .iter()
        .zip(&low_vals)
        .zip(&close_vals)
        .map(|((h, l), c)| (h + l + c) / 3.0)
        .collect();

    let typprice_series = Series::new(output_col.into(), &typprice_vals);
    let mut result_df = df;
    result_df.with_column(typprice_series.into())?;
    Ok(result_df)
}

// ============================================================================
// WCLPRICE - Weighted Close Price
// ============================================================================

/// WCLPRICE - Weighted Close Price
///
/// Calcula el precio ponderado dando doble peso al precio de cierre.
/// El cierre es considerado el precio más importante porque representa
/// el consenso final del mercado después de toda la negociación.
///
/// Este indicador da más importancia al cierre mientras todavía
/// considera el rango de negociación del día.
///
/// # Parámetros
/// * `df` - DataFrame con columnas: high, low, close (case insensitive)
/// * `output_col` - Nombre de la columna de salida (default: "wclprice")
///
/// # Retorna
/// DataFrame con columna "wclprice" añadida
///
/// # Fórmula
/// WCLPRICE = (high + low + 2 * close) / 4
///
/// # Ejemplo
/// ```rust
/// let df_with_wcl = wclprice(df, None).await?;
/// ```
pub async fn wclprice(df: DataFrame, output_col: Option<&str>) -> PolarsResult<DataFrame> {
    let output_col = output_col.unwrap_or("wclprice");

    let high = get_high(&df)?;
    let low = get_low(&df)?;
    let close = get_close(&df)?;

    let high_ca: ChunkedArray<Float64Type> = high.f64().unwrap().clone();
    let low_ca: ChunkedArray<Float64Type> = low.f64().unwrap().clone();
    let close_ca: ChunkedArray<Float64Type> = close.f64().unwrap().clone();

    let high_vals: Vec<f64> = high_ca.into_no_null_iter().collect();
    let low_vals: Vec<f64> = low_ca.into_no_null_iter().collect();
    let close_vals: Vec<f64> = close_ca.into_no_null_iter().collect();

    let wclprice_vals: Vec<f64> = high_vals
        .iter()
        .zip(&low_vals)
        .zip(&close_vals)
        .map(|((h, l), c)| (h + l + 2.0 * c) / 4.0)
        .collect();

    let wclprice_series = Series::new(output_col.into(), &wclprice_vals);
    let mut result_df = df;
    result_df.with_column(wclprice_series.into())?;
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
    async fn test_avgprice() {
        match load_data().await {
            Ok(df) => match avgprice(df, None).await {
                Ok(result) => {
                    save_data(&result, "download/test_avgprice.csv")
                        .await
                        .unwrap();
                }
                Err(e) => panic!("Failed to compute avgprice: {:?}", e),
            },
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_medprice() {
        match load_data().await {
            Ok(df) => match medprice(df, None).await {
                Ok(result) => {
                    save_data(&result, "download/test_medprice.csv")
                        .await
                        .unwrap();
                }
                Err(e) => panic!("Failed to compute medprice: {:?}", e),
            },
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_typprice() {
        match load_data().await {
            Ok(df) => match typprice(df, None).await {
                Ok(result) => {
                    save_data(&result, "download/test_typprice.csv")
                        .await
                        .unwrap();
                }
                Err(e) => panic!("Failed to compute typprice: {:?}", e),
            },
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_wclprice() {
        match load_data().await {
            Ok(df) => match wclprice(df, None).await {
                Ok(result) => {
                    save_data(&result, "download/test_wclprice.csv")
                        .await
                        .unwrap();
                }
                Err(e) => panic!("Failed to compute wclprice: {:?}", e),
            },
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }
}
