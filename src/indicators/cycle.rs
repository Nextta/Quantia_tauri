use polars::prelude::*;

/// Lista de indicadores:
/// HT_DCPERIOD          Hilbert Transform - Dominant Cycle Period
/// HT_DCPHASE           Hilbert Transform - Dominant Cycle Phase
/// HT_PHASOR            Hilbert Transform - Phasor Components
/// HT_SINE              Hilbert Transform - SineWave
/// HT_TRENDMODE         Hilbert Transform - Trend vs Cycle Mode

// ============================================================================
// Helper Functions
// ============================================================================

/// Get close column from DataFrame (case insensitive)
fn get_close(df: &DataFrame) -> PolarsResult<Series> {
    let s = df.column("close").or_else(|_| df.column("Close"))?;
    Ok(s.cast(&DataType::Float64)?.take_materialized_series())
}

/// Hilbert Transform preprocessing state
struct HTState {
    // Smoothing filter
    period_wma_sum: f64,
    period_wma_sub: f64,
    trailing_price: f64,

    // Hilbert Transform buffers (circular buffer approach)
    smoothed: [f64; 6],
    detrender: [f64; 6],
    q1: [f64; 6],
    i1: [f64; 10],

    // Phasor components
    j_i: f64,
    j_q: f64,
    prev_i2: f64,
    prev_q2: f64,
    i2: f64,
    q2: f64,

    // Phase computation
    re: f64,
    im: f64,

    // Period tracking
    period: f64,
    prev_period: f64,
    smooth_period: f64,
    prev_smooth_period: f64,

    // Phase tracking
    dc_phase: f64,
    prev_dc_phase: f64,

    // Hilbert coefficients
    a: f64,
    b: f64,

    // Index tracking
    idx: usize,
    count: usize,
}

impl HTState {
    fn new() -> Self {
        Self {
            period_wma_sum: 0.0,
            period_wma_sub: 0.0,
            trailing_price: 0.0,
            smoothed: [0.0; 6],
            detrender: [0.0; 6],
            q1: [0.0; 6],
            i1: [0.0; 10],
            j_i: 0.0,
            j_q: 0.0,
            prev_i2: 0.0,
            prev_q2: 0.0,
            i2: 0.0,
            q2: 0.0,
            re: 0.0,
            im: 0.0,
            period: 0.0,
            prev_period: 0.0,
            smooth_period: 0.0,
            prev_smooth_period: 0.0,
            dc_phase: 0.0,
            prev_dc_phase: 0.0,
            a: 0.0962,
            b: 0.5769,
            idx: 0,
            count: 0,
        }
    }

