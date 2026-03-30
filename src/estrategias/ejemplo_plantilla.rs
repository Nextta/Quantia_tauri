// =============================================================================
// ARQUITECTURA: PLANTILLA DE ESTRATEGIA DINÁMICA DESDE SQLITE
// =============================================================================
//
// FLUJO COMPLETO:
//
//  Rete.js (JS)                SQLite                   Rust
//  ─────────────────    ──────────────────────    ────────────────────────
//  Nodos visuales   →   strategies              →  cargar_estrategia()
//  Condiciones      →   strategy_conditions     →  evaluar_condicion()
//  Acciones         →   strategy_actions        →  ejecutar_accion()
//  Indicadores      →   strategy_indicators     →  calcular_indicador()
//
// =============================================================================

use rusqlite::{Connection, Result as SqliteResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ─────────────────────────────────────────────────────────────────────────────
// 1. MODELOS — reflejan la estructura de las tablas SQLite
// ─────────────────────────────────────────────────────────────────────────────

/// Estrategia completa cargada desde la base de datos
#[derive(Debug, Clone)]
pub struct Estrategia {
    pub id: i64,
    pub nombre: String,
    pub descripcion: String,
    pub condiciones_entrada: Vec<Condicion>,
    pub condiciones_salida: Vec<Condicion>,
    pub acciones_entrada: Vec<Accion>,
    pub acciones_salida: Vec<Accion>,
    pub indicadores: Vec<Indicador>,
}

/// Una condición: campo OPERADOR valor  (ej: "close > sma_20")
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condicion {
    pub id: i64,
    pub campo_a: String,  // ej: "close", "sma_20", "rsi"
    pub operador: String, // ej: ">", "<", ">=", "<=", "==", "cross_above"
    pub campo_b: String,  // ej: "sma_50", "30.0" (valor literal o campo)
    pub logica: String,   // "AND" | "OR"
    pub orden: i32,       // orden de evaluación
}

/// Una acción a ejecutar cuando se cumple una condición (buy, sell, set_sl, set_tp...)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Accion {
    pub id: i64,
    pub tipo: String,      // "buy" | "sell" | "close" | "set_sl" | "set_tp"
    pub parametro: String, // JSON con parámetros extra: {"lot": 0.1, "sl_pips": 20}
}

/// Un indicador que necesita ser calculado antes de evaluar condiciones
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Indicador {
    pub id: i64,
    pub nombre: String,     // nombre del campo generado: "sma_20"
    pub tipo: String,       // "SMA" | "EMA" | "RSI" | "MACD" | "BB"
    pub parametros: String, // JSON: {"period": 20} | {"fast": 12, "slow": 26}
}

// ─────────────────────────────────────────────────────────────────────────────
// 2. ESQUEMA SQLite RECOMENDADO
// (Ejecutar una vez para crear las tablas)
// ─────────────────────────────────────────────────────────────────────────────

pub fn crear_esquema(conn: &Connection) -> SqliteResult<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS strategies (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            id_user     INTEGER NOT NULL,
            nombre      TEXT NOT NULL,
            descripcion TEXT,
            activa      INTEGER DEFAULT 1,
            creada_en   TEXT DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS strategy_indicators (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            strategy_id  INTEGER NOT NULL REFERENCES strategies(id),
            nombre       TEXT NOT NULL,  -- nombre del campo: 'sma_20'
            tipo         TEXT NOT NULL,  -- 'SMA', 'EMA', 'RSI', 'MACD', 'BB'
            parametros   TEXT NOT NULL   -- JSON: '{\"period\": 20}'
        );

        CREATE TABLE IF NOT EXISTS strategy_actions (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            strategy_id  INTEGER NOT NULL REFERENCES strategies(id),
            tipo_signal  TEXT NOT NULL,  -- 'entry' | 'exit'
            tipo         TEXT NOT NULL,  -- 'buy', 'sell', 'close', 'set_sl', 'set_tp'
            parametro    TEXT DEFAULT '{}'  -- JSON con parámetros extra
        );

        CREATE TABLE IF NOT EXISTS strategy_conditions (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            strategy_id  INTEGER NOT NULL REFERENCES strategies(id),
            action_id    INTEGER NOT NULL REFERENCES strategy_actions(id),
            campo_a      TEXT NOT NULL,  -- 'close', 'sma_20', 'rsi'
            operador     TEXT NOT NULL,  -- '>', '<', '>=', '<=', '==', 'cross_above', 'cross_below'
            campo_b      TEXT NOT NULL,  -- 'sma_50' o valor literal '30.0'
            logica       TEXT DEFAULT 'NULL',
            orden        INTEGER DEFAULT 0
        );
    ",
    )
}

