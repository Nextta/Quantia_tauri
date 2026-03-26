// use sqlx::Connection;
use crate::backtest::backtest::Backtest;
use crate::backtest::broker::BrokerCFD;
use crate::backtest::dias::Dias;
use crate::backtest::resultados::Resultados;
use crate::backtest::symbol::SymbolInfoCFD;
use crate::backtest::trade::Trade;

use crate::utils::tools::truncate_decimal;
use rust_decimal::Decimal;
use sqlx::migrate::MigrateDatabase;
use sqlx::sqlite::{Sqlite, SqlitePool, SqliteRow};
use sqlx::{Column, Row, TypeInfo};

#[derive(Debug)]
pub struct DbSqlite {
    pool: SqlitePool,
}

impl DbSqlite {
    pub async fn new(db_path: &str) -> Result<Self, sqlx::Error> {
        if !Sqlite::database_exists(db_path).await? {
            Sqlite::create_database(db_path).await?;
            println!("Database '{}' creada correctamente.", db_path);
        }

        let pool = SqlitePool::connect(db_path).await?;
        Ok(Self { pool })
    }

    pub async fn execute(&self, sql: &str) -> Result<Vec<SqliteRow>, sqlx::Error> {
        let result = sqlx::query(sql).fetch_all(&self.pool).await?;

        Ok(result)
    }

    pub async fn table_trades(&self) -> Result<(), sqlx::Error> {
        let sql = "CREATE TABLE IF NOT EXISTS trades
                    (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    id_backtest INTEGER NOT NULL,
                    id_symbol INTEGER NOT NULL,
                    symbol TEXT NOT NULL,
                    tipo TEXT NOT NULL,
                    lotaje REAL NOT NULL,
                    multiplicador REAL NOT NULL,
                    t0 TEXT NOT NULL,
                    tp REAL NOT NULL,
                    sl REAL NOT NULL,
                    precio_entrada REAL NOT NULL,
                    t1 TEXT NOT NULL,
                    precio_cierre REAL NOT NULL,
                    precio_maximo REAL NOT NULL,
                    precio_minimo REAL NOT NULL,
                    duracion_segundos TEXT NOT NULL,
                    duracion_minutos TEXT NOT NULL,
                    duracion_horas TEXT NOT NULL,
                    duracion_dias TEXT NOT NULL,
                    label INTEGER NOT NULL,
                    pl REAL NOT NULL,
                    plsc REAL NOT NULL,
                    pips_pl REAL NOT NULL
                    )";

        let table = self.execute(sql).await;

        match table {
            Ok(_) => println!("TABLE: Trades is ok."),
            Err(e) => println!("TABLE: Trades error: {}", e),
        }

        Ok(())
    }

    pub async fn table_symbol_cfd(&self) -> Result<(), sqlx::Error> {
        let sql = "CREATE TABLE IF NOT EXISTS symbol_cfd
                    (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    broker_id INTEGER NOT NULL,
                    name TEXT NOT NULL,
                    valor_contrato REAL NOT NULL,
                    comision_lote REAL NOT NULL,
                    swap_long REAL NOT NULL,
                    swap_short REAL NOT NULL,
                    dia_triple_swap TEXT NOT NULL,
                    lotaje_minimo REAL NOT NULL,
                    lotaje_maximo REAL NOT NULL,
                    digitos INTEGER NOT NULL,
                    open_weekend BOOLEAN NOT NULL,
                    spread REAL NOT NULL
                    )"
        .to_string();

        let table = self.execute(&sql).await;

        match table {
            Ok(_) => {
                println!("TABLE: symbol_cfd is ok.");
            }
            Err(e) => println!("TABLE: symbol_cfd error: {}", e),
        }

        Ok(())
    }

    pub async fn insert_symbol_cfd(&self, symbol_cfd: SymbolInfoCFD) -> Result<i32, sqlx::Error> {
        let sql = format!(
            "INSERT INTO symbol_cfd (broker_id, name, valor_contrato, comision_lote, swap_long, swap_short, dia_triple_swap, lotaje_minimo, lotaje_maximo, digitos, open_weekend, spread) VALUES ({}, '{}', {}, {}, {}, {}, '{}', {}, {}, {}, {}, {}) RETURNING id",
            symbol_cfd.get_broker_id(),
            symbol_cfd.get_name(),
            symbol_cfd.get_valor_contrato(),
            symbol_cfd.get_comision_lote(),
            symbol_cfd.get_swap_long(),
            symbol_cfd.get_swap_short(),
            symbol_cfd.get_dia_triple_swap().to_string(),
            symbol_cfd.get_lotaje_minimo(),
            symbol_cfd.get_lotaje_maximo(),
            symbol_cfd.get_digitos(),
            symbol_cfd.get_open_weekend(),
            symbol_cfd.get_spread()
        );

        let mut id: i32 = 0;
        let insert = self.execute(&sql).await;
        match insert {
            Ok(row) => {
                for r in row {
                    id = r.get::<i32, _>("id");
                }
            }
            Err(e) => println!("INSERT: symbol_cfd error: {}", e),
        }

        Ok(id)
    }

    pub async fn table_resultados(&self) -> Result<(), sqlx::Error> {
        let sql = "CREATE TABLE IF NOT EXISTS resultados
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
                    )"
        .to_string();

        let table = self.execute(&sql).await;