    /// Process one price bar through the Hilbert Transform algorithm
    fn process(&mut self, price: f64) -> (f64, f64, f64, f64, f64, f64, i32) {
        self.count += 1;

        // Step 1: Price Smoothing (4-period WMA)
        self.period_wma_sub += price;
        self.period_wma_sub -= self.trailing_price;
        self.period_wma_sum += price * 4.0;

        let smoothed = self.period_wma_sum * 0.1;
        self.period_wma_sum -= self.period_wma_sub;
        self.trailing_price = price;

        // Store smoothed value in circular buffer
        for i in (1..6).rev() {
            self.smoothed[i] = self.smoothed[i - 1];
        }
        self.smoothed[0] = smoothed;

        // Need at least 6 bars for smoothing
        if self.count < 6 {
            return (
                f64::NAN,
                f64::NAN,
                f64::NAN,
                f64::NAN,
                f64::NAN,
                f64::NAN,
                0,
            );
        }

        // Step 2: Hilbert Transform - Detrender
        let detrender = self.a * self.smoothed[0] + self.b * self.smoothed[2]
            - self.a * self.smoothed[4]
            - self.b * self.smoothed[5];

        for i in (1..6).rev() {
            self.detrender[i] = self.detrender[i - 1];
        }
        self.detrender[0] = detrender;

        // Q1 computation
        let q1 = self.a * self.detrender[0] + self.b * self.detrender[2]
            - self.a * self.detrender[4]
            - self.b * self.detrender[5];

        for i in (1..6).rev() {
            self.q1[i] = self.q1[i - 1];
        }
        self.q1[0] = q1;

        // I1 is detrender delayed 3 bars
        if self.count >= 9 {
            for i in (1..10).rev() {
                self.i1[i] = self.i1[i - 1];
            }
            self.i1[0] = self.detrender[0];
        }

        // jI and jQ computation
        if self.count >= 9 {
            self.j_i = self.a * self.i1[3] + self.b * self.i1[5]
                - self.a * self.i1[7]
                - self.b * self.i1[9];

            self.j_q = self.a * self.q1[0] + self.b * self.q1[2]
                - self.a * self.q1[4]
                - self.b * self.q1[5];
        }

        // Step 3: Phasor components (I2, Q2) with 1-pole IIR filter
        if self.count >= 9 {
            self.i2 = 0.2 * (self.i1[0] - self.j_q) + 0.8 * self.prev_i2;
            self.q2 = 0.2 * (self.q1[0] + self.j_i) + 0.8 * self.prev_q2;
            self.prev_i2 = self.i2;
            self.prev_q2 = self.q2;
        }

        // Step 4: Phase and Period computation
        if self.count >= 10 {
            // Smoothed dot and cross products
            self.re = 0.2 * (self.i2 * self.prev_i2 + self.q2 * self.prev_q2) + 0.8 * self.re;
            self.im = 0.2 * (self.i2 * self.prev_q2 - self.q2 * self.prev_i2) + 0.8 * self.im;

            // Ensure im and re are not both zero
            if self.im.abs() < 0.001 {
                self.im = 0.001;
            }
            if self.re.abs() < 0.001 {
                self.re = 0.001;
            }

            // Period from arctangent
            let temp_period = 360.0 / (self.im / self.re).atan().to_degrees().abs();

            // Dynamic bounding: limit change to ±50% of previous period
            let mut bounded_period = if self.prev_period > 0.0 {
                let lower = 0.67 * self.prev_period;
                let upper = 1.5 * self.prev_period;
                temp_period.max(lower).min(upper)
            } else {
                temp_period
            };

            // Hard limits
            bounded_period = bounded_period.max(6.0).min(50.0);

            // Double exponential smoothing
            let period_filtered = 0.2 * bounded_period + 0.8 * self.prev_period;
            self.smooth_period = 0.33 * period_filtered + 0.67 * self.prev_smooth_period;

            self.prev_period = period_filtered;
            self.prev_smooth_period = self.smooth_period;
            self.period = self.smooth_period;

            // DC Phase computation
            let phase_temp = if self.i1[0].abs() > 0.001 {
                (self.q1[0] / self.i1[0]).atan()
            } else {
                0.0
            };

            // Convert to degrees and adjust
            let mut dc_phase = phase_temp.to_degrees() + 90.0;
            if dc_phase < 0.0 {
                dc_phase += 360.0;
            }
            if dc_phase > 360.0 {
                dc_phase -= 360.0;
            }

            // Smooth the phase
            self.dc_phase = 0.33 * dc_phase + 0.67 * self.prev_dc_phase;
            self.prev_dc_phase = self.dc_phase;
        }

        // Step 5: Trend mode detection
        let trend_mode = if self.count >= 10 && self.prev_smooth_period > 0.0 {
            if self.smooth_period > 1.5 * self.prev_smooth_period {
                1 // Trend mode
            } else {
                0 // Cycle mode
            }
        } else {
            0
        };

        // Return: (dc_period, dc_phase, in_phase, quadrature, sine, lead_sine, trend_mode)
        let in_phase = if self.count >= 9 {
            self.i1[0]
        } else {
            f64::NAN
        };
        let quadrature = if self.count >= 9 {
            self.q1[0]
        } else {
            f64::NAN
        };

        let sine = if self.count >= 10 {
            (self.dc_phase.to_radians()).sin()
        } else {
            f64::NAN
        };

        let lead_sine = if self.count >= 10 {
            ((self.dc_phase + 45.0).to_radians()).sin()
        } else {
            f64::NAN
        };

        let dc_period = if self.count >= 10 {
            self.smooth_period
        } else {
            f64::NAN
        };

        let dc_phase_out = if self.count >= 10 {
            self.dc_phase
        } else {
            f64::NAN
        };

        (
            dc_period,
            dc_phase_out,
            in_phase,
            quadrature,
            sine,
            lead_sine,
            trend_mode,
        )
    }
}

