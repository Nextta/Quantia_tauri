use crate::structs::user::User;
use crate::utils::configuracion::DB_LOCAL;
use crate::utils::configuracion::{get_db_config, Error};
use libsql::{params, Builder};

/// Crea la tabla de usuarios en la base de datos.
///
/// # Returns
/// * `Result<String>` - Mensaje de éxito si la tabla se crea correctamente.
///
/// # Errores
/// Retorna error si falla la conexión a la base de datos o la ejecución de la query.
pub async fn table_users() -> Result<String, Error> {
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
        "CREATE TABLE IF NOT EXISTS users (
            id_clerk TEXT PRIMARY KEY,
            nombre TEXT NOT NULL,
            apellidos TEXT NOT NULL,
            username TEXT NOT NULL,
            descripcion TEXT,
            clave_activacion TEXT,
            usuario_activo BOOLEAN NOT NULL DEFAULT 0
        )",
        (),
    )
    .await?;

    Ok("Tabla de usuarios OK".to_string())
}

/// Inserta un nuevo usuario en la base de datos.
///
/// # Parámetros
/// * `&User`: Objeto User con los datos del usuario a insertar.
///
/// # Returns
/// * `Result<()>` - Ok si la inserción es correcta.
///
/// # Errores
/// Retorna error si falla la conexión a la base de datos, si ya existe un
/// usuario con el mismo `id_clerk` o si la inserción falla.
pub async fn insert_user(user: &User) -> Result<(), Error> {
    match table_users().await {
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

            conn.execute(
                "INSERT INTO users (id_clerk, nombre, apellidos, username, descripcion, clave_activacion, usuario_activo) VALUES (?, ?, ?, ?, ?, ?, ?)",
                params![
                    user.id_clerk.as_str(),
                    user.nombre.as_str(),
                    user.apellidos.as_str(),
                    user.username.as_str(),
                    user.descripcion.clone(),
                    user.clave_activacion.clone(),
                    user.usuario_activo,
                ],
            )
            .await?;

            Ok(())
        }
        Err(e) => Err(e),
    }
}

/// Obtiene todos los usuarios de la base de datos.
///
/// # Returns
/// * `Result<Vec<User>>` - Vector de usuarios.
///
/// # Errores
/// Retorna error si falla la conexión a la base de datos o la consulta.
pub async fn get_users() -> Result<Vec<User>, Error> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = if !DB_LOCAL {
        Builder::new_remote_replica(db_path, sync_url, auth_token)
            .build()
            .await?
    } else {
        Builder::new_local(db_path).build().await?
    };

    let conn = db.connect()?;

    let mut rows = conn.query("SELECT * FROM users", ()).await?;

    let mut usuarios: Vec<User> = Vec::new();

    while let Some(row) = rows.next().await? {
        let user = User {
            id_clerk: row.get::<String>(0)?,
            nombre: row.get::<String>(1)?,
            apellidos: row.get::<String>(2)?,
            username: row.get::<String>(3)?,
            descripcion: row.get::<Option<String>>(4)?,
            clave_activacion: row.get::<Option<String>>(5)?,
            usuario_activo: row.get::<i32>(6)? != 0,
        };
        usuarios.push(user);
    }

    Ok(usuarios)
}

/// Obtiene un usuario por su identificador de Clerk.
///
/// # Parámetros
/// * `id_clerk`: Identificador del usuario en Clerk.
///
/// # Returns
/// * `Result<User>` - Usuario encontrado con todos sus campos.
///
/// # Errores
/// Retorna error si no se encuentra el usuario o falla la conexión.
pub async fn get_user_by_id_clerk(id_clerk: String) -> Result<User, Error> {
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
        .query("SELECT * FROM users WHERE id_clerk = ?", params![id_clerk])
        .await?;

    let row = rows.next().await?.ok_or_else(|| Error {
        msg: format!("Usuario no encontrado: id_clerk = {id_clerk}"),
    })?;

    let user = User {
        id_clerk: row.get::<String>(0)?,
        nombre: row.get::<String>(1)?,
        apellidos: row.get::<String>(2)?,
        username: row.get::<String>(3)?,
        descripcion: row.get::<Option<String>>(4)?,
        clave_activacion: row.get::<Option<String>>(5)?,
        usuario_activo: row.get::<i32>(6)? != 0,
    };

    Ok(user)
}

