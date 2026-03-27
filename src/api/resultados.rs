use crate::backtest::resultados::Resultados;
use dotenvy::dotenv;
use libsql::{params, Builder};
use serde::Serialize;
use std::env;

#[derive(Serialize, Debug)]
pub struct Error {
    msg: String,
}

type Result<T> = std::result::Result<T, Error>;

impl<T> From<T> for Error
where
    T: std::error::Error,
{
    fn from(value: T) -> Self {
        Self {
            msg: value.to_string(),
        }
    }
}

fn get_db_config() -> Result<(String, String, String)> {
    dotenv().expect(".env file not found");
    let db_path = env::var("DB_PATH").unwrap();
    let sync_url = env::var("TURSO_SYNC_URL").unwrap();
    let auth_token = env::var("TURSO_AUTH_TOKEN").unwrap();
    Ok((db_path, sync_url, auth_token))
}

#[tauri::command]
pub async fn table_resultados() -> Result<String> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = Builder::new_remote_replica(db_path, sync_url, auth_token)
        .build()
        .await?;

    let conn = db.connect()?;

    conn.query(
        "CREATE TABLE IF NOT EXISTS resultados
                    (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    id_backtest INTEGER NOT NULL,
                    return REAL NOT NULL,
                    return_percent REAL NOT NULL,
                    cagr REAL NOT NULL,
                    sharpe_ratio REAL NOT NULL,
                    sortino_ratio REAL NOT NULL,
                    omega_ratio REAL NOT NULL,
                    expected_daily REAL NOT NULL,
                    expected_monthly REAL NOT NULL,
                    expected_yearly REAL NOT NULL,
                    best_day REAL NOT NULL,
                    worst_day REAL NOT NULL,
                    best_month REAL NOT NULL,
                    worst_month REAL NOT NULL,
                    best_year REAL NOT NULL,
                    worst_year REAL NOT NULL,
                    time_in_market REAL NOT NULL,
                    max_drawdown REAL NOT NULL,
                    max_drawdown_divisa REAL NOT NULL,
                    max_drawdown_duration INTEGER NOT NULL,
                    avg_drawdown_duration REAL NOT NULL,
                    max_drawdown_avg REAL NOT NULL,
                    avg_drawdown REAL NOT NULL,
                    ulcer_index REAL NOT NULL,
                    serenity_index REAL NOT NULL,
                    daily_var REAL NOT NULL,
                    daily_var_95 REAL NOT NULL,
                    daily_var_99 REAL NOT NULL,
                    cvar REAL NOT NULL,
                    risk_of_ruin REAL NOT NULL,
                    volatility_ann REAL NOT NULL,
                    calmar_ratio REAL NOT NULL,
                    skew_ratio REAL NOT NULL,
                    kurtosis_ratio REAL NOT NULL,
                    tail_ratio REAL NOT NULL,
                    outlier_win REAL NOT NULL,
                    outlier_loss REAL NOT NULL,
                    payoff_ratio REAL NOT NULL,
                    profit_factor REAL NOT NULL,
                    gain_pain_ratio REAL NOT NULL,
                    common_sense_ratio REAL NOT NULL,
                    cpc_index REAL NOT NULL,
                    kelly_criterion REAL NOT NULL,
                    win_days REAL NOT NULL,
                    win_months REAL NOT NULL,
                    win_quarters REAL NOT NULL,
                    win_years REAL NOT NULL,
                    beta REAL NOT NULL,
                    alpha REAL NOT NULL,
                    correlation REAL NOT NULL,
                    information_ratio REAL NOT NULL,
                    recovery_factor REAL NOT NULL,
                    n_trades INTEGER NOT NULL,
                    return_drawdown_ratio REAL NOT NULL,
                    wins_percentage REAL NOT NULL,
                    avg_trade_return REAL NOT NULL,
                    avg_win_return REAL NOT NULL,
                    avg_loss_return REAL NOT NULL,
                    avg_win_loss_ratio REAL NOT NULL,
                    r_expectancy REAL NOT NULL,
                    r_exp_score REAL NOT NULL,
                    z_score REAL NOT NULL,
                    z_probability REAL NOT NULL,
                    n_wins INTEGER NOT NULL,
                    n_losses INTEGER NOT NULL,
                    avg_bars_win REAL NOT NULL,
                    avg_bars_loss REAL NOT NULL
                    )",
        (),
    )
    .await?;

    Ok("Tabla resultados is ok.".to_string())
}

