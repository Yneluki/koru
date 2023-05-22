use crate::api::response::error;
use crate::api::routes::middleware::user_session::{UserId, UserSession};
use crate::error_chain;
use actix_web::body::{BoxBody, MessageBody};
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::http::StatusCode;
use actix_web::{FromRequest, HttpMessage, HttpResponse, ResponseError};
use actix_web_lab::middleware::Next;
use anyhow::anyhow;
use log::info;

pub async fn reject_anonymous_users(
    mut req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, actix_web::Error> {
    let session = {
        let (http_request, payload) = req.parts_mut();
        UserSession::from_request(http_request, payload).await
    }?;
    session
        .get_user_id()
        .map_err(|e| AuthorizationError::Unexpected(anyhow!(e)))
        .and_then(|id| match id {
            None => Err(AuthorizationError::Unexpected(anyhow!(
                "User is not logged in."
            ))),
            Some(user_id) => {
                info!("Session user is {}", user_id);
                req.extensions_mut().insert(UserId(user_id));
                Ok(())
            }
        })
        .map_err(actix_web::Error::from)?;

    next.call(req).await
}

error_chain! {
    #[derive(thiserror::Error)]
    pub enum AuthorizationError {
        #[error(transparent)]
        Unexpected(#[from] anyhow::Error),
    }
}

impl ResponseError for AuthorizationError {
    fn error_response(&self) -> HttpResponse<BoxBody> {
        let (code, msg) = match self {
            AuthorizationError::Unexpected(_) => {
                (StatusCode::UNAUTHORIZED, "You are not logged in.")
            }
        };
        HttpResponse::build(code).json(&error(msg))
    }
}
