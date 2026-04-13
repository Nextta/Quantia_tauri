use polars::prelude::*;

/// Lista de indicadores:
/// HT_DCPERIOD          Hilbert Transform - Dominant Cycle Period
/// HT_DCPHASE           Hilbert Transform - Dominant Cycle Phase
/// HT_PHASOR            Hilbert Transform - Phasor Components
/// HT_SINE              Hilbert Transform - SineWave
/// HT_TRENDMODE         Hilbert Transform - Trend vs Cycle Mode

pub async fn ht_dcperiod(df: DataFrame) -> PolarsResult<DataFrame> {
    Ok(df)
}

pub async fn ht_dcphase(df: DataFrame) -> PolarsResult<DataFrame> {
    Ok(df)
}

pub async fn ht_phasor(df: DataFrame) -> PolarsResult<DataFrame> {
    Ok(df)
}

pub async fn ht_sine(df: DataFrame) -> PolarsResult<DataFrame> {
    Ok(df)
}

pub async fn ht_trendmode(df: DataFrame) -> PolarsResult<DataFrame> {
    Ok(df)
}