#[tauri::command]
pub async fn insert_resultados(resultados: Resultados) -> Result<i32> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = Builder::new_remote_replica(db_path, sync_url, auth_token)
        .build()
        .await?;

    let conn = db.connect()?;

    let parametros = params![
        resultados.id_backtest,
        resultados.retorno,
        resultados.return_percent,
        resultados.cagr,
        resultados.sharpe_ratio,
        resultados.sortino_ratio,
        resultados.omega_ratio,
        resultados.expected_daily,
        resultados.expected_monthly,
        resultados.expected_yearly,
        resultados.best_day,
        resultados.worst_day,
        resultados.best_month,
        resultados.worst_month,
        resultados.best_year,
        resultados.worst_year,
        resultados.time_in_market,
        resultados.max_drawdown,
        resultados.max_drawdown_divisa,
        resultados.max_drawdown_duration,
        resultados.avg_drawdown_duration,
        resultados.max_drawdown_avg,
        resultados.avg_drawdown,
        resultados.ulcer_index,
        resultados.serenity_index,
        resultados.daily_var,
        resultados.daily_var_95,
        resultados.daily_var_99,
        resultados.cvar,
        resultados.risk_of_ruin,
        resultados.volatility_ann,
        resultados.calmar_ratio,
        resultados.skew_ratio,
        resultados.kurtosis_ratio,
        resultados.tail_ratio,
        resultados.outlier_win,
        resultados.outlier_loss,
        resultados.payoff_ratio,
        resultados.profit_factor,
        resultados.gain_pain_ratio,
        resultados.common_sense_ratio,
        resultados.cpc_index,
        resultados.kelly_criterion,
        resultados.win_days,
        resultados.win_months,
        resultados.win_quarters,
        resultados.win_years,
        resultados.beta,
        resultados.alpha,
        resultados.correlation,
        resultados.information_ratio,
        resultados.recovery_factor,
        resultados.n_trades,
        resultados.return_drawdown_ratio,
        resultados.wins_percentage,
        resultados.avg_trade_return,
        resultados.avg_win_return,
        resultados.avg_loss_return,
        resultados.avg_win_loss_ratio,
        resultados.r_expectancy,
        resultados.r_exp_score,
        resultados.z_score,
        resultados.z_probability,
        resultados.n_wins,
        resultados.n_losses,
        resultados.avg_bars_win,
        resultados.avg_bars_loss,
    ];

    conn.query(
        "INSERT INTO resultados (
            id_backtest, return, return_percent, cagr,
            sharpe_ratio, sortino_ratio, omega_ratio,
            expected_daily, expected_monthly, expected_yearly,
            best_day, worst_day, best_month, worst_month,
            best_year, worst_year, time_in_market, max_drawdown,
            max_drawdown_divisa, max_drawdown_duration, avg_drawdown_duration, max_drawdown_avg,
            avg_drawdown, ulcer_index, serenity_index, daily_var, daily_var_95, daily_var_99,
            cvar, risk_of_ruin, volatility_ann, calmar_ratio, skew_ratio,
            kurtosis_ratio, tail_ratio, outlier_win, outlier_loss,
            payoff_ratio, profit_factor, gain_pain_ratio, common_sense_ratio,
            cpc_index, kelly_criterion, win_days, win_months, win_quarters,
            win_years, beta, alpha, correlation, information_ratio,
            recovery_factor, n_trades, return_drawdown_ratio, wins_percentage,
            avg_trade_return, avg_win_return, avg_loss_return, avg_win_loss_ratio,
            r_expectancy, r_exp_score, z_score, z_probability, n_wins, n_losses,
            avg_bars_win, avg_bars_loss) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) RETURNING id",
        parametros,
    )
    .await?;

    let id = conn.last_insert_rowid() as i32;
    Ok(id)
}

