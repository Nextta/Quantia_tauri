use crate::enums::actions::Action;
use crate::enums::chart_type::ChartType;
use crate::enums::logics::Logic;
use crate::strategy::strategy::Strategy;
use crate::strategy::strategy_action::StrategyAction;
use crate::strategy::strategy_condition::StrategyCondition;
use crate::strategy::strategy_indicator::StrategyIndicator;
use crate::strategy::strategy_options::{StopLoss, StrategyOptions, TakeProfit, TradingDirection};
use crate::utils::configuracion::DB_LOCAL;
use crate::utils::configuracion::{get_db_config, Error};
use crate::utils::tools::add_condition;
use chrono::DateTime;
use chrono::Utc;
use libsql::{params, Builder};

//================================Strategies================================

/// Crea la tabla de estrategias en la base de datos.
///
/// # Returns
/// * `Result<String>` - Mensaje de éxito si la tabla se crea correctamente.
///
/// # Errores
/// Retorna error si falla la conexión a la base de datos o la ejecución de la query.
#[tauri::command]
pub async fn table_strategies() -> Result<String, Error> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = if !DB_LOCAL {
        Builder::new_remote_replica(db_path, sync_url, auth_token)
            .build()
            .await?
    } else {
        Builder::new_local(db_path).build().await?
    };

    let conn = db.connect()?;

    conn.query(
        "CREATE TABLE IF NOT EXISTS strategies (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            id_user     INTEGER NOT NULL,
            nombre      TEXT NOT NULL,
            descripcion TEXT,
            activa      INTEGER DEFAULT 1,
            creada_en   TEXT DEFAULT (datetime('now'))
        )",
        (),
    )
    .await?;

    Ok("Tabla strategies is ok.".to_string())
}

/// Inserta una nueva estrategia en la base de datos.
///
/// # Parámetros
/// * `strategy`: Objeto Strategy con los datos de la estrategia a insertar.
///
/// # Returns
/// * `Result<i32>` - ID de la estrategia insertada.
///
/// # Errores
/// Retorna error si falla la conexión a la base de datos o la inserción.
#[tauri::command]
pub async fn insert_strategies(strategy: &Strategy) -> Result<i32, Error> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = if !DB_LOCAL {
        Builder::new_remote_replica(db_path, sync_url, auth_token)
            .build()
            .await?
    } else {
        Builder::new_local(db_path).build().await?
    };

    let conn = db.connect()?;

    let parametros = params![
        strategy.id_user,
        strategy.nombre.clone(),
        strategy.descripcion.clone(),
        strategy.activa,
        strategy.creada_en.clone()
    ];
    conn.query(
        "INSERT INTO strategies (id_user, nombre, descripcion, activa, creada_en) VALUES (?, ?, ?, ?, ?) RETURNING id",
        parametros,
    )
    .await?;

    let id = conn.last_insert_rowid() as i32;
    Ok(id)
}

/// Obtiene todas las estrategias de la base de datos, incluyendo sus indicadores, condiciones y acciones.
///
/// # Returns
/// * `Result<Vec<Strategy>>` - Vector de estrategias con todos sus componentes.
///
/// # Errores
/// Retorna error si falla la conexión a la base de datos o la consulta.
#[tauri::command]
pub async fn get_strategies() -> Result<Vec<Strategy>, Error> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = if !DB_LOCAL {
        Builder::new_remote_replica(db_path, sync_url, auth_token)
            .build()
            .await?
    } else {
        Builder::new_local(db_path).build().await?
    };

    let conn = db.connect()?;

    let sql = "SELECT * FROM strategies";

    let mut result = conn.query(sql, ()).await?;

    let mut strategies = Vec::new();
    while let Some(row) = result.next().await? {
        let mut str_indicators: Vec<StrategyIndicator> = Vec::<StrategyIndicator>::new();
        // let mut str_conditions: Vec<StrategyCondition> = Vec::<StrategyCondition>::new();
        let mut str_actions: Vec<StrategyAction> = Vec::<StrategyAction>::new();
        let mut str_options: StrategyOptions = StrategyOptions::new_empty();

        let id_startegy = row.get::<i32>(0)?;

        match get_strategies_indicators_by_strategy_id(id_startegy).await {
            Ok(indicators) => str_indicators = indicators,
            Err(e) => println!(
                "Error al optener los indicadores de strategy_id: {}. Error: {:?}",
                id_startegy, e
            ),
        }

        match get_strategies_actions_by_strategy_id(id_startegy).await {
            Ok(actions) => str_actions = actions,
            Err(e) => println!(
                "Error al optener los actions de strategy_id: {}. Error: {:?}",
                id_startegy, e
            ),
        }

        match get_strategy_options_by_strategy_id(id_startegy).await {
            Ok(options) => str_options = options,
            Err(e) => println!(
                "Error al optener los options de strategy_id: {}. Error: {:?}",
                id_startegy, e
            ),
        }

        let mut activa = false;
        if row.get::<i32>(4)? != 0 {
            activa = true;
        }
        let strategy = Strategy {
            id: id_startegy,
            id_user: row.get::<i32>(1)?,
            nombre: row.get::<String>(2)?,
            descripcion: row.get::<Option<String>>(3)?,
            activa: activa,
            creada_en: row.get::<String>(5)?,
            indicadores: str_indicators,
            acciones: str_actions,
            opciones: str_options,
        };
        strategies.push(strategy);
    }

    Ok(strategies)
}

/// Obtiene una estrategia específica por su ID.
///
/// # Parámetros
/// * `id`: ID de la estrategia a buscar.
///
/// # Returns
/// * `Result<Strategy>` - Estrategia encontrada con todos sus componentes.
///
/// # Errores
/// Retorna error si no se encuentra la estrategia o falla la conexión.
#[tauri::command]
pub async fn get_strategies_by_id(id: i32) -> Result<Strategy, Error> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = if !DB_LOCAL {
        Builder::new_remote_replica(db_path, sync_url, auth_token)
            .build()
            .await?
    } else {
        Builder::new_local(db_path).build().await?
    };

    let conn = db.connect()?;

    let sql = "SELECT * FROM strategies WHERE id = ?";

    let parametros = params![id];

    let mut result = conn.query(sql, parametros).await?;

    let row = result.next().await?.unwrap();

    let mut str_indicators: Vec<StrategyIndicator> = Vec::<StrategyIndicator>::new();
    let mut str_actions: Vec<StrategyAction> = Vec::<StrategyAction>::new();
    let mut str_options: StrategyOptions = StrategyOptions::new_empty();

    let id_startegy = row.get::<i32>(0)?;

    match get_strategies_indicators_by_strategy_id(id_startegy).await {
        Ok(indicators) => str_indicators = indicators,
        Err(e) => println!(
            "Error al optener los indicadores de strategy_id: {}. Error: {:?}",
            id_startegy, e
        ),
    }

    match get_strategies_actions_by_strategy_id(id_startegy).await {
        Ok(actions) => str_actions = actions,
        Err(e) => println!(
            "Error al optener los actions de strategy_id: {}. Error: {:?}",
            id_startegy, e
        ),
    }

    match get_strategy_options_by_strategy_id(id_startegy).await {
        Ok(options) => str_options = options,
        Err(e) => println!(
            "Error al optener los options de strategy_id: {}. Error: {:?}",
            id_startegy, e
        ),
    }

    let mut activa = false;
    if row.get::<i32>(4)? != 0 {
        activa = true;
    }
    let strategy = Strategy {
        id: id_startegy,
        id_user: row.get::<i32>(1)?,
        nombre: row.get::<String>(2)?,
        descripcion: row.get::<Option<String>>(3)?,
        activa: activa,
        creada_en: row.get::<String>(5)?,
        indicadores: str_indicators,
        acciones: str_actions,
        opciones: str_options,
    };

    Ok(strategy)
}

