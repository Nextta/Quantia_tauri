/// Usuario de la aplicación.
///
/// Representa a un usuario registrado en la base de datos (`users`).
/// La autenticación la gestiona Clerk en el frontend; aquí solo se
/// almacena su identificador (`id_clerk`) y los datos de perfil.
/// El acceso al programa depende de `usuario_activo`, que se activa
/// mediante una `clave_activacion` válida.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    /// Identificador del usuario en Clerk (clave primaria en la tabla `users`).
    pub id_clerk: String,
    /// Nombre del usuario.
    pub nombre: String,
    /// Apellidos del usuario.
    pub apellidos: String,
    /// Nombre de usuario (username).
    pub username: String,
    /// Descripción del usuario (opcional hasta que el usuario la añada).
    pub descripcion: Option<String>,
    /// Clave de activación del producto (opcional hasta la compra/activación).
    pub clave_activacion: Option<String>,
    /// Indica si el usuario está activado (tiene acceso al programa).
    pub usuario_activo: bool,
}
