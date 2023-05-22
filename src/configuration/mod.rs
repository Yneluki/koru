use crate::configuration::application::{ApiSettings, ApplicationSettings};
use crate::configuration::event_bus::EventBusSettings;
use crate::configuration::store::DatabaseSettings;

pub mod application;
pub mod event_bus;
#[cfg(feature = "notification")]
pub mod notification;
pub mod store;

#[derive(serde::Deserialize, Debug)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub event_bus: EventBusSettings,
    pub api: ApiSettings,
    pub application: ApplicationSettings,
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let environment: Environment = std::env::var("KORU_ENV")
        .unwrap_or_else(|_| "local".into())
        .try_into()
        .expect("Failed to parse KORU_ENV.");

    let base_path = std::env::current_dir().expect("Failed to determine the current directory");
    let config_dir = base_path.join("config");

    let conf = config::Config::builder()
        .add_source(config::File::from(config_dir.join("default")))
        .add_source(config::File::from(config_dir.join(environment.as_str())))
        .add_source(config::Environment::with_prefix("KORU").separator("__"))
        .build()?;
    conf.try_deserialize()
}

pub enum Environment {
    Local,
    #[cfg(any(feature = "production", feature = "development"))]
    Integration,
    #[cfg(any(feature = "production", feature = "development"))]
    Production,
}
impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Local => "local",
            #[cfg(any(feature = "production", feature = "development"))]
            Environment::Integration => "integration",
            #[cfg(any(feature = "production", feature = "development"))]
            Environment::Production => "prod",
        }
    }
}
impl TryFrom<String> for Environment {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            #[cfg(any(feature = "production", feature = "development"))]
            "prod" => Ok(Self::Production),
            #[cfg(any(feature = "production", feature = "development"))]
            "integration" => Ok(Self::Integration),
            other => Err(format!(
                "{} is not a supported environment. Use either `local`, `integration` (requires `--features full`) or `prod` (requires `--features full`).",
                other
            )),
        }
    }
}
