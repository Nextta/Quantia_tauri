use polars::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Datos {
    datos: DataFrame,
}

impl Datos {
    pub fn new(datos: DataFrame) -> Self {
        Datos { datos }
    }

    //GETTERS
    pub fn get_datos(&self) -> DataFrame {
        self.datos.clone()
    }

    //SETTERS
    pub fn set_datos(&mut self, datos: DataFrame) {
        self.datos = datos;
    }

    //Funciones
    pub fn timeframe(&self, timeframe: &str) -> PolarsResult<DataFrame> {
        let lz: LazyFrame = self.datos.clone().lazy();

        let schema = lz
            .clone()
            .limit(0)
            .collect()?
            .schema()
            .get("open")
            .is_some();

        let df: DataFrame = if schema {
            lz.group_by_dynamic(
                col("time"),
                [],
                DynamicGroupOptions {
                    every: Duration::parse(timeframe),
                    period: Duration::parse(timeframe),
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
            .collect()?
        } else {
            lz.select([
                col("time"),
                col("bidPrice").alias("open"),
                col("bidPrice").alias("high"),
                col("bidPrice").alias("low"),
                col("bidPrice").alias("close"),
                (col("bidVolume") + col("askVolume")).alias("volume"),
            ])
            .group_by_dynamic(
                col("time"),
                [],
                DynamicGroupOptions {
                    every: Duration::parse(timeframe),
                    period: Duration::parse(timeframe),
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
            .collect()?
        };

        Ok(df)
    }
}