/// Obtiene todas las estrategias activas de un usuario específico.
///
/// # Parámetros
/// * `id_user`: ID del usuario propietario de las estrategias.
///
/// # Returns
/// * `Result<Vec<Strategy>>` - Vector de estrategias activas del usuario.
///
/// # Errores
/// Retorna error si falla la conexión a la base de datos o la consulta.
#[tauri::command]
pub async fn get_strategies_by_id_user(id_user: i32) -> Result<Vec<Strategy>, Error> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = if !DB_LOCAL {
        Builder::new_remote_replica(db_path, sync_url, auth_token)
            .build()
            .await?
    } else {
        Builder::new_local(db_path).build().await?
    };

    let conn = db.connect()?;

    let parametros = params![id_user];

    let sql = "SELECT * FROM strategies WHERE id_user = ?  AND activa = 1";

    let mut result = conn.query(sql, parametros).await?;

    let mut strategies = Vec::new();
    while let Some(row) = result.next().await? {
        let mut str_indicators: Vec<StrategyIndicator> = Vec::<StrategyIndicator>::new();
        let mut str_actions: Vec<StrategyAction> = Vec::<StrategyAction>::new();
        let mut str_options: StrategyOptions = StrategyOptions::new_empty();

        let id_startegy = row.get::<i32>(0)?;

        match get_strategies_indicators_by_strategy_id(id_startegy).await {
            Ok(indicators) => str_indicators = indicators,
            Err(e) => println!(
                "Error al optener los indicadores de strategy_id: {}. Error: {:?}",
                id_startegy, e
            ),
        }

        match get_strategies_actions_by_strategy_id(id_startegy).await {
            Ok(actions) => str_actions = actions,
            Err(e) => println!(
                "Error al optener los actions de strategy_id: {}. Error: {:?}",
                id_startegy, e
            ),
        }

        match get_strategy_options_by_strategy_id(id_startegy).await {
            Ok(options) => str_options = options,
            Err(e) => println!(
                "Error al optener los options de strategy_id: {}. Error: {:?}",
                id_startegy, e
            ),
        }

        let mut activa = false;
        if row.get::<i32>(4)? != 0 {
            activa = true;
        }
        let strategy = Strategy {
            id: id_startegy,
            id_user: row.get::<i32>(1)?,
            nombre: row.get::<String>(2)?,
            descripcion: row.get::<Option<String>>(3)?,
            activa: activa,
            creada_en: row.get::<String>(5)?,
            indicadores: str_indicators,
            acciones: str_actions,
            opciones: str_options,
        };
        strategies.push(strategy);
    }

    Ok(strategies)
}

/// Obtiene estrategias por nombre que estén activas.
///
/// # Parámetros
/// * `nombre`: Nombre de la estrategia a buscar.
///
/// # Returns
/// * `Result<Vec<Strategy>>` - Vector de estrategias que coinciden con el nombre.
///
/// # Errores
/// Retorna error si falla la conexión a la base de datos o la consulta.
#[tauri::command]
pub async fn get_strategies_by_nombre(nombre: String) -> Result<Vec<Strategy>, Error> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = if !DB_LOCAL {
        Builder::new_remote_replica(db_path, sync_url, auth_token)
            .build()
            .await?
    } else {
        Builder::new_local(db_path).build().await?
    };

    let conn = db.connect()?;

    let parametros = params![nombre];

    let sql = "SELECT * FROM strategies WHERE nombre = ?  AND activa = 1";

    let mut result = conn.query(sql, parametros).await?;

    let mut strategies = Vec::new();
    while let Some(row) = result.next().await? {
        let mut str_indicators: Vec<StrategyIndicator> = Vec::<StrategyIndicator>::new();
        // let mut str_conditions: Vec<StrategyCondition> = Vec::<StrategyCondition>::new();
        let mut str_actions: Vec<StrategyAction> = Vec::<StrategyAction>::new();
        let mut str_options: StrategyOptions = StrategyOptions::new_empty();

        let id_startegy = row.get::<i32>(0)?;

        match get_strategies_indicators_by_strategy_id(id_startegy).await {
            Ok(indicators) => str_indicators = indicators,
            Err(e) => println!(
                "Error al optener los indicadores de strategy_id: {}. Error: {:?}",
                id_startegy, e
            ),
        }

        // match get_strategies_conditions_by_strategy_id(id_startegy).await {
        //     Ok(conditions) => str_conditions = conditions,
        //     Err(e) => println!(
        //         "Error al optener los conditions de strategy_id: {}. Error: {:?}",
        //         id_startegy, e
        //     ),
        // }

        match get_strategies_actions_by_strategy_id(id_startegy).await {
            Ok(actions) => str_actions = actions,
            Err(e) => println!(
                "Error al optener los actions de strategy_id: {}. Error: {:?}",
                id_startegy, e
            ),
        }

        match get_strategy_options_by_strategy_id(id_startegy).await {
            Ok(options) => str_options = options,
            Err(e) => println!(
                "Error al optener los options de strategy_id: {}. Error: {:?}",
                id_startegy, e
            ),
        }

        let mut activa = false;
        if row.get::<i32>(4)? != 0 {
            activa = true;
        }
        let strategy = Strategy {
            id: id_startegy,
            id_user: row.get::<i32>(1)?,
            nombre: row.get::<String>(2)?,
            descripcion: row.get::<Option<String>>(3)?,
            activa: activa,
            creada_en: row.get::<String>(5)?,
            indicadores: str_indicators,
            // condiciones: str_conditions,
            acciones: str_actions,
            opciones: str_options,
        };
        strategies.push(strategy);
    }

    Ok(strategies)
}

/// Obtiene todas las estrategias activas de un usuario específico.
///
/// # Parámetros
/// * `id_user`: ID del usuario propietario de las estrategias.
///
/// # Returns
/// * `Result<Vec<Strategy>>` - Vector de estrategias activas del usuario.
///
/// # Errores
/// Retorna error si falla la conexión a la base de datos o la consulta.
#[tauri::command]
pub async fn get_active_strategies_by_user(id_user: i32) -> Result<Vec<Strategy>, Error> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = if !DB_LOCAL {
        Builder::new_remote_replica(db_path, sync_url, auth_token)
            .build()
            .await?
    } else {
        Builder::new_local(db_path).build().await?
    };

    let conn = db.connect()?;

    let parametros = params![id_user];

    let sql = "SELECT * FROM strategies WHERE id_user = ? AND activa = 1";

    let mut result = conn.query(sql, parametros).await?;

    let mut strategies = Vec::new();
    while let Some(row) = result.next().await? {
        let mut str_indicators: Vec<StrategyIndicator> = Vec::<StrategyIndicator>::new();
        // let mut str_conditions: Vec<StrategyCondition> = Vec::<StrategyCondition>::new();
        let mut str_actions: Vec<StrategyAction> = Vec::<StrategyAction>::new();
        let mut str_options: StrategyOptions = StrategyOptions::new_empty();

        let id_startegy = row.get::<i32>(0)?;

        match get_strategies_indicators_by_strategy_id(id_startegy).await {
            Ok(indicators) => str_indicators = indicators,
            Err(e) => println!(
                "Error al optener los indicadores de strategy_id: {}. Error: {:?}",
                id_startegy, e
            ),
        }

        // match get_strategies_conditions_by_strategy_id(id_startegy).await {
        //     Ok(conditions) => str_conditions = conditions,
        //     Err(e) => println!(
        //         "Error al optener los conditions de strategy_id: {}. Error: {:?}",
        //         id_startegy, e
        //     ),
        // }

        match get_strategies_actions_by_strategy_id(id_startegy).await {
            Ok(actions) => str_actions = actions,
            Err(e) => println!(
                "Error al optener los actions de strategy_id: {}. Error: {:?}",
                id_startegy, e
            ),
        }

        match get_strategy_options_by_strategy_id(id_startegy).await {
            Ok(options) => str_options = options,
            Err(e) => println!(
                "Error al optener los options de strategy_id: {}. Error: {:?}",
                id_startegy, e
            ),
        }

        let mut activa = false;
        if row.get::<i32>(4)? != 0 {
            activa = true;
        }
        let strategy = Strategy {
            id: id_startegy,
            id_user: row.get::<i32>(1)?,
            nombre: row.get::<String>(2)?,
            descripcion: row.get::<Option<String>>(3)?,
            activa: activa,
            creada_en: row.get::<String>(5)?,
            indicadores: str_indicators,
            // condiciones: str_conditions,
            acciones: str_actions,
            opciones: str_options,
        };
        strategies.push(strategy);
    }

    Ok(strategies)
}

/// Obtiene todas las estrategias activas de la base de datos.
///
/// # Returns
/// * `Result<Vec<Strategy>>` - Vector de todas las estrategias activas.
///
/// # Errores
/// Retorna error si falla la conexión a la base de datos o la consulta.
#[tauri::command]
pub async fn get_all_active_strategies() -> Result<Vec<Strategy>, Error> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = if !DB_LOCAL {
        Builder::new_remote_replica(db_path, sync_url, auth_token)
            .build()
            .await?
    } else {
        Builder::new_local(db_path).build().await?
    };

    let conn = db.connect()?;

    // let parametros = params![id_user];

    let sql = "SELECT * FROM strategies WHERE activa = 1";

    let mut result = conn.query(sql, ()).await?;

    let mut strategies = Vec::new();
    while let Some(row) = result.next().await? {
        let mut str_indicators: Vec<StrategyIndicator> = Vec::<StrategyIndicator>::new();
        // let mut str_conditions: Vec<StrategyCondition> = Vec::<StrategyCondition>::new();
        let mut str_actions: Vec<StrategyAction> = Vec::<StrategyAction>::new();
        let mut str_options: StrategyOptions = StrategyOptions::new_empty();

        let id_startegy = row.get::<i32>(0)?;

        match get_strategies_indicators_by_strategy_id(id_startegy).await {
            Ok(indicators) => str_indicators = indicators,
            Err(e) => println!(
                "Error al optener los indicadores de strategy_id: {}. Error: {:?}",
                id_startegy, e
            ),
        }

        // match get_strategies_conditions_by_strategy_id(id_startegy).await {
        //     Ok(conditions) => str_conditions = conditions,
        //     Err(e) => println!(
        //         "Error al optener los conditions de strategy_id: {}. Error: {:?}",
        //         id_startegy, e
        //     ),
        // }

        match get_strategies_actions_by_strategy_id(id_startegy).await {
            Ok(actions) => str_actions = actions,
            Err(e) => println!(
                "Error al optener los actions de strategy_id: {}. Error: {:?}",
                id_startegy, e
            ),
        }

        match get_strategy_options_by_strategy_id(id_startegy).await {
            Ok(options) => str_options = options,
            Err(e) => println!(
                "Error al optener los options de strategy_id: {}. Error: {:?}",
                id_startegy, e
            ),
        }

        let mut activa = false;
        if row.get::<i32>(4)? != 0 {
            activa = true;
        }
        let strategy = Strategy {
            id: id_startegy,
            id_user: row.get::<i32>(1)?,
            nombre: row.get::<String>(2)?,
            descripcion: row.get::<Option<String>>(3)?,
            activa: activa,
            creada_en: row.get::<String>(5)?,
            indicadores: str_indicators,
            // condiciones: str_conditions,
            acciones: str_actions,
            opciones: str_options,
        };
        strategies.push(strategy);
    }

    Ok(strategies)
}

