use crate::api::resultados::{insert_resultados, table_resultados};
use crate::backtest::trade::Trade;
use crate::structs::logs::RegistroLog;
use crate::utils::configuracion::LOGS_REGISTRO;
use chrono::{Datelike, NaiveDateTime}; // Utc, Month, DateTime
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Resultados {
    pub id: i32,
    pub id_backtest: i32,
    pub retorno: f64,
    pub return_percent: f64,
    pub cagr: f64,
    pub sharpe_ratio: f64,
    pub sortino_ratio: f64,
    pub omega_ratio: f64,
    pub expected_daily: f64,
    pub expected_monthly: f64,
    pub expected_yearly: f64,
    pub best_day: f64,
    pub worst_day: f64,
    pub best_month: f64,
    pub worst_month: f64,
    pub best_year: f64,
    pub worst_year: f64,
    pub time_in_market: f64,
    pub max_drawdown: f64,
    pub max_drawdown_divisa: f64,
    pub max_drawdown_duration: u64,
    pub avg_drawdown_duration: f64,
    pub max_drawdown_avg: f64,
    pub avg_drawdown: f64,
    pub ulcer_index: f64,
    pub serenity_index: f64,
    pub daily_var: f64,
    pub daily_var_95: f64,
    pub daily_var_99: f64,
    pub cvar: f64,
    pub risk_of_ruin: f64,
    pub volatility_ann: f64,
    pub calmar_ratio: f64,
    pub skew_ratio: f64,
    pub kurtosis_ratio: f64,
    pub tail_ratio: f64,
    pub outlier_win: f64,
    pub outlier_loss: f64,
    pub payoff_ratio: f64,
    pub profit_factor: f64,
    pub gain_pain_ratio: f64,
    pub common_sense_ratio: f64,
    pub cpc_index: f64,
    pub kelly_criterion: f64,
    pub win_days: f64,
    pub win_months: f64,
    pub win_quarters: f64,
    pub win_years: f64,
    pub beta: f64,
    pub alpha: f64,
    pub correlation: f64,
    pub information_ratio: f64,
    pub recovery_factor: f64,
    pub n_trades: u64,
    pub return_drawdown_ratio: f64,
    pub wins_percentage: f64,
    pub avg_trade_return: f64,
    pub avg_win_return: f64,
    pub avg_loss_return: f64,
    pub avg_win_loss_ratio: f64,
    pub r_expectancy: f64,
    pub r_exp_score: f64,
    pub z_score: f64,
    pub z_probability: f64,
    pub n_wins: u64,
    pub n_losses: u64,
    pub avg_bars_win: f64,
    pub avg_bars_loss: f64,
}