// ============================================================================
// HT_DCPERIOD - Hilbert Transform - Dominant Cycle Period
// ============================================================================

/// HT_DCPERIOD - Hilbert Transform - Dominant Cycle Period
///
/// Mide el período del ciclo dominante actual en el mercado usando la Transformada de Hilbert.
/// Este indicador devuelve la longitud del ciclo predominante en barras, útil para ajustar
/// otros indicadores al ciclo actual del mercado.
///
/// Basado en el algoritmo de John Ehlers para procesamiento digital de señales aplicado
/// a series temporales financieras.
///
/// # Parámetros
/// * `df` - DataFrame con columna: close (case insensitive)
/// * `output_col` - Nombre de la columna de salida (default: "ht_dcperiod")
///
/// # Retorna
/// DataFrame con columna "ht_dcperiod" añadida
///
/// # Fórmula
/// Aplica filtro de suavizado WMA(4) → Transformada de Hilbert → Componentes de fase →
/// Período = 360 / |arctan(Im/Re)| → Suavizado doble exponencial
///
/// # Ejemplo
/// ```rust
/// let df_with_period = ht_dcperiod(df, None).await?;
/// ```
pub async fn ht_dcperiod(df: DataFrame, output_col: Option<&str>) -> PolarsResult<DataFrame> {
    let output_col = output_col.unwrap_or("ht_dcperiod");
    let close = get_close(&df)?;

    let close_ca: ChunkedArray<Float64Type> = close.f64().unwrap().clone();
    let close_vals: Vec<f64> = close_ca.into_no_null_iter().collect();

    let n = close_vals.len();
    let mut dc_period_vals: Vec<f64> = vec![f64::NAN; n];

    let mut ht_state = HTState::new();

    for (i, &price) in close_vals.iter().enumerate() {
        let (dc_period, _, _, _, _, _, _) = ht_state.process(price);
        dc_period_vals[i] = dc_period;
    }

    let dc_period_series = Series::new(output_col.into(), &dc_period_vals);
    let mut result_df = df;
    result_df.with_column(dc_period_series.into())?;
    Ok(result_df)
}

// ============================================================================
/// HT_DCPHASE - Hilbert Transform - Dominant Cycle Phase
///
/// Calcula la fase instantánea del ciclo dominante usando la Transformada de Hilbert.
/// La fase representa la posición actual dentro del ciclo del mercado, con valores
/// que oscilan típicamente de 0° a 360°. Útil para identificar la posición en el
/// ciclo actual y anticipar cambios de tendencia.
///
/// Basado en el algoritmo de John Ehlers para procesamiento digital de señales.
///
/// # Parámetros
/// * `df` - DataFrame con columna: close (case insensitive)
/// * `output_col` - Nombre de la columna de salida (default: "ht_dcphase")
///
/// # Retorna
/// DataFrame con columna "ht_dcphase" añadida
///
/// # Fórmula
/// Fase = arctan(Q1/I1) en grados → suavizado exponencial
/// donde I1 y Q1 son los componentes de fase y cuadratura de la Transformada de Hilbert
///
/// # Ejemplo
/// ```rust
/// let df_with_phase = ht_dcphase(df, None).await?;
/// ```
pub async fn ht_dcphase(df: DataFrame, output_col: Option<&str>) -> PolarsResult<DataFrame> {
    let output_col = output_col.unwrap_or("ht_dcphase");
    let close = get_close(&df)?;

    let close_ca: ChunkedArray<Float64Type> = close.f64().unwrap().clone();
    let close_vals: Vec<f64> = close_ca.into_no_null_iter().collect();

    let n = close_vals.len();
    let mut dc_phase_vals: Vec<f64> = vec![f64::NAN; n];

    let mut ht_state = HTState::new();

    for (i, &price) in close_vals.iter().enumerate() {
        let (_, dc_phase, _, _, _, _, _) = ht_state.process(price);
        dc_phase_vals[i] = dc_phase;
    }

    let dc_phase_series = Series::new(output_col.into(), &dc_phase_vals);
    let mut result_df = df;
    result_df.with_column(dc_phase_series.into())?;
    Ok(result_df)
}

