use crate::api::users::{get_user_by_id_clerk, update_user_activo, update_user_clave};
use crate::structs::activation_key::ActivationKey;
use crate::utils::configuracion::DB_LOCAL;
use crate::utils::configuracion::{get_db_config, Error};
use libsql::{params, Builder};
use rand::Rng;

/// Juego de caracteres para generar claves: mayúsculas y dígitos,
/// excluyendo los ambiguos `0`, `O`, `1` e `I`.
const CARACTERES_CLAVE: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";

/// Longitud de los grupos de caracteres de una clave, separados por `-`
/// (formato `XXXX-XX-XXX-XXXXXXXXXX`).
const GRUPOS_CLAVE: [usize; 4] = [4, 2, 3, 10];

/// Genera una clave de activación aleatoria con formato
/// `XXXX-XX-XXX-XXXXXXXXXX`.
///
/// Usa el juego de caracteres sin ambiguos definido en `CARACTERES_CLAVE`.
fn generar_clave() -> String {
    let mut rng = rand::rng();
    let mut clave = String::new();

    for (indice, &longitud) in GRUPOS_CLAVE.iter().enumerate() {
        if indice > 0 {
            clave.push('-');
        }
        for _ in 0..longitud {
            let posicion = rng.random_range(0..CARACTERES_CLAVE.len());
            clave.push(CARACTERES_CLAVE[posicion] as char);
        }
    }

    clave
}

/// Crea la tabla de claves de activación en la base de datos.
///
/// # Returns
/// * `Result<String>` - Mensaje de éxito si la tabla se crea correctamente.
///
/// # Errores
/// Retorna error si falla la conexión a la base de datos o la ejecución de la query.
pub async fn table_activation_keys() -> Result<String, Error> {
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
        "CREATE TABLE IF NOT EXISTS activation_keys (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            clave TEXT NOT NULL UNIQUE,
            usada BOOLEAN NOT NULL DEFAULT 0,
            id_clerk TEXT,
            fecha_activacion TEXT
        )",
        (),
    )
    .await?;

    Ok("Tabla de claves de activación OK".to_string())
}

/// Inserta una nueva clave de activación en la base de datos.
///
/// # Parámetros
/// * `clave`: Valor de la clave de activación a insertar.
///
/// # Returns
/// * `Result<i32>` - ID de la clave insertada.
///
/// # Errores
/// Retorna error si falla la conexión a la base de datos, si ya existe una
/// clave con el mismo valor o si la inserción falla.
pub async fn insert_key(clave: String) -> Result<i32, Error> {
    match table_activation_keys().await {
        Ok(_) => {
            let (db_path, sync_url, auth_token) = get_db_config()?;

            let db = if !DB_LOCAL {
                Builder::new_remote_replica(db_path, sync_url, auth_token)
                    .build()
                    .await?
            } else {
                Builder::new_local(db_path).build().await?
            };

            let conn = db.connect()?;

            let mut rows = conn
                .query(
                    "INSERT INTO activation_keys (clave) VALUES (?) RETURNING id",
                    params![clave.as_str()],
                )
                .await?;

            let row = rows.next().await?.ok_or_else(|| Error {
                msg: "No se ha insertado la clave de activación.".to_string(),
            })?;

            let id = row.get::<i32>(0)?;

            Ok(id)
        }
        Err(e) => Err(e),
    }
}

