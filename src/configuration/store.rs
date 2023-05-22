#[cfg(feature = "postgres")]
use secrecy::{ExposeSecret, Secret};

#[derive(serde::Deserialize, Debug)]
pub enum DatabaseSettings {
    #[cfg(feature = "postgres")]
    #[serde(rename = "postgres")]
    Postgres(PostgresSettings),
    #[serde(rename = "memory")]
    Memory,
}

#[cfg(feature = "postgres")]
#[derive(serde::Deserialize, Debug, Clone)]
pub struct PostgresSettings {
    pub username: String,
    pub password: Secret<String>,
    pub port: u16,
    pub host: String,
    pub name: String,
}

#[cfg(feature = "postgres")]
impl PostgresSettings {
    pub fn connection_string(&self) -> Secret<String> {
        Secret::new(format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port,
            self.name
        ))
    }
    pub fn connection_string_without_db(&self) -> Secret<String> {
        Secret::new(format!(
            "postgres://{}:{}@{}:{}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port
        ))
    }
}