// ============================================================================
/// HT_PHASOR - Hilbert Transform - Phasor Components
///
/// Descompone la serie de precios en sus componentes analíticos fundamentales:
/// el componente en fase (In-Phase) y el componente en cuadratura (Quadrature).
/// Estos componentes representan la señal analítica resultante de aplicar la
/// Transformada de Hilbert, donde Quadrature está desfasado 90° respecto a In-Phase.
///
/// Basado en el algoritmo de John Ehlers para procesamiento digital de señales.
///
/// # Parámetros
/// * `df` - DataFrame con columna: close (case insensitive)
/// * `output_col_in_phase` - Nombre de la columna para In-Phase (default: "ht_phasor_inphase")
/// * `output_col_quadrature` - Nombre de la columna para Quadrature (default: "ht_phasor_quadrature")
///
/// # Retorna
/// DataFrame con columnas "ht_phasor_inphase" y "ht_phasor_quadrature" añadidas
///
/// # Fórmula
/// I1 = Detrender retardado 3 barras
/// Q1 = arctan(Q1/I1) aplicado al Detrender
/// donde Detrender es un filtro FIR de Hilbert con coefientes a=0.0962, b=0.5769
///
/// # Ejemplo
/// ```rust
/// let df_with_phasors = ht_phasor(df, None, None).await?;
/// ```
pub async fn ht_phasor(
    df: DataFrame,
    output_col_in_phase: Option<&str>,
    output_col_quadrature: Option<&str>,
) -> PolarsResult<DataFrame> {
    let output_col_in_phase = output_col_in_phase.unwrap_or("ht_phasor_inphase");
    let output_col_quadrature = output_col_quadrature.unwrap_or("ht_phasor_quadrature");
    let close = get_close(&df)?;

    let close_ca: ChunkedArray<Float64Type> = close.f64().unwrap().clone();
    let close_vals: Vec<f64> = close_ca.into_no_null_iter().collect();

    let n = close_vals.len();
    let mut in_phase_vals: Vec<f64> = vec![f64::NAN; n];
    let mut quadrature_vals: Vec<f64> = vec![f64::NAN; n];

    let mut ht_state = HTState::new();

    for (i, &price) in close_vals.iter().enumerate() {
        let (_, _, in_phase, quadrature, _, _, _) = ht_state.process(price);
        in_phase_vals[i] = in_phase;
        quadrature_vals[i] = quadrature;
    }

    let in_phase_series = Series::new(output_col_in_phase.into(), &in_phase_vals);
    let quadrature_series = Series::new(output_col_quadrature.into(), &quadrature_vals);

    let mut result_df = df;
    result_df
        .with_column(in_phase_series.into())?
        .with_column(quadrature_series.into())?;
    Ok(result_df)
}

