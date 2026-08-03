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
    // Price buffers for smoothing
    price_buf: [f64; 4],

    // Hilbert Transform buffers (7-tap FIR filter needs up to index 6)
    smoothed: [f64; 7],
    detrender: [f64; 7],
    q1: [f64; 7],
    i1: [f64; 7],

    // Phasor components
    prev_i2: f64,
    prev_q2: f64,

    // Beat note (Homodyne) smoothing
    re: f64,
    im: f64,

    // Period tracking
    prev_period: f64,
    prev_smooth_period: f64,

    // Phase tracking
    prev_dc_phase: f64,

    // SineWave tracking for Trend vs Cycle mode
    prev_sine: f64,
    prev_lead_sine: f64,
    bars_since_cross: usize,

    // Index tracking
    count: usize,
}

impl HTState {
    fn new() -> Self {
        Self {
            price_buf: [0.0; 4],
            smoothed: [0.0; 7],
            detrender: [0.0; 7],
            q1: [0.0; 7],
            i1: [0.0; 7],
            prev_i2: 0.0,
            prev_q2: 0.0,
            re: 0.0,
            im: 0.0,
            prev_period: 0.0,
            prev_smooth_period: 0.0,
            prev_dc_phase: 0.0,
            prev_sine: 0.0,
            prev_lead_sine: 0.0,
            bars_since_cross: 0,
            count: 0,
        }
    }

