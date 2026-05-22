use std::fs::OpenOptions;
use std::io::Write;

#[derive(Debug, Clone)]
pub struct RegistroLog {
    pub timestamp: String,
    pub message: String,
}

impl RegistroLog {
    pub fn new(message: String) -> RegistroLog {
        let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        RegistroLog { timestamp, message }
    }

    pub fn guardar_logs(&self) -> std::io::Result<()> {
        let path = "logs.txt";
        let mut file = OpenOptions::new()
            .create(true) // crea si no existe
            .append(true) // añade sin borrar lo anterior
            .open(path)?;
        writeln!(file, "{} | {}", self.timestamp, self.message)
    }

    pub fn delete_log_file() -> std::io::Result<()> {
        let path = "logs.txt";
        std::fs::remove_file(path)
    }
}
