use crate::api::response::{error, ok_message};
use crate::application::app::Application;
use crate::application::store::MultiRepository;
use crate::domain::errors::CreateUserError;
use crate::domain::usecases::user::{RegistrationRequest, UserUseCase};
use actix_web::body::BoxBody;
use actix_web::http::StatusCode;
use actix_web::{web, HttpResponse, ResponseError};
use secrecy::Secret;
use serde::Deserialize;
#[cfg(feature = "openapi")]
use utoipa::ToSchema;

/// Registers a new user.
///
/// Example:
/// ```
/// curl -i -H 'Content-Type: application/json' -d '{"password":"123","email":"r@r1.com","name":"Bob"}' -c cookie "http://localhost:8000/register"
///```
#[cfg_attr(feature = "openapi", utoipa::path(
    post,
    path = "/register",
    request_body = RegisterPayload,
    responses(
        (status = 201, description = "Successful registration", body = MessageResponse),
        (status = 400, description = "Validation errors in registration request", body = ErrorResponse),
        (status = 500, description = "Unexpected server error", body = ErrorResponse),
    ),
    tag = "Authentication",
))]
#[tracing::instrument(
    name = "Registering new user",
    skip(payload, app),
    fields(
        user_email = %payload.email,
        user_name = %payload.name
    )
)]
pub async fn register<Store: MultiRepository>(
    payload: web::Json<RegisterPayload>,
    app: web::Data<Application<Store>>,
) -> Result<HttpResponse, CreateUserError> {
    let req = RegistrationRequest {
        name: payload.0.name.clone(),
        email: payload.0.email.clone(),
        password: payload.0.password.clone(),
    };
    app.users().register(req).await?;
    Ok(HttpResponse::Created().json(&ok_message("User created.")))
}

#[derive(Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct RegisterPayload {
    pub name: String,
    pub email: String,
    #[cfg_attr(feature = "openapi", schema(value_type=Option<String>))]
    pub password: Option<Secret<String>>,
}

impl ResponseError for CreateUserError {
    fn error_response(&self) -> HttpResponse<BoxBody> {
        let (code, msg) = match self {
            CreateUserError::Validation(msg) => (StatusCode::BAD_REQUEST, *msg),
            CreateUserError::Unexpected(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Unexpected server error.",
            ),
            CreateUserError::Conflict() => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Unexpected server error.",
            ),
        };
        HttpResponse::build(code).json(&error(msg))
    }
}
