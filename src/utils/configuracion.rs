use dotenvy::dotenv;
use serde::Serialize;
use std::env;

pub const LOGS_REGISTRO: bool = false;
pub const DB_LOCAL: bool = true;

#[derive(Debug, Serialize)]
pub struct Error {
    pub msg: String,
}

type Result<T> = std::result::Result<T, Error>;

impl Error {
    pub fn msg(&self) -> &str {
        &self.msg
    }
}

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

pub fn get_db_config() -> Result<(String, String, String)> {
    dotenv().expect(".env file not found");
    let db_path = env::var("DB_PATH").unwrap();
    let sync_url = env::var("TURSO_SYNC_URL").unwrap();
    let auth_token = env::var("TURSO_AUTH_TOKEN").unwrap();
    Ok((db_path, sync_url, auth_token))
}
