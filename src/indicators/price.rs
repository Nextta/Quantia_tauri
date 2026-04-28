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
pub async fn avgprice(mut df: DataFrame, output_col: Option<&str>) -> PolarsResult<DataFrame> {
    let output_col = output_col.unwrap_or("avgprice");

    let open = get_open(&df)?;
    let high = get_high(&df)?;
    let low = get_low(&df)?;
    let close = get_close(&df)?;

    let sum1 = (&open + &high)?;
    let sum2 = (&sum1 + &low)?;
    let sum3 = (&sum2 + &close)?;
    let avgprice_series = sum3 / 4.0;

    let mut avgprice_series = avgprice_series;
    avgprice_series.rename(output_col.into());

    df.with_column(avgprice_series.into())?;
    Ok(df)
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
pub async fn medprice(mut df: DataFrame, output_col: Option<&str>) -> PolarsResult<DataFrame> {
    let output_col = output_col.unwrap_or("medprice");

    let high = get_high(&df)?;
    let low = get_low(&df)?;

    let sum = (&high + &low)?;
    let medprice_series = sum / 2.0;

    let mut medprice_series = medprice_series;
    medprice_series.rename(output_col.into());

    df.with_column(medprice_series.into())?;
    Ok(df)
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
pub async fn typprice(mut df: DataFrame, output_col: Option<&str>) -> PolarsResult<DataFrame> {
    let output_col = output_col.unwrap_or("typprice");

    let high = get_high(&df)?;
    let low = get_low(&df)?;
    let close = get_close(&df)?;

    let sum1 = (&high + &low)?;
    let sum2 = (&sum1 + &close)?;
    let typprice_series = sum2 / 3.0;

    let mut typprice_series = typprice_series;
    typprice_series.rename(output_col.into());

    df.with_column(typprice_series.into())?;
    Ok(df)
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
pub async fn wclprice(mut df: DataFrame, output_col: Option<&str>) -> PolarsResult<DataFrame> {
    let output_col = output_col.unwrap_or("wclprice");

    let high = get_high(&df)?;
    let low = get_low(&df)?;
    let close = get_close(&df)?;

    let close_weighted = &close * 2.0;
    let sum1 = (&high + &low)?;
    let sum2 = (&sum1 + &close_weighted)?;
    let wclprice_series = sum2 / 4.0;

    let mut wclprice_series = wclprice_series;
    wclprice_series.rename(output_col.into());

    df.with_column(wclprice_series.into())?;
    Ok(df)
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