// ============================================================================
/// HT_SINE - Hilbert Transform - SineWave
///
/// Genera ondas sinusoidales basadas en la fase del ciclo dominante extraída.
/// Produce dos salidas: una onda sinusoidal estándar y una onda sinusoidal líder
/// (desfasada 45°) que ayuda a visualizar los picos y valles del ciclo.
///
/// Basado en el algoritmo de John Ehlers para procesamiento digital de señales.
///
/// # Parámetros
/// * `df` - DataFrame con columna: close (case insensitive)
/// * `output_col_sine` - Nombre de la columna para Sine (default: "ht_sine_sine")
/// * `output_col_lead_sine` - Nombre de la columna para LeadSine (default: "ht_sine_leadsine")
///
/// # Retorna
/// DataFrame con columnas "ht_sine_sine" y "ht_sine_leadsine" añadidas
///
/// # Fórmula
/// Sine = sin(Fase * π/180)
/// LeadSine = sin((Fase + 45°) * π/180)
/// donde Fase es la fase instantánea del ciclo dominante
///
/// # Ejemplo
/// ```rust
/// let df_with_sine = ht_sine(df, None, None).await?;
/// ```
pub async fn ht_sine(
    df: DataFrame,
    output_col_sine: Option<&str>,
    output_col_lead_sine: Option<&str>,
) -> PolarsResult<DataFrame> {
    let output_col_sine = output_col_sine.unwrap_or("ht_sine_sine");
    let output_col_lead_sine = output_col_lead_sine.unwrap_or("ht_sine_leadsine");
    let close = get_close(&df)?;

    let close_ca: ChunkedArray<Float64Type> = close.f64().unwrap().clone();
    let close_vals: Vec<f64> = close_ca.into_no_null_iter().collect();

    let n = close_vals.len();
    let mut sine_vals: Vec<f64> = vec![f64::NAN; n];
    let mut lead_sine_vals: Vec<f64> = vec![f64::NAN; n];

    let mut ht_state = HTState::new();

    for (i, &price) in close_vals.iter().enumerate() {
        let (_, _, _, _, sine, lead_sine, _) = ht_state.process(price);
        sine_vals[i] = sine;
        lead_sine_vals[i] = lead_sine;
    }

    let sine_series = Series::new(output_col_sine.into(), &sine_vals);
    let lead_sine_series = Series::new(output_col_lead_sine.into(), &lead_sine_vals);

    let mut result_df = df;
    result_df
        .with_column(sine_series.into())?
        .with_column(lead_sine_series.into())?;
    Ok(result_df)
}

// ============================================================================
/// HT_TRENDMODE - Hilbert Transform - Trend vs Cycle Mode
///
/// Identifica si el mercado está actualmente en modo tendencia o modo ciclo.
/// Evalúa la suavidad y estabilidad del cambio de fase para determinar el régimen
/// del mercado. Devuelve 1 para modo tendencia y 0 para modo ciclo.
///
/// Este indicador es útil para ajustar estrategias: en modo tendencia, los indicadores
/// basados en ciclos pueden no ser confiables; en modo ciclo, las estrategias de
/// reversión a la media pueden funcionar mejor.
///
/// Basado en el algoritmo de John Ehlers para procesamiento digital de señales.
///
/// # Parámetros
/// * `df` - DataFrame con columna: close (case insensitive)
/// * `output_col` - Nombre de la columna de salida (default: "ht_trendmode")
///
/// # Retorna
/// DataFrame con columna "ht_trendmode" añadida (valores 0 o 1)
///
/// # Fórmula
/// Si PeríodoSuavizado > 1.5 * PeríodoSuavizadoAnterior → 1 (Tendencia)
/// Si no → 0 (Ciclo)
///
/// # Ejemplo
/// ```rust
/// let df_with_trend = ht_trendmode(df, None).await?;
/// ```
pub async fn ht_trendmode(df: DataFrame, output_col: Option<&str>) -> PolarsResult<DataFrame> {
    let output_col = output_col.unwrap_or("ht_trendmode");
    let close = get_close(&df)?;

    let close_ca: ChunkedArray<Float64Type> = close.f64().unwrap().clone();
    let close_vals: Vec<f64> = close_ca.into_no_null_iter().collect();

    let n = close_vals.len();
    let mut trend_mode_vals: Vec<i32> = vec![0; n];

    let mut ht_state = HTState::new();

    for (i, &price) in close_vals.iter().enumerate() {
        let (_, _, _, _, _, _, trend_mode) = ht_state.process(price);
        trend_mode_vals[i] = trend_mode;
    }

    let trend_mode_series = Series::new(output_col.into(), &trend_mode_vals);
    let mut result_df = df;
    result_df.with_column(trend_mode_series.into())?;
    Ok(result_df)
}
