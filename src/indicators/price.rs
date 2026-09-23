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
    let s = df.column("high").or_else(|_| df.column("High")).unwrap();
    Ok(s.cast(&DataType::Float64)
        .unwrap()
        .take_materialized_series())
}

/// Get low column from DataFrame (case insensitive)
fn get_low(df: &DataFrame) -> PolarsResult<Series> {
    let s = df.column("low").or_else(|_| df.column("Low")).unwrap();
    Ok(s.cast(&DataType::Float64)
        .unwrap()
        .take_materialized_series())
}

/// Get close column from DataFrame (case insensitive)
fn get_close(df: &DataFrame) -> PolarsResult<Series> {
    let s = df.column("close").or_else(|_| df.column("Close")).unwrap();
    Ok(s.cast(&DataType::Float64)
        .unwrap()
        .take_materialized_series())
}

/// Get open column from DataFrame (case insensitive)
fn get_open(df: &DataFrame) -> PolarsResult<Series> {
    let s = df.column("open").or_else(|_| df.column("Open")).unwrap();
    Ok(s.cast(&DataType::Float64)
        .unwrap()
        .take_materialized_series())
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
/// let df_with_avg = avgprice(df, None).unwrap();
/// ```
pub fn avgprice(df: &mut DataFrame, output_col: Option<&str>) {
    let output_col = output_col.unwrap_or("avgprice");

    let open = get_open(&df).unwrap();
    let high = get_high(&df).unwrap();
    let low = get_low(&df).unwrap();
    let close = get_close(&df).unwrap();

    let sum1 = (&open + &high).unwrap();
    let sum2 = (&sum1 + &low).unwrap();
    let sum3 = (&sum2 + &close).unwrap();
    let avgprice_series = sum3 / 4.0;

    let mut avgprice_series = avgprice_series;
    avgprice_series.rename(output_col.into());

    df.with_column(avgprice_series.into()).unwrap();
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
/// let df_with_med = medprice(df, None).unwrap();
/// ```
pub fn medprice(df: &mut DataFrame, output_col: Option<&str>) {
    let output_col = output_col.unwrap_or("medprice");

    let high = get_high(&df).unwrap();
    let low = get_low(&df).unwrap();

    let sum = (&high + &low).unwrap();
    let medprice_series = sum / 2.0;

    let mut medprice_series = medprice_series;
    medprice_series.rename(output_col.into());

    df.with_column(medprice_series.into()).unwrap();
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
/// let df_with_typ = typprice(df, None).unwrap();
/// ```
pub fn typprice(df: &mut DataFrame, output_col: Option<&str>) {
    let output_col = output_col.unwrap_or("typprice");

    let high = get_high(&df).unwrap();
    let low = get_low(&df).unwrap();
    let close = get_close(&df).unwrap();

    let sum1 = (&high + &low).unwrap();
    let sum2 = (&sum1 + &close).unwrap();
    let typprice_series = sum2 / 3.0;

    let mut typprice_series = typprice_series;
    typprice_series.rename(output_col.into());

    df.with_column(typprice_series.into()).unwrap();
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
/// let df_with_wcl = wclprice(df, None).unwrap();
/// ```
pub fn wclprice(df: &mut DataFrame, output_col: Option<&str>) {
    let output_col = output_col.unwrap_or("wclprice");

    let high = get_high(&df).unwrap();
    let low = get_low(&df).unwrap();
    let close = get_close(&df).unwrap();

    let close_weighted = &close * 2.0;
    let sum1 = (&high + &low).unwrap();
    let sum2 = (&sum1 + &close_weighted).unwrap();
    let wclprice_series = sum2 / 4.0;

    let mut wclprice_series = wclprice_series;
    wclprice_series.rename(output_col.into());

    df.with_column(wclprice_series.into()).unwrap();
}

/// Calcula las velas OHLC diarias a partir de un DataFrame de velas horarias.
///
/// # Argumentos
///
/// * `df` - DataFrame de velas horarias.
/// * `output_col_open` - Nombre de la columna de apertura diaria.
/// * `output_col_high` - Nombre de la columna de alta diaria.
/// * `output_col_low` - Nombre de la columna de baja diaria.
/// * `output_col_close` - Nombre de la columna de cierre diaria.
///
/// # Retorna
///
/// Un DataFrame con las velas OHLC diarias.
///
/// # Ejemplo
/// ```rust
/// let df_daily = daily_ohlc(df, None, None, None, None).unwrap();
/// ```
pub fn daily_ohlc(
    df: &mut DataFrame,
    output_col_open: Option<&str>,
    output_col_high: Option<&str>,
    output_col_low: Option<&str>,
    output_col_close: Option<&str>,
) {
    let output_col_open = output_col_open.unwrap_or("open_daily");
    let output_col_high = output_col_high.unwrap_or("high_daily");
    let output_col_low = output_col_low.unwrap_or("low_daily");
    let output_col_close = output_col_close.unwrap_or("close_daily");

    let df_original: DataFrame = df
        .clone()
        .lazy()
        .with_column(
            (col("time") / lit(86400000i64)) // Convertir ms a días desde epoch
                .cast(DataType::Date) // Convertir a tipo Date
                .alias("date"),
        )
        .collect()
        .unwrap();

    let ohlc_daily = df_original
        .clone()
        .lazy()
        .group_by([col("date")])
        .agg([
            col("open").first().alias(output_col_open),
            col("high").max().alias(output_col_high),
            col("low").min().alias(output_col_low),
            col("close").last().alias(output_col_close),
        ])
        .collect()
        .unwrap();

    let mut result: DataFrame = df_original
        .clone()
        .lazy()
        .join(
            ohlc_daily.lazy(),             // DataFrame derecho (OHLC diario)
            [col("date")],                 // Clave join izquierdo
            [col("date")],                 // Clave join derecho
            JoinArgs::new(JoinType::Left), // Left join para mantener todas las velas
        )
        .collect()
        .unwrap();

    result = result.drop("date").unwrap();
    for name in [
        output_col_open,
        output_col_high,
        output_col_low,
        output_col_close,
    ] {
        df.with_column(result.column(name).unwrap().clone())
            .unwrap();
    }
}

/// Calcula las velas OHLC semanales a partir de un DataFrame de velas.
///
/// # Argumentos
///
/// * `df` - DataFrame de velas diarias.
/// * `output_col_open` - Nombre de la columna de apertura semanal.
/// * `output_col_high` - Nombre de la columna de alta semanal.
/// * `output_col_low` - Nombre de la columna de baja semanal.
/// * `output_col_close` - Nombre de la columna de cierre semanal.
///
/// # Retorna
///
/// Un DataFrame con las velas OHLC semanales.
///
/// # Ejemplo
/// ```rust
/// let df_weekly = weekly_ohlc(df, None, None, None, None).unwrap();
/// ```
pub fn weekly_ohlc(
    df: &mut DataFrame,
    output_col_open: Option<&str>,
    output_col_high: Option<&str>,
    output_col_low: Option<&str>,
    output_col_close: Option<&str>,
) {
    let output_col_open = output_col_open.unwrap_or("open_weekly");
    let output_col_high = output_col_high.unwrap_or("high_weekly");
    let output_col_low = output_col_low.unwrap_or("low_weekly");
    let output_col_close = output_col_close.unwrap_or("close_weekly");

    let lf = df.clone().lazy();

    // Crear columna week
    let df_with_week = lf.with_column(
        col("time")
            .cast(DataType::Datetime(TimeUnit::Milliseconds, None))
            .dt()
            .truncate(lit("1w"))
            .alias("week"),
    );

    // OHLC semanal
    let ohlc_weekly = df_with_week.clone().group_by([col("week")]).agg([
        col("open").first().alias(output_col_open),
        col("high").max().alias(output_col_high),
        col("low").min().alias(output_col_low),
        col("close").last().alias(output_col_close),
    ]);

    // Join
    let mut result = df_with_week
        .join(
            ohlc_weekly,
            [col("week")],
            [col("week")],
            JoinArgs::new(JoinType::Left),
        )
        .collect()
        .unwrap();

    result = result.drop("week").unwrap();
    for name in [
        output_col_open,
        output_col_high,
        output_col_low,
        output_col_close,
    ] {
        df.with_column(result.column(name).unwrap().clone())
            .unwrap();
    }
}

/// Calcula las velas OHLC mensuales a partir de un DataFrame de velas.
///
/// # Argumentos
///
/// * `df` - DataFrame de velas diarias.
/// * `output_col_open` - Nombre de la columna de apertura mensual.
/// * `output_col_high` - Nombre de la columna de alta mensual.
/// * `output_col_low` - Nombre de la columna de baja mensual.
/// * `output_col_close` - Nombre de la columna de cierre mensual.
///
/// # Retorna
///
/// Un DataFrame con las velas OHLC mensuales.
///
/// # Ejemplo
/// ```rust
/// let df_monthly = monthly_ohlc(df, None, None, None, None);
/// ```
pub fn monthly_ohlc(
    df: &mut DataFrame,
    output_col_open: Option<&str>,
    output_col_high: Option<&str>,
    output_col_low: Option<&str>,
    output_col_close: Option<&str>,
) {
    let output_col_open = output_col_open.unwrap_or("open_monthly");
    let output_col_high = output_col_high.unwrap_or("high_monthly");
    let output_col_low = output_col_low.unwrap_or("low_monthly");
    let output_col_close = output_col_close.unwrap_or("close_monthly");

    // PASO 1: Crear columna 'month' (fecha inicio mes) desde timestamp
    let df_original: DataFrame = df
        .clone()
        .lazy()
        .with_column(
            col("time")
                .cast(DataType::Datetime(TimeUnit::Milliseconds, None))
                .dt()
                .truncate(lit("1mo"))
                .cast(DataType::Date)
                .alias("month"),
        )
        .collect()
        .unwrap();

    // PASO 2: Group by 'month' → calcular OHLC mensual
    let ohlc_monthly = df_original
        .clone()
        .lazy()
        .group_by([col("month")])
        .agg([
            col("open").first().alias(output_col_open),
            col("high").max().alias(output_col_high),
            col("low").min().alias(output_col_low),
            col("close").last().alias(output_col_close),
        ])
        .collect()
        .unwrap();

    // PASO 3: Left join con df original por 'month'
    let mut result: DataFrame = df_original
        .clone()
        .lazy()
        .join(
            ohlc_monthly.lazy(),
            [col("month")],
            [col("month")],
            JoinArgs::new(JoinType::Left),
        )
        .collect()
        .unwrap();

    // PASO 4: Eliminar columna 'month'
    result = result.drop("month").unwrap();

    for name in [
        output_col_open,
        output_col_high,
        output_col_low,
        output_col_close,
    ] {
        df.with_column(result.column(name).unwrap().clone())
            .unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::data_format::DataFormatSymbol;
    use crate::utils::data_test::create_test_data;
    use std::fs::remove_file;

    //Para los test crear una carpeta llamada data en la raiz de este proyecto
    // y llamar a los datos test.csv
    fn load_data() -> PolarsResult<DataFrame> {
        create_test_data(&DataFormatSymbol::Csv);

        let df = CsvReadOptions::default()
            .try_into_reader_with_file_path(Some("data/test.csv".into()))
            .unwrap()
            .finish()
            .unwrap();
        Ok(df)
    }

    // fn save_data(df: &DataFrame, path: &str) -> PolarsResult<()> {
    //     let mut df: DataFrame = df.clone();
    //     let mut file = std::fs::File::create(path).unwrap();
    //     CsvWriter::new(&mut file).finish(&mut df).unwrap();
    //     Ok(())
    // }

    #[test]
    fn test_weekly_ohlc() {
        let _guardia = crate::utils::data_test::BLOQUEO_RECURSOS.blocking_lock();
        match load_data() {
            Ok(mut df) => {
                weekly_ohlc(&mut df, None, None, None, None);
                // save_data(&df, "data/test_weekly_ohlc.csv").unwrap();
                remove_file("data/test.csv").unwrap();
                remove_file("data/test.parquet").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[test]
    fn test_daily_ohlc() {
        let _guardia = crate::utils::data_test::BLOQUEO_RECURSOS.blocking_lock();
        match load_data() {
            Ok(mut df) => {
                daily_ohlc(&mut df, None, None, None, None);
                // save_data(&df, "data/test_daily_ohlc.csv").unwrap();
                remove_file("data/test.csv").unwrap();
                remove_file("data/test.parquet").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[test]
    fn test_monthly_ohlc() {
        let _guardia = crate::utils::data_test::BLOQUEO_RECURSOS.blocking_lock();
        match load_data() {
            Ok(mut df) => {
                monthly_ohlc(&mut df, None, None, None, None);
                // save_data(&df, "data/test_monthly_ohlc.csv").unwrap();
                remove_file("data/test.csv").unwrap();
                remove_file("data/test.parquet").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[test]
    fn test_avgprice() {
        let _guardia = crate::utils::data_test::BLOQUEO_RECURSOS.blocking_lock();
        match load_data() {
            Ok(mut df) => {
                avgprice(&mut df, None);
                // save_data(&df, "data/test_avgprice.csv").unwrap();
                remove_file("data/test.csv").unwrap();
                remove_file("data/test.parquet").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[test]
    fn test_medprice() {
        let _guardia = crate::utils::data_test::BLOQUEO_RECURSOS.blocking_lock();
        match load_data() {
            Ok(mut df) => {
                medprice(&mut df, None);
                // save_data(&df, "data/test_medprice.csv").unwrap();
                remove_file("data/test.csv").unwrap();
                remove_file("data/test.parquet").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[test]
    fn test_typprice() {
        let _guardia = crate::utils::data_test::BLOQUEO_RECURSOS.blocking_lock();
        match load_data() {
            Ok(mut df) => {
                typprice(&mut df, None);
                // save_data(&df, "data/test_typprice.csv").unwrap();
                remove_file("data/test.csv").unwrap();
                remove_file("data/test.parquet").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[test]
    fn test_wclprice() {
        let _guardia = crate::utils::data_test::BLOQUEO_RECURSOS.blocking_lock();
        match load_data() {
            Ok(mut df) => {
                wclprice(&mut df, None);
                // save_data(&df, "data/test_wclprice.csv").unwrap();
                remove_file("data/test.csv").unwrap();
                remove_file("data/test.parquet").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }
}