impl Resultados {
    pub async fn new(id_backtest: i32) -> Self {
        let tabla = table_resultados().await;

        match tabla {
            Ok(_) => {}
            Err(e) => {
                println!("Error al obtener tabla de resultados: {:?}", e);
            }
        }

        Self {
            id: 0,
            id_backtest,
            retorno: 0.0,
            return_percent: 0.0,
            cagr: 0.0,
            sharpe_ratio: 0.0,
            sortino_ratio: 0.0,
            omega_ratio: 0.0,
            expected_daily: 0.0,
            expected_monthly: 0.0,
            expected_yearly: 0.0,
            best_day: 0.0,
            worst_day: 0.0,
            best_month: 0.0,
            worst_month: 0.0,
            best_year: 0.0,
            worst_year: 0.0,
            time_in_market: 0.0,
            max_drawdown: 0.0,
            max_drawdown_divisa: 0.0,
            max_drawdown_duration: 0,
            avg_drawdown_duration: 0.0,
            max_drawdown_avg: 0.0, // Eliminar este
            avg_drawdown: 0.0,
            ulcer_index: 0.0,
            serenity_index: 0.0,
            daily_var: 0.0,
            daily_var_95: 0.0,
            daily_var_99: 0.0,
            cvar: 0.0,
            risk_of_ruin: 0.0,
            volatility_ann: 0.0,
            calmar_ratio: 0.0,
            skew_ratio: 0.0,
            kurtosis_ratio: 0.0,
            tail_ratio: 0.0,
            outlier_win: 0.0,
            outlier_loss: 0.0,
            payoff_ratio: 0.0,
            profit_factor: 0.0,
            gain_pain_ratio: 0.0,
            common_sense_ratio: 0.0,
            cpc_index: 0.0,
            kelly_criterion: 0.0,
            win_days: 0.0,
            win_months: 0.0,
            win_quarters: 0.0,
            win_years: 0.0,
            beta: 0.0,
            alpha: 0.0,
            correlation: 0.0,
            information_ratio: 0.0,
            recovery_factor: 0.0,
            n_trades: 0,
            return_drawdown_ratio: 0.0,
            wins_percentage: 0.0,
            avg_trade_return: 0.0,
            avg_win_return: 0.0,
            avg_loss_return: 0.0,
            avg_win_loss_ratio: 0.0,
            r_expectancy: 0.0,
            r_exp_score: 0.0,
            z_score: 0.0,
            z_probability: 0.0,
            n_wins: 0,
            n_losses: 0,
            avg_bars_win: 0.0,
            avg_bars_loss: 0.0,
        }
    }

