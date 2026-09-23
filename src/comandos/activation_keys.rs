use crate::api::activation_keys as api_keys;
use crate::structs::activation_key::ActivationKey;
use crate::utils::configuracion::Error;

/// Crea la tabla de claves de activación en la base de datos.
///
/// # Return
/// Mensaje de éxito si la tabla se crea correctamente.
#[tauri::command]
pub async fn table_activation_keys() -> Result<String, Error> {
    api_keys::table_activation_keys().await
}

/// Inserta una nueva clave de activación en la base de datos.
///
/// # Arguments
/// clave: Valor de la clave de activación a insertar.
///
/// # Return
/// ID de la clave insertada.
///
/// # Errores
/// Error si ya existe una clave con ese valor o si falla la base de datos.
#[tauri::command]
pub async fn insert_key(clave: String) -> Result<i32, Error> {
    api_keys::insert_key(clave).await
}

/// Obtiene todas las claves de activación con su estado.
///
/// # Return
/// Vector con todas las claves.
#[tauri::command]
pub async fn get_keys() -> Result<Vec<ActivationKey>, Error> {
    api_keys::get_keys().await
}

/// Obtiene las claves de activación disponibles (no usadas).
///
/// # Return
/// Vector con las claves disponibles.
#[tauri::command]
pub async fn get_available_keys() -> Result<Vec<ActivationKey>, Error> {
    api_keys::get_available_keys().await
}

/// Obtiene una clave de activación por su ID.
///
/// # Arguments
/// id: Identificador de la clave.
///
/// # Return
/// La clave con todos sus campos.
///
/// # Errores
/// Error si no existe ninguna clave con ese ID.
#[tauri::command]
pub async fn get_key_by_id(id: i32) -> Result<ActivationKey, Error> {
    api_keys::get_key_by_id(id).await
}

/// Elimina una clave de activación por su ID.
///
/// # Arguments
/// id: Identificador de la clave a eliminar.
///
/// # Return
/// Ok si la eliminación es correcta.
#[tauri::command]
pub async fn delete_key(id: i32) -> Result<(), Error> {
    api_keys::delete_key(id).await
}

/// Genera claves de activación aleatorias con formato
/// `XXXX-XX-XXX-XXXXXXXXXX` y las guarda en la base de datos.
///
/// # Arguments
/// cantidad: Número de claves a generar.
///
/// # Return
/// Vector con las claves generadas.
#[tauri::command]
pub async fn generate_keys(cantidad: u32) -> Result<Vec<String>, Error> {
    api_keys::generate_keys(cantidad).await
}

/// Activa el acceso de un usuario mediante una clave de activación.
///
/// # Arguments
/// id_clerk: Identificador del usuario en Clerk.
/// clave: Clave de activación introducida por el usuario.
///
/// # Return
/// Ok si la activación es correcta: la clave queda vinculada al usuario,
/// guardada también en su perfil, y el usuario queda activado.
///
/// # Errores
/// Error (sin modificar la base de datos) si no existe el usuario, si ya
/// está activo o si la clave no existe o ya está usada.
#[tauri::command]
pub async fn activar_usuario(id_clerk: String, clave: String) -> Result<(), Error> {
    api_keys::activar_usuario(id_clerk, clave).await
}

/// Libera una clave de activación usada para poder reutilizarla.
///
/// # Arguments
/// id: Identificador de la clave a liberar.
///
/// # Return
/// Ok si la liberación es correcta.
///
/// # Errores
/// Error si falla la base de datos.
#[tauri::command]
pub async fn release_key(id: i32) -> Result<(), Error> {
    api_keys::release_key(id).await
}