#[tauri::command]
pub async fn get_resultados() -> Result<Vec<Resultados>> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = Builder::new_remote_replica(db_path, sync_url, auth_token)
        .build()
        .await?;

    let conn = db.connect()?;

    let mut rows = conn.query("SELECT * FROM resultados", ()).await?;

    let mut resultados: Vec<Resultados> = Vec::new();
    while let Some(row) = rows.next().await? {
        let resultado: Resultados = Resultados {
            id: row.get::<i32>(0)?,
            id_backtest: row.get::<i32>(0)?,
            retorno: row.get::<f64>(0)?,
            return_percent: row.get::<f64>(0)?,
            cagr: row.get::<f64>(0)?,
            sharpe_ratio: row.get::<f64>(0)?,
            sortino_ratio: row.get::<f64>(0)?,
            omega_ratio: row.get::<f64>(0)?,
            expected_daily: row.get::<f64>(0)?,
            expected_monthly: row.get::<f64>(0)?,
            expected_yearly: row.get::<f64>(0)?,
            best_day: row.get::<f64>(0)?,
            worst_day: row.get::<f64>(0)?,
            best_month: row.get::<f64>(0)?,
            worst_month: row.get::<f64>(0)?,
            best_year: row.get::<f64>(0)?,
            worst_year: row.get::<f64>(0)?,
            time_in_market: row.get::<f64>(0)?,
            max_drawdown: row.get::<f64>(0)?,
            max_drawdown_divisa: row.get::<f64>(0)?,
            max_drawdown_duration: row.get::<u64>(0)?,
            avg_drawdown_duration: row.get::<f64>(0)?,
            max_drawdown_avg: row.get::<f64>(0)?,
            avg_drawdown: row.get::<f64>(0)?,
            ulcer_index: row.get::<f64>(0)?,
            serenity_index: row.get::<f64>(0)?,
            daily_var: row.get::<f64>(0)?,
            daily_var_95: row.get::<f64>(0)?,
            daily_var_99: row.get::<f64>(0)?,
            cvar: row.get::<f64>(0)?,
            risk_of_ruin: row.get::<f64>(0)?,
            volatility_ann: row.get::<f64>(0)?,
            calmar_ratio: row.get::<f64>(0)?,
            skew_ratio: row.get::<f64>(0)?,
            kurtosis_ratio: row.get::<f64>(0)?,
            tail_ratio: row.get::<f64>(0)?,
            outlier_win: row.get::<f64>(0)?,
            outlier_loss: row.get::<f64>(0)?,
            payoff_ratio: row.get::<f64>(0)?,
            profit_factor: row.get::<f64>(0)?,
            gain_pain_ratio: row.get::<f64>(0)?,
            common_sense_ratio: row.get::<f64>(0)?,
            cpc_index: row.get::<f64>(0)?,
            kelly_criterion: row.get::<f64>(0)?,
            win_days: row.get::<f64>(0)?,
            win_months: row.get::<f64>(0)?,
            win_quarters: row.get::<f64>(0)?,
            win_years: row.get::<f64>(0)?,
            beta: row.get::<f64>(0)?,
            alpha: row.get::<f64>(0)?,
            correlation: row.get::<f64>(0)?,
            information_ratio: row.get::<f64>(0)?,
            recovery_factor: row.get::<f64>(0)?,
            n_trades: row.get::<u64>(0)?,
            return_drawdown_ratio: row.get::<f64>(0)?,
            wins_percentage: row.get::<f64>(0)?,
            avg_trade_return: row.get::<f64>(0)?,
            avg_win_return: row.get::<f64>(0)?,
            avg_loss_return: row.get::<f64>(0)?,
            avg_win_loss_ratio: row.get::<f64>(0)?,
            r_expectancy: row.get::<f64>(0)?,
            r_exp_score: row.get::<f64>(0)?,
            z_score: row.get::<f64>(0)?,
            z_probability: row.get::<f64>(0)?,
            n_wins: row.get::<u64>(0)?,
            n_losses: row.get::<u64>(0)?,
            avg_bars_win: row.get::<f64>(0)?,
            avg_bars_loss: row.get::<f64>(0)?,
        };
        resultados.push(resultado);
    }
    Ok(resultados)
}