/// Obtiene todas las claves de activación de la base de datos.
///
/// # Returns
/// * `Result<Vec<ActivationKey>>` - Vector con todas las claves y su estado.
///
/// # Errores
/// Retorna error si falla la conexión a la base de datos o la consulta.
pub async fn get_keys() -> Result<Vec<ActivationKey>, Error> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = if !DB_LOCAL {
        Builder::new_remote_replica(db_path, sync_url, auth_token)
            .build()
            .await?
    } else {
        Builder::new_local(db_path).build().await?
    };

    let conn = db.connect()?;

    let mut rows = conn.query("SELECT * FROM activation_keys", ()).await?;

    let mut claves: Vec<ActivationKey> = Vec::new();

    while let Some(row) = rows.next().await? {
        let key = ActivationKey {
            id: row.get::<i32>(0)?,
            clave: row.get::<String>(1)?,
            usada: row.get::<i32>(2)? != 0,
            id_clerk: row.get::<Option<String>>(3)?,
            fecha_activacion: row.get::<Option<String>>(4)?,
        };
        claves.push(key);
    }

    Ok(claves)
}

/// Obtiene las claves de activación disponibles (no usadas).
///
/// # Returns
/// * `Result<Vec<ActivationKey>>` - Vector con las claves disponibles.
///
/// # Errores
/// Retorna error si falla la conexión a la base de datos o la consulta.
pub async fn get_available_keys() -> Result<Vec<ActivationKey>, Error> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = if !DB_LOCAL {
        Builder::new_remote_replica(db_path, sync_url, auth_token)
            .build()
            .await?
    } else {
        Builder::new_local(db_path).build().await?
    };

    let conn = db.connect()?;

    let mut rows = conn
        .query("SELECT * FROM activation_keys WHERE usada = 0", ())
        .await?;

    let mut claves: Vec<ActivationKey> = Vec::new();

    while let Some(row) = rows.next().await? {
        let key = ActivationKey {
            id: row.get::<i32>(0)?,
            clave: row.get::<String>(1)?,
            usada: row.get::<i32>(2)? != 0,
            id_clerk: row.get::<Option<String>>(3)?,
            fecha_activacion: row.get::<Option<String>>(4)?,
        };
        claves.push(key);
    }

    Ok(claves)
}

/// Obtiene una clave de activación por su ID.
///
/// # Parámetros
/// * `id`: ID de la clave a buscar.
///
/// # Returns
/// * `Result<ActivationKey>` - Clave encontrada con todos sus campos.
///
/// # Errores
/// Retorna error si no se encuentra la clave o falla la conexión.
pub async fn get_key_by_id(id: i32) -> Result<ActivationKey, Error> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = if !DB_LOCAL {
        Builder::new_remote_replica(db_path, sync_url, auth_token)
            .build()
            .await?
    } else {
        Builder::new_local(db_path).build().await?
    };

    let conn = db.connect()?;

    let mut rows = conn
        .query("SELECT * FROM activation_keys WHERE id = ?", params![id])
        .await?;

    let row = rows.next().await?.ok_or_else(|| Error {
        msg: format!("Clave de activación no encontrada: id = {id}"),
    })?;

    let key = ActivationKey {
        id: row.get::<i32>(0)?,
        clave: row.get::<String>(1)?,
        usada: row.get::<i32>(2)? != 0,
        id_clerk: row.get::<Option<String>>(3)?,
        fecha_activacion: row.get::<Option<String>>(4)?,
    };

    Ok(key)
}

/// Obtiene una clave de activación por su valor.
///
/// Función interna usada por la lógica de activación y generación; no se
/// expone como comando Tauri.
///
/// # Parámetros
/// * `clave`: Valor de la clave a buscar.
///
/// # Returns
/// * `Result<ActivationKey>` - Clave encontrada con todos sus campos.
///
/// # Errores
/// Retorna un error genérico si no se encuentra la clave o falla la conexión.
/// Nunca incluye el valor de la clave en el mensaje (Art. 6 de la constitución).
pub async fn get_key_by_clave(clave: String) -> Result<ActivationKey, Error> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = if !DB_LOCAL {
        Builder::new_remote_replica(db_path, sync_url, auth_token)
            .build()
            .await?
    } else {
        Builder::new_local(db_path).build().await?
    };

    let conn = db.connect()?;

    let mut rows = conn
        .query(
            "SELECT * FROM activation_keys WHERE clave = ?",
            params![clave.as_str()],
        )
        .await?;

    let row = rows.next().await?.ok_or_else(|| Error {
        msg: "Clave de activación no encontrada.".to_string(),
    })?;

    let key = ActivationKey {
        id: row.get::<i32>(0)?,
        clave: row.get::<String>(1)?,
        usada: row.get::<i32>(2)? != 0,
        id_clerk: row.get::<Option<String>>(3)?,
        fecha_activacion: row.get::<Option<String>>(4)?,
    };

    Ok(key)
}