    pub fn calcular_resultados(&mut self, trades: Vec<Trade>, capital_inicial: f64) {
        if trades.is_empty() {
            return;
        }

        let n_trades = trades.len();

        // ─────────────────────────────────────────────────────────────
        // ACUMULADORES — se rellenan en el for loop
        // ─────────────────────────────────────────────────────────────

        // Equity y drawdown
        let mut equity = capital_inicial;
        let mut peak = capital_inicial;
        let mut squared_drawdowns: Vec<f64> = Vec::new();
        let mut all_drawdowns_pct: Vec<f64> = Vec::new(); // para avg_drawdown
        let mut in_drawdown = false;
        let mut current_dd_low = capital_inicial;
        let mut dd_start_time: Option<NaiveDateTime> = None;
        let mut dd_durations: Vec<i64> = Vec::new(); // segundos

        // Retornos
        let mut returns: Vec<f64> = Vec::new(); // retorno % por trade
        let mut capital_actual = capital_inicial;
        let mut total_pl = 0.0_f64;

        // Wins / losses
        let mut n_wins = 0u64;
        let mut n_losses = 0u64;
        let mut win_pls: Vec<f64> = Vec::new();
        let mut loss_pls: Vec<f64> = Vec::new();
        let mut win_returns: Vec<f64> = Vec::new();
        let mut loss_returns: Vec<f64> = Vec::new();
        let mut win_bars: Vec<u64> = Vec::new();
        let mut loss_bars: Vec<u64> = Vec::new();

        // Tiempo en mercado
        let mut time_in_market_secs = 0i64;
        let backtest_start =
            NaiveDateTime::parse_from_str(&trades[0].t0, "%Y-%m-%d %H:%M:%S").unwrap();
        let backtest_end =
            NaiveDateTime::parse_from_str(&trades[n_trades - 1].t1, "%Y-%m-%d %H:%M:%S").unwrap();
        let backtest_duration_secs = (backtest_end - backtest_start).num_seconds();

        // Mejor / peor día, mes, año (en divisa)
        let mut day_buckets: std::collections::HashMap<String, f64> =
            std::collections::HashMap::new();
        let mut month_buckets: std::collections::HashMap<String, f64> =
            std::collections::HashMap::new();
        let mut quarter_buckets: std::collections::HashMap<String, f64> =
            std::collections::HashMap::new();
        let mut year_buckets: std::collections::HashMap<String, f64> =
            std::collections::HashMap::new();

        // Omega / gain-pain
        let mut sum_gains = 0.0_f64;
        let mut sum_losses_abs = 0.0_f64;

        // ─────────────────────────────────────────────────────────────
        // FOR LOOP PRINCIPAL — un único recorrido
        // ─────────────────────────────────────────────────────────────
        for trade in &trades {
            let pl = trade.pl;
            let t0 = NaiveDateTime::parse_from_str(&trade.t0, "%Y-%m-%d %H:%M:%S").unwrap();
            let t1 = NaiveDateTime::parse_from_str(&trade.t1, "%Y-%m-%d %H:%M:%S").unwrap();
            let bars = (t1 - t0).num_seconds().unsigned_abs(); // duración en segundos como "bars"

            // ── Retorno porcentual del trade ──────────────────────────
            let ret = pl / capital_actual;
            returns.push(ret);
            capital_actual += pl;
            total_pl += pl;

            // ── Wins / losses ─────────────────────────────────────────
            if trade.label == 1 {
                n_wins += 1;
                win_pls.push(pl);
                win_returns.push(ret);
                win_bars.push(bars);
                sum_gains += ret;
            } else if trade.label == 0 {
                n_losses += 1;
                loss_pls.push(pl);
                loss_returns.push(ret.abs());
                loss_bars.push(bars);
                sum_losses_abs += ret.abs();
            }

            // ── Equity y drawdown ─────────────────────────────────────
            equity += pl;

            if equity > peak {
                // Nuevo pico — cerrar drawdown si estábamos en uno
                if in_drawdown {
                    let dd_pct = (current_dd_low - peak) / peak * 100.0;
                    all_drawdowns_pct.push(dd_pct);

                    if let Some(start) = dd_start_time {
                        dd_durations.push((t1 - start).num_seconds());
                    }

                    in_drawdown = false;
                    dd_start_time = None;
                }
                peak = equity;
                current_dd_low = equity;
            } else {
                // Seguimos en drawdown
                if !in_drawdown {
                    in_drawdown = true;
                    dd_start_time = Some(t0);
                }
                if equity < current_dd_low {
                    current_dd_low = equity;
                }
            }

            // Drawdown puntual (para Ulcer Index y max DD)
            let dd_pct = (equity - peak) / peak * 100.0;
            let dd_divisa = equity - peak;
            squared_drawdowns.push(dd_pct.powi(2));

            if dd_divisa < self.max_drawdown_divisa {
                self.max_drawdown_divisa = dd_divisa;
            }
            if dd_pct < self.max_drawdown {
                self.max_drawdown = dd_pct;
            }

            // ── Tiempo en mercado ─────────────────────────────────────
            time_in_market_secs += (t1 - t0).num_seconds();

            // ── Buckets temporales (día, mes, trimestre, año) ─────────
            let day_key = t1.format("%Y-%m-%d").to_string();
            let month_key = t1.format("%Y-%m").to_string();
            let quarter_key = format!("{}-Q{}", t1.year(), (t1.month() - 1) / 3 + 1);
            let year_key = t1.format("%Y").to_string();

            *day_buckets.entry(day_key).or_insert(0.0) += pl;
            *month_buckets.entry(month_key).or_insert(0.0) += pl;
            *quarter_buckets.entry(quarter_key).or_insert(0.0) += pl;
            *year_buckets.entry(year_key).or_insert(0.0) += pl;
        }

        // Cerrar drawdown activo al final del backtest
        if in_drawdown {
            let dd_pct = (current_dd_low - peak) / peak * 100.0;
            all_drawdowns_pct.push(dd_pct);
            if let Some(start) = dd_start_time {
                dd_durations.push((backtest_end - start).num_seconds());
            }
        }

        // ─────────────────────────────────────────────────────────────
        // MÉTRICAS DERIVADAS (post loop)
        // ─────────────────────────────────────────────────────────────

        let n = n_trades as f64;
        let n_wins_f = n_wins as f64;
        let n_losses_f = n_losses as f64;

        // ── Retornos básicos ──────────────────────────────────────────
        self.retorno = total_pl;
        self.return_percent = (total_pl / capital_inicial) * 100.0;
        self.n_trades = n_trades as u64;
        self.n_wins = n_wins;
        self.n_losses = n_losses;

        // ── CAGR ──────────────────────────────────────────────────────
        self.cagr = self.calcular_carg(
            trades[0].t0.clone(),
            trades[n_trades - 1].t1.clone(),
            capital_inicial,
        );

        // ── Tiempo en mercado ─────────────────────────────────────────
        self.time_in_market = if backtest_duration_secs > 0 {
            (time_in_market_secs as f64 / backtest_duration_secs as f64) * 100.0
        } else {
            0.0
        };

        // ── Drawdown ──────────────────────────────────────────────────
        let n_dd = squared_drawdowns.len() as f64;
        self.ulcer_index = (squared_drawdowns.iter().sum::<f64>() / n_dd).sqrt();

        self.avg_drawdown = if !all_drawdowns_pct.is_empty() {
            all_drawdowns_pct.iter().sum::<f64>() / all_drawdowns_pct.len() as f64
        } else {
            0.0
        };

        self.max_drawdown_duration = dd_durations.iter().cloned().max().unwrap_or(0) as u64;
        self.avg_drawdown_duration = if !dd_durations.is_empty() {
            dd_durations.iter().sum::<i64>() as f64 / dd_durations.len() as f64
        } else {
            0.0
        };

        // ── Estadísticas de retornos ──────────────────────────────────
        let mean_ret = returns.iter().sum::<f64>() / n;

        let variance = returns.iter().map(|r| (r - mean_ret).powi(2)).sum::<f64>() / (n - 1.0);
        let std_dev = variance.sqrt();

        // Volatilidad anualizada
        self.volatility_ann = std_dev * 252.0_f64.sqrt() * 100.0;

        // ── Skewness ──────────────────────────────────────────────────
        self.skew_ratio = if std_dev > 0.0 {
            returns
                .iter()
                .map(|r| ((r - mean_ret) / std_dev).powi(3))
                .sum::<f64>()
                / n
        } else {
            0.0
        };

        // ── Kurtosis (excess) ─────────────────────────────────────────
        self.kurtosis_ratio = if std_dev > 0.0 {
            returns
                .iter()
                .map(|r| ((r - mean_ret) / std_dev).powi(4))
                .sum::<f64>()
                / n
                - 3.0
        } else {
            0.0
        };

        if self.kurtosis_ratio.is_nan() {
            self.kurtosis_ratio = 0.0;
        }

        // ── Sharpe Ratio ──────────────────────────────────────────────
        self.sharpe_ratio = if std_dev > 0.0 {
            (mean_ret / std_dev) * 252.0_f64.sqrt()
        } else {
            0.0
        };

        // ── Sortino Ratio ─────────────────────────────────────────────
        let downside_var = returns
            .iter()
            .filter(|&&r| r < 0.0)
            .map(|&r| r.powi(2))
            .sum::<f64>()
            / n;
        let downside_std = downside_var.sqrt();
        self.sortino_ratio = if downside_std > 0.0 {
            (mean_ret / downside_std) * 252.0_f64.sqrt()
        } else {
            f64::INFINITY
        };

        // ── Calmar Ratio ──────────────────────────────────────────────
        self.calmar_ratio = if self.max_drawdown != 0.0 {
            (self.cagr * 100.0) / self.max_drawdown.abs()
        } else {
            f64::INFINITY
        };

        // ── Omega Ratio ───────────────────────────────────────────────
        self.omega_ratio = if sum_losses_abs > 0.0 {
            sum_gains / sum_losses_abs
        } else {
            f64::INFINITY
        };

        // ── Profit Factor ─────────────────────────────────────────────
        let gross_profit = win_pls.iter().sum::<f64>();
        let gross_loss = loss_pls.iter().map(|x| x.abs()).sum::<f64>();
        self.profit_factor = if gross_loss > 0.0 {
            gross_profit / gross_loss
        } else {
            f64::INFINITY
        };

        // ── Win Rate ──────────────────────────────────────────────────
        self.wins_percentage = (n_wins_f / n) * 100.0;

        // ── Payoff Ratio (avg win / avg loss) ─────────────────────────
        let avg_win = if n_wins > 0 {
            gross_profit / n_wins_f
        } else {
            0.0
        };
        let avg_loss = if n_losses > 0 {
            gross_loss / n_losses_f
        } else {
            0.0
        };
        self.payoff_ratio = if avg_loss > 0.0 {
            avg_win / avg_loss
        } else {
            f64::INFINITY
        };
        self.avg_trade_return = mean_ret * 100.0;
        self.avg_win_return = if !win_returns.is_empty() {
            win_returns.iter().sum::<f64>() / win_returns.len() as f64 * 100.0
        } else {
            0.0
        };
        self.avg_loss_return = if !loss_returns.is_empty() {
            loss_returns.iter().sum::<f64>() / loss_returns.len() as f64 * 100.0
        } else {
            0.0
        };
        self.avg_win_loss_ratio = if self.avg_loss_return != 0.0 {
            self.avg_win_return / self.avg_loss_return
        } else {
            f64::INFINITY
        };

        // ── Avg bars en wins y losses ─────────────────────────────────
        self.avg_bars_win = if !win_bars.is_empty() {
            win_bars.iter().sum::<u64>() as f64 / win_bars.len() as f64
        } else {
            0.0
        };
        self.avg_bars_loss = if !loss_bars.is_empty() {
            loss_bars.iter().sum::<u64>() as f64 / loss_bars.len() as f64
        } else {
            0.0
        };

        // ── Expected Returns ──────────────────────────────────────────
        self.expected_daily = mean_ret * 100.0;
        self.expected_monthly = mean_ret * 21.0 * 100.0;
        self.expected_yearly = mean_ret * 252.0 * 100.0;

        // ── R-Expectancy  E[R] = (WR × avg_win_R) - (LR × avg_loss_R) ─
        let win_rate = n_wins_f / n;
        let loss_rate = n_losses_f / n;
        // Normalizado a R (1R = avg loss)
        let avg_win_r = if avg_loss > 0.0 {
            avg_win / avg_loss
        } else {
            0.0
        };
        let avg_loss_r = 1.0;
        self.r_expectancy = (win_rate * avg_win_r) - (loss_rate * avg_loss_r);
        // Score: expectancy ponderada por número de trades
        self.r_exp_score = self.r_expectancy * n.sqrt();

        // ── Kelly Criterion ───────────────────────────────────────────
        // f = W - (1-W)/R  donde R = avg_win / avg_loss
        self.kelly_criterion = if self.payoff_ratio > 0.0 {
            win_rate - (loss_rate / self.payoff_ratio)
        } else {
            0.0
        };

        // ── Gain Pain Ratio  (sum_gains - sum_losses) / sum_losses ────
        self.gain_pain_ratio = if sum_losses_abs > 0.0 {
            (sum_gains - sum_losses_abs) / sum_losses_abs
        } else {
            f64::INFINITY
        };

        // ── Tail Ratio  (percentil 95 / |percentil 5|) ───────────────
        {
            let mut sorted = returns.clone();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let idx_5 = ((0.05 * n) as usize).max(0);
            let idx_95 = ((0.95 * n) as usize).min(n_trades - 1);
            let p5 = sorted[idx_5].abs();
            let p95 = sorted[idx_95];
            self.tail_ratio = if p5 > 0.0 { p95 / p5 } else { f64::INFINITY };

            // Outlier win/loss: ratio del retorno más extremo vs percentil 95/5
            self.outlier_win = if p95 > 0.0 {
                sorted[n_trades - 1] / p95
            } else {
                0.0
            };
            self.outlier_loss = if p5 > 0.0 { sorted[0].abs() / p5 } else { 0.0 };
        }

        // ── Common Sense Ratio  = Profit Factor × Tail Ratio ─────────
        self.common_sense_ratio = self.profit_factor * self.tail_ratio;

        // ── CPC Index  = Profit Factor × Win Rate × Payoff Ratio ─────
        self.cpc_index = self.profit_factor * win_rate * self.payoff_ratio;

        // ── Daily VaR (paramétrico 95% y 99%) ────────────────────────
        let z_95 = 1.645_f64;
        let z_99 = 2.326_f64;
        self.daily_var_95 = (mean_ret - z_95 * std_dev) * capital_inicial;
        self.daily_var_99 = (mean_ret - z_99 * std_dev) * capital_inicial;

        // ── CVaR / Expected Shortfall 95% ────────────────────────────
        {
            let mut sorted = returns.clone();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let cutoff = ((1.0 - 0.95) * n) as usize;
            let tail = &sorted[..cutoff.max(1)];
            let cvar_pct = tail.iter().sum::<f64>() / tail.len() as f64;
            self.cvar = cvar_pct * capital_inicial;

            if self.cvar.is_nan() {
                self.cvar = 0.0;
            }
        }

        // ── Ulcer Index ya calculado — Serenity Index ─────────────────
        // Serenity = Martin Ratio × factor de penalización por distribución
        let martin_ratio = if self.ulcer_index > 0.0 {
            (self.cagr * 100.0) / self.ulcer_index
        } else {
            0.0
        };
        let penalty = 1.0 / (1.0 + (self.skew_ratio.powi(2) + self.kurtosis_ratio) / 4.0);
        self.serenity_index = martin_ratio * penalty;

        // ── Z-Score  (autocorrelación de rachas) ──────────────────────
        // Z = (N × (N_series - (2×W×L / N))) / sqrt(2×W×L×(2×W×L - N) / (N²×(N-1)))
        // N_series = número de cambios de signo + 1
        let n_series = returns
            .windows(2)
            .filter(|w| (w[0] >= 0.0) != (w[1] >= 0.0))
            .count() as f64
            + 1.0;
        let wl2 = 2.0 * n_wins_f * n_losses_f;
        let z_num = n * (n_series - (wl2 / n + 1.0));
        let z_den_sq = (wl2 * (wl2 - n)) / (n * n * (n - 1.0));
        self.z_score = if z_den_sq > 0.0 {
            z_num / z_den_sq.sqrt()
        } else {
            0.0
        };
        // Probabilidad aproximada (distribución normal estándar, two-tail)
        self.z_probability = 2.0 * (1.0 - self.normal_cdf(self.z_score.abs()));

        // ── Win días, meses, trimestres, años ─────────────────────────
        self.win_days = day_buckets.values().filter(|&&v| v > 0.0).count() as f64;
        self.win_months = month_buckets.values().filter(|&&v| v > 0.0).count() as f64;
        self.win_quarters = quarter_buckets.values().filter(|&&v| v > 0.0).count() as f64;
        self.win_years = year_buckets.values().filter(|&&v| v > 0.0).count() as f64;

        // Mejor y peor período
        self.best_day = day_buckets
            .values()
            .cloned()
            .fold(f64::NEG_INFINITY, f64::max);
        self.worst_day = day_buckets.values().cloned().fold(f64::INFINITY, f64::min);
        self.best_month = month_buckets
            .values()
            .cloned()
            .fold(f64::NEG_INFINITY, f64::max);
        self.worst_month = month_buckets
            .values()
            .cloned()
            .fold(f64::INFINITY, f64::min);
        self.best_year = year_buckets
            .values()
            .cloned()
            .fold(f64::NEG_INFINITY, f64::max);
        self.worst_year = year_buckets.values().cloned().fold(f64::INFINITY, f64::min);

        // ── Recovery Factor  = total_pl / |max_drawdown_divisa| ───────
        self.recovery_factor = if self.max_drawdown_divisa != 0.0 {
            total_pl / self.max_drawdown_divisa.abs()
        } else {
            f64::INFINITY
        };

        // ── Return / Drawdown Ratio  = return_percent / |max_drawdown| ─
        self.return_drawdown_ratio = if self.max_drawdown != 0.0 {
            self.return_percent / self.max_drawdown.abs()
        } else {
            f64::INFINITY
        };

        // ── Beta, Alpha, Correlation, Information Ratio ───────────────
        // Requieren benchmark returns. Si no tienes benchmark, se dejan en 0.
        // Descomenta y pasa `benchmark_returns: Vec<f64>` como parámetro cuando lo tengas.
        //
        // let (beta, alpha, corr) = calcular_beta_alpha_corr(&returns, &benchmark_returns, mean_ret);
        // self.beta        = beta;
        // self.alpha       = alpha;
        // self.correlation = corr;
        //
        // let tracking_error = calcular_tracking_error(&returns, &benchmark_returns);
        // let excess_returns = mean_ret - benchmark_mean;
        // self.information_ratio = if tracking_error > 0.0 {
        //     (excess_returns / tracking_error) * 252.0_f64.sqrt()
        // } else { 0.0 };
        self.beta = 0.0;
        self.alpha = 0.0;
        self.correlation = 0.0;
        self.information_ratio = 0.0;

        // ── Risk of Ruin (fórmula analítica) ──────────────────────────
        let edge = (win_rate * avg_win) - (loss_rate * avg_loss);
        self.risk_of_ruin = if edge <= 0.0 {
            100.0
        } else {
            let avg_risk = (win_rate * avg_win + loss_rate * avg_loss) / 2.0;
            let ruin_threshold = 0.5; // 50% del capital, ajusta si quieres
            if avg_risk > 0.0 {
                ((1.0 - edge) / (1.0 + edge)).powf(ruin_threshold / avg_risk) * 100.0
            } else {
                100.0
            }
        };

        if self.risk_of_ruin.is_nan() {
            self.risk_of_ruin = 0.0;
        }
    }

