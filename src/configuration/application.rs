#[cfg(feature = "notification")]
use crate::configuration::notification::NotificationSettings;
use crate::domain::TokenGenerator;
#[cfg(feature = "jwt")]
use crate::infrastructure::token_generator::JwtTokenGenerator;
use secrecy::Secret;

#[derive(serde::Deserialize, Debug)]
pub struct ApiSettings {
    pub port: u16,
    pub host: String,
    pub session: SessionSettings,
}

#[derive(serde::Deserialize, Debug)]
pub struct ApplicationSettings {
    pub auth: AuthSettings,
    pub token: TokenSettings,
    #[cfg(feature = "notification")]
    pub notification: Option<NotificationSettings>,
}

#[derive(serde::Deserialize, Debug)]
pub enum AuthSettings {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "internal")]
    Internal,
}

#[derive(serde::Deserialize, Debug)]
pub struct SessionSettings {
    pub duration: u16,
    pub hmac: Secret<String>,
    pub store: SessionStoreSettings,
}

#[derive(serde::Deserialize, Debug)]
pub enum SessionStoreSettings {
    #[cfg(feature = "redis-session")]
    #[serde(rename = "redis")]
    Redis(RedisSessionSettings),
    #[serde(rename = "memory")]
    Memory,
}

#[cfg(feature = "redis-session")]
#[derive(serde::Deserialize, Debug)]
pub struct RedisSessionSettings {
    pub host: String,
    pub port: u16,
}

#[derive(serde::Deserialize, Debug)]
pub enum TokenSettings {
    #[cfg(feature = "jwt")]
    #[serde(rename = "jwt")]
    Jwt(JwtTokenSettings),
}

#[cfg(feature = "jwt")]
#[derive(serde::Deserialize, Debug)]
pub struct JwtTokenSettings {
    pub secret: Secret<String>,
}

impl ApiSettings {
    pub fn address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

impl TokenSettings {
    pub fn setup_token_generator(&self) -> anyhow::Result<impl TokenGenerator> {
        match self {
            #[cfg(feature = "jwt")]
            TokenSettings::Jwt(conf) => Ok(JwtTokenGenerator::new(conf.secret.clone())),
        }
    }
}

#[cfg(feature = "redis-session")]
impl RedisSessionSettings {
    pub fn connection_string(&self) -> Secret<String> {
        Secret::new(format!("redis://{}:{}", self.host, self.port))
    }
}