/// Obtiene todas las estrategias desactivadas de un usuario específico.
///
/// # Parámetros
/// * `id_user`: ID del usuario propietario de las estrategias.
///
/// # Returns
/// * `Result<Vec<Strategy>>` - Vector de estrategias desactivadas del usuario.
///
/// # Errores
/// Retorna error si falla la conexión a la base de datos o la consulta.
#[tauri::command]
pub async fn get_desactive_strategies_by_user(id_user: i32) -> Result<Vec<Strategy>, Error> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = if !DB_LOCAL {
        Builder::new_remote_replica(db_path, sync_url, auth_token)
            .build()
            .await?
    } else {
        Builder::new_local(db_path).build().await?
    };

    let conn = db.connect()?;

    let parametros = params![id_user];

    let sql = "SELECT * FROM strategies WHERE id_user = ? AND activa = 0";

    let mut result = conn.query(sql, parametros).await?;

    let mut strategies = Vec::new();
    while let Some(row) = result.next().await? {
        let mut str_indicators: Vec<StrategyIndicator> = Vec::<StrategyIndicator>::new();
        // let mut str_conditions: Vec<StrategyCondition> = Vec::<StrategyCondition>::new();
        let mut str_actions: Vec<StrategyAction> = Vec::<StrategyAction>::new();
        let mut str_options: StrategyOptions = StrategyOptions::new_empty();

        let id_startegy = row.get::<i32>(0)?;

        match get_strategies_indicators_by_strategy_id(id_startegy).await {
            Ok(indicators) => str_indicators = indicators,
            Err(e) => println!(
                "Error al optener los indicadores de strategy_id: {}. Error: {:?}",
                id_startegy, e
            ),
        }

        // match get_strategies_conditions_by_strategy_id(id_startegy).await {
        //     Ok(conditions) => str_conditions = conditions,
        //     Err(e) => println!(
        //         "Error al optener los conditions de strategy_id: {}. Error: {:?}",
        //         id_startegy, e
        //     ),
        // }

        match get_strategies_actions_by_strategy_id(id_startegy).await {
            Ok(actions) => str_actions = actions,
            Err(e) => println!(
                "Error al optener los actions de strategy_id: {}. Error: {:?}",
                id_startegy, e
            ),
        }

        match get_strategy_options_by_strategy_id(id_startegy).await {
            Ok(options) => str_options = options,
            Err(e) => println!(
                "Error al optener los options de strategy_id: {}. Error: {:?}",
                id_startegy, e
            ),
        }

        let mut activa = false;
        if row.get::<i32>(4)? != 0 {
            activa = true;
        }
        let strategy = Strategy {
            id: id_startegy,
            id_user: row.get::<i32>(1)?,
            nombre: row.get::<String>(2)?,
            descripcion: row.get::<Option<String>>(3)?,
            activa: activa,
            creada_en: row.get::<String>(5)?,
            indicadores: str_indicators,
            // condiciones: str_conditions,
            acciones: str_actions,
            opciones: str_options,
        };
        strategies.push(strategy);
    }

    Ok(strategies)
}

/// Obtiene todas las estrategias desactivadas de la base de datos.
///
/// # Returns
/// * `Result<Vec<Strategy>>` - Vector de todas las estrategias desactivadas.
///
/// # Errores
/// Retorna error si falla la conexión a la base de datos o la consulta.
#[tauri::command]
pub async fn get_all_desactive_strategies() -> Result<Vec<Strategy>, Error> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = if !DB_LOCAL {
        Builder::new_remote_replica(db_path, sync_url, auth_token)
            .build()
            .await?
    } else {
        Builder::new_local(db_path).build().await?
    };

    let conn = db.connect()?;

    // let parametros = params![id_user];

    let sql = "SELECT * FROM strategies WHERE activa = 0";

    let mut result = conn.query(sql, ()).await?;

    let mut strategies = Vec::new();
    while let Some(row) = result.next().await? {
        let mut str_indicators: Vec<StrategyIndicator> = Vec::<StrategyIndicator>::new();
        // let mut str_conditions: Vec<StrategyCondition> = Vec::<StrategyCondition>::new();
        let mut str_actions: Vec<StrategyAction> = Vec::<StrategyAction>::new();
        let mut str_options: StrategyOptions = StrategyOptions::new_empty();

        let id_startegy = row.get::<i32>(0)?;

        match get_strategies_indicators_by_strategy_id(id_startegy).await {
            Ok(indicators) => str_indicators = indicators,
            Err(e) => println!(
                "Error al optener los indicadores de strategy_id: {}. Error: {:?}",
                id_startegy, e
            ),
        }

        // match get_strategies_conditions_by_strategy_id(id_startegy).await {
        //     Ok(conditions) => str_conditions = conditions,
        //     Err(e) => println!(
        //         "Error al optener los conditions de strategy_id: {}. Error: {:?}",
        //         id_startegy, e
        //     ),
        // }

        match get_strategies_actions_by_strategy_id(id_startegy).await {
            Ok(actions) => str_actions = actions,
            Err(e) => println!(
                "Error al optener los actions de strategy_id: {}. Error: {:?}",
                id_startegy, e
            ),
        }

        match get_strategy_options_by_strategy_id(id_startegy).await {
            Ok(options) => str_options = options,
            Err(e) => println!(
                "Error al optener los options de strategy_id: {}. Error: {:?}",
                id_startegy, e
            ),
        }

        let mut activa = false;
        if row.get::<i32>(4)? != 0 {
            activa = true;
        }
        let strategy = Strategy {
            id: id_startegy,
            id_user: row.get::<i32>(1)?,
            nombre: row.get::<String>(2)?,
            descripcion: row.get::<Option<String>>(3)?,
            activa: activa,
            creada_en: row.get::<String>(5)?,
            indicadores: str_indicators,
            // condiciones: str_conditions,
            acciones: str_actions,
            opciones: str_options,
        };
        strategies.push(strategy);
    }

    Ok(strategies)
}

/// Obtiene estrategias activas por fecha de creación.
///
/// # Parámetros
/// * `fecha`: Fecha de creación en formato texto.
///
/// # Returns
/// * `Result<Vec<Strategy>>` - Vector de estrategias activas creadas en esa fecha.
///
/// # Errores
/// Retorna error si falla la conexión a la base de datos o la consulta.
#[tauri::command]
pub async fn get_active_strategies_by_date(fecha: String) -> Result<Vec<Strategy>, Error> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = if !DB_LOCAL {
        Builder::new_remote_replica(db_path, sync_url, auth_token)
            .build()
            .await?
    } else {
        Builder::new_local(db_path).build().await?
    };

    let conn = db.connect()?;

    let parametros = params![fecha];

    let sql = "SELECT * FROM strategies WHERE creada_en = ? AND activa = 1";

    let mut result = conn.query(sql, parametros).await?;

    let mut strategies = Vec::new();
    while let Some(row) = result.next().await? {
        let mut str_indicators: Vec<StrategyIndicator> = Vec::<StrategyIndicator>::new();
        // let mut str_conditions: Vec<StrategyCondition> = Vec::<StrategyCondition>::new();
        let mut str_actions: Vec<StrategyAction> = Vec::<StrategyAction>::new();
        let mut str_options: StrategyOptions = StrategyOptions::new_empty();

        let id_startegy = row.get::<i32>(0)?;

        match get_strategies_indicators_by_strategy_id(id_startegy).await {
            Ok(indicators) => str_indicators = indicators,
            Err(e) => println!(
                "Error al optener los indicadores de strategy_id: {}. Error: {:?}",
                id_startegy, e
            ),
        }

        // match get_strategies_conditions_by_strategy_id(id_startegy).await {
        //     Ok(conditions) => str_conditions = conditions,
        //     Err(e) => println!(
        //         "Error al optener los conditions de strategy_id: {}. Error: {:?}",
        //         id_startegy, e
        //     ),
        // }

        match get_strategies_actions_by_strategy_id(id_startegy).await {
            Ok(actions) => str_actions = actions,
            Err(e) => println!(
                "Error al optener los actions de strategy_id: {}. Error: {:?}",
                id_startegy, e
            ),
        }

        match get_strategy_options_by_strategy_id(id_startegy).await {
            Ok(options) => str_options = options,
            Err(e) => println!(
                "Error al optener los options de strategy_id: {}. Error: {:?}",
                id_startegy, e
            ),
        }

        let mut activa = false;
        if row.get::<i32>(4)? != 0 {
            activa = true;
        }
        let strategy = Strategy {
            id: id_startegy,
            id_user: row.get::<i32>(1)?,
            nombre: row.get::<String>(2)?,
            descripcion: row.get::<Option<String>>(3)?,
            activa: activa,
            creada_en: row.get::<String>(5)?,
            indicadores: str_indicators,
            // condiciones: str_conditions,
            acciones: str_actions,
            opciones: str_options,
        };
        strategies.push(strategy);
    }

    Ok(strategies)
}