#[tauri::command]
pub async fn get_resultados_by_id(id: i32) -> Result<Resultados> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = Builder::new_remote_replica(db_path, sync_url, auth_token)
        .build()
        .await?;

    let conn = db.connect()?;

    let mut rows = conn
        .query("SELECT * FROM resultados WHERE id = ?", [id])
        .await?;

    let row = rows.next().await?.unwrap();

    let resultado = Resultados {
        id: row.get::<i32>(0)?,
        id_backtest: row.get::<i32>(0)?,
        retorno: row.get::<f64>(0)?,
        return_percent: row.get::<f64>(0)?,
        cagr: row.get::<f64>(0)?,
        sharpe_ratio: row.get::<f64>(0)?,
        sortino_ratio: row.get::<f64>(0)?,
        omega_ratio: row.get::<f64>(0)?,
        expected_daily: row.get::<f64>(0)?,
        expected_monthly: row.get::<f64>(0)?,
        expected_yearly: row.get::<f64>(0)?,
        best_day: row.get::<f64>(0)?,
        worst_day: row.get::<f64>(0)?,
        best_month: row.get::<f64>(0)?,
        worst_month: row.get::<f64>(0)?,
        best_year: row.get::<f64>(0)?,
        worst_year: row.get::<f64>(0)?,
        time_in_market: row.get::<f64>(0)?,
        max_drawdown: row.get::<f64>(0)?,
        max_drawdown_divisa: row.get::<f64>(0)?,
        max_drawdown_duration: row.get::<u64>(0)?,
        avg_drawdown_duration: row.get::<f64>(0)?,
        max_drawdown_avg: row.get::<f64>(0)?,
        avg_drawdown: row.get::<f64>(0)?,
        ulcer_index: row.get::<f64>(0)?,
        serenity_index: row.get::<f64>(0)?,
        daily_var: row.get::<f64>(0)?,
        daily_var_95: row.get::<f64>(0)?,
        daily_var_99: row.get::<f64>(0)?,
        cvar: row.get::<f64>(0)?,
        risk_of_ruin: row.get::<f64>(0)?,
        volatility_ann: row.get::<f64>(0)?,
        calmar_ratio: row.get::<f64>(0)?,
        skew_ratio: row.get::<f64>(0)?,
        kurtosis_ratio: row.get::<f64>(0)?,
        tail_ratio: row.get::<f64>(0)?,
        outlier_win: row.get::<f64>(0)?,
        outlier_loss: row.get::<f64>(0)?,
        payoff_ratio: row.get::<f64>(0)?,
        profit_factor: row.get::<f64>(0)?,
        gain_pain_ratio: row.get::<f64>(0)?,
        common_sense_ratio: row.get::<f64>(0)?,
        cpc_index: row.get::<f64>(0)?,
        kelly_criterion: row.get::<f64>(0)?,
        win_days: row.get::<f64>(0)?,
        win_months: row.get::<f64>(0)?,
        win_quarters: row.get::<f64>(0)?,
        win_years: row.get::<f64>(0)?,
        beta: row.get::<f64>(0)?,
        alpha: row.get::<f64>(0)?,
        correlation: row.get::<f64>(0)?,
        information_ratio: row.get::<f64>(0)?,
        recovery_factor: row.get::<f64>(0)?,
        n_trades: row.get::<u64>(0)?,
        return_drawdown_ratio: row.get::<f64>(0)?,
        wins_percentage: row.get::<f64>(0)?,
        avg_trade_return: row.get::<f64>(0)?,
        avg_win_return: row.get::<f64>(0)?,
        avg_loss_return: row.get::<f64>(0)?,
        avg_win_loss_ratio: row.get::<f64>(0)?,
        r_expectancy: row.get::<f64>(0)?,
        r_exp_score: row.get::<f64>(0)?,
        z_score: row.get::<f64>(0)?,
        z_probability: row.get::<f64>(0)?,
        n_wins: row.get::<u64>(0)?,
        n_losses: row.get::<u64>(0)?,
        avg_bars_win: row.get::<f64>(0)?,
        avg_bars_loss: row.get::<f64>(0)?,
    };
    Ok(resultado)
}