/// Elimina una clave de activación por su ID.
///
/// # Parámetros
/// * `id`: ID de la clave a eliminar.
///
/// # Returns
/// * `Result<()>` - Ok si la eliminación es correcta.
///
/// # Errores
/// Retorna error si falla la conexión a la base de datos o la eliminación.
pub async fn delete_key(id: i32) -> Result<(), Error> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = if !DB_LOCAL {
        Builder::new_remote_replica(db_path, sync_url, auth_token)
            .build()
            .await?
    } else {
        Builder::new_local(db_path).build().await?
    };

    let conn = db.connect()?;

    conn.execute("DELETE FROM activation_keys WHERE id = ?", params![id])
        .await?;

    Ok(())
}

/// Genera claves de activación aleatorias con formato
/// `XXXX-XX-XXX-XXXXXXXXXX` y las guarda en la base de datos.
///
/// # Parámetros
/// * `cantidad`: Número de claves a generar.
///
/// # Returns
/// * `Result<Vec<String>>` - Vector con las claves generadas.
///
/// # Errores
/// Retorna error si falla la conexión a la base de datos o alguna inserción.
/// Si una clave generada ya existiera (prácticamente imposible) se regenera.
pub async fn generate_keys(cantidad: u32) -> Result<Vec<String>, Error> {
    let mut claves: Vec<String> = Vec::new();

    while claves.len() < cantidad as usize {
        let clave = generar_clave();

        // Comprobar colisión antes de insertar: si la clave ya existe se
        // regenera (robustez, el espacio de claves hace la colisión casi
        // imposible).
        if get_key_by_clave(clave.clone()).await.is_ok() {
            continue;
        }

        insert_key(clave.clone()).await?;
        claves.push(clave);
    }

    Ok(claves)
}

/// Activa el acceso de un usuario mediante una clave de activación.
///
/// Operación crítica: valida el usuario y la clave en el backend y vincula
/// la clave al usuario si todo es correcto.
///
/// # Parámetros
/// * `id_clerk`: Identificador del usuario en Clerk.
/// * `clave`: Clave de activación introducida por el usuario.
///
/// # Returns
/// * `Result<()>` - Ok si la activación es correcta.
///
/// # Errores
/// Retorna error (sin modificar la base de datos) si:
/// - No existe el usuario.
/// - El usuario ya está activo.
/// - La clave no existe o ya está usada (error genérico, sin revelar cuál de
///   las dos situaciones es ni el valor de la clave).
pub async fn activar_usuario(id_clerk: String, clave: String) -> Result<(), Error> {
    // 1. El usuario debe existir.
    let user = get_user_by_id_clerk(id_clerk.clone()).await?;

    // 2. Un usuario ya activo no puede activarse de nuevo.
    if user.usuario_activo {
        return Err(Error {
            msg: format!("El usuario ya está activo: id_clerk = {id_clerk}"),
        });
    }

    // 3. La clave debe existir y estar disponible.
    let key = get_key_by_clave(clave.clone()).await?;
    if key.usada {
        return Err(Error {
            msg: "Clave de activación no válida.".to_string(),
        });
    }

    // 4. Vincular la clave de forma segura: el `WHERE usada = 0` evita que
    // dos activaciones simultáneas consuman la misma clave.
    let fecha = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = if !DB_LOCAL {
        Builder::new_remote_replica(db_path, sync_url, auth_token)
            .build()
            .await?
    } else {
        Builder::new_local(db_path).build().await?
    };

    let conn = db.connect()?;

    let filas = conn
        .execute(
            "UPDATE activation_keys SET usada = 1, id_clerk = ?, fecha_activacion = ? WHERE id = ? AND usada = 0",
            params![id_clerk.as_str(), fecha.as_str(), key.id],
        )
        .await?;

    if filas != 1 {
        return Err(Error {
            msg: "Clave de activación no válida.".to_string(),
        });
    }

    // 5. Activar al usuario y guardar la clave consumida en su perfil.
    update_user_clave(id_clerk.clone(), clave).await?;
    update_user_activo(id_clerk, true).await?;

    Ok(())
}