/// Elimina una estrategia y todos sus elementos relacionados (indicadores, acciones y condiciones).
///
/// # Parámetros
/// * `id`: ID de la estrategia a eliminar.
///
/// # Returns
/// * `Result<String>` - Mensaje de éxito si la eliminación es correcta.
///
/// # Errores
/// Retorna error si falla la conexión a la base de datos o la eliminación.
///
/// # Nota
/// Esta función elimina en cascada los indicadores, acciones y condiciones asociados a la estrategia.
#[tauri::command]
pub async fn delete_strategy(id: i32) -> Result<String, Error> {
    let _ = delete_strategy_condition_by_strategy(id).await;
    let _ = delete_strategy_indicator_by_strategy(id).await;
    let _ = delete_strategy_action_by_strategy(id).await;
    let _ = delete_strategy_options_by_strategy(id).await;

    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = if !DB_LOCAL {
        Builder::new_remote_replica(db_path, sync_url, auth_token)
            .build()
            .await?
    } else {
        Builder::new_local(db_path).build().await?
    };

    let conn = db.connect()?;

    let parametros = params![id];

    conn.query("DELETE FROM strategies WHERE id = ?", parametros)
        .await?;

    Ok("Estrategia eliminada con exito!".to_string())
}
//================================Indicators================================

/// Crea la tabla de indicadores de estrategias en la base de datos.
///
/// # Returns
/// * `Result<String>` - Mensaje de éxito si la tabla se crea correctamente.
///
/// # Errores
/// Retorna error si falla la conexión a la base de datos o la ejecución de la query.
#[tauri::command]
pub async fn table_strategy_indicators() -> Result<String, Error> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = if !DB_LOCAL {
        Builder::new_remote_replica(db_path, sync_url, auth_token)
            .build()
            .await?
    } else {
        Builder::new_local(db_path).build().await?
    };

    let conn = db.connect()?;

    conn.query(
        "CREATE TABLE IF NOT EXISTS strategy_indicators (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            strategy_id  INTEGER NOT NULL REFERENCES strategies(id),
            nombre       TEXT NOT NULL,  -- nombre del campo: 'sma_20'
            tipo         TEXT NOT NULL,  -- 'SMA', 'EMA', 'RSI', 'MACD', 'BB'
            chart_type   TEXT NOT NULL,
            parametros   TEXT DEFAULT '{}'   -- JSON: '{\"period\": 20}'
        )",
        (),
    )
    .await?;

    Ok("Tabla strategy_indicators is ok.".to_string())
}

/// Inserta un nuevo indicador en la base de datos.
///
/// # Parámetros
/// * `indicator`: Objeto StrategyIndicator con los datos del indicador a insertar.
///
/// # Returns
/// * `Result<i32>` - ID del indicador insertado.
///
/// # Errores
/// Retorna error si falla la conexión a la base de datos o la inserción.
#[tauri::command]
pub async fn insert_strategies_indicator(indicator: &StrategyIndicator) -> Result<i32, Error> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = if !DB_LOCAL {
        Builder::new_remote_replica(db_path, sync_url, auth_token)
            .build()
            .await?
    } else {
        Builder::new_local(db_path).build().await?
    };

    let conn = db.connect()?;

    let parametros = params![
        indicator.strategy_id,
        indicator.nombre.as_str(),
        indicator.tipo.as_str(),
        indicator.chart_type.as_str(),
        indicator.parametros.to_string()
    ];
    conn.query(
        "INSERT INTO strategy_indicators (strategy_id, nombre, tipo, chart_type, parametros) VALUES (?, ?, ?, ?, ?) RETURNING id",
        parametros,
    )
    .await?;

    let id = conn.last_insert_rowid() as i32;
    Ok(id)
}

/// Obtiene un indicador específico por su ID.
///
/// # Parámetros
/// * `id`: ID del indicador a buscar.
///
/// # Returns
/// * `Result<StrategyIndicator>` - Indicador encontrado.
///
/// # Errores
/// Retorna error si no se encuentra el indicador o falla la conexión.
#[tauri::command]
pub async fn get_strategy_indicator_by_id(id: i32) -> Result<StrategyIndicator, Error> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = if !DB_LOCAL {
        Builder::new_remote_replica(db_path, sync_url, auth_token)
            .build()
            .await?
    } else {
        Builder::new_local(db_path).build().await?
    };

    let conn = db.connect()?;

    let parametros = params![id];

    let mut result = conn
        .query("SELECT * FROM strategy_indicators WHERE id = ?", parametros)
        .await?;

    let row = result.next().await?.unwrap();

    let indicator = StrategyIndicator {
        id: row.get::<i32>(0)?,
        strategy_id: row.get::<i32>(1)?,
        nombre: row.get::<String>(2)?,
        tipo: row.get::<String>(3)?,
        chart_type: ChartType::as_ct(row.get::<String>(4)?.as_str()),
        parametros: serde_json::from_str(&row.get::<String>(5)?)?,
    };

    Ok(indicator)
}

/// Obtiene todos los indicadores asociados a una estrategia.
///
/// # Parámetros
/// * `startegy_id`: ID de la estrategia cuyos indicadores se obtendrán.
///
/// # Returns
/// * `Result<Vec<StrategyIndicator>>` - Vector de indicadores de la estrategia.
///
/// # Errores
/// Retorna error si falla la conexión a la base de datos o la consulta.
#[tauri::command]
pub async fn get_strategies_indicators_by_strategy_id(
    startegy_id: i32,
) -> Result<Vec<StrategyIndicator>, Error> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = if !DB_LOCAL {
        Builder::new_remote_replica(db_path, sync_url, auth_token)
            .build()
            .await?
    } else {
        Builder::new_local(db_path).build().await?
    };

    let conn = db.connect()?;

    let parametros = params![startegy_id];

    let sql = "SELECT * FROM strategy_indicators WHERE strategy_id = ?";

    let mut result = conn.query(sql, parametros).await?;

    let mut indicators = Vec::new();
    while let Some(row) = result.next().await? {
        let indicator = StrategyIndicator {
            id: row.get::<i32>(0)?,
            strategy_id: row.get::<i32>(1)?,
            nombre: row.get::<String>(2)?,
            tipo: row.get::<String>(3)?,
            chart_type: ChartType::as_ct(row.get::<String>(4)?.as_str()),
            parametros: serde_json::from_str(&row.get::<String>(5)?)?,
        };
        indicators.push(indicator);
    }

    Ok(indicators)
}

/// Elimina un indicador específico por su ID.
///
/// # Parámetros
/// * `id`: ID del indicador a eliminar.
///
/// # Returns
/// * `Result<String>` - Mensaje de éxito si la eliminación es correcta.
///
/// # Errores
/// Retorna error si falla la conexión a la base de datos o la eliminación.
#[tauri::command]
pub async fn delete_strategy_indicator(id: i32) -> Result<String, Error> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = if !DB_LOCAL {
        Builder::new_remote_replica(db_path, sync_url, auth_token)
            .build()
            .await?
    } else {
        Builder::new_local(db_path).build().await?
    };

    let conn = db.connect()?;

    let parametros = params![id];

    conn.query("DELETE FROM strategy_indicators WHERE id = ?", parametros)
        .await?;

    Ok("Indicador eliminado con exito!".to_string())
}

/// Elimina todos los indicadores asociados a una estrategia.
///
/// # Parámetros
/// * `strategy_id`: ID de la estrategia cuyos indicadores se eliminarán.
///
/// # Returns
/// * `Result<String>` - Mensaje de éxito si la eliminación es correcta.
///
/// # Errores
/// Retorna error si falla la conexión a la base de datos o la eliminación.
#[tauri::command]
pub async fn delete_strategy_indicator_by_strategy(strategy_id: i32) -> Result<String, Error> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = if !DB_LOCAL {
        Builder::new_remote_replica(db_path, sync_url, auth_token)
            .build()
            .await?
    } else {
        Builder::new_local(db_path).build().await?
    };

    let conn = db.connect()?;

    let parametros = params![strategy_id];

    conn.query(
        "DELETE FROM strategy_indicators WHERE strategy_id = ?",
        parametros,
    )
    .await?;

    Ok("Indicadores eliminados con exito!".to_string())
}