#[tauri::command]
pub async fn get_resultados_by_id_backtest(id: i32) -> Result<Resultados> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = Builder::new_remote_replica(db_path, sync_url, auth_token)
        .build()
        .await?;

    let conn = db.connect()?;

    let mut rows = conn
        .query("SELECT * FROM resultados WHERE id_backtest = ?", [id])
        .await?;

    let row = rows.next().await?.unwrap();

    let resultado = Resultados {
        id: row.get::<i32>(0)?,
        id_backtest: row.get::<i32>(0)?,
        retorno: row.get::<f64>(0)?,
        return_percent: row.get::<f64>(0)?,
        cagr: row.get::<f64>(0)?,
        sharpe_ratio: row.get::<f64>(0)?,
        sortino_ratio: row.get::<f64>(0)?,
        omega_ratio: row.get::<f64>(0)?,
        expected_daily: row.get::<f64>(0)?,
        expected_monthly: row.get::<f64>(0)?,
        expected_yearly: row.get::<f64>(0)?,
        best_day: row.get::<f64>(0)?,
        worst_day: row.get::<f64>(0)?,
        best_month: row.get::<f64>(0)?,
        worst_month: row.get::<f64>(0)?,
        best_year: row.get::<f64>(0)?,
        worst_year: row.get::<f64>(0)?,
        time_in_market: row.get::<f64>(0)?,
        max_drawdown: row.get::<f64>(0)?,
        max_drawdown_divisa: row.get::<f64>(0)?,
        max_drawdown_duration: row.get::<u64>(0)?,
        avg_drawdown_duration: row.get::<f64>(0)?,
        max_drawdown_avg: row.get::<f64>(0)?,
        avg_drawdown: row.get::<f64>(0)?,
        ulcer_index: row.get::<f64>(0)?,
        serenity_index: row.get::<f64>(0)?,
        daily_var: row.get::<f64>(0)?,
        daily_var_95: row.get::<f64>(0)?,
        daily_var_99: row.get::<f64>(0)?,
        cvar: row.get::<f64>(0)?,
        risk_of_ruin: row.get::<f64>(0)?,
        volatility_ann: row.get::<f64>(0)?,
        calmar_ratio: row.get::<f64>(0)?,
        skew_ratio: row.get::<f64>(0)?,
        kurtosis_ratio: row.get::<f64>(0)?,
        tail_ratio: row.get::<f64>(0)?,
        outlier_win: row.get::<f64>(0)?,
        outlier_loss: row.get::<f64>(0)?,
        payoff_ratio: row.get::<f64>(0)?,
        profit_factor: row.get::<f64>(0)?,
        gain_pain_ratio: row.get::<f64>(0)?,
        common_sense_ratio: row.get::<f64>(0)?,
        cpc_index: row.get::<f64>(0)?,
        kelly_criterion: row.get::<f64>(0)?,
        win_days: row.get::<f64>(0)?,
        win_months: row.get::<f64>(0)?,
        win_quarters: row.get::<f64>(0)?,
        win_years: row.get::<f64>(0)?,
        beta: row.get::<f64>(0)?,
        alpha: row.get::<f64>(0)?,
        correlation: row.get::<f64>(0)?,
        information_ratio: row.get::<f64>(0)?,
        recovery_factor: row.get::<f64>(0)?,
        n_trades: row.get::<u64>(0)?,
        return_drawdown_ratio: row.get::<f64>(0)?,
        wins_percentage: row.get::<f64>(0)?,
        avg_trade_return: row.get::<f64>(0)?,
        avg_win_return: row.get::<f64>(0)?,
        avg_loss_return: row.get::<f64>(0)?,
        avg_win_loss_ratio: row.get::<f64>(0)?,
        r_expectancy: row.get::<f64>(0)?,
        r_exp_score: row.get::<f64>(0)?,
        z_score: row.get::<f64>(0)?,
        z_probability: row.get::<f64>(0)?,
        n_wins: row.get::<u64>(0)?,
        n_losses: row.get::<u64>(0)?,
        avg_bars_win: row.get::<f64>(0)?,
        avg_bars_loss: row.get::<f64>(0)?,
    };
    Ok(resultado)
}