    /// Process one price bar through the Hilbert Transform algorithm
    fn process(&mut self, price: f64) -> (f64, f64, f64, f64, f64, f64, i32) {
        self.count += 1;

        // Shift price buffer
        for i in (1..4).rev() {
            self.price_buf[i] = self.price_buf[i - 1];
        }
        self.price_buf[0] = price;

        // Step 1: Price Smoothing (Ehlers' standard 4-bar WMA)
        let smoothed = (self.price_buf[0]
            + 2.0 * self.price_buf[1]
            + 2.0 * self.price_buf[2]
            + self.price_buf[3])
            / 6.0;

        // Store smoothed value in circular buffer
        for i in (1..7).rev() {
            self.smoothed[i] = self.smoothed[i - 1];
        }
        self.smoothed[0] = smoothed;

        // Initializing variables
        let mut dc_period = f64::NAN;
        let mut dc_phase_out = f64::NAN;
        let mut in_phase = f64::NAN;
        let mut quadrature = f64::NAN;
        let mut sine = f64::NAN;
        let mut lead_sine = f64::NAN;
        let mut trend_mode = 0;

        if self.count < 6 {
            return (
                dc_period,
                dc_phase_out,
                in_phase,
                quadrature,
                sine,
                lead_sine,
                trend_mode,
            );
        }

        // Step 2: Hilbert Transform - Detrender
        // Coefficients: 0.0962, 0.5769, -0.5769, -0.0962 at lags 0, 2, 4, 6
        let mult = 0.075 * self.prev_period + 0.54;
        let detrender = (0.0962 * self.smoothed[0] + 0.5769 * self.smoothed[2]
            - 0.5769 * self.smoothed[4]
            - 0.0962 * self.smoothed[6])
            * mult;

        for i in (1..7).rev() {
            self.detrender[i] = self.detrender[i - 1];
        }
        self.detrender[0] = detrender;

        // Q1 (Quadrature) computation
        let q1 = (0.0962 * self.detrender[0] + 0.5769 * self.detrender[2]
            - 0.5769 * self.detrender[4]
            - 0.0962 * self.detrender[6])
            * mult;

        for i in (1..7).rev() {
            self.q1[i] = self.q1[i - 1];
        }
        self.q1[0] = q1;

        // I1 (In-Phase) is detrender delayed 3 bars
        let i1 = self.detrender[3];
        for i in (1..7).rev() {
            self.i1[i] = self.i1[i - 1];
        }
        self.i1[0] = i1;

        // Step 3: Complex components (jI and jQ) to refine phasor
        let j_i =
            (0.0962 * self.i1[0] + 0.5769 * self.i1[2] - 0.5769 * self.i1[4] - 0.0962 * self.i1[6])
                * mult;
        let j_q =
            (0.0962 * self.q1[0] + 0.5769 * self.q1[2] - 0.5769 * self.q1[4] - 0.0962 * self.q1[6])
                * mult;

        // Step 4: Phasor components (I2, Q2) with smoothing
        let i2 = i1 - j_q;
        let q2 = q1 + j_i;

        let smoothed_i2 = 0.2 * i2 + 0.8 * self.prev_i2;
        let smoothed_q2 = 0.2 * q2 + 0.8 * self.prev_q2;

        // Step 5: Homodyne Discriminator (Phase Change)
        // Re = I2*prev_I2 + Q2*prev_Q2
        // Im = I2*prev_Q2 - Q2*prev_I2
        let re = smoothed_i2 * self.prev_i2 + smoothed_q2 * self.prev_q2;
        let im = smoothed_i2 * self.prev_q2 - smoothed_q2 * self.prev_i2;

        self.re = 0.2 * re + 0.8 * self.re;
        self.im = 0.2 * im + 0.8 * self.im;

        // Store for next bar
        self.prev_i2 = smoothed_i2;
        self.prev_q2 = smoothed_q2;

        // Step 6: Period Calculation
        let mut period = if self.im != 0.0 && self.re != 0.0 {
            360.0 / (self.im / self.re).atan().to_degrees()
        } else {
            self.prev_period
        };

        // Rate limiting: clamp to ±50% of previous period
        if self.prev_period > 0.0 {
            period = period
                .max(0.67 * self.prev_period)
                .min(1.5 * self.prev_period);
        }

        // Range limiting
        period = period.max(6.0).min(50.0);

        // Final smoothing (Double smoothing)
        let filtered_period = 0.2 * period + 0.8 * self.prev_period;
        let smooth_period = 0.33 * filtered_period + 0.67 * self.prev_smooth_period;

        self.prev_period = filtered_period;
        self.prev_smooth_period = smooth_period;

        // Step 7: DC Phase computation (Stable Accumulation)
        let mut dc_phase = if i1.abs() > 0.0 {
            (q1 / i1).atan().to_degrees()
        } else {
            0.0
        };
        dc_phase += 90.0;

        // Correct for wrap-around of raw phase
        if dc_phase < 0.0 {
            dc_phase += 360.0;
        }
        if dc_phase > 360.0 {
            dc_phase -= 360.0;
        }

        // Calculate Delta Phase (change in phase)
        let mut delta_phase = self.prev_dc_phase - dc_phase;
        if self.prev_dc_phase < dc_phase {
            delta_phase = 360.0 + self.prev_dc_phase - dc_phase;
        }

        // Clip delta phase to a reasonable range based on the dominant period
        // If period is 20, expected delta is 360/20 = 18 degrees
        if delta_phase < 1.0 {
            delta_phase = 1.0;
        }

        // Update the stable accumulated phase
        let mut smooth_phase = self.prev_dc_phase - delta_phase;
        if smooth_phase < 0.0 {
            smooth_phase += 360.0;
        }

        // Final smoothing
        smooth_phase = 0.33 * smooth_phase + 0.67 * self.prev_dc_phase;
        self.prev_dc_phase = smooth_phase;

        // Step 8: Trend mode detection (Sine Wave Crossings + Period Stability)
        let sine_val = (smooth_phase.to_radians()).sin();
        let lead_sine_val = ((smooth_phase + 45.0).to_radians()).sin();

        // Detect crossing: if current diff and previous diff have different signs
        let cross =
            (sine_val - lead_sine_val).signum() != (self.prev_sine - self.prev_lead_sine).signum();

        if cross {
            self.bars_since_cross = 0;
        } else {
            self.bars_since_cross += 1;
        }

        self.prev_sine = sine_val;
        self.prev_lead_sine = lead_sine_val;

        if self.count >= 12 {
            // Trend if:
            // 1. No Sine/LeadSine cross for more than 50% of the period
            // 2. OR Period has jumped significantly (>50%)
            if self.bars_since_cross as f64 > 0.6 * smooth_period
                || smooth_period > 1.5 * self.prev_smooth_period
            {
                trend_mode = 1;
            }
        }

        // Output formatting
        if self.count >= 30 {
            dc_period = smooth_period;
            dc_phase_out = smooth_phase;
            in_phase = smoothed_i2;
            quadrature = smoothed_q2;
            sine = sine_val;
            lead_sine = lead_sine_val;
        }

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
/// let df_with_period = ht_dcperiod(df, None);
/// ```
pub fn ht_dcperiod(df: &mut DataFrame, output_col: Option<&str>) {
    let output_col = output_col.unwrap_or("ht_dcperiod");
    let close = get_close(&df).unwrap();

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
    df.with_column(dc_period_series.into()).unwrap();
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
/// let df_with_phase = ht_dcphase(df, None);
/// ```
pub fn ht_dcphase(df: &mut DataFrame, output_col: Option<&str>) {
    let output_col = output_col.unwrap_or("ht_dcphase");
    let close = get_close(&df).unwrap();

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
    df.with_column(dc_phase_series.into()).unwrap();
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
/// let df_with_phasors = ht_phasor(df, None, None);
/// ```
pub fn ht_phasor(
    df: &mut DataFrame,
    output_col_in_phase: Option<&str>,
    output_col_quadrature: Option<&str>,
) {
    let output_col_in_phase = output_col_in_phase.unwrap_or("ht_phasor_inphase");
    let output_col_quadrature = output_col_quadrature.unwrap_or("ht_phasor_quadrature");
    let close = get_close(&df).unwrap();

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

    df.with_column(in_phase_series.into())
        .unwrap()
        .with_column(quadrature_series.into())
        .unwrap();
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
/// let df_with_sine = ht_sine(df, None, None);
/// ```
pub fn ht_sine(
    df: &mut DataFrame,
    output_col_sine: Option<&str>,
    output_col_lead_sine: Option<&str>,
) {
    let output_col_sine = output_col_sine.unwrap_or("ht_sine_sine");
    let output_col_lead_sine = output_col_lead_sine.unwrap_or("ht_sine_leadsine");
    let close = get_close(&df).unwrap();

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

    df.with_column(sine_series.into())
        .unwrap()
        .with_column(lead_sine_series.into())
        .unwrap();
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
/// let df_with_trend = ht_trendmode(df, None);
/// ```
pub fn ht_trendmode(df: &mut DataFrame, output_col: Option<&str>) {
    let output_col = output_col.unwrap_or("ht_trendmode");
    let close = get_close(&df).unwrap();

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
    df.with_column(trend_mode_series.into()).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::data_format::DataFormatSymbol;
    use crate::utils::data_test::create_test_data;
    use std::fs::remove_file;

    //Para los test crear una carpeta llamada download en la raiz de este proyecto
    // y llamar a los datos test.csv
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
    fn test_ht_dcperiod() {
        match load_data() {
            Ok(mut df) => {
                ht_dcperiod(&mut df, None);
                // save_data(&df, "download/test_ht_dcperiod.csv").unwrap();
                // remove_file("download/test_ht_dcperiod.csv").unwrap();
                remove_file("download/test.csv").unwrap();
                remove_file("download/test.parquet").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[test]
    fn test_ht_dcphase() {
        match load_data() {
            Ok(mut df) => {
                ht_dcphase(&mut df, None);
                // save_data(&df, "download/test_ht_dcphase.csv").unwrap();
                // remove_file("download/test_ht_dcphase.csv").unwrap();
                remove_file("download/test.csv").unwrap();
                remove_file("download/test.parquet").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[test]
    fn test_ht_phasor() {
        match load_data() {
            Ok(mut df) => {
                ht_phasor(&mut df, None, None);
                // save_data(&df, "download/test_ht_phasor.csv").unwrap();
                // remove_file("download/test_ht_phasor.csv").unwrap();
                remove_file("download/test.csv").unwrap();
                remove_file("download/test.parquet").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[test]
    fn test_ht_sine() {
        match load_data() {
            Ok(mut df) => {
                ht_sine(&mut df, None, None);
                // save_data(&df, "download/test_ht_sine.csv").unwrap();
                // remove_file("download/test_ht_sine.csv").unwrap();
                remove_file("download/test.csv").unwrap();
                remove_file("download/test.parquet").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }

    #[test]
    fn test_ht_trendmode() {
        match load_data() {
            Ok(mut df) => {
                ht_trendmode(&mut df, None);
                // save_data(&df, "download/test_ht_trendmode.csv").unwrap();
                // remove_file("download/test_ht_trendmode.csv").unwrap();
                remove_file("download/test.csv").unwrap();
                remove_file("download/test.parquet").unwrap();
            }
            Err(e) => panic!("Failed to load data: {:?}", e),
        }
    }
}