//================================Actions================================

/// Crea la tabla de acciones de estrategias en la base de datos.
///
/// # Returns
/// * `Result<String>` - Mensaje de éxito si la tabla se crea correctamente.
///
/// # Errores
/// Retorna error si falla la conexión a la base de datos o la ejecución de la query.
#[tauri::command]
pub async fn table_strategy_actions() -> Result<String, Error> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = if !DB_LOCAL {
        Builder::new_remote_replica(db_path, sync_url, auth_token)
            .build()
            .await?
    } else {
        Builder::new_local(db_path).build().await?
    };

    let conn = db.connect()?;

    conn.query(
        "CREATE TABLE IF NOT EXISTS strategy_actions (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            strategy_id  INTEGER NOT NULL REFERENCES strategies(id),
            tipo_signal  TEXT NOT NULL,  -- 'entry' | 'exit'
            tipo         TEXT NOT NULL,  -- 'buy', 'sell', 'close', 'set_sl', 'set_tp'
            parametros    TEXT DEFAULT '{}'  -- JSON con parámetros extra
        )",
        (),
    )
    .await?;

    Ok("Tabla strategy_actions is ok.".to_string())
}

/// Inserta una nueva acción en la base de datos.
///
/// # Parámetros
/// * `action`: Objeto StrategyAction con los datos de la acción a insertar.
///
/// # Returns
/// * `Result<i32>` - ID de la acción insertada.
///
/// # Errores
/// Retorna error si falla la conexión a la base de datos o la inserción.
#[tauri::command]
pub async fn insert_strategies_action(action: &StrategyAction) -> Result<i32, Error> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = if !DB_LOCAL {
        Builder::new_remote_replica(db_path, sync_url, auth_token)
            .build()
            .await?
    } else {
        Builder::new_local(db_path).build().await?
    };

    let conn = db.connect()?;

    let parametros = params![
        action.strategy_id,
        action.tipo_signal.clone(),
        action.tipo.to_string(),
        action.parametros.to_string()
    ];
    conn.query(
        "INSERT INTO strategy_actions (strategy_id, tipo_signal, tipo, parametros) VALUES (?, ?, ?, ?) RETURNING id",
        parametros,
    )
    .await?;

    let id = conn.last_insert_rowid() as i32;
    Ok(id)
}

/// Obtiene una acción específica por su ID.
///
/// # Parámetros
/// * `id`: ID de la acción a buscar.
///
/// # Returns
/// * `Result<StrategyAction>` - Acción encontrada.
///
/// # Errores
/// Retorna error si no se encuentra la acción o falla la conexión.
#[tauri::command]
pub async fn get_strategy_action_by_id(id: i32) -> Result<StrategyAction, Error> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = if !DB_LOCAL {
        Builder::new_remote_replica(db_path, sync_url, auth_token)
            .build()
            .await?
    } else {
        Builder::new_local(db_path).build().await?
    };

    let conn = db.connect()?;

    let mut sql = "SELECT * FROM strategy_actions WHERE id = ?";

    let mut parametros = params![id];

    let mut result = conn.query(sql, parametros).await?;

    let row = result.next().await?.unwrap();

    let mut action = StrategyAction {
        id: row.get::<i32>(0)?,
        strategy_id: row.get::<i32>(1)?,
        tipo_signal: row.get::<String>(2)?,
        tipo: match row.get::<String>(3)?.as_str() {
            "Buy" => Action::Buy,
            "Sell" => Action::Sell,
            "Buy_limit" => Action::BuyLimit,
            "Sell_limit" => Action::SellLimit,
            "Buy_stop" => Action::BuyStop,
            "Sell_stop" => Action::SellStop,
            "Close" => Action::Close,
            _ => Action::Buy,
        },
        parametros: serde_json::from_str(&row.get::<String>(4)?)?,
        conditions: None,
    };

    sql = "SELECT * FROM strategy_conditions WHERE action_id = ?";

    parametros = params![action.id];

    result = conn.query(sql, parametros).await?;

    while let Some(row_condition) = result.next().await? {
        if let Some(condition) = &mut action.conditions {
            add_condition(condition, &row_condition);
        } else {
            action.conditions = Some(StrategyCondition {
                id: row_condition.get::<i32>(0)?,
                strategy_id: row_condition.get::<i32>(1)?,
                action_id: row_condition.get::<i32>(2)?,
                campo_a: row_condition.get::<String>(3)?,
                shift_a: row_condition.get::<i32>(4)?,
                operador: row_condition.get::<String>(5)?,
                campo_b: row_condition.get::<String>(6)?,
                shift_b: row_condition.get::<i32>(7)?,
                logica: match row_condition.get::<String>(8)?.as_str() {
                    "AND" => Some(Logic::AND),
                    "OR" => Some(Logic::OR),
                    _ => None,
                },
                orden: row_condition.get::<i32>(9)?,
                next_condition: None,
            });
        }
    }

    Ok(action)
}

/// Obtiene todas las acciones asociadas a una estrategia.
///
/// # Parámetros
/// * `startegy_id`: ID de la estrategia cuyas acciones se obtendrán.
///
/// # Returns
/// * `Result<Vec<StrategyAction>>` - Vector de acciones de la estrategia.
///
/// # Errores
/// Retorna error si falla la conexión a la base de datos o la consulta.
#[tauri::command]
pub async fn get_strategies_actions_by_strategy_id(
    startegy_id: i32,
) -> Result<Vec<StrategyAction>, Error> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = if !DB_LOCAL {
        Builder::new_remote_replica(db_path, sync_url, auth_token)
            .build()
            .await?
    } else {
        Builder::new_local(db_path).build().await?
    };

    let conn = db.connect()?;

    let parametros = params![startegy_id];

    let sql = "SELECT * FROM strategy_actions WHERE strategy_id = ?";

    let mut result = conn.query(sql, parametros).await?;

    let mut actions = Vec::new();
    while let Some(row) = result.next().await? {
        let mut action = StrategyAction {
            id: row.get::<i32>(0)?,
            strategy_id: row.get::<i32>(1)?,
            tipo_signal: row.get::<String>(2)?,
            tipo: match row.get::<String>(3)?.as_str() {
                "Buy" => Action::Buy,
                "Sell" => Action::Sell,
                "Buy_limit" => Action::BuyLimit,
                "Sell_limit" => Action::SellLimit,
                "Buy_stop" => Action::BuyStop,
                "Sell_stop" => Action::SellStop,
                "Close" => Action::Close,
                _ => Action::Buy,
            },
            parametros: serde_json::from_str(&row.get::<String>(4)?)?,
            conditions: None,
        };

        let sql = "SELECT * FROM strategy_conditions WHERE action_id = ?";

        let parametros = params![action.id];

        let mut result = conn.query(sql, parametros).await?;

        while let Some(row_condition) = result.next().await? {
            if let Some(condition) = &mut action.conditions {
                add_condition(condition, &row_condition);
            } else {
                action.conditions = Some(StrategyCondition {
                    id: row_condition.get::<i32>(0)?,
                    strategy_id: row_condition.get::<i32>(1)?,
                    action_id: row_condition.get::<i32>(2)?,
                    campo_a: row_condition.get::<String>(3)?,
                    shift_a: row_condition.get::<i32>(4)?,
                    operador: row_condition.get::<String>(5)?,
                    campo_b: row_condition.get::<String>(6)?,
                    shift_b: row_condition.get::<i32>(7)?,
                    logica: match row_condition.get::<String>(8)?.as_str() {
                        "AND" => Some(Logic::AND),
                        "OR" => Some(Logic::OR),
                        _ => None,
                    },
                    orden: row_condition.get::<i32>(9)?,
                    next_condition: None,
                });
            }
        }
        actions.push(action);
    }

    Ok(actions)
}

/// Elimina una acción específica por su ID.
///
/// # Parámetros
/// * `id`: ID de la acción a eliminar.
///
/// # Returns
/// * `Result<String>` - Mensaje de éxito si la eliminación es correcta.
///
/// # Errores
/// Retorna error si falla la conexión a la base de datos o la eliminación.
#[tauri::command]
pub async fn delete_strategy_action(id: i32) -> Result<String, Error> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = if !DB_LOCAL {
        Builder::new_remote_replica(db_path, sync_url, auth_token)
            .build()
            .await?
    } else {
        Builder::new_local(db_path).build().await?
    };

    let conn = db.connect()?;

    let parametros = params![id];

    conn.query("DELETE FROM strategy_actions WHERE id = ?", parametros)
        .await?;

    Ok("Action eliminado con exito!".to_string())
}

