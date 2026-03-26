use polars::prelude::*;

#[derive(Debug, Clone)]
pub struct Datos {
    datos: DataFrame,
}

impl Datos {
    pub fn new(datos: DataFrame) -> Self {
        Datos { datos }
    }

    //GETTERS
    pub fn get_datos(&self) -> &DataFrame {
        &self.datos
    }

    //SETTERS
    pub fn set_datos(&mut self, datos: DataFrame) {
        self.datos = datos;
    }

    //Funciones
    pub fn timeframe(&self, timeframe: &str) -> PolarsResult<DataFrame> {
        let lz: LazyFrame = self.datos.clone().lazy();

        let df: DataFrame;
        let schema = lz
            .clone()
            .limit(0)
            .collect()?
            .schema()
            .get("open")
            .is_some();

        if schema {
            df = lz
                .group_by_dynamic(
                    col("timestamp"),
                    [],
                    DynamicGroupOptions {
                        every: Duration::parse(&timeframe),
                        period: Duration::parse(&timeframe),
                        offset: Duration::parse("0ms"),
                        ..Default::default()
                    },
                )
                .agg([
                    col("open").first(),
                    col("high").max(),
                    col("low").min(),
                    col("close").last(),
                    col("volume").sum(),
                ])
                .collect()?;
        } else {
            df = lz
                .select([
                    col("timestamp"),
                    col("bidPrice").alias("open"),
                    col("bidPrice").alias("high"),
                    col("bidPrice").alias("low"),
                    col("bidPrice").alias("close"),
                    (col("bidVolume") + col("askVolume")).alias("volume"),
                ])
                .group_by_dynamic(
                    col("timestamp"),
                    [],
                    DynamicGroupOptions {
                        every: Duration::parse(&timeframe),
                        period: Duration::parse(&timeframe),
                        offset: Duration::parse("0ms"),
                        ..Default::default()
                    },
                )
                .agg([
                    col("open").first(),
                    col("high").max(),
                    col("low").min(),
                    col("close").last(),
                    col("volume").sum(),
                ])
                .collect()?;
        }

        Ok(df)
    }
}