/// Obtiene un usuario por su username.
///
/// # Parámetros
/// * `username`: Nombre de usuario a buscar.
///
/// # Returns
/// * `Result<User>` - Usuario encontrado con todos sus campos.
///
/// # Errores
/// Retorna error si no se encuentra el usuario o falla la conexión.
pub async fn get_user_by_username(username: String) -> Result<User, Error> {
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
        .query("SELECT * FROM users WHERE username = ?", params![username])
        .await?;

    let row = rows.next().await?.ok_or_else(|| Error {
        msg: format!("Usuario no encontrado: username = {username}"),
    })?;

    let user = User {
        id_clerk: row.get::<String>(0)?,
        nombre: row.get::<String>(1)?,
        apellidos: row.get::<String>(2)?,
        username: row.get::<String>(3)?,
        descripcion: row.get::<Option<String>>(4)?,
        clave_activacion: row.get::<Option<String>>(5)?,
        usuario_activo: row.get::<i32>(6)? != 0,
    };

    Ok(user)
}

/// Actualiza los datos de perfil de un usuario (nombre, apellidos, username y descripción).
///
/// # Parámetros
/// * `&User`: Objeto User con el `id_clerk` del usuario a actualizar y los
///   nuevos datos de perfil.
///
/// # Returns
/// * `Result<()>` - Ok si la actualización es correcta.
///
/// # Errores
/// Retorna error si falla la conexión a la base de datos o la actualización.
pub async fn update_user(user: &User) -> Result<(), Error> {
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
        "UPDATE users SET nombre = ?, apellidos = ?, username = ?, descripcion = ? WHERE id_clerk = ?",
        params![
            user.nombre.as_str(),
            user.apellidos.as_str(),
            user.username.as_str(),
            user.descripcion.clone(),
            user.id_clerk.as_str(),
        ],
    )
    .await?;

    Ok(())
}

/// Actualiza la clave de activación de un usuario.
///
/// # Parámetros
/// * `id_clerk`: Identificador del usuario en Clerk.
/// * `clave`: Nueva clave de activación.
///
/// # Returns
/// * `Result<()>` - Ok si la actualización es correcta.
///
/// # Errores
/// Retorna error si falla la conexión a la base de datos o la actualización.
pub async fn update_user_clave(id_clerk: String, clave: String) -> Result<(), Error> {
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
        "UPDATE users SET clave_activacion = ? WHERE id_clerk = ?",
        params![clave.as_str(), id_clerk],
    )
    .await?;

    Ok(())
}

/// Actualiza el estado de activación de un usuario.
///
/// # Parámetros
/// * `id_clerk`: Identificador del usuario en Clerk.
/// * `activo`: Bool que indica si el usuario está activado o no.
///
/// # Returns
/// * `Result<()>` - Ok si la actualización es correcta.
///
/// # Errores
/// Retorna error si falla la conexión a la base de datos o la actualización.
pub async fn update_user_activo(id_clerk: String, activo: bool) -> Result<(), Error> {
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
        "UPDATE users SET usuario_activo = ? WHERE id_clerk = ?",
        params![if activo { 1 } else { 0 }, id_clerk],
    )
    .await?;

    Ok(())
}

/// Elimina un usuario por su identificador de Clerk.
///
/// # Parámetros
/// * `id_clerk`: Identificador del usuario en Clerk.
///
/// # Returns
/// * `Result<()>` - Ok si la eliminación es correcta.
///
/// # Errores
/// Retorna error si falla la conexión a la base de datos o la eliminación.
pub async fn delete_user(id_clerk: String) -> Result<(), Error> {
    let (db_path, sync_url, auth_token) = get_db_config()?;

    let db = if !DB_LOCAL {
        Builder::new_remote_replica(db_path, sync_url, auth_token)
            .build()
            .await?
    } else {
        Builder::new_local(db_path).build().await?
    };

    let conn = db.connect()?;

    conn.execute("DELETE FROM users WHERE id_clerk = ?", params![id_clerk])
        .await?;

    Ok(())
}