/// Elimina todas las acciones asociadas a una estrategia.
///
/// # Parámetros
/// * `strategy_id`: ID de la estrategia cuyas acciones se eliminarán.
///
/// # Returns
/// * `Result<String>` - Mensaje de éxito si la eliminación es correcta.
///
/// # Errores
/// Retorna error si falla la conexión a la base de datos o la eliminación.
#[tauri::command]
pub async fn delete_strategy_action_by_strategy(strategy_id: i32) -> Result<String, Error> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = if !DB_LOCAL {
        Builder::new_remote_replica(db_path, sync_url, auth_token)
            .build()
            .await?
    } else {
        Builder::new_local(db_path).build().await?
    };

    let conn = db.connect()?;

    let parametros = params![strategy_id];

    conn.query(
        "DELETE FROM strategy_actions WHERE strategy_id = ?",
        parametros,
    )
    .await?;

    Ok("Actions eliminados con exito!".to_string())
}
//================================Conditions================================

/// Crea la tabla de condiciones de estrategias en la base de datos.
///
/// # Returns
/// * `Result<String>` - Mensaje de éxito si la tabla se crea correctamente.
///
/// # Errores
/// Retorna error si falla la conexión a la base de datos o la ejecución de la query.
#[tauri::command]
pub async fn table_strategy_conditions() -> Result<String, Error> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = if !DB_LOCAL {
        Builder::new_remote_replica(db_path, sync_url, auth_token)
            .build()
            .await?
    } else {
        Builder::new_local(db_path).build().await?
    };

    let conn = db.connect()?;

    conn.query(
        "CREATE TABLE IF NOT EXISTS strategy_conditions (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            strategy_id  INTEGER NOT NULL REFERENCES strategies(id),
            action_id    INTEGER NOT NULL REFERENCES strategy_actions(id),
            campo_a      TEXT NOT NULL,  -- 'close', 'sma_20', 'rsi'
            shift_a      INTEGER DEFAULT 0,
            operador     TEXT NOT NULL,  -- '>', '<', '>=', '<=', '==', 'cross_above', 'cross_below'
            campo_b      TEXT NOT NULL,  -- 'sma_50' o valor literal '30.0'
            shift_b      INTEGER DEFAULT 0,
            logica       TEXT DEFAULT 'NULL',
            orden        INTEGER DEFAULT 0
        )",
        (),
    )
    .await?;

    Ok("Tabla strategy_conditions is ok.".to_string())
}

/// Inserta una nueva condición en la base de datos.
///
/// # Parámetros
/// * `condition`: Objeto StrategyCondition con los datos de la condición a insertar.
///
/// # Returns
/// * `Result<i32>` - ID de la condición insertada.
///
/// # Errores
/// Retorna error si falla la conexión a la base de datos o la inserción.
#[tauri::command]
pub async fn insert_strategy_condition(condition: &StrategyCondition) -> Result<i32, Error> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = if !DB_LOCAL {
        Builder::new_remote_replica(db_path, sync_url, auth_token)
            .build()
            .await?
    } else {
        Builder::new_local(db_path).build().await?
    };

    let conn = db.connect()?;

    let parametros = params![
        condition.strategy_id,
        condition.action_id,
        condition.campo_a.clone(),
        condition.shift_a,
        condition.operador.clone(),
        condition.campo_b.clone(),
        condition.shift_b,
        condition.logica.unwrap().to_string(),
        condition.orden
    ];
    conn.query(
        "INSERT INTO strategy_conditions (strategy_id, action_id, campo_a, shift_a, operador, campo_b, shift_b, logica, orden) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?) RETURNING id",
        parametros,
    )
    .await?;

    let id = conn.last_insert_rowid() as i32;
    Ok(id)
}

/// Obtiene una condición específica por su ID.
///
/// # Parámetros
/// * `id`: ID de la condición a buscar.
///
/// # Returns
/// * `Result<StrategyCondition>` - Condición encontrada.
///
/// # Errores
/// Retorna error si no se encuentra la condición o falla la conexión.
#[tauri::command]
pub async fn get_strategy_condition_by_id(id: i32) -> Result<StrategyCondition, Error> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = if !DB_LOCAL {
        Builder::new_remote_replica(db_path, sync_url, auth_token)
            .build()
            .await?
    } else {
        Builder::new_local(db_path).build().await?
    };

    let conn = db.connect()?;

    let sql = "SELECT * FROM strategy_conditions WHERE id = ?";

    let parametros = params![id];

    let mut result = conn.query(sql, parametros).await?;

    let row = result.next().await?.unwrap();

    let condition = StrategyCondition {
        id: row.get::<i32>(0)?,
        strategy_id: row.get::<i32>(1)?,
        action_id: row.get::<i32>(2)?,
        campo_a: row.get::<String>(3)?,
        shift_a: row.get::<i32>(4)?,
        operador: row.get::<String>(5)?,
        campo_b: row.get::<String>(6)?,
        shift_b: row.get::<i32>(7)?,
        logica: match row.get::<String>(8)?.as_str() {
            "AND" => Some(Logic::AND),
            "OR" => Some(Logic::OR),
            _ => None,
        },
        orden: row.get::<i32>(9)?,
        next_condition: None,
    };

    Ok(condition)
}

/// Obtiene todas las condiciones asociadas a una estrategia.
///
/// # Parámetros
/// * `startegy_id`: ID de la estrategia cuyas condiciones se obtendrán.
///
/// # Returns
/// * `Result<Vec<StrategyCondition>>` - Vector de condiciones de la estrategia.
///
/// # Errores
/// Retorna error si falla la conexión a la base de datos o la consulta.
#[tauri::command]
pub async fn get_strategies_conditions_by_strategy_id(
    startegy_id: i32,
) -> Result<Vec<StrategyCondition>, Error> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = if !DB_LOCAL {
        Builder::new_remote_replica(db_path, sync_url, auth_token)
            .build()
            .await?
    } else {
        Builder::new_local(db_path).build().await?
    };

    let conn = db.connect()?;

    let parametros = params![startegy_id];

    let sql = "SELECT * FROM strategy_conditions WHERE strategy_id = ?";

    let mut result = conn.query(sql, parametros).await?;

    let mut conditions = Vec::new();
    while let Some(row) = result.next().await? {
        let condition = StrategyCondition {
            id: row.get::<i32>(0)?,
            strategy_id: row.get::<i32>(1)?,
            action_id: row.get::<i32>(2)?,
            campo_a: row.get::<String>(3)?,
            shift_a: row.get::<i32>(4)?,
            operador: row.get::<String>(5)?,
            campo_b: row.get::<String>(6)?,
            shift_b: row.get::<i32>(7)?,
            logica: match row.get::<String>(8)?.as_str() {
                "AND" => Some(Logic::AND),
                "OR" => Some(Logic::OR),
                _ => None,
            },
            orden: row.get::<i32>(9)?,
            next_condition: None,
        };
        conditions.push(condition);
    }

    Ok(conditions)
}

/// Elimina una condición específica por su ID.
///
/// # Parámetros
/// * `id`: ID de la condición a eliminar.
///
/// # Returns
/// * `Result<String>` - Mensaje de éxito si la eliminación es correcta.
///
/// # Errores
/// Retorna error si falla la conexión a la base de datos o la eliminación.
#[tauri::command]
pub async fn delete_strategy_condition(id: i32) -> Result<String, Error> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = if !DB_LOCAL {
        Builder::new_remote_replica(db_path, sync_url, auth_token)
            .build()
            .await?
    } else {
        Builder::new_local(db_path).build().await?
    };

    let conn = db.connect()?;

    let parametros = params![id];

    conn.query("DELETE FROM strategy_conditions WHERE id = ?", parametros)
        .await?;

    Ok("Condition eliminado con exito!".to_string())
}

/// Elimina todas las condiciones asociadas a una estrategia.
///
/// # Parámetros
/// * `strategy_id`: ID de la estrategia cuyas condiciones se eliminarán.
///
/// # Returns
/// * `Result<String>` - Mensaje de éxito si la eliminación es correcta.
///
/// # Errores
/// Retorna error si falla la conexión a la base de datos o la eliminación.
#[tauri::command]
pub async fn delete_strategy_condition_by_strategy(strategy_id: i32) -> Result<String, Error> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = if !DB_LOCAL {
        Builder::new_remote_replica(db_path, sync_url, auth_token)
            .build()
            .await?
    } else {
        Builder::new_local(db_path).build().await?
    };

    let conn = db.connect()?;

    let parametros = params![strategy_id];

    conn.query(
        "DELETE FROM strategy_conditions WHERE strategy_id = ?",
        parametros,
    )
    .await?;

    Ok("Conditions eliminados con exito!".to_string())
}

