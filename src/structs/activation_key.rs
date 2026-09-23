use serde::{Deserialize, Serialize};

/// Clave de activación del producto.
///
/// Representa una fila de la tabla `activation_keys`: una clave que habilita
/// el acceso al programa para un usuario. Una clave se genera con formato
/// `XXXX-XX-XXX-XXXXXXXXXX`, se entrega al usuario tras la compra y queda
/// vinculada a él al activarse (`usada`, `id_clerk`, `fecha_activacion`).
/// Un administrador puede liberarla para reutilizarla con otro usuario.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ActivationKey {
    /// Identificador de la clave en la base de datos.
    pub id: i32,
    /// Valor de la clave de activación (formato `XXXX-XX-XXX-XXXXXXXXXX`).
    pub clave: String,
    /// Indica si la clave ya ha sido consumida por algún usuario.
    pub usada: bool,
    /// Identificador de Clerk del usuario que consumió la clave
    /// (`None` mientras la clave esté disponible).
    pub id_clerk: Option<String>,
    /// Fecha y hora de activación de la clave, en formato
    /// `YYYY-MM-DD HH:MM:SS` (`None` mientras esté disponible).
    pub fecha_activacion: Option<String>,
}
