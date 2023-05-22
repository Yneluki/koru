use crate::api::response::{error, ok_id};
use crate::api::routes::middleware::user_session::UserSession;
use crate::application::app::Application;
use crate::application::store::MultiRepository;
use crate::domain::errors::LoginError;
use crate::domain::usecases::user::{LoginRequest, UserUseCase};
use actix_web::body::BoxBody;
use actix_web::http::StatusCode;
use actix_web::{web, HttpResponse, ResponseError};
use secrecy::Secret;
#[cfg(feature = "openapi")]
use utoipa::ToSchema;

/// Logs in the user given valid credentials.
///
/// It will return a `Set-Cookie` header, that should be used in next authenticated requests
///
/// Example:
/// ```
/// curl -i -H 'Content-Type: application/json' -d '{"password":"123","email":"r@r1.com"}' -c cookie "http://localhost:8000/login"
///```
#[cfg_attr(feature = "openapi", utoipa::path(
    post,
    path = "/login",
    request_body = LoginPayload,
    responses(
        (
            status = 200,
            description = "Successful login",
            body = IdResponse,
            headers(
                ("Set-Cookie" = String, description = "Auth cookie")
            ),
        ),
        (status = 400, description = "Validation errors in login request", body = ErrorResponse),
        (status = 401, description = "Invalid credentials", body = ErrorResponse),
        (status = 500, description = "Unexpected server error", body = ErrorResponse),
    ),
    tag = "Authentication",
))]
#[tracing::instrument(
    name = "Logging in",
    skip(payload, app, session),
    fields(
        user_email = %payload.email,
    )
)]
pub async fn login<Store: MultiRepository>(
    payload: web::Json<LoginPayload>,
    app: web::Data<Application<Store>>,
    session: UserSession,
) -> Result<HttpResponse, LoginError> {
    let request = LoginRequest {
        email: payload.0.email,
        password: payload.0.password,
    };
    let user_id = app.users().login(request).await?;
    session.renew();
    session
        .insert_user_id(user_id)
        .map_err(|e| LoginError::Unexpected(e.into()))?;
    Ok(HttpResponse::Ok().json(&ok_id(user_id)))
}

#[derive(serde::Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct LoginPayload {
    pub email: String,
    #[cfg_attr(feature = "openapi", schema(value_type=Option<String>))]
    pub password: Option<Secret<String>>,
}

impl ResponseError for LoginError {
    fn error_response(&self) -> HttpResponse<BoxBody> {
        let (code, msg) = match self {
            LoginError::Validation(msg) => (StatusCode::BAD_REQUEST, *msg),
            LoginError::InvalidCredentials() => (StatusCode::UNAUTHORIZED, "Invalid credentials."),
            LoginError::Unexpected(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Unexpected server error.",
            ),
        };
        HttpResponse::build(code).json(&error(msg))
    }
}