//================================Options================================
/// Crea la tabla de opciones de estrategias en la base de datos.
///
/// # Returns
/// * `Result<String>` - Mensaje de éxito si la tabla se crea correctamente.
///
/// # Errores
/// Retorna error si falla la conexión a la base de datos o la ejecución de la query.
#[tauri::command]
pub async fn table_strategy_options() -> Result<String, Error> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = if !DB_LOCAL {
        Builder::new_remote_replica(db_path, sync_url, auth_token)
            .build()
            .await?
    } else {
        Builder::new_local(db_path).build().await?
    };

    let conn = db.connect()?;

    conn.query(
        "CREATE TABLE IF NOT EXISTS strategy_options (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            strategy_id  INTEGER NOT NULL REFERENCES strategies(id),
            multiples_trades BOOLEAN DEFAULT FALSE,
            trading_direccion TEXT DEFAULT 'Both',
            operar_finde BOOLEAN DEFAULT FALSE,
            cerrar_fin_de_dia BOOLEAN DEFAULT FALSE,
            hora_fin_de_dia TEXT DEFAULT 'NULL',
            cerrar_viernes BOOLEAN DEFAULT FALSE,
            hora_cierre_viernes TEXT DEFAULT 'NULL',
            rango_operativo BOOLEAN DEFAULT FALSE,
            rango_operativo_inicio TEXT DEFAULT 'NULL',
            rango_operativo_fin TEXT DEFAULT 'NULL',
            cerrar_fin_rango_operativo BOOLEAN DEFAULT FALSE,
            activar_cierre_numero_velas BOOLEAN DEFAULT FALSE,
            numero_velas_cierre INTEGER DEFAULT 0,
            cierre_limite_hora BOOLEAN DEFAULT FALSE,
            hora_cierre_limite TEXT DEFAULT 'NULL',
            parametros_stoploss TEXT DEFAULT 'NULL',
            parametros_takeprofit TEXT DEFAULT 'NULL'
        )",
        (),
    )
    .await?;

    Ok("Tabla strategy_options is ok.".to_string())
}

/// Inserta las opciones de estrategia en la base de datos.
///
/// # Parámetros
/// * `options` - Las opciones de estrategia a insertar.
///
/// # Returns
/// * `Ok(())` - Las opciones de estrategia fueron insertadas correctamente.
/// * `Err(Error)` - Ocurrió un error al insertar las opciones de estrategia.
#[tauri::command]
pub async fn insert_strategy_options(options: &StrategyOptions) -> Result<i32, Error> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = if !DB_LOCAL {
        Builder::new_remote_replica(db_path, sync_url, auth_token)
            .build()
            .await?
    } else {
        Builder::new_local(db_path).build().await?
    };

    let conn = db.connect()?;

    let parametros = params![
        options.strategy_id,
        options.multiples_trades,
        options.trading_direccion.to_string(),
        options.operar_finde,
        options.cerrar_fin_de_dia,
        options.hora_fin_de_dia.to_string(),
        options.cerrar_viernes,
        options.hora_cierre_viernes.to_string(),
        options.rango_operativo,
        options.rango_operativo_inicio.to_string(),
        options.rango_operativo_fin.to_string(),
        options.cerrar_fin_rango_operativo,
        options.activar_cierre_numero_velas,
        options.numero_velas_cierre,
        options.cierre_limite_hora,
        options.hora_cierre_limite.to_string(),
        options.parametros_stoploss.as_ref().unwrap().to_json(),
        options.parametros_takeprofit.as_ref().unwrap().to_json(),
    ];

    conn.execute("INSERT INTO strategy_options (strategy_id, multiples_trades, trading_direccion, operar_finde, cerrar_fin_de_dia, hora_fin_de_dia, cerrar_viernes, hora_cierre_viernes, rango_operativo, rango_operativo_inicio, rango_operativo_fin, cerrar_fin_rango_operativo, activar_cierre_numero_velas, numero_velas_cierre, cierre_limite_hora, hora_cierre_limite, parametros_stoploss, parametros_takeprofit) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)", parametros).await?;

    let id = conn.last_insert_rowid() as i32;
    Ok(id)
}

/// Obtiene las opciones de estrategia específica por su ID.
///
/// # Parámetros
/// * `id`: ID de las opciones de estrategia a buscar.
///
/// # Returns
/// * `Result<StrategyOptions>` - Opciones de estrategia encontradas.
///
/// # Errores
/// Retorna error si no se encuentra la estrategia o falla la conexión.
#[tauri::command]
pub async fn get_strategy_options_by_id(id: i32) -> Result<StrategyOptions, Error> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = if !DB_LOCAL {
        Builder::new_remote_replica(db_path, sync_url, auth_token)
            .build()
            .await?
    } else {
        Builder::new_local(db_path).build().await?
    };

    let conn = db.connect()?;

    let sql = "SELECT * FROM strategy_options WHERE id = ?";

    let parametros = params![id];

    let mut result = conn.query(sql, parametros).await?;

    let row = result.next().await?.unwrap();

    let options = StrategyOptions {
        id: row.get::<i32>(0)?,
        strategy_id: row.get::<i32>(1)?,
        multiples_trades: if row.get::<i32>(2)? == 1 { true } else { false },
        trading_direccion: if row.get::<String>(3)? == "long" {
            TradingDirection::Long
        } else if row.get::<String>(3)? == "short" {
            TradingDirection::Short
        } else {
            TradingDirection::Both
        },
        operar_finde: if row.get::<i32>(4)? == 1 { true } else { false },
        cerrar_fin_de_dia: if row.get::<i32>(5)? == 1 { true } else { false },
        hora_fin_de_dia: DateTime::parse_from_rfc3339(&row.get::<String>(6)?)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| DateTime::<Utc>::MIN_UTC),
        cerrar_viernes: if row.get::<i32>(7)? == 1 { true } else { false },
        hora_cierre_viernes: DateTime::parse_from_rfc3339(&row.get::<String>(8)?)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| DateTime::<Utc>::MIN_UTC),
        rango_operativo: if row.get::<i32>(9)? == 1 { true } else { false },
        rango_operativo_inicio: DateTime::parse_from_rfc3339(&row.get::<String>(10)?)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| DateTime::<Utc>::MIN_UTC),
        rango_operativo_fin: DateTime::parse_from_rfc3339(&row.get::<String>(11)?)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| DateTime::<Utc>::MIN_UTC),
        cerrar_fin_rango_operativo: if row.get::<i32>(12)? == 1 {
            true
        } else {
            false
        },
        activar_cierre_numero_velas: if row.get::<i32>(13)? == 1 {
            true
        } else {
            false
        },
        numero_velas_cierre: row.get::<i32>(14)?,
        cierre_limite_hora: if row.get::<i32>(15)? == 1 {
            true
        } else {
            false
        },
        hora_cierre_limite: DateTime::parse_from_rfc3339(&row.get::<String>(16)?)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| DateTime::<Utc>::MIN_UTC),
        parametros_stoploss: Some(serde_json::from_str::<StopLoss>(&row.get::<String>(17)?)?),
        parametros_takeprofit: Some(serde_json::from_str::<TakeProfit>(&row.get::<String>(18)?)?),
    };

    Ok(options)
}

/// Obtiene las opciones de estrategia específica por su strategy_id.
///
/// # Parámetros
/// * `strategy_id`: ID de la estrategia a buscar.
///
/// # Returns
/// * `Result<StrategyOptions>` - Opciones de estrategia encontradas.
///
/// # Errores
/// Retorna error si no se encuentra la estrategia o falla la conexión.
#[tauri::command]
pub async fn get_strategy_options_by_strategy_id(
    strategy_id: i32,
) -> Result<StrategyOptions, Error> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = if !DB_LOCAL {
        Builder::new_remote_replica(db_path, sync_url, auth_token)
            .build()
            .await?
    } else {
        Builder::new_local(db_path).build().await?
    };

    let conn = db.connect()?;

    let sql = "SELECT * FROM strategy_options WHERE strategy_id = ?";

    let parametros = params![strategy_id];

    let mut result = conn.query(sql, parametros).await?;

    let row = result.next().await?.unwrap();

    let options = StrategyOptions {
        id: row.get::<i32>(0)?,
        strategy_id: row.get::<i32>(1)?,
        multiples_trades: if row.get::<i32>(2)? == 1 { true } else { false },
        trading_direccion: if row.get::<String>(3)? == "long" {
            TradingDirection::Long
        } else if row.get::<String>(3)? == "short" {
            TradingDirection::Short
        } else {
            TradingDirection::Both
        },
        operar_finde: if row.get::<i32>(4)? == 1 { true } else { false },
        cerrar_fin_de_dia: if row.get::<i32>(5)? == 1 { true } else { false },
        hora_fin_de_dia: DateTime::parse_from_rfc3339(&row.get::<String>(6)?)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| DateTime::<Utc>::MIN_UTC),
        cerrar_viernes: if row.get::<i32>(7)? == 1 { true } else { false },
        hora_cierre_viernes: DateTime::parse_from_rfc3339(&row.get::<String>(8)?)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| DateTime::<Utc>::MIN_UTC),
        rango_operativo: if row.get::<i32>(9)? == 1 { true } else { false },
        rango_operativo_inicio: DateTime::parse_from_rfc3339(&row.get::<String>(10)?)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| DateTime::<Utc>::MIN_UTC),
        rango_operativo_fin: DateTime::parse_from_rfc3339(&row.get::<String>(11)?)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| DateTime::<Utc>::MIN_UTC),
        cerrar_fin_rango_operativo: if row.get::<i32>(12)? == 1 {
            true
        } else {
            false
        },
        activar_cierre_numero_velas: if row.get::<i32>(13)? == 1 {
            true
        } else {
            false
        },
        numero_velas_cierre: row.get::<i32>(14)?,
        cierre_limite_hora: if row.get::<i32>(15)? == 1 {
            true
        } else {
            false
        },
        hora_cierre_limite: DateTime::parse_from_rfc3339(&row.get::<String>(16)?)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| DateTime::<Utc>::MIN_UTC),
        parametros_stoploss: Some(serde_json::from_str::<StopLoss>(&row.get::<String>(17)?)?),
        parametros_takeprofit: Some(serde_json::from_str::<TakeProfit>(&row.get::<String>(18)?)?),
    };

    Ok(options)
}