/// Libera una clave de activación usada para poder reutilizarla.
///
/// Limpia el estado de la clave (`usada`, `id_clerk`, `fecha_activacion`)
/// pero no modifica al usuario que la tenía: si procede, el administrador
/// debe desactivarlo aparte.
///
/// # Parámetros
/// * `id`: ID de la clave a liberar.
///
/// # Returns
/// * `Result<()>` - Ok si la liberación es correcta.
///
/// # Errores
/// Retorna error si falla la conexión a la base de datos o la actualización.
pub async fn release_key(id: i32) -> Result<(), Error> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = if !DB_LOCAL {
        Builder::new_remote_replica(db_path, sync_url, auth_token)
            .build()
            .await?
    } else {
        Builder::new_local(db_path).build().await?
    };

    let conn = db.connect()?;

    conn.execute(
        "UPDATE activation_keys SET usada = 0, id_clerk = NULL, fecha_activacion = NULL WHERE id = ?",
        params![id],
    )
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::users::{delete_user, insert_user};
    use crate::structs::user::User;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_crud_keys() -> Result<(), Error> {
        let _guardia = crate::utils::data_test::BLOQUEO_RECURSOS.lock().await;
        let clave_test = "TEST-XX-XXX-CRUDTEST001".to_string();

        // Limpieza defensiva por si quedó algún residuo de un test anterior.
        if let Ok(k) = get_key_by_clave(clave_test.clone()).await {
            let _ = delete_key(k.id).await;
        }

        // Crear la tabla.
        let _ = table_activation_keys().await?;

        // Insertar la clave de prueba.
        let id = insert_key(clave_test.clone()).await?;

        // Obtenerla por id: debe estar disponible y sin usuario vinculado.
        let key = get_key_by_id(id).await?;
        assert_eq!(key.clave, clave_test);
        assert!(!key.usada);
        assert!(key.id_clerk.is_none());
        assert!(key.fecha_activacion.is_none());

        // Listados.
        let _ = get_keys().await?;
        let disponibles = get_available_keys().await?;
        assert!(disponibles.iter().any(|k| k.id == id));

        // Eliminar la clave de prueba.
        delete_key(id).await?;
        assert!(get_key_by_id(id).await.is_err());

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_generacion_claves() -> Result<(), Error> {
        let _guardia = crate::utils::data_test::BLOQUEO_RECURSOS.lock().await;
        let claves = generate_keys(3).await?;
        assert_eq!(claves.len(), 3);

        for clave in &claves {
            // Formato XXXX-XX-XXX-XXXXXXXXXX: longitud 22 y guiones en las
            // posiciones 4, 7 y 11.
            assert_eq!(clave.len(), 22);
            let bytes = clave.as_bytes();
            assert_eq!(bytes[4], b'-');
            assert_eq!(bytes[7], b'-');
            assert_eq!(bytes[11], b'-');

            // Solo caracteres del juego permitido.
            for &b in bytes {
                assert!(
                    b == b'-' || CARACTERES_CLAVE.contains(&b),
                    "Carácter no válido en la clave generada: {clave}"
                );
            }
        }

        // Las claves generadas deben ser distintas entre sí.
        assert_ne!(claves[0], claves[1]);
        assert_ne!(claves[1], claves[2]);
        assert_ne!(claves[0], claves[2]);

        // Limpieza: eliminar las claves generadas.
        for clave in claves {
            let k = get_key_by_clave(clave).await?;
            delete_key(k.id).await?;
        }

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_activacion_usuario() -> Result<(), Error> {
        let _guardia = crate::utils::data_test::BLOQUEO_RECURSOS.lock().await;
        let id_clerk_1 = "test_user_keys_001".to_string();
        let id_clerk_2 = "test_user_keys_002".to_string();
        let clave_test = "TEST-XX-XXX-TESTCLAVE01".to_string();
        let clave_otra = "TEST-XX-XXX-TESTCLAVE02".to_string();

        // Limpieza defensiva por si quedó algún residuo de un test anterior.
        let _ = delete_user(id_clerk_1.clone()).await;
        let _ = delete_user(id_clerk_2.clone()).await;
        if let Ok(k) = get_key_by_clave(clave_test.clone()).await {
            let _ = delete_key(k.id).await;
        }
        if let Ok(k) = get_key_by_clave(clave_otra.clone()).await {
            let _ = delete_key(k.id).await;
        }

        // Preparar usuarios de prueba (inactivos) y claves.
        let _ = crate::api::users::table_users().await?;
        for id_clerk in [&id_clerk_1, &id_clerk_2] {
            insert_user(&User {
                id_clerk: id_clerk.clone(),
                nombre: "Nombre".to_string(),
                apellidos: "Apellidos".to_string(),
                username: id_clerk.clone(),
                descripcion: None,
                clave_activacion: None,
                usuario_activo: false,
            })
            .await?;
        }
        let id_clave = insert_key(clave_test.clone()).await?;
        let id_clave_otra = insert_key(clave_otra.clone()).await?;

        // Activación correcta: usuario activo, clave vinculada y guardada
        // también en el perfil del usuario.
        activar_usuario(id_clerk_1.clone(), clave_test.clone()).await?;

        let key = get_key_by_id(id_clave).await?;
        assert!(key.usada);
        assert_eq!(key.id_clerk.as_deref(), Some(id_clerk_1.as_str()));
        assert!(key.fecha_activacion.is_some());

        let user = get_user_by_id_clerk(id_clerk_1.clone()).await?;
        assert!(user.usuario_activo);
        assert_eq!(user.clave_activacion.as_deref(), Some(clave_test.as_str()));

        // Un usuario ya activo no puede activarse de nuevo.
        assert!(activar_usuario(id_clerk_1.clone(), clave_otra.clone())
            .await
            .is_err());

        // Clave inexistente: error.
        assert!(
            activar_usuario(id_clerk_2.clone(), "CLAVE-INEXISTENTE".to_string())
                .await
                .is_err()
        );

        // Clave ya usada: error.
        assert!(activar_usuario(id_clerk_2.clone(), clave_test.clone())
            .await
            .is_err());

        // Liberar la clave y reutilizarla con otro usuario.
        release_key(id_clave).await?;

        let key = get_key_by_id(id_clave).await?;
        assert!(!key.usada);
        assert!(key.id_clerk.is_none());
        assert!(key.fecha_activacion.is_none());

        activar_usuario(id_clerk_2.clone(), clave_test.clone()).await?;

        let user = get_user_by_id_clerk(id_clerk_2.clone()).await?;
        assert!(user.usuario_activo);
        assert_eq!(user.clave_activacion.as_deref(), Some(clave_test.as_str()));

        // Limpieza final: usuarios y claves de prueba.
        delete_user(id_clerk_1).await?;
        delete_user(id_clerk_2).await?;
        delete_key(id_clave).await?;
        delete_key(id_clave_otra).await?;

        Ok(())
    }
}