// ─────────────────────────────────────────────────────────────────────────────
// 3. CARGA DESDE SQLITE
// ─────────────────────────────────────────────────────────────────────────────

pub fn cargar_estrategia(conn: &Connection, strategy_id: i64) -> SqliteResult<Estrategia> {
    // Datos base de la estrategia
    let (nombre, descripcion) = conn.query_row(
        "SELECT nombre, descripcion FROM strategies WHERE id = ?1",
        [strategy_id],
        |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
    )?;

    // Indicadores
    let mut stmt = conn.prepare(
        "SELECT id, nombre, tipo, parametros
         FROM strategy_indicators WHERE strategy_id = ?1",
    )?;
    let indicadores: Vec<Indicador> = stmt
        .query_map([strategy_id], |row| {
            Ok(Indicador {
                id: row.get(0)?,
                nombre: row.get(1)?,
                tipo: row.get(2)?,
                parametros: row.get(3)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    // Condiciones de entrada
    let mut stmt = conn.prepare(
        "SELECT id, campo_a, operador, campo_b, logica, orden
         FROM strategy_conditions
         WHERE strategy_id = ?1 AND tipo = 'entry'
         ORDER BY orden",
    )?;
    let condiciones_entrada: Vec<Condicion> = stmt
        .query_map([strategy_id], |row| {
            Ok(Condicion {
                id: row.get(0)?,
                campo_a: row.get(1)?,
                operador: row.get(2)?,
                campo_b: row.get(3)?,
                logica: row.get(4)?,
                orden: row.get(5)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    // Condiciones de salida
    let mut stmt = conn.prepare(
        "SELECT id, campo_a, operador, campo_b, logica, orden
         FROM strategy_conditions
         WHERE strategy_id = ?1 AND tipo = 'exit'
         ORDER BY orden",
    )?;
    let condiciones_salida: Vec<Condicion> = stmt
        .query_map([strategy_id], |row| {
            Ok(Condicion {
                id: row.get(0)?,
                campo_a: row.get(1)?,
                operador: row.get(2)?,
                campo_b: row.get(3)?,
                logica: row.get(4)?,
                orden: row.get(5)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    // Acciones de entrada
    let mut stmt = conn.prepare(
        "SELECT id, tipo, parametro FROM strategy_actions
         WHERE strategy_id = ?1 AND tipo_signal = 'entry'",
    )?;
    let acciones_entrada: Vec<Accion> = stmt
        .query_map([strategy_id], |row| {
            Ok(Accion {
                id: row.get(0)?,
                tipo: row.get(1)?,
                parametro: row.get(2)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    // Acciones de salida
    let mut stmt = conn.prepare(
        "SELECT id, tipo, parametro FROM strategy_actions
         WHERE strategy_id = ?1 AND tipo_signal = 'exit'",
    )?;
    let acciones_salida: Vec<Accion> = stmt
        .query_map([strategy_id], |row| {
            Ok(Accion {
                id: row.get(0)?,
                tipo: row.get(1)?,
                parametro: row.get(2)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(Estrategia {
        id: strategy_id,
        nombre,
        descripcion,
        condiciones_entrada,
        condiciones_salida,
        acciones_entrada,
        acciones_salida,
        indicadores,
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// 4. MOTOR DE EVALUACIÓN DE CONDICIONES
// ─────────────────────────────────────────────────────────────────────────────

/// Contexto de mercado: todos los valores disponibles para evaluar condiciones
/// Incluye OHLCV + indicadores calculados (SMA, EMA, RSI...)
pub type ContextoMercado = HashMap<String, f64>;

/// Evalúa una condición contra el contexto de mercado actual
fn evaluar_condicion(cond: &Condicion, ctx: &ContextoMercado, ctx_prev: &ContextoMercado) -> bool {
    // Resolver campo_a (siempre un campo del contexto)
    let valor_a = match ctx.get(&cond.campo_a) {
        Some(v) => *v,
        None => return false, // campo no existe, condición no cumplida
    };

    // Resolver campo_b (puede ser un campo del contexto o un literal numérico)
    let valor_b = match ctx.get(&cond.campo_b) {
        Some(v) => *v,
        None => match cond.campo_b.parse::<f64>() {
            Ok(v) => v,
            Err(_) => return false,
        },
    };

    match cond.operador.as_str() {
        ">" => valor_a > valor_b,
        "<" => valor_a < valor_b,
        ">=" => valor_a >= valor_b,
        "<=" => valor_a <= valor_b,
        "==" => (valor_a - valor_b).abs() < f64::EPSILON,
        "!=" => (valor_a - valor_b).abs() >= f64::EPSILON,

        // Cruce al alza: en la vela anterior A estaba por debajo de B,
        // en la vela actual A está por encima de B
        "cross_above" => {
            let prev_a = ctx_prev.get(&cond.campo_a).copied().unwrap_or(valor_a);
            let prev_b = ctx_prev.get(&cond.campo_b).copied().unwrap_or(valor_b);
            prev_a <= prev_b && valor_a > valor_b
        }

        // Cruce a la baja
        "cross_below" => {
            let prev_a = ctx_prev.get(&cond.campo_a).copied().unwrap_or(valor_a);
            let prev_b = ctx_prev.get(&cond.campo_b).copied().unwrap_or(valor_b);
            prev_a >= prev_b && valor_a < valor_b
        }

        _ => false, // operador desconocido
    }
}

/// Evalúa una lista de condiciones respetando AND / OR
fn evaluar_condiciones(
    condiciones: &[Condicion],
    ctx: &ContextoMercado,
    ctx_prev: &ContextoMercado,
) -> bool {
    if condiciones.is_empty() {
        return false;
    }

    // La primera condición siempre se evalúa
    let mut resultado = evaluar_condicion(&condiciones[0], ctx, ctx_prev);

    for cond in condiciones.iter().skip(1) {
        let eval = evaluar_condicion(cond, ctx, ctx_prev);
        resultado = match cond.logica.as_str() {
            "OR" => resultado || eval,
            _ => resultado && eval, // AND por defecto
        };
    }

    resultado
}

// ─────────────────────────────────────────────────────────────────────────────
// 5. MOTOR DE INDICADORES
// ─────────────────────────────────────────────────────────────────────────────

/// Calcula todos los indicadores de la estrategia y los añade al contexto
fn calcular_indicadores(
    indicadores: &[Indicador],
    data: &BacktestData,
    i: usize,
    ctx: &mut ContextoMercado,
) {
    for ind in indicadores {
        let params: serde_json::Value =
            serde_json::from_str(&ind.parametros).unwrap_or(serde_json::json!({}));

        let valor = match ind.tipo.as_str() {
            "SMA" => {
                let period = params["period"].as_u64().unwrap_or(20) as usize;
                calcular_sma(&data.close, i, period)
            }
            "EMA" => {
                let period = params["period"].as_u64().unwrap_or(20) as usize;
                calcular_ema(&data.close, i, period)
            }
            "RSI" => {
                let period = params["period"].as_u64().unwrap_or(14) as usize;
                calcular_rsi(&data.close, i, period)
            }
            // Añade aquí más indicadores según los necesites:
            // "MACD" => calcular_macd(...),
            // "BB"   => calcular_bb(...),
            _ => None,
        };

        if let Some(v) = valor {
            ctx.insert(ind.nombre.clone(), v);
        }
    }
}

fn calcular_sma(close: &[f64], i: usize, period: usize) -> Option<f64> {
    if i + 1 < period {
        return None;
    }
    let sum: f64 = close[(i + 1 - period)..=i].iter().sum();
    Some(sum / period as f64)
}

fn calcular_ema(close: &[f64], i: usize, period: usize) -> Option<f64> {
    if i + 1 < period {
        return None;
    }
    let k = 2.0 / (period as f64 + 1.0);
    let mut ema = close[(i + 1 - period)..=i].iter().sum::<f64>() / period as f64;
    for j in (i + 1 - period + 1)..=i {
        ema = close[j] * k + ema * (1.0 - k);
    }
    Some(ema)
}

fn calcular_rsi(close: &[f64], i: usize, period: usize) -> Option<f64> {
    if i < period {
        return None;
    }
    let mut gains = 0.0_f64;
    let mut losses = 0.0_f64;
    for j in (i - period + 1)..=i {
        let diff = close[j] - close[j - 1];
        if diff > 0.0 {
            gains += diff;
        } else {
            losses -= diff;
        }
    }
    let avg_gain = gains / period as f64;
    let avg_loss = losses / period as f64;
    if avg_loss == 0.0 {
        return Some(100.0);
    }
    let rs = avg_gain / avg_loss;
    Some(100.0 - (100.0 / (1.0 + rs)))
}

// ─────────────────────────────────────────────────────────────────────────────
// 6. PLANTILLA PRINCIPAL DEL BACKTEST
// ─────────────────────────────────────────────────────────────────────────────

pub async fn run_backtest_desde_db(
    titulo: &str,
    balance: f64,
    tipo: &str,
    datos_path: &str,
    symbol: SymbolInfoCFD,
    strategy_id: i64, // ID de la estrategia en SQLite
    db_path: &str,    // ruta al archivo SQLite
) -> PolarsResult<()> {
    // ── Cargar estrategia desde SQLite ────────────────────────────
    let conn = Connection::open(db_path).expect("Error al abrir SQLite");
    let estrategia = cargar_estrategia(&conn, strategy_id).expect("Error al cargar estrategia");

    println!("Estrategia cargada: {}", estrategia.nombre);

    // ── Setup del backtest (igual que antes) ──────────────────────
    let mut backtest = Backtest::new(titulo.to_string(), balance, tipo.to_string()).await;
    let df = backtest
        .add_datos(datos_path)
        .expect("Error al cargar datos");
    let data = prepare_backtest_slices(&df.get_datos())?;

    let mut trade = Trade::new(backtest.get_id(), symbol).await;
    let mut in_position = false;
    let mut trade_type = String::from("None");

    // Contexto actual y previo para evaluar condiciones y cruces
    let mut ctx_actual: ContextoMercado = HashMap::new();
    let mut ctx_previo: ContextoMercado = HashMap::new();

    println!("Iniciando Backtest dinámico...");
    let inicio = std::time::Instant::now();

    for i in 0..data.close.len() {
        let close = data.close[i];
        let high = data.high[i];
        let low = data.low[i];
        let open = data.open[i];
        let time = data.time[i];

        let naive_time = chrono::DateTime::from_timestamp_millis(time).expect("timestamp inválido");
        let time_str = naive_time.format("%Y-%m-%d %H:%M:%S").to_string();

        // ── Guardar contexto previo y construir el actual ─────────
        ctx_previo = ctx_actual.clone();

        ctx_actual.clear();
        ctx_actual.insert("open".to_string(), open);
        ctx_actual.insert("high".to_string(), high);
        ctx_actual.insert("low".to_string(), low);
        ctx_actual.insert("close".to_string(), close);

        // Añadir velas anteriores al contexto (para lookback)
        if i > 0 {
            ctx_actual.insert("prev_close".to_string(), data.close[i - 1]);
            ctx_actual.insert("prev_high".to_string(), data.high[i - 1]);
            ctx_actual.insert("prev_low".to_string(), data.low[i - 1]);
        }

        // ── Calcular indicadores de la estrategia ─────────────────
        calcular_indicadores(&estrategia.indicadores, &data, i, &mut ctx_actual);

        // ── Lógica de entrada ─────────────────────────────────────
        if !in_position {
            if evaluar_condiciones(&estrategia.condiciones_entrada, &ctx_actual, &ctx_previo) {
                // Ejecutar acciones de entrada
                for accion in &estrategia.acciones_entrada {
                    let params: serde_json::Value =
                        serde_json::from_str(&accion.parametro).unwrap_or(serde_json::json!({}));

                    let lot = params["lot"].as_f64().unwrap_or(1.0);
                    let sl = params["sl"].as_f64().unwrap_or(0.0);
                    let tp = params["tp"].as_f64().unwrap_or(0.0);
                    let sl_pct = params["sl_pct"].as_f64().unwrap_or(0.0);
                    let tp_pct = params["tp_pct"].as_f64().unwrap_or(0.0);

                    // Calcular SL/TP dinámicos si se usan porcentajes
                    let sl_final = if sl_pct > 0.0 {
                        close * (1.0 - sl_pct / 100.0)
                    } else {
                        sl
                    };
                    let tp_final = if tp_pct > 0.0 {
                        close * (1.0 + tp_pct / 100.0)
                    } else {
                        tp
                    };

                    match accion.tipo.as_str() {
                        "buy" => {
                            trade.buy(
                                0.0,
                                lot,
                                time_str.clone(),
                                close,
                                tp_final,
                                sl_final,
                                &backtest,
                            );
                            trade_type = "Buy".to_string();
                            in_position = true;
                        }
                        "sell" => {
                            trade.sell(
                                0.0,
                                lot,
                                time_str.clone(),
                                close,
                                tp_final,
                                sl_final,
                                &backtest,
                            );
                            trade_type = "Sell".to_string();
                            in_position = true;
                        }
                        _ => {}
                    }
                }
            }

        // ── Lógica de salida ──────────────────────────────────────
        } else if evaluar_condiciones(&estrategia.condiciones_salida, &ctx_actual, &ctx_previo) {
            for accion in &estrategia.acciones_salida {
                if accion.tipo == "close" {
                    trade.close(time_str.clone(), close);
                    backtest.add_trade(trade.clone());
                    in_position = false;
                    trade_type = "None".to_string();
                }
            }
        }
    }

    let duracion = inicio.elapsed();
    println!(
        "Backtest '{}' finalizado en {:?}",
        estrategia.nombre, duracion
    );

    backtest.guardar_trades().await;

    let mut resultados = Resultados::new(backtest.get_id()).await;
    resultados.calcular_resultados(backtest.get_trades().clone(), backtest.get_balance());
    resultados.guardar_resultados().await;

    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// 7. EJEMPLO: INSERTAR UNA ESTRATEGIA DESDE RETE.JS
//
// Rete.js genera un JSON así:
// {
//   "nombre": "SMA Crossover",
//   "indicadores": [
//     { "nombre": "sma_20", "tipo": "SMA", "parametros": {"period": 20} },
//     { "nombre": "sma_50", "tipo": "SMA", "parametros": {"period": 50} }
//   ],
//   "condiciones_entrada": [
//     { "campo_a": "sma_20", "operador": "cross_above", "campo_b": "sma_50", "logica": "AND", "orden": 0 }
//   ],
//   "condiciones_salida": [
//     { "campo_a": "sma_20", "operador": "cross_below", "campo_b": "sma_50", "logica": "AND", "orden": 0 }
//   ],
//   "acciones_entrada": [
//     { "tipo": "buy", "parametro": {"lot": 1.0, "sl_pct": 2.0, "tp_pct": 4.0} }
//   ],
//   "acciones_salida": [
//     { "tipo": "close", "parametro": {} }
//   ]
// }
//
// Y desde Rust lo insertas en SQLite así:
// ─────────────────────────────────────────────────────────────────────────────

pub fn insertar_estrategia_desde_json(conn: &Connection, json_str: &str) -> SqliteResult<i64> {
    let json: serde_json::Value = serde_json::from_str(json_str).expect("JSON inválido");

    // Insertar estrategia base
    conn.execute(
        "INSERT INTO strategies (nombre, descripcion) VALUES (?1, ?2)",
        rusqlite::params![
            json["nombre"].as_str().unwrap_or("Sin nombre"),
            json["descripcion"].as_str().unwrap_or(""),
        ],
    )?;
    let strategy_id = conn.last_insert_rowid();

    // Insertar indicadores
    if let Some(inds) = json["indicadores"].as_array() {
        for ind in inds {
            conn.execute(
                "INSERT INTO strategy_indicators (strategy_id, nombre, tipo, parametros)
                 VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params![
                    strategy_id,
                    ind["nombre"].as_str().unwrap_or(""),
                    ind["tipo"].as_str().unwrap_or(""),
                    ind["parametros"].to_string(),
                ],
            )?;
        }
    }

    // Insertar condiciones de entrada
    if let Some(conds) = json["condiciones_entrada"].as_array() {
        for cond in conds {
            conn.execute(
                "INSERT INTO strategy_conditions
                 (strategy_id, tipo, campo_a, operador, campo_b, logica, orden)
                 VALUES (?1, 'entry', ?2, ?3, ?4, ?5, ?6)",
                rusqlite::params![
                    strategy_id,
                    cond["campo_a"].as_str().unwrap_or(""),
                    cond["operador"].as_str().unwrap_or(""),
                    cond["campo_b"].as_str().unwrap_or(""),
                    cond["logica"].as_str().unwrap_or("AND"),
                    cond["orden"].as_i64().unwrap_or(0),
                ],
            )?;
        }
    }

    // Insertar condiciones de salida
    if let Some(conds) = json["condiciones_salida"].as_array() {
        for cond in conds {
            conn.execute(
                "INSERT INTO strategy_conditions
                 (strategy_id, tipo, campo_a, operador, campo_b, logica, orden)
                 VALUES (?1, 'exit', ?2, ?3, ?4, ?5, ?6)",
                rusqlite::params![
                    strategy_id,
                    cond["campo_a"].as_str().unwrap_or(""),
                    cond["operador"].as_str().unwrap_or(""),
                    cond["campo_b"].as_str().unwrap_or(""),
                    cond["logica"].as_str().unwrap_or("AND"),
                    cond["orden"].as_i64().unwrap_or(0),
                ],
            )?;
        }
    }

    // Insertar acciones de entrada
    if let Some(acts) = json["acciones_entrada"].as_array() {
        for act in acts {
            conn.execute(
                "INSERT INTO strategy_actions (strategy_id, tipo_signal, tipo, parametro)
                 VALUES (?1, 'entry', ?2, ?3)",
                rusqlite::params![
                    strategy_id,
                    act["tipo"].as_str().unwrap_or(""),
                    act["parametro"].to_string(),
                ],
            )?;
        }
    }

    // Insertar acciones de salida
    if let Some(acts) = json["acciones_salida"].as_array() {
        for act in acts {
            conn.execute(
                "INSERT INTO strategy_actions (strategy_id, tipo_signal, tipo, parametro)
                 VALUES (?1, 'exit', ?2, ?3)",
                rusqlite::params![
                    strategy_id,
                    act["tipo"].as_str().unwrap_or(""),
                    act["parametro"].to_string(),
                ],
            )?;
        }
    }

    Ok(strategy_id)
}

// ─────────────────────────────────────────────────────────────────────────────
// DEPENDENCIAS EN Cargo.toml:
// ─────────────────────────────────────────────────────────────────────────────
//
// [dependencies]
// rusqlite  = { version = "0.31", features = ["bundled"] }
// serde     = { version = "1",    features = ["derive"] }
// serde_json = "1"
// chrono    = "0.4"
// polars    = { version = "0.41", features = ["lazy"] }
// tokio     = { version = "1",    features = ["full"] }