/// Elimina las opciones de estrategia específica por su ID.
///
/// # Parámetros
/// * `id`: ID de las opciones de estrategia a eliminar.
///
/// # Returns
/// * `Result<String>` - Mensaje de éxito si la eliminación es correcta.
///
/// # Errores
/// Retorna error si falla la conexión a la base de datos o la eliminación.
#[tauri::command]
pub async fn delete_strategy_option(id: i32) -> Result<String, Error> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = if !DB_LOCAL {
        Builder::new_remote_replica(db_path, sync_url, auth_token)
            .build()
            .await?
    } else {
        Builder::new_local(db_path).build().await?
    };

    let conn = db.connect()?;

    let parametros = params![id];

    conn.query("DELETE FROM strategy_options WHERE id = ?", parametros)
        .await?;

    Ok("Option eliminado con exito!".to_string())
}

/// Elimina todas las opciones de estrategia asociadas a una estrategia.
///
/// # Parámetros
/// * `strategy_id`: ID de la estrategia cuyas opciones se eliminarán.
///
/// # Returns
/// * `Result<String>` - Mensaje de éxito si la eliminación es correcta.
///
/// # Errores
/// Retorna error si falla la conexión a la base de datos o la eliminación.
#[tauri::command]
pub async fn delete_strategy_options_by_strategy(strategy_id: i32) -> Result<String, Error> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = if !DB_LOCAL {
        Builder::new_remote_replica(db_path, sync_url, auth_token)
            .build()
            .await?
    } else {
        Builder::new_local(db_path).build().await?
    };

    let conn = db.connect()?;

    let parametros = params![strategy_id];

    conn.query(
        "DELETE FROM strategy_options WHERE strategy_id = ?",
        parametros,
    )
    .await?;

    Ok("Options eliminados con exito!".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    //================================TEST: Strategies================================
    #[tokio::test(flavor = "multi_thread")]
    async fn test_crud_strategies() -> Result<(), Error> {
        let _guardia = crate::utils::data_test::BLOQUEO_RECURSOS.lock().await;
        match table_strategies().await {
            Ok(_) => {
                let estrategia: Strategy = Strategy {
                    id: 1,
                    id_user: 1,
                    nombre: "Ema_tres".to_string(),
                    descripcion: Some("Cruce de 3 emas".to_string()),
                    activa: true,
                    creada_en: "01-04-2026".to_string(),
                    indicadores: Vec::<StrategyIndicator>::new(),
                    acciones: Vec::<StrategyAction>::new(),
                    opciones: StrategyOptions::new_empty(),
                };

                match insert_strategies(&estrategia).await {
                    Ok(id) => {
                        tokio::time::sleep(std::time::Duration::from_secs(5)).await;

                        match table_strategy_indicators().await {
                            Ok(_) => {
                                let indicator = StrategyIndicator {
                                    id: 1,
                                    strategy_id: id,
                                    nombre: "SMA_20".to_string(),
                                    tipo: "SMA".to_string(),
                                    chart_type: ChartType::Inchart,
                                    parametros: serde_json::Value::String(
                                        "{
                                        period: 20
                                        }"
                                        .to_string(),
                                    ),
                                };

                                match insert_strategies_indicator(&indicator).await {
                                    Ok(id_indicador) => {
                                        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                                        //================================TEST: GETTERS Indicators================================
                                        let _ = get_strategy_indicator_by_id(id_indicador).await;

                                        let _ = get_strategies_indicators_by_strategy_id(
                                            indicator.strategy_id.clone(),
                                        )
                                        .await;
                                    }
                                    Err(e) => return Err(e),
                                }
                            }
                            Err(e) => return Err(e),
                        };

                        match table_strategy_actions().await {
                            Ok(_) => {
                                let action: StrategyAction = StrategyAction {
                                    id: 1,
                                    strategy_id: id,
                                    tipo_signal: "Buy".to_string(),
                                    tipo: Action::Close,
                                    parametros: serde_json::Value::String(
                                        "{parametro:20}".to_string(),
                                    ),
                                    conditions: None,
                                };

                                match insert_strategies_action(&action).await {
                                    Ok(id_action) => {
                                        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                                        let condition: StrategyCondition = StrategyCondition {
                                            id: 1,
                                            strategy_id: id,
                                            action_id: id_action,
                                            campo_a: "SMA_20".to_string(),
                                            shift_a: 0,
                                            operador: ">".to_string(),
                                            campo_b: "SMA_ 50".to_string(),
                                            shift_b: 0,
                                            logica: Some(Logic::AND),
                                            orden: 0,
                                            next_condition: None,
                                        };

                                        match table_strategy_conditions().await {
                                            Ok(_) => {
                                                match insert_strategy_condition(&condition).await {
                                                    Ok(id_condition) => {
                                                        //================================TEST: GETTERS Conditions================================
                                                        let _ = get_strategy_condition_by_id(
                                                            id_condition,
                                                        )
                                                        .await;

                                                        let _ = get_strategies_conditions_by_strategy_id(
                                                            condition.strategy_id.clone(),
                                                        )
                                                        .await;
                                                    }
                                                    Err(e) => return Err(e),
                                                }
                                            }
                                            Err(e) => return Err(e),
                                        };

                                        //================================TEST: GETTERS Actions================================
                                        let _ = get_strategy_action_by_id(id_action).await;

                                        let _ = get_strategies_actions_by_strategy_id(
                                            action.strategy_id.clone(),
                                        )
                                        .await;
                                    }
                                    Err(e) => return Err(e),
                                }
                            }
                            Err(e) => return Err(e),
                        };

                        match table_strategy_options().await {
                            Ok(_) => {
                                let option: StrategyOptions = StrategyOptions {
                                    id: 1,
                                    strategy_id: id,
                                    multiples_trades: false,
                                    trading_direccion: TradingDirection::Long,
                                    operar_finde: false,
                                    cerrar_fin_de_dia: false,
                                    hora_fin_de_dia: Utc::now(),
                                    cerrar_viernes: false,
                                    hora_cierre_viernes: Utc::now(),
                                    rango_operativo: false,
                                    rango_operativo_inicio: Utc::now(),
                                    rango_operativo_fin: Utc::now(),
                                    cerrar_fin_rango_operativo: false,
                                    activar_cierre_numero_velas: false,
                                    numero_velas_cierre: 0,
                                    cierre_limite_hora: false,
                                    hora_cierre_limite: Utc::now(),
                                    parametros_stoploss: Some(StopLoss::new_empty()),
                                    parametros_takeprofit: Some(TakeProfit::new_empty()),
                                };

                                match insert_strategy_options(&option).await {
                                    Ok(id_option) => {
                                        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                                        //================================TEST: GETTERS Options================================

                                        let _ = get_strategy_options_by_id(id_option).await;

                                        let _ = get_strategy_options_by_strategy_id(
                                            option.strategy_id.clone(),
                                        )
                                        .await;
                                    }
                                    Err(e) => return Err(e),
                                }
                            }
                            Err(e) => return Err(e),
                        };

                        //================================TEST: GETTERS Strategies================================
                        let _ = get_strategies().await;

                        let _ = get_strategies_by_id(id).await;

                        let _ = get_strategies_by_id_user(estrategia.id_user.clone()).await;

                        let _ = get_strategies_by_nombre(estrategia.nombre.clone()).await;

                        let _ = get_active_strategies_by_user(estrategia.id_user.clone()).await;

                        let _ = get_all_active_strategies().await;

                        let _ = get_desactive_strategies_by_user(estrategia.id_user.clone()).await;

                        let _ = get_all_desactive_strategies().await;

                        let _ = get_active_strategies_by_date(estrategia.creada_en.clone()).await;

                        //================================TEST: DELETE Strategies================================

                        let _ = delete_strategy_indicator_by_strategy(id).await;
                        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                        let _ = delete_strategy_action_by_strategy(id).await;
                        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                        let _ = delete_strategy_condition_by_strategy(id).await;
                        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                        let _ = delete_strategy_options_by_strategy(id).await;
                        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                        let _ = delete_strategy(id).await;
                    }
                    Err(e) => return Err(e),
                };
            }
            Err(e) => return Err(e),
        }

        Ok(())
    }
}
