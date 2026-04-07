use crate::strategy::strategy::Strategy;
use crate::strategy::strategy_action::StrategyAction;
use crate::strategy::strategy_condition::StrategyCondition;
use crate::strategy::strategy_indicator::StrategyIndicator;
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
//================================Strategies================================

/// Crea la tabla de estrategias en la base de datos.
///
/// # Returns
/// * `Result<String>` - Mensaje de éxito si la tabla se crea correctamente.
///
/// # Errores
/// Retorna error si falla la conexión a la base de datos o la ejecución de la query.
#[tauri::command]
pub async fn table_strategies() -> Result<String> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = Builder::new_remote_replica(db_path, sync_url, auth_token)
        .build()
        .await?;

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
pub async fn insert_strategies(strategy: Strategy) -> Result<i32> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = Builder::new_remote_replica(db_path, sync_url, auth_token)
        .build()
        .await?;

    let conn = db.connect()?;

    let parametros = params![
        strategy.id_user,
        strategy.nombre,
        strategy.descripcion,
        strategy.activa,
        strategy.creada_en
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
pub async fn get_strategies() -> Result<Vec<Strategy>> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = Builder::new_remote_replica(db_path, sync_url, auth_token)
        .build()
        .await?;

    let conn = db.connect()?;

    let sql = "SELECT * FROM strategies";

    let mut result = conn.query(sql, ()).await?;

    let mut strategies = Vec::new();
    while let Some(row) = result.next().await? {
        let mut str_indicators: Vec<StrategyIndicator> = Vec::<StrategyIndicator>::new();
        let mut str_conditions: Vec<StrategyCondition> = Vec::<StrategyCondition>::new();
        let mut str_actions: Vec<StrategyAction> = Vec::<StrategyAction>::new();

        let id_startegy = row.get::<i32>(0)?;

        match get_strategies_indicators_by_strategy_id(id_startegy).await {
            Ok(indicators) => str_indicators = indicators,
            Err(e) => println!(
                "Error al optener los indicadores de strategy_id: {}. Error: {:?}",
                id_startegy, e
            ),
        }

        match get_strategies_conditions_by_strategy_id(id_startegy).await {
            Ok(conditions) => str_conditions = conditions,
            Err(e) => println!(
                "Error al optener los conditions de strategy_id: {}. Error: {:?}",
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
            condiciones: str_conditions,
            acciones: str_actions,
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
pub async fn get_strategies_by_id(id: i32) -> Result<Strategy> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = Builder::new_remote_replica(db_path, sync_url, auth_token)
        .build()
        .await?;

    let conn = db.connect()?;

    let sql = "SELECT * FROM strategies WHERE id = ?";

    let parametros = params![id];

    let mut result = conn.query(sql, parametros).await?;

    let row = result.next().await?.unwrap();

    let mut str_indicators: Vec<StrategyIndicator> = Vec::<StrategyIndicator>::new();
    let mut str_conditions: Vec<StrategyCondition> = Vec::<StrategyCondition>::new();
    let mut str_actions: Vec<StrategyAction> = Vec::<StrategyAction>::new();

    let id_startegy = row.get::<i32>(0)?;

    match get_strategies_indicators_by_strategy_id(id_startegy).await {
        Ok(indicators) => str_indicators = indicators,
        Err(e) => println!(
            "Error al optener los indicadores de strategy_id: {}. Error: {:?}",
            id_startegy, e
        ),
    }

    match get_strategies_conditions_by_strategy_id(id_startegy).await {
        Ok(conditions) => str_conditions = conditions,
        Err(e) => println!(
            "Error al optener los conditions de strategy_id: {}. Error: {:?}",
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
        condiciones: str_conditions,
        acciones: str_actions,
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
pub async fn get_strategies_by_id_user(id_user: i32) -> Result<Vec<Strategy>> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = Builder::new_remote_replica(db_path, sync_url, auth_token)
        .build()
        .await?;

    let conn = db.connect()?;

    let parametros = params![id_user];

    let sql = "SELECT * FROM strategies WHERE id_user = ?  AND activa = 1";

    let mut result = conn.query(sql, parametros).await?;

    let mut strategies = Vec::new();
    while let Some(row) = result.next().await? {
        let mut str_indicators: Vec<StrategyIndicator> = Vec::<StrategyIndicator>::new();
        let mut str_conditions: Vec<StrategyCondition> = Vec::<StrategyCondition>::new();
        let mut str_actions: Vec<StrategyAction> = Vec::<StrategyAction>::new();

        let id_startegy = row.get::<i32>(0)?;

        match get_strategies_indicators_by_strategy_id(id_startegy).await {
            Ok(indicators) => str_indicators = indicators,
            Err(e) => println!(
                "Error al optener los indicadores de strategy_id: {}. Error: {:?}",
                id_startegy, e
            ),
        }

        match get_strategies_conditions_by_strategy_id(id_startegy).await {
            Ok(conditions) => str_conditions = conditions,
            Err(e) => println!(
                "Error al optener los conditions de strategy_id: {}. Error: {:?}",
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
            condiciones: str_conditions,
            acciones: str_actions,
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
pub async fn get_strategies_by_nombre(nombre: String) -> Result<Vec<Strategy>> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = Builder::new_remote_replica(db_path, sync_url, auth_token)
        .build()
        .await?;

    let conn = db.connect()?;

    let parametros = params![nombre];

    let sql = "SELECT * FROM strategies WHERE nombre = ?  AND activa = 1";

    let mut result = conn.query(sql, parametros).await?;

    let mut strategies = Vec::new();
    while let Some(row) = result.next().await? {
        let mut str_indicators: Vec<StrategyIndicator> = Vec::<StrategyIndicator>::new();
        let mut str_conditions: Vec<StrategyCondition> = Vec::<StrategyCondition>::new();
        let mut str_actions: Vec<StrategyAction> = Vec::<StrategyAction>::new();

        let id_startegy = row.get::<i32>(0)?;

        match get_strategies_indicators_by_strategy_id(id_startegy).await {
            Ok(indicators) => str_indicators = indicators,
            Err(e) => println!(
                "Error al optener los indicadores de strategy_id: {}. Error: {:?}",
                id_startegy, e
            ),
        }

        match get_strategies_conditions_by_strategy_id(id_startegy).await {
            Ok(conditions) => str_conditions = conditions,
            Err(e) => println!(
                "Error al optener los conditions de strategy_id: {}. Error: {:?}",
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
            condiciones: str_conditions,
            acciones: str_actions,
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
pub async fn get_active_strategies_by_user(id_user: i32) -> Result<Vec<Strategy>> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = Builder::new_remote_replica(db_path, sync_url, auth_token)
        .build()
        .await?;

    let conn = db.connect()?;

    let parametros = params![id_user];

    let sql = "SELECT * FROM strategies WHERE id_user = ? AND activa = 1";

    let mut result = conn.query(sql, parametros).await?;

    let mut strategies = Vec::new();
    while let Some(row) = result.next().await? {
        let mut str_indicators: Vec<StrategyIndicator> = Vec::<StrategyIndicator>::new();
        let mut str_conditions: Vec<StrategyCondition> = Vec::<StrategyCondition>::new();
        let mut str_actions: Vec<StrategyAction> = Vec::<StrategyAction>::new();

        let id_startegy = row.get::<i32>(0)?;

        match get_strategies_indicators_by_strategy_id(id_startegy).await {
            Ok(indicators) => str_indicators = indicators,
            Err(e) => println!(
                "Error al optener los indicadores de strategy_id: {}. Error: {:?}",
                id_startegy, e
            ),
        }

        match get_strategies_conditions_by_strategy_id(id_startegy).await {
            Ok(conditions) => str_conditions = conditions,
            Err(e) => println!(
                "Error al optener los conditions de strategy_id: {}. Error: {:?}",
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
            condiciones: str_conditions,
            acciones: str_actions,
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
pub async fn get_all_active_strategies() -> Result<Vec<Strategy>> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = Builder::new_remote_replica(db_path, sync_url, auth_token)
        .build()
        .await?;

    let conn = db.connect()?;

    // let parametros = params![id_user];

    let sql = "SELECT * FROM strategies WHERE activa = 1";

    let mut result = conn.query(sql, ()).await?;

    let mut strategies = Vec::new();
    while let Some(row) = result.next().await? {
        let mut str_indicators: Vec<StrategyIndicator> = Vec::<StrategyIndicator>::new();
        let mut str_conditions: Vec<StrategyCondition> = Vec::<StrategyCondition>::new();
        let mut str_actions: Vec<StrategyAction> = Vec::<StrategyAction>::new();

        let id_startegy = row.get::<i32>(0)?;

        match get_strategies_indicators_by_strategy_id(id_startegy).await {
            Ok(indicators) => str_indicators = indicators,
            Err(e) => println!(
                "Error al optener los indicadores de strategy_id: {}. Error: {:?}",
                id_startegy, e
            ),
        }

        match get_strategies_conditions_by_strategy_id(id_startegy).await {
            Ok(conditions) => str_conditions = conditions,
            Err(e) => println!(
                "Error al optener los conditions de strategy_id: {}. Error: {:?}",
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
            condiciones: str_conditions,
            acciones: str_actions,
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
pub async fn get_desactive_strategies_by_user(id_user: i32) -> Result<Vec<Strategy>> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = Builder::new_remote_replica(db_path, sync_url, auth_token)
        .build()
        .await?;

    let conn = db.connect()?;

    let parametros = params![id_user];

    let sql = "SELECT * FROM strategies WHERE id_user = ? AND activa = 0";

    let mut result = conn.query(sql, parametros).await?;

    let mut strategies = Vec::new();
    while let Some(row) = result.next().await? {
        let mut str_indicators: Vec<StrategyIndicator> = Vec::<StrategyIndicator>::new();
        let mut str_conditions: Vec<StrategyCondition> = Vec::<StrategyCondition>::new();
        let mut str_actions: Vec<StrategyAction> = Vec::<StrategyAction>::new();

        let id_startegy = row.get::<i32>(0)?;

        match get_strategies_indicators_by_strategy_id(id_startegy).await {
            Ok(indicators) => str_indicators = indicators,
            Err(e) => println!(
                "Error al optener los indicadores de strategy_id: {}. Error: {:?}",
                id_startegy, e
            ),
        }

        match get_strategies_conditions_by_strategy_id(id_startegy).await {
            Ok(conditions) => str_conditions = conditions,
            Err(e) => println!(
                "Error al optener los conditions de strategy_id: {}. Error: {:?}",
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
            condiciones: str_conditions,
            acciones: str_actions,
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
pub async fn get_all_desactive_strategies() -> Result<Vec<Strategy>> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = Builder::new_remote_replica(db_path, sync_url, auth_token)
        .build()
        .await?;

    let conn = db.connect()?;

    // let parametros = params![id_user];

    let sql = "SELECT * FROM strategies WHERE activa = 0";

    let mut result = conn.query(sql, ()).await?;

    let mut strategies = Vec::new();
    while let Some(row) = result.next().await? {
        let mut str_indicators: Vec<StrategyIndicator> = Vec::<StrategyIndicator>::new();
        let mut str_conditions: Vec<StrategyCondition> = Vec::<StrategyCondition>::new();
        let mut str_actions: Vec<StrategyAction> = Vec::<StrategyAction>::new();

        let id_startegy = row.get::<i32>(0)?;

        match get_strategies_indicators_by_strategy_id(id_startegy).await {
            Ok(indicators) => str_indicators = indicators,
            Err(e) => println!(
                "Error al optener los indicadores de strategy_id: {}. Error: {:?}",
                id_startegy, e
            ),
        }

        match get_strategies_conditions_by_strategy_id(id_startegy).await {
            Ok(conditions) => str_conditions = conditions,
            Err(e) => println!(
                "Error al optener los conditions de strategy_id: {}. Error: {:?}",
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
            condiciones: str_conditions,
            acciones: str_actions,
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
pub async fn get_active_strategies_by_date(fecha: String) -> Result<Vec<Strategy>> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = Builder::new_remote_replica(db_path, sync_url, auth_token)
        .build()
        .await?;

    let conn = db.connect()?;

    let parametros = params![fecha];

    let sql = "SELECT * FROM strategies WHERE creada_en = ? AND activa = 1";

    let mut result = conn.query(sql, parametros).await?;

    let mut strategies = Vec::new();
    while let Some(row) = result.next().await? {
        let mut str_indicators: Vec<StrategyIndicator> = Vec::<StrategyIndicator>::new();
        let mut str_conditions: Vec<StrategyCondition> = Vec::<StrategyCondition>::new();
        let mut str_actions: Vec<StrategyAction> = Vec::<StrategyAction>::new();

        let id_startegy = row.get::<i32>(0)?;

        match get_strategies_indicators_by_strategy_id(id_startegy).await {
            Ok(indicators) => str_indicators = indicators,
            Err(e) => println!(
                "Error al optener los indicadores de strategy_id: {}. Error: {:?}",
                id_startegy, e
            ),
        }

        match get_strategies_conditions_by_strategy_id(id_startegy).await {
            Ok(conditions) => str_conditions = conditions,
            Err(e) => println!(
                "Error al optener los conditions de strategy_id: {}. Error: {:?}",
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
            condiciones: str_conditions,
            acciones: str_actions,
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
pub async fn delete_strategy(id: i32) -> Result<String> {
    let _ = delete_strategy_condition_by_strategy(id).await;
    let _ = delete_strategy_indicator_by_strategy(id).await;
    let _ = delete_strategy_action_by_strategy(id).await;

    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = Builder::new_remote_replica(db_path, sync_url, auth_token)
        .build()
        .await?;

    let conn = db.connect()?;

    let parametros = params![id];

    conn.query("DELETE FROM strategies WHERE id = ?", parametros)
        .await?;

    Ok("Condition eliminado con exito!".to_string())
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
pub async fn table_strategy_indicators() -> Result<String> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = Builder::new_remote_replica(db_path, sync_url, auth_token)
        .build()
        .await?;

    let conn = db.connect()?;

    conn.query(
        "CREATE TABLE IF NOT EXISTS strategy_indicators (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            strategy_id  INTEGER NOT NULL REFERENCES strategies(id),
            nombre       TEXT NOT NULL,  -- nombre del campo: 'sma_20'
            tipo         TEXT NOT NULL,  -- 'SMA', 'EMA', 'RSI', 'MACD', 'BB'
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
pub async fn insert_strategies_indicator(indicator: StrategyIndicator) -> Result<i32> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = Builder::new_remote_replica(db_path, sync_url, auth_token)
        .build()
        .await?;

    let conn = db.connect()?;

    let parametros = params![
        indicator.strategy_id,
        indicator.nombre,
        indicator.tipo,
        indicator.parametros.to_string()
    ];
    conn.query(
        "INSERT INTO strategy_indicators (strategy_id, nombre, tipo, parametros) VALUES (?, ?, ?, ?) RETURNING id",
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
pub async fn get_strategy_indicator_by_id(id: i32) -> Result<StrategyIndicator> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = Builder::new_remote_replica(db_path, sync_url, auth_token)
        .build()
        .await?;

    let conn = db.connect()?;

    let sql = "SELECT * FROM strategy_indicators WHERE id = ?";

    let parametros = params![id];

    let mut result = conn.query(sql, parametros).await?;

    let row = result.next().await?.unwrap();

    let indicator = StrategyIndicator {
        id: row.get::<i32>(0)?,
        strategy_id: row.get::<i32>(1)?,
        nombre: row.get::<String>(2)?,
        tipo: row.get::<String>(3)?,
        parametros: serde_json::from_str(&row.get::<String>(4)?).unwrap(),
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
) -> Result<Vec<StrategyIndicator>> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = Builder::new_remote_replica(db_path, sync_url, auth_token)
        .build()
        .await?;

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
            parametros: serde_json::from_str(&row.get::<String>(4)?).unwrap(),
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
pub async fn delete_strategy_indicator(id: i32) -> Result<String> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = Builder::new_remote_replica(db_path, sync_url, auth_token)
        .build()
        .await?;

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
pub async fn delete_strategy_indicator_by_strategy(strategy_id: i32) -> Result<String> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = Builder::new_remote_replica(db_path, sync_url, auth_token)
        .build()
        .await?;

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
pub async fn table_strategy_actions() -> Result<String> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = Builder::new_remote_replica(db_path, sync_url, auth_token)
        .build()
        .await?;

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
pub async fn insert_strategies_action(action: StrategyAction) -> Result<i32> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = Builder::new_remote_replica(db_path, sync_url, auth_token)
        .build()
        .await?;

    let conn = db.connect()?;

    let parametros = params![
        action.strategy_id,
        action.tipo_signal,
        action.tipo,
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
pub async fn get_strategy_action_by_id(id: i32) -> Result<StrategyAction> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = Builder::new_remote_replica(db_path, sync_url, auth_token)
        .build()
        .await?;

    let conn = db.connect()?;

    let sql = "SELECT * FROM strategy_actions WHERE id = ?";

    let parametros = params![id];

    let mut result = conn.query(sql, parametros).await?;

    let row = result.next().await?.unwrap();

    let action = StrategyAction {
        id: row.get::<i32>(0)?,
        strategy_id: row.get::<i32>(1)?,
        tipo_signal: row.get::<String>(2)?,
        tipo: row.get::<String>(3)?,
        parametros: serde_json::from_str(&row.get::<String>(4)?).unwrap(),
    };

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
) -> Result<Vec<StrategyAction>> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = Builder::new_remote_replica(db_path, sync_url, auth_token)
        .build()
        .await?;

    let conn = db.connect()?;

    let parametros = params![startegy_id];

    let sql = "SELECT * FROM strategy_actions WHERE strategy_id = ?";

    let mut result = conn.query(sql, parametros).await?;

    let mut actions = Vec::new();
    while let Some(row) = result.next().await? {
        let action = StrategyAction {
            id: row.get::<i32>(0)?,
            strategy_id: row.get::<i32>(1)?,
            tipo_signal: row.get::<String>(2)?,
            tipo: row.get::<String>(3)?,
            parametros: serde_json::from_str(&row.get::<String>(4)?).unwrap(),
        };
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
pub async fn delete_strategy_action(id: i32) -> Result<String> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = Builder::new_remote_replica(db_path, sync_url, auth_token)
        .build()
        .await?;

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
pub async fn delete_strategy_action_by_strategy(strategy_id: i32) -> Result<String> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = Builder::new_remote_replica(db_path, sync_url, auth_token)
        .build()
        .await?;

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
pub async fn table_strategy_conditions() -> Result<String> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = Builder::new_remote_replica(db_path, sync_url, auth_token)
        .build()
        .await?;

    let conn = db.connect()?;

    conn.query(
        "CREATE TABLE IF NOT EXISTS strategy_conditions (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            strategy_id  INTEGER NOT NULL REFERENCES strategies(id),
            action_id    INTEGER NOT NULL REFERENCES strategy_actions(id),
            campo_a      TEXT NOT NULL,  -- 'close', 'sma_20', 'rsi'
            operador     TEXT NOT NULL,  -- '>', '<', '>=', '<=', '==', 'cross_above', 'cross_below'
            campo_b      TEXT NOT NULL,  -- 'sma_50' o valor literal '30.0'
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
pub async fn insert_strategy_condition(condition: StrategyCondition) -> Result<i32> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = Builder::new_remote_replica(db_path, sync_url, auth_token)
        .build()
        .await?;

    let conn = db.connect()?;

    let parametros = params![
        condition.strategy_id,
        condition.action_id,
        condition.campo_a,
        condition.operador,
        condition.campo_b,
        condition.logica,
        condition.orden
    ];
    conn.query(
        "INSERT INTO strategy_conditions (strategy_id, action_id, campo_a, operador, campo_b, logica, orden) VALUES (?, ?, ?, ?, ?, ?, ?) RETURNING id",
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
pub async fn get_strategy_condition_by_id(id: i32) -> Result<StrategyCondition> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = Builder::new_remote_replica(db_path, sync_url, auth_token)
        .build()
        .await?;

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
        operador: row.get::<String>(4)?,
        campo_b: row.get::<String>(5)?,
        logica: row.get::<String>(6)?,
        orden: row.get::<i32>(7)?,
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
) -> Result<Vec<StrategyCondition>> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = Builder::new_remote_replica(db_path, sync_url, auth_token)
        .build()
        .await?;

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
            operador: row.get::<String>(4)?,
            campo_b: row.get::<String>(5)?,
            logica: row.get::<String>(6)?,
            orden: row.get::<i32>(7)?,
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
pub async fn delete_strategy_condition(id: i32) -> Result<String> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = Builder::new_remote_replica(db_path, sync_url, auth_token)
        .build()
        .await?;

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
pub async fn delete_strategy_condition_by_strategy(strategy_id: i32) -> Result<String> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = Builder::new_remote_replica(db_path, sync_url, auth_token)
        .build()
        .await?;

    let conn = db.connect()?;

    let parametros = params![strategy_id];

    conn.query(
        "DELETE FROM strategy_conditions WHERE strategy_id = ?",
        parametros,
    )
    .await?;

    Ok("Conditions eliminados con exito!".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    //================================TEST: Strategies================================
    #[tokio::test(flavor = "multi_thread")]
    async fn test_crud_strategies() -> Result<()> {
        let estrategia: Strategy = Strategy {
            id: 1,
            id_user: 1,
            nombre: "Ema_tres".to_string(),
            descripcion: Some("Cruce de 3 emas".to_string()),
            activa: true,
            creada_en: "01-04-2026".to_string(),
            indicadores: Vec::<StrategyIndicator>::new(),
            condiciones: Vec::<StrategyCondition>::new(),
            acciones: Vec::<StrategyAction>::new(),
        };

        let _ = table_strategies().await?;

        let id = insert_strategies(estrategia.clone()).await?;

        let _ = get_strategies().await;

        let _ = get_strategies_by_id(id).await;

        let _ = get_strategies_by_id_user(estrategia.id_user.clone()).await;

        let _ = get_strategies_by_nombre(estrategia.nombre.clone()).await;

        let _ = get_active_strategies_by_user(estrategia.id_user.clone()).await;

        let _ = get_all_active_strategies().await;

        let _ = get_desactive_strategies_by_user(estrategia.id_user.clone()).await;

        let _ = get_all_desactive_strategies().await;

        let _ = get_active_strategies_by_date(estrategia.creada_en.clone()).await;

        let _ = delete_strategy(id).await;

        Ok(())
    }

    //================================TEST: Indicators================================
    #[tokio::test(flavor = "multi_thread")]
    async fn test_crud_strategy_indicators() -> Result<()> {
        let indicator = StrategyIndicator {
            id: 1,
            strategy_id: 1,
            nombre: "SMA_20".to_string(),
            tipo: "SMA".to_string(),
            parametros: serde_json::Value::String(
                "{
                period: 20
                }"
                .to_string(),
            ),
        };

        let _ = table_strategy_indicators().await?;

        let id: i32 = insert_strategies_indicator(indicator.clone())
            .await
            .unwrap();

        let _ = get_strategy_indicator_by_id(id).await;

        let _ = get_strategies_indicators_by_strategy_id(indicator.strategy_id.clone()).await;

        let _ = delete_strategy_indicator(id).await;

        let _ = insert_strategies_indicator(indicator.clone())
            .await
            .unwrap();

        let _ = delete_strategy_indicator_by_strategy(indicator.strategy_id.clone()).await;

        Ok(())
    }
    //================================TEST: Actions================================
    #[tokio::test(flavor = "multi_thread")]
    async fn test_crud_startegy_actions() -> Result<()> {
        let action: StrategyAction = StrategyAction {
            id: 1,
            strategy_id: 1,
            tipo_signal: "Buy".to_string(),
            tipo: "Close".to_string(),
            parametros: serde_json::Value::String("{parametro:20}".to_string()),
        };

        let _ = table_strategy_actions().await?;

        let id: i32 = insert_strategies_action(action.clone()).await.unwrap();

        let _ = get_strategy_action_by_id(id).await;

        let _ = get_strategies_actions_by_strategy_id(action.strategy_id.clone()).await;

        let _ = delete_strategy_action(id).await;

        let _ = insert_strategies_action(action.clone()).await.unwrap();

        let _ = delete_strategy_action_by_strategy(action.strategy_id).await;

        Ok(())
    }

    //================================TEST: Conditions================================
    #[tokio::test(flavor = "multi_thread")]
    async fn test_crud_strategy_conditions() -> Result<()> {
        let condition: StrategyCondition = StrategyCondition {
            id: 1,
            strategy_id: 1,
            action_id: 1,
            campo_a: "SMA_20".to_string(),
            operador: ">".to_string(),
            campo_b: "SMA_ 50".to_string(),
            logica: "AND".to_string(),
            orden: 0,
        };

        let _ = table_strategy_conditions().await?;

        let id: i32 = insert_strategy_condition(condition.clone()).await.unwrap();

        let _ = get_strategy_condition_by_id(id).await;

        let _ = get_strategies_conditions_by_strategy_id(condition.strategy_id.clone()).await;

        let _ = delete_strategy_condition(id).await;

        let _ = insert_strategy_condition(condition.clone()).await.unwrap();

        let _ = delete_strategy_condition_by_strategy(condition.strategy_id).await;

        Ok(())
    }
}