    // ─────────────────────────────────────────────────────────────────────────────
    // HELPER — CDF normal estándar (aproximación de Abramowitz & Stegun)
    // Necesaria para z_probability
    // ─────────────────────────────────────────────────────────────────────────────
    fn normal_cdf(&self, x: f64) -> f64 {
        let t = 1.0 / (1.0 + 0.2316419 * x.abs());
        let poly = t
            * (0.319381530
                + t * (-0.356563782 + t * (1.781477937 + t * (-1.821255978 + t * 1.330274429))));
        let pdf = (-x * x / 2.0).exp() / (2.0 * std::f64::consts::PI).sqrt();
        let cdf = 1.0 - pdf * poly;
        if x >= 0.0 {
            cdf
        } else {
            1.0 - cdf
        }
    }

    fn calcular_carg(&self, t0: String, t1: String, capital_inicial: f64) -> f64 {
        let date_time_t0 = NaiveDateTime::parse_from_str(&t0, "%Y-%m-%d %H:%M:%S").unwrap();
        let date_time_t1 = NaiveDateTime::parse_from_str(&t1, "%Y-%m-%d %H:%M:%S").unwrap();
        let duration = date_time_t1 - date_time_t0;
        let years = duration.num_days() as f64 / 365.0;
        ((self.retorno + capital_inicial) / capital_inicial).powf(1.0 / years) - 1.0
    }

    //FUNCIONES
    pub async fn guardar_resultados(&self) {
        let result = insert_resultados(&self).await;
        match result {
            Ok(_) => {
                if LOGS_REGISTRO {
                    let entry =
                        RegistroLog::new("Resultados guardados en la base de datos.".to_string());
                    entry.guardar_logs().unwrap();
                }
            }
            Err(e) => {
                if LOGS_REGISTRO {
                    let entry = RegistroLog::new(format!("Error al guardar resultados: {:?}", e));
                    entry.guardar_logs().unwrap();
                }
            }
        }
    }
}