        match table {
            Ok(_) => {
                println!("TABLE: resultados is ok.");
            }
            Err(e) => {
                println!("TABLE: resultados error: {}", e);
            }
        }

        Ok(())
    }

    pub async fn insert_resultados(&self, resultados: &Resultados) -> Result<i32, sqlx::Error> {
        let sql = format!("INSERT INTO resultados (
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
            avg_bars_win, avg_bars_loss) VALUES ({}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}) RETURNING id", resultados.get_id_backtest(),
            resultados.get_return(), resultados.get_return_percent(), resultados.get_cagr(), resultados.get_sharpe_ratio(), resultados.get_sortino_ratio(), resultados.get_omega_ratio(),
            resultados.get_expected_daily(), resultados.get_expected_monthly(), resultados.get_expected_yearly(),
            resultados.get_best_day(), resultados.get_worst_day(), resultados.get_best_month(), resultados.get_worst_month(),
            resultados.get_best_year(), resultados.get_worst_year(), resultados.get_time_in_market(), resultados.get_max_drawdown(),
            resultados.get_max_drawdown_divisa(), resultados.get_max_drawdown_duration(), resultados.get_avg_drawdown_duration(), resultados.get_max_drawdown_avg(),
            resultados.get_avg_drawdown(), resultados.get_ulcer_index(), resultados.get_serenity_index(), resultados.get_daily_var(), resultados.get_daily_var_95(), resultados.get_daily_var_99(),
            resultados.get_cvar(), resultados.get_risk_of_ruin(), resultados.get_volatility_ann(), resultados.get_calmar_ratio(), resultados.get_skew_ratio(),
            resultados.get_kurtosis_ratio(), resultados.get_tail_ratio(), resultados.get_outlier_win(), resultados.get_outlier_loss(),
            resultados.get_payoff_ratio(), resultados.get_profit_factor(), resultados.get_gain_pain_ratio(), resultados.get_common_sense_ratio(),
            resultados.get_cpc_index(), resultados.get_kelly_criterion(), resultados.get_win_days(), resultados.get_win_months(), resultados.get_win_quarters(),
            resultados.get_win_years(), resultados.get_beta(), resultados.get_alpha(), resultados.get_correlation(), resultados.get_information_ratio(),
            resultados.get_recovery_factor(), resultados.get_n_trades(), resultados.get_return_drawdown_ratio(), resultados.get_wins_percentage(),
            resultados.get_avg_trade_return(), resultados.get_avg_win_return(), resultados.get_avg_loss_return(), resultados.get_avg_win_loss_ratio(),
            resultados.get_r_expectancy(), resultados.get_r_exp_score(), resultados.get_z_score(), resultados.get_z_probability(), resultados.get_n_wins(), resultados.get_n_losses(),
            resultados.get_avg_bars_win(), resultados.get_avg_bars_loss()
        );

        let result = self.execute(&sql).await;
        let mut id: i32 = 0;
        match result {
            Ok(row) => {
                for r in row {
                    id = r.get::<i32, _>("id");
                }
            }
            Err(e) => {
                println!("SQL: {}", sql);
                println!("Error al guardar los resultados: {}", e)
            }
        }

        Ok(id)
    }

    pub async fn table_broker_cfd(&self) -> Result<(), sqlx::Error> {
        let sql = "CREATE TABLE IF NOT EXISTS broker_cfd
                    (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    name TEXT NOT NULL
                    )"
        .to_string();

        let table = self.execute(&sql).await;

        match table {
            Ok(_) => {
                println!("TABLE: broker_cfd is ok.");
            }
            Err(e) => {
                println!("Error al crear la tabla broker_cfd: {}", e);
            }
        }

        Ok(())
    }

    pub async fn insert_broker_cfd(&self, broker: BrokerCFD) -> Result<i32, sqlx::Error> {
        let sql = format!(
            "INSERT INTO broker_cfd (name) VALUES ('{}') RETURNING id",
            broker.get_name()
        );
        let result = self.execute(&sql).await;
        let mut id = 0;
        match result {
            Ok(row) => {
                for r in row {
                    id = r.get::<i32, _>("id");
                }
            }
            Err(e) => println!("SELECT: broker_cfd error: {}", e),
        }

        Ok(id)
    }

    pub async fn table_backtest(&self) -> Result<(), sqlx::Error> {
        let sql: String = "CREATE TABLE IF NOT EXISTS backtest
                    (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    titulo TEXT NOT NULL,
                    balance REAL NOT NULL,
                    tipo TEXT NOT NULL
                    )"
        .to_string();

        let table = self.execute(&sql).await;

        match table {
            Ok(_) => {
                println!("TABLE: backtest is ok.");
            }
            Err(e) => {
                println!("Error al crear la tabla backtest: {}", e);
            }
        }

        Ok(())
    }

    pub async fn insert_backtest(&self, backtest: &Backtest) -> Result<i32, sqlx::Error> {
        let sql = format!(
            "INSERT INTO backtest (titulo, balance, tipo) VALUES ('{}', {}, '{}') RETURNING id",
            backtest.get_titulo(),
            backtest.get_balance(),
            backtest.get_tipo()
        );

        let result = self.execute(&sql).await;
        let mut id = 0;

        match result {
            Ok(row) => {
                for r in row {
                    id = r.get::<i32, _>("id");
                }
            }
            Err(e) => println!("TABLE: backtest error: {}", e),
        }

        Ok(id)
    }

    pub async fn insert_trades(&self, id_backtest: i32, trade: &Trade) -> Result<i32, sqlx::Error> {
        let pl: Decimal = trade.get_pl().to_string().parse().unwrap();
        let plsc: Decimal = trade.get_plsc().to_string().parse().unwrap();
        let pips_pl: Decimal = trade.get_pip_pl().to_string().parse().unwrap();

        let sql = format!(
            "INSERT INTO trades (id_backtest, id_symbol, symbol, tipo, lotaje, multiplicador, t0, precio_entrada, tp, sl, t1, precio_cierre, precio_maximo, precio_minimo, duracion_segundos, duracion_minutos, duracion_horas, duracion_dias, label, pl, plsc, pips_pl) VALUES ({}, {}, '{}', '{}', {}, {}, '{}', {}, {}, {}, '{}', {}, {}, {}, '{}', '{}', '{}', '{}', {}, {}, {}, {}) RETURNING id",
            id_backtest,
            trade.get_symbol().get_id(),
            trade.get_symbol().get_name(),
            trade.get_tipo(),
            trade.get_lotaje(),
            trade.get_multiplier(),
            trade.get_t0(),
            trade.get_precio_entrada(),
            trade.get_tp(),
            trade.get_sl(),
            trade.get_t1(),
            trade.get_precio_cierre(),
            trade.get_precio_maximo(),
            trade.get_precio_minimo(),
            trade.get_duracion_segundos(),
            trade.get_duracion_minutos(),
            trade.get_duracion_horas(),
            trade.get_duracion_dias(),
            trade.get_label(),
            truncate_decimal(pl, 2),
            truncate_decimal(plsc, 2),
            truncate_decimal(pips_pl, 5)
        );

        let result = self.execute(&sql).await;

        let mut id: i32 = 0;
        match result {
            Ok(result) => {
                for row in result {
                    id = row.get::<i32, _>("id");
                }
            }
            Err(err) => {
                println!("SQL: {}", sql);
                println!("Error al guardar el trade: {}", err)
            }
        }

        Ok(id)
    }

    ///Selects symbols_cfd
    pub async fn get_symbols_cfd(&self) -> Result<Vec<SymbolInfoCFD>, sqlx::Error> {
        let sql = "SELECT * FROM symbol_cfd";
        let result = self.execute(&sql).await;

        let mut symbols = Vec::<SymbolInfoCFD>::new();
        match result {
            Ok(result) => {
                for row in result {
                    let symbol: SymbolInfoCFD = SymbolInfoCFD::new(
                        row.get::<i32, _>("id"),
                        row.get::<i32, _>("broker_id"),
                        row.get::<String, _>("name"),
                        row.get::<f64, _>("valor_contrato"),
                        row.get::<f64, _>("comision_lote"),
                        row.get::<f64, _>("swap_long"),
                        row.get::<f64, _>("swap_short"),
                        match row.get::<&str, _>("dia_triple_swap") {
                            "Lu" => Dias::Lu,
                            "Ma" => Dias::Ma,
                            "Mi" => Dias::Mi,
                            "Ju" => Dias::Ju,
                            "Vi" => Dias::Vi,
                            "Sa" => Dias::Sa,
                            "Do" => Dias::Do,
                            _ => Dias::Vi,
                        },
                        row.get::<f64, _>("lotaje_minimo"),
                        row.get::<f64, _>("lotaje_maximo"),
                        row.get::<u8, _>("digitos"),
                        row.get::<f64, _>("spread"),
                        row.get::<bool, _>("open_weekend"),
                    )
                    .await;

                    symbols.push(symbol);
                }
            }
            Err(err) => {
                println!("SQL: {}", sql);
                println!("Error al obtener los activos: {}", err)
            }
        }

        Ok(symbols)
    }

    pub async fn get_symbol_cfd_by_name(&self, name: &str) -> Result<SymbolInfoCFD, sqlx::Error> {
        let sql = format!("SELECT * FROM symbol_cfd WHERE name = '{}'", name);
        let result = self.execute(&sql).await;

        match result {
            Ok(result) => {
                for row in result {
                    let symbol: SymbolInfoCFD = SymbolInfoCFD::new(
                        row.get::<i32, _>("id"),
                        row.get::<i32, _>("broker_id"),
                        row.get::<String, _>("name"),
                        row.get::<f64, _>("valor_contrato"),
                        row.get::<f64, _>("comision_lote"),
                        row.get::<f64, _>("swap_long"),
                        row.get::<f64, _>("swap_short"),
                        match row.get::<&str, _>("dia_triple_swap") {
                            "Lu" => Dias::Lu,
                            "Ma" => Dias::Ma,
                            "Mi" => Dias::Mi,
                            "Ju" => Dias::Ju,
                            "Vi" => Dias::Vi,
                            "Sa" => Dias::Sa,
                            "Do" => Dias::Do,
                            _ => Dias::Vi,
                        },
                        row.get::<f64, _>("lotaje_minimo"),
                        row.get::<f64, _>("lotaje_maximo"),
                        row.get::<u8, _>("digitos"),
                        row.get::<f64, _>("spread"),
                        row.get::<bool, _>("open_weekend"),
                    )
                    .await;
                    return Ok(symbol);
                }
            }
            Err(err) => {
                println!("SQL: {}", sql);
                println!("Error al obtener los activos: {}", err);
            }
        }
        Err(sqlx::Error::RowNotFound)
    }

    pub async fn get_symbol_cfd_by_id(&self, id: i32) -> Result<SymbolInfoCFD, sqlx::Error> {
        let sql = format!("SELECT * FROM symbol_cfd WHERE id = {}", id);
        let result = self.execute(&sql).await;

        match result {
            Ok(result) => {
                for row in result {
                    let symbol: SymbolInfoCFD = SymbolInfoCFD::new(
                        row.get::<i32, _>("id"),
                        row.get::<i32, _>("broker_id"),
                        row.get::<String, _>("name"),
                        row.get::<f64, _>("valor_contrato"),
                        row.get::<f64, _>("comision_lote"),
                        row.get::<f64, _>("swap_long"),
                        row.get::<f64, _>("swap_short"),
                        match row.get::<&str, _>("dia_triple_swap") {
                            "Lu" => Dias::Lu,
                            "Ma" => Dias::Ma,
                            "Mi" => Dias::Mi,
                            "Ju" => Dias::Ju,
                            "Vi" => Dias::Vi,
                            "Sa" => Dias::Sa,
                            "Do" => Dias::Do,
                            _ => Dias::Vi,
                        },
                        row.get::<f64, _>("lotaje_minimo"),
                        row.get::<f64, _>("lotaje_maximo"),
                        row.get::<u8, _>("digitos"),
                        row.get::<f64, _>("spread"),
                        row.get::<bool, _>("open_weekend"),
                    )
                    .await;
                    return Ok(symbol);
                }
            }
            Err(err) => {
                println!("SQL: {}", sql);
                println!("Error al obtener los activos: {}", err);
            }
        }
        Err(sqlx::Error::RowNotFound)
    }

    pub async fn get_symbols_by_broker(
        &self,
        broker_id: i32,
    ) -> Result<Vec<SymbolInfoCFD>, sqlx::Error> {
        let sql = format!("SELECT * FROM symbol WHERE broker_id = {}", broker_id);
        let result = self.execute(&sql).await;
        let mut symbols = Vec::<SymbolInfoCFD>::new();
        match result {
            Ok(result) => {
                for row in result {
                    let symbol: SymbolInfoCFD = SymbolInfoCFD::new(
                        row.get::<i32, _>("id"),
                        row.get::<i32, _>("broker_id"),
                        row.get::<String, _>("name"),
                        row.get::<f64, _>("valor_contrato"),
                        row.get::<f64, _>("comision_lote"),
                        row.get::<f64, _>("swap_long"),
                        row.get::<f64, _>("swap_short"),
                        match row.get::<&str, _>("dia_triple_swap") {
                            "Lu" => Dias::Lu,
                            "Ma" => Dias::Ma,
                            "Mi" => Dias::Mi,
                            "Ju" => Dias::Ju,
                            "Vi" => Dias::Vi,
                            "Sa" => Dias::Sa,
                            "Do" => Dias::Do,
                            _ => Dias::Vi,
                        },
                        row.get::<f64, _>("lotaje_minimo"),
                        row.get::<f64, _>("lotaje_maximo"),
                        row.get::<u8, _>("digitos"),
                        row.get::<f64, _>("spread"),
                        row.get::<bool, _>("open_weekend"),
                    )
                    .await;
                    symbols.push(symbol);
                }
            }
            Err(err) => {
                println!("SQL: {}", sql);
                println!("Error al obtener los activos: {}", err);
            }
        }
        Ok(symbols)
    }

    ///Selects brokers
    pub async fn get_brokers(&self) -> Result<Vec<BrokerCFD>, sqlx::Error> {
        let sql = "SELECT * FROM broker";
        let result = self.execute(&sql).await;
        let mut brokers = Vec::<BrokerCFD>::new();
        match result {
            Ok(result) => {
                for row in result {
                    let broker: BrokerCFD =
                        BrokerCFD::new(row.get::<i32, _>("id"), row.get::<String, _>("name")).await;
                    brokers.push(broker);
                }
            }
            Err(err) => {
                println!("SQL: {}", sql);
                println!("Error al obtener los brokers: {}", err);
            }
        }
        Ok(brokers)
    }

    pub async fn get_broker_by_name(&self, name: &str) -> Result<BrokerCFD, sqlx::Error> {
        let sql = format!("SELECT * FROM broker WHERE name = '{}'", name);
        let result = self.execute(&sql).await;
        match result {
            Ok(result) => {
                for row in result {
                    let broker: BrokerCFD =
                        BrokerCFD::new(row.get::<i32, _>("id"), row.get::<String, _>("name")).await;
                    return Ok(broker);
                }
            }
            Err(err) => {
                println!("SQL: {}", sql);
                println!("Error al obtener el broker: {}", err);
            }
        }
        Err(sqlx::Error::RowNotFound)
    }

    pub async fn get_broker_by_id(&self, id: i32) -> Result<BrokerCFD, sqlx::Error> {
        let sql = format!("SELECT * FROM broker WHERE id = {}", id);
        let result = self.execute(&sql).await;
        match result {
            Ok(result) => {
                for row in result {
                    let broker: BrokerCFD =
                        BrokerCFD::new(row.get::<i32, _>("id"), row.get::<String, _>("name")).await;
                    return Ok(broker);
                }
            }
            Err(err) => {
                println!("SQL: {}", sql);
                println!("Error al obtener el broker: {}", err);
            }
        }
        Err(sqlx::Error::RowNotFound)
    }

    ///Selects resultados
    pub async fn get_resultados_by_id(&self, id: i32) -> Result<Resultados, sqlx::Error> {
        let sql = format!("SELECT * FROM resultados WHERE id = {}", id);
        let result = self.execute(&sql).await;
        match result {
            Ok(result) => {
                for row in result {
                    let mut resultados: Resultados =
                        Resultados::new(row.get::<i32, _>("id_backtest")).await;
                    resultados.set_id(row.get::<i32, _>("id"));
                    resultados.set_return(row.get::<f64, _>("retorno"));
                    resultados.set_return_percent(row.get::<f64, _>("return_percent"));
                    resultados.set_cagr(row.get::<f64, _>("cagr"));
                    resultados.set_sharpe_ratio(row.get::<f64, _>("sharpe_ratio"));
                    resultados.set_sortino_ratio(row.get::<f64, _>("sortino_ratio"));
                    resultados.set_omega_ratio(row.get::<f64, _>("omega_ratio"));
                    resultados.set_expected_daily(row.get::<f64, _>("expected_daily"));
                    resultados.set_expected_monthly(row.get::<f64, _>("expected_monthly"));
                    resultados.set_expected_yearly(row.get::<f64, _>("expected_yearly"));
                    resultados.set_best_day(row.get::<f64, _>("best_day"));
                    resultados.set_worst_day(row.get::<f64, _>("worst_day"));
                    resultados.set_best_month(row.get::<f64, _>("best_month"));
                    resultados.set_worst_month(row.get::<f64, _>("worst_month"));
                    resultados.set_best_year(row.get::<f64, _>("best_year"));
                    resultados.set_worst_year(row.get::<f64, _>("worst_year"));
                    resultados.set_time_in_market(row.get::<f64, _>("time_in_market"));
                    resultados.set_max_drawdown(row.get::<f64, _>("max_drawdown"));
                    resultados.set_max_drawdown_divisa(row.get::<f64, _>("max_drawdown_divisa"));
                    resultados
                        .set_max_drawdown_duration(row.get::<u64, _>("max_drawdown_duration"));
                    resultados
                        .set_avg_drawdown_duration(row.get::<f64, _>("avg_drawdown_duration"));
                    resultados.set_max_drawdown_avg(row.get::<f64, _>("max_drawdown_avg"));
                    resultados.set_avg_drawdown(row.get::<f64, _>("avg_drawdown"));
                    resultados.set_ulcer_index(row.get::<f64, _>("ulcer_index"));
                    resultados.set_serenity_index(row.get::<f64, _>("serenity_index"));
                    resultados.set_daily_var(row.get::<f64, _>("daily_var"));
                    resultados.set_daily_var_95(row.get::<f64, _>("daily_var_95"));
                    resultados.set_daily_var_99(row.get::<f64, _>("daily_var_99"));
                    resultados.set_cvar(row.get::<f64, _>("cvar"));
                    resultados.set_risk_of_ruin(row.get::<f64, _>("risk_of_ruin"));
                    resultados.set_volatility_ann(row.get::<f64, _>("volatility_ann"));
                    resultados.set_calmar_ratio(row.get::<f64, _>("calmar_ratio"));
                    resultados.set_skew_ratio(row.get::<f64, _>("skew_ratio"));
                    resultados.set_kurtosis_ratio(row.get::<f64, _>("kurtosis_ratio"));
                    resultados.set_tail_ratio(row.get::<f64, _>("tail_ratio"));
                    resultados.set_outlier_win(row.get::<f64, _>("outlier_win"));
                    resultados.set_outlier_loss(row.get::<f64, _>("outlier_loss"));
                    resultados.set_payoff_ratio(row.get::<f64, _>("payoff_ratio"));
                    resultados.set_profit_factor(row.get::<f64, _>("profit_factor"));
                    resultados.set_gain_pain_ratio(row.get::<f64, _>("gain_pain_ratio"));
                    resultados.set_common_sense_ratio(row.get::<f64, _>("common_sense_ratio"));
                    resultados.set_cpc_index(row.get::<f64, _>("cpc_index"));
                    resultados.set_kelly_criterion(row.get::<f64, _>("kelly_criterion"));
                    resultados.set_win_days(row.get::<f64, _>("win_days"));
                    resultados.set_win_months(row.get::<f64, _>("win_months"));
                    resultados.set_win_quarters(row.get::<f64, _>("win_quarters"));
                    resultados.set_win_years(row.get::<f64, _>("win_years"));
                    resultados.set_beta(row.get::<f64, _>("beta"));
                    resultados.set_alpha(row.get::<f64, _>("alpha"));
                    resultados.set_correlation(row.get::<f64, _>("correlation"));
                    resultados.set_information_ratio(row.get::<f64, _>("information_ratio"));
                    resultados.set_recovery_factor(row.get::<f64, _>("recovery_factor"));
                    resultados.set_n_trades(row.get::<u64, _>("n_trades"));
                    resultados
                        .set_return_drawdown_ratio(row.get::<f64, _>("return_drawdown_ratio"));
                    resultados.set_wins_percentage(row.get::<f64, _>("wins_percentage"));
                    resultados.set_avg_trade_return(row.get::<f64, _>("avg_trade_return"));
                    resultados.set_avg_win_return(row.get::<f64, _>("avg_win_return"));
                    resultados.set_avg_loss_return(row.get::<f64, _>("avg_loss_return"));
                    resultados.set_avg_win_loss_ratio(row.get::<f64, _>("avg_win_loss_ratio"));
                    resultados.set_r_expectancy(row.get::<f64, _>("r_expectancy"));
                    resultados.set_r_exp_score(row.get::<f64, _>("r_exp_score"));
                    resultados.set_z_score(row.get::<f64, _>("z_score"));
                    resultados.set_z_probability(row.get::<f64, _>("z_probability"));
                    resultados.set_n_wins(row.get::<u64, _>("n_wins"));
                    resultados.set_n_losses(row.get::<u64, _>("n_losses"));
                    resultados.set_avg_bars_win(row.get::<f64, _>("avg_bars_win"));
                    resultados.set_avg_bars_loss(row.get::<f64, _>("avg_bars_loss"));
                    return Ok(resultados);
                }
            }
            Err(err) => {
                println!("SQL: {}", sql);
                println!("Error al obtener los resultados: {}", err);
            }
        }
        Err(sqlx::Error::RowNotFound)
    }

    pub async fn get_resultados_by_id_backtest(
        &self,
        id: i32,
    ) -> Result<Vec<Resultados>, sqlx::Error> {
        let sql = format!("SELECT * FROM resultados WHERE id = {}", id);
        let result = self.execute(&sql).await;

        let mut resultados = Vec::<Resultados>::new();
        match result {
            Ok(result) => {
                for row in result {
                    let mut resultado: Resultados =
                        Resultados::new(row.get::<i32, _>("id_backtest")).await;
                    resultado.set_id(row.get::<i32, _>("id"));
                    resultado.set_return(row.get::<f64, _>("retorno"));
                    resultado.set_return_percent(row.get::<f64, _>("return_percent"));
                    resultado.set_cagr(row.get::<f64, _>("cagr"));
                    resultado.set_sharpe_ratio(row.get::<f64, _>("sharpe_ratio"));
                    resultado.set_sortino_ratio(row.get::<f64, _>("sortino_ratio"));
                    resultado.set_omega_ratio(row.get::<f64, _>("omega_ratio"));
                    resultado.set_expected_daily(row.get::<f64, _>("expected_daily"));
                    resultado.set_expected_monthly(row.get::<f64, _>("expected_monthly"));
                    resultado.set_expected_yearly(row.get::<f64, _>("expected_yearly"));
                    resultado.set_best_day(row.get::<f64, _>("best_day"));
                    resultado.set_worst_day(row.get::<f64, _>("worst_day"));
                    resultado.set_best_month(row.get::<f64, _>("best_month"));
                    resultado.set_worst_month(row.get::<f64, _>("worst_month"));
                    resultado.set_best_year(row.get::<f64, _>("best_year"));
                    resultado.set_worst_year(row.get::<f64, _>("worst_year"));
                    resultado.set_time_in_market(row.get::<f64, _>("time_in_market"));
                    resultado.set_max_drawdown(row.get::<f64, _>("max_drawdown"));
                    resultado.set_max_drawdown_divisa(row.get::<f64, _>("max_drawdown_divisa"));
                    resultado.set_max_drawdown_duration(row.get::<u64, _>("max_drawdown_duration"));
                    resultado.set_avg_drawdown_duration(row.get::<f64, _>("avg_drawdown_duration"));
                    resultado.set_max_drawdown_avg(row.get::<f64, _>("max_drawdown_avg"));
                    resultado.set_avg_drawdown(row.get::<f64, _>("avg_drawdown"));
                    resultado.set_ulcer_index(row.get::<f64, _>("ulcer_index"));
                    resultado.set_serenity_index(row.get::<f64, _>("serenity_index"));
                    resultado.set_daily_var(row.get::<f64, _>("daily_var"));
                    resultado.set_daily_var_95(row.get::<f64, _>("daily_var_95"));
                    resultado.set_daily_var_99(row.get::<f64, _>("daily_var_99"));
                    resultado.set_cvar(row.get::<f64, _>("cvar"));
                    resultado.set_risk_of_ruin(row.get::<f64, _>("risk_of_ruin"));
                    resultado.set_volatility_ann(row.get::<f64, _>("volatility_ann"));
                    resultado.set_calmar_ratio(row.get::<f64, _>("calmar_ratio"));
                    resultado.set_skew_ratio(row.get::<f64, _>("skew_ratio"));
                    resultado.set_kurtosis_ratio(row.get::<f64, _>("kurtosis_ratio"));
                    resultado.set_tail_ratio(row.get::<f64, _>("tail_ratio"));
                    resultado.set_outlier_win(row.get::<f64, _>("outlier_win"));
                    resultado.set_outlier_loss(row.get::<f64, _>("outlier_loss"));
                    resultado.set_payoff_ratio(row.get::<f64, _>("payoff_ratio"));
                    resultado.set_profit_factor(row.get::<f64, _>("profit_factor"));
                    resultado.set_gain_pain_ratio(row.get::<f64, _>("gain_pain_ratio"));
                    resultado.set_common_sense_ratio(row.get::<f64, _>("common_sense_ratio"));
                    resultado.set_cpc_index(row.get::<f64, _>("cpc_index"));
                    resultado.set_kelly_criterion(row.get::<f64, _>("kelly_criterion"));
                    resultado.set_win_days(row.get::<f64, _>("win_days"));
                    resultado.set_win_months(row.get::<f64, _>("win_months"));
                    resultado.set_win_quarters(row.get::<f64, _>("win_quarters"));
                    resultado.set_win_years(row.get::<f64, _>("win_years"));
                    resultado.set_beta(row.get::<f64, _>("beta"));
                    resultado.set_alpha(row.get::<f64, _>("alpha"));
                    resultado.set_correlation(row.get::<f64, _>("correlation"));
                    resultado.set_information_ratio(row.get::<f64, _>("information_ratio"));
                    resultado.set_recovery_factor(row.get::<f64, _>("recovery_factor"));
                    resultado.set_n_trades(row.get::<u64, _>("n_trades"));
                    resultado.set_return_drawdown_ratio(row.get::<f64, _>("return_drawdown_ratio"));
                    resultado.set_wins_percentage(row.get::<f64, _>("wins_percentage"));
                    resultado.set_avg_trade_return(row.get::<f64, _>("avg_trade_return"));
                    resultado.set_avg_win_return(row.get::<f64, _>("avg_win_return"));
                    resultado.set_avg_loss_return(row.get::<f64, _>("avg_loss_return"));
                    resultado.set_avg_win_loss_ratio(row.get::<f64, _>("avg_win_loss_ratio"));
                    resultado.set_r_expectancy(row.get::<f64, _>("r_expectancy"));
                    resultado.set_r_exp_score(row.get::<f64, _>("r_exp_score"));
                    resultado.set_z_score(row.get::<f64, _>("z_score"));
                    resultado.set_z_probability(row.get::<f64, _>("z_probability"));
                    resultado.set_n_wins(row.get::<u64, _>("n_wins"));
                    resultado.set_n_losses(row.get::<u64, _>("n_losses"));
                    resultado.set_avg_bars_win(row.get::<f64, _>("avg_bars_win"));
                    resultado.set_avg_bars_loss(row.get::<f64, _>("avg_bars_loss"));

                    resultados.push(resultado);
                }
            }
            Err(err) => {
                println!("SQL: {}", sql);
                println!("Error al obtener los resultados backtest: {}", err);
            }
        }
        Ok(resultados)
    }

    pub async fn get_resultados(&self) -> Result<Vec<Resultados>, sqlx::Error> {
        let sql = format!("SELECT * FROM resultados");
        let result = self.execute(&sql).await;

        let mut resultados = Vec::<Resultados>::new();
        match result {
            Ok(result) => {
                for row in result {
                    let mut resultado: Resultados =
                        Resultados::new(row.get::<i32, _>("id_backtest")).await;
                    resultado.set_id(row.get::<i32, _>("id"));
                    resultado.set_return(row.get::<f64, _>("retorno"));
                    resultado.set_return_percent(row.get::<f64, _>("return_percent"));
                    resultado.set_cagr(row.get::<f64, _>("cagr"));
                    resultado.set_sharpe_ratio(row.get::<f64, _>("sharpe_ratio"));
                    resultado.set_sortino_ratio(row.get::<f64, _>("sortino_ratio"));
                    resultado.set_omega_ratio(row.get::<f64, _>("omega_ratio"));
                    resultado.set_expected_daily(row.get::<f64, _>("expected_daily"));
                    resultado.set_expected_monthly(row.get::<f64, _>("expected_monthly"));
                    resultado.set_expected_yearly(row.get::<f64, _>("expected_yearly"));
                    resultado.set_best_day(row.get::<f64, _>("best_day"));
                    resultado.set_worst_day(row.get::<f64, _>("worst_day"));
                    resultado.set_best_month(row.get::<f64, _>("best_month"));
                    resultado.set_worst_month(row.get::<f64, _>("worst_month"));
                    resultado.set_best_year(row.get::<f64, _>("best_year"));
                    resultado.set_worst_year(row.get::<f64, _>("worst_year"));
                    resultado.set_time_in_market(row.get::<f64, _>("time_in_market"));
                    resultado.set_max_drawdown(row.get::<f64, _>("max_drawdown"));
                    resultado.set_max_drawdown_divisa(row.get::<f64, _>("max_drawdown_divisa"));
                    resultado.set_max_drawdown_duration(row.get::<u64, _>("max_drawdown_duration"));
                    resultado.set_avg_drawdown_duration(row.get::<f64, _>("avg_drawdown_duration"));
                    resultado.set_max_drawdown_avg(row.get::<f64, _>("max_drawdown_avg"));
                    resultado.set_avg_drawdown(row.get::<f64, _>("avg_drawdown"));
                    resultado.set_ulcer_index(row.get::<f64, _>("ulcer_index"));
                    resultado.set_serenity_index(row.get::<f64, _>("serenity_index"));
                    resultado.set_daily_var(row.get::<f64, _>("daily_var"));
                    resultado.set_daily_var_95(row.get::<f64, _>("daily_var_95"));
                    resultado.set_daily_var_99(row.get::<f64, _>("daily_var_99"));
                    resultado.set_cvar(row.get::<f64, _>("cvar"));
                    resultado.set_risk_of_ruin(row.get::<f64, _>("risk_of_ruin"));
                    resultado.set_volatility_ann(row.get::<f64, _>("volatility_ann"));
                    resultado.set_calmar_ratio(row.get::<f64, _>("calmar_ratio"));
                    resultado.set_skew_ratio(row.get::<f64, _>("skew_ratio"));
                    resultado.set_kurtosis_ratio(row.get::<f64, _>("kurtosis_ratio"));
                    resultado.set_tail_ratio(row.get::<f64, _>("tail_ratio"));
                    resultado.set_outlier_win(row.get::<f64, _>("outlier_win"));
                    resultado.set_outlier_loss(row.get::<f64, _>("outlier_loss"));
                    resultado.set_payoff_ratio(row.get::<f64, _>("payoff_ratio"));
                    resultado.set_profit_factor(row.get::<f64, _>("profit_factor"));
                    resultado.set_gain_pain_ratio(row.get::<f64, _>("gain_pain_ratio"));
                    resultado.set_common_sense_ratio(row.get::<f64, _>("common_sense_ratio"));
                    resultado.set_cpc_index(row.get::<f64, _>("cpc_index"));
                    resultado.set_kelly_criterion(row.get::<f64, _>("kelly_criterion"));
                    resultado.set_win_days(row.get::<f64, _>("win_days"));
                    resultado.set_win_months(row.get::<f64, _>("win_months"));
                    resultado.set_win_quarters(row.get::<f64, _>("win_quarters"));
                    resultado.set_win_years(row.get::<f64, _>("win_years"));
                    resultado.set_beta(row.get::<f64, _>("beta"));
                    resultado.set_alpha(row.get::<f64, _>("alpha"));
                    resultado.set_correlation(row.get::<f64, _>("correlation"));
                    resultado.set_information_ratio(row.get::<f64, _>("information_ratio"));
                    resultado.set_recovery_factor(row.get::<f64, _>("recovery_factor"));
                    resultado.set_n_trades(row.get::<u64, _>("n_trades"));
                    resultado.set_return_drawdown_ratio(row.get::<f64, _>("return_drawdown_ratio"));
                    resultado.set_wins_percentage(row.get::<f64, _>("wins_percentage"));
                    resultado.set_avg_trade_return(row.get::<f64, _>("avg_trade_return"));
                    resultado.set_avg_win_return(row.get::<f64, _>("avg_win_return"));
                    resultado.set_avg_loss_return(row.get::<f64, _>("avg_loss_return"));
                    resultado.set_avg_win_loss_ratio(row.get::<f64, _>("avg_win_loss_ratio"));
                    resultado.set_r_expectancy(row.get::<f64, _>("r_expectancy"));
                    resultado.set_r_exp_score(row.get::<f64, _>("r_exp_score"));
                    resultado.set_z_score(row.get::<f64, _>("z_score"));
                    resultado.set_z_probability(row.get::<f64, _>("z_probability"));
                    resultado.set_n_wins(row.get::<u64, _>("n_wins"));
                    resultado.set_n_losses(row.get::<u64, _>("n_losses"));
                    resultado.set_avg_bars_win(row.get::<f64, _>("avg_bars_win"));
                    resultado.set_avg_bars_loss(row.get::<f64, _>("avg_bars_loss"));

                    resultados.push(resultado);
                }
            }
            Err(err) => {
                println!("SQL: {}", sql);
                println!("Error al obtener los resultados backtest: {}", err);
            }
        }
        Ok(resultados)
    }

    ///Selects trades
    pub async fn get_trades_by_backtest(
        &self,
        id_backtest: i32,
    ) -> Result<Vec<Trade>, sqlx::Error> {
        let sql = format!("SELECT * FROM trades WHERE id_backtest = {}", id_backtest);
        let result = self.execute(&sql).await;
        let mut resultados = Vec::new();
        match result {
            Ok(rows) => {
                for row in rows {
                    let symbol: SymbolInfoCFD = self
                        .get_symbol_cfd_by_id(row.get::<i32, _>("id_symbol"))
                        .await
                        .unwrap();
                    let mut trade: Trade =
                        Trade::new(row.get::<i32, _>("id_backtest"), symbol).await;

                    trade.set_id(row.get::<i32, _>("id"));
                    trade.set_tipo(row.get::<String, _>("tipo"));
                    trade.set_lotaje_fijo(row.get::<f64, _>("lotaje"));
                    trade.set_multiplicador(row.get::<f64, _>("multiplicador"));
                    trade.set_t0(row.get::<String, _>("t0"));
                    trade.set_precio_entrada(row.get::<f64, _>("precio_entrada"));
                    trade.set_tp(row.get::<f64, _>("tp"));
                    trade.set_sl(row.get::<f64, _>("sl"));
                    trade.set_t1(row.get::<String, _>("t1"));
                    trade.set_precio_cierre(row.get::<f64, _>("precio_cierre"));
                    trade.set_precio_maximo(row.get::<f64, _>("precio_maximo"));
                    trade.set_precio_minimo(row.get::<f64, _>("precio_minimo"));
                    trade.set_duracion_segundos(row.get::<String, _>("duracion_segundos"));
                    trade.set_duracion_minutos(row.get::<String, _>("duracion_minutos"));
                    trade.set_duracion_horas(row.get::<String, _>("duracion_horas"));
                    trade.set_duracion_dias(row.get::<String, _>("duracion_dias"));
                    trade.set_label(row.get::<u8, _>("label"));
                    trade.set_pl(row.get::<f64, _>("pl"));
                    trade.set_plsc(row.get::<f64, _>("plsc"));
                    trade.set_pips_pl(row.get::<f64, _>("pips_pl"));
                    resultados.push(trade);
                }
            }
            Err(err) => {
                println!("SQL: {}", sql);
                println!("Error al obtener los trades: {}", err);
            }
        }
        Ok(resultados)
    }
}
