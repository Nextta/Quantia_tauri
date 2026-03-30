#[derive(Debug, Clone)]
pub struct Strategy {
    pub id: i32,
    pub id_user: i32,
    pub nombre: String,
    pub descripcion: Option<String>,
    pub activa: bool,
    pub creada_en: String,
}
