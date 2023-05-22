#[cfg(feature = "redis-bus")]
use secrecy::Secret;

#[derive(serde::Deserialize, Debug)]
pub enum EventBusSettings {
    #[cfg(feature = "redis-bus")]
    #[serde(rename = "redis")]
    Redis(RedisSettings),
    #[serde(rename = "memory")]
    Memory,
}

#[cfg(feature = "redis-bus")]
#[derive(serde::Deserialize, Debug, Clone)]
pub struct RedisSettings {
    pub port: u16,
    pub host: String,
    pub event_channel: String,
}

#[cfg(feature = "redis-bus")]
impl RedisSettings {
    pub fn connection_string(&self) -> Secret<String> {
        Secret::new(format!("redis://{}:{}", self.host, self.port))
    }
}
