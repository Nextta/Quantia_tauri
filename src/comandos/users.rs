use crate::api::users as api_users;
use crate::structs::user::User;
use crate::utils::configuracion::Error;

/// Crea la tabla de usuarios en la base de datos.
///
/// # Return
/// Mensaje de éxito si la tabla se crea correctamente.
#[tauri::command]
pub async fn table_users() -> Result<String, Error> {
    api_users::table_users().await
}

/// Inserta un nuevo usuario en la base de datos.
///
/// # Arguments
/// user: Datos del usuario a insertar, con su `id_clerk` como clave primaria.
///
/// # Return
/// Ok si la inserción es correcta.
///
/// # Errores
/// Error si ya existe un usuario con ese `id_clerk` o si falla la base de datos.
#[tauri::command]
pub async fn insert_user(user: User) -> Result<(), Error> {
    api_users::insert_user(&user).await
}

/// Obtiene todos los usuarios de la base de datos.
///
/// # Return
/// Vector con todos los usuarios.
#[tauri::command]
pub async fn get_users() -> Result<Vec<User>, Error> {
    api_users::get_users().await
}

/// Obtiene un usuario por su identificador de Clerk.
///
/// # Arguments
/// id_clerk: Identificador del usuario en Clerk.
///
/// # Return
/// El usuario con todos sus campos.
///
/// # Errores
/// Error si no existe ningún usuario con ese `id_clerk`.
#[tauri::command]
pub async fn get_user_by_id_clerk(id_clerk: String) -> Result<User, Error> {
    api_users::get_user_by_id_clerk(id_clerk).await
}

/// Obtiene un usuario por su username.
///
/// # Arguments
/// username: Nombre de usuario a buscar.
///
/// # Return
/// El usuario con todos sus campos.
///
/// # Errores
/// Error si no existe ningún usuario con ese username.
#[tauri::command]
pub async fn get_user_by_username(username: String) -> Result<User, Error> {
    api_users::get_user_by_username(username).await
}

/// Actualiza los datos de perfil de un usuario (nombre, apellidos, username y descripción).
///
/// # Arguments
/// user: Datos nuevos de perfil, con el `id_clerk` del usuario a actualizar.
///
/// # Return
/// Ok si la actualización es correcta.
#[tauri::command]
pub async fn update_user(user: User) -> Result<(), Error> {
    api_users::update_user(&user).await
}

/// Actualiza la clave de activación de un usuario.
///
/// # Arguments
/// id_clerk: Identificador del usuario en Clerk.
/// clave: Nueva clave de activación.
///
/// # Return
/// Ok si la actualización es correcta.
#[tauri::command]
pub async fn update_user_clave(id_clerk: String, clave: String) -> Result<(), Error> {
    api_users::update_user_clave(id_clerk, clave).await
}

/// Actualiza el estado de activación de un usuario.
///
/// # Arguments
/// id_clerk: Identificador del usuario en Clerk.
/// activo: Indica si el usuario está activado (true) o no (false).
///
/// # Return
/// Ok si la actualización es correcta.
#[tauri::command]
pub async fn update_user_activo(id_clerk: String, activo: bool) -> Result<(), Error> {
    api_users::update_user_activo(id_clerk, activo).await
}

/// Elimina un usuario por su identificador de Clerk.
///
/// # Arguments
/// id_clerk: Identificador del usuario en Clerk.
///
/// # Return
/// Ok si la eliminación es correcta.
#[tauri::command]
pub async fn delete_user(id_clerk: String) -> Result<(), Error> {
    api_users::delete_user(id_clerk).await
}
