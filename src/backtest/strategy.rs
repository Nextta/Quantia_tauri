#[derive(Debug, Clone)]
pub struct Strategy {
    pub id: i32,
    pub id_user: i32,
    pub nombre: String,
    pub descripcion: Option<String>,
    pub activa: bool,
    pub creada_en: String,
}

impl Strategy {
    pub fn new(
        id: i32,
        id_user: i32,
        nombre: String,
        descripcion: Option<String>,
        activa: bool,
        creada_en: String,
    ) -> Self {
        Self {
            id,
            id_user,
            nombre,
            descripcion,
            activa,
            creada_en,
        }
    }
}
