use actix_session::{Session, SessionExt, SessionGetError, SessionInsertError};
use actix_web::dev::Payload;
use actix_web::{FromRequest, HttpRequest};
use std::future::{ready, Ready};
use std::ops::Deref;
use uuid::Uuid;

pub struct UserSession(Session);

impl UserSession {
    const USER_ID_KEY: &'static str = "user_id";
    pub fn renew(&self) {
        self.0.renew();
    }
    pub fn purge(&self) {
        self.0.purge();
    }
    pub fn insert_user_id(&self, user_id: Uuid) -> Result<(), SessionInsertError> {
        self.0.insert(Self::USER_ID_KEY, user_id)
    }
    pub fn get_user_id(&self) -> Result<Option<Uuid>, SessionGetError> {
        self.0.get(Self::USER_ID_KEY)
    }
}

impl FromRequest for UserSession {
    type Error = <Session as FromRequest>::Error;
    type Future = Ready<Result<UserSession, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        ready(Ok(UserSession(req.get_session())))
    }
}

#[derive(Copy, Clone, Debug)]
pub struct UserId(pub Uuid);

impl std::fmt::Display for UserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl Deref for UserId {
    type Target = Uuid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
