use crate::api::response::{error, ok};
use crate::api::routes::middleware::user_session::UserId;
use crate::application::app::Application;
use crate::application::store::MultiRepository;
use crate::domain::errors::GetUsersError;
use crate::domain::usecases::admin::AdminUseCase;
use crate::domain::usecases::dto::dtos::DetailedUserDto;
use actix_web::body::BoxBody;
use actix_web::http::StatusCode;
use actix_web::{web, HttpResponse, ResponseError};
#[cfg(feature = "openapi")]
use utoipa::ToSchema;

/// Fetches all users, only available if user is ADMIN
///
/// Requires the auth cookie from `/login` to be attached to the request.
///
/// Examples:
/// ```
/// curl -i -b cookie "http://localhost:8000/admin/users"
/// ```
///
#[cfg_attr(feature = "openapi", utoipa::path(
    get,
    path = "/admin/users",
    responses(
        (status = 200, description = "List of all users", body = GetAllUsersResponse),
        (status = 401, description = "User is not logged in", body = ErrorResponse),
        (status = 403, description = "User is not Admin", body = ErrorResponse),
        (status = 500, description = "Unexpected server error", body = ErrorResponse),
    ),
    security(
        ("cookieAuth" = [])
    ),
    tag = "Admin",
))]
#[tracing::instrument(
    name = "Get all users",
    skip(app, user_id),
    fields(
        user_id = %user_id.0,
    )
)]
pub async fn get_all_users<Store: MultiRepository>(
    app: web::Data<Application<Store>>,
    user_id: web::ReqData<UserId>,
) -> Result<HttpResponse, GetUsersError> {
    let user_id = *user_id.into_inner();
    let users = app.admin().get_users(&user_id).await?;

    Ok(HttpResponse::Ok().json(&ok(AllUsersResponse { users })))
}

#[derive(serde::Serialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct AllUsersResponse {
    users: Vec<DetailedUserDto>,
}

impl ResponseError for GetUsersError {
    fn error_response(&self) -> HttpResponse<BoxBody> {
        let (code, msg) = match self {
            GetUsersError::Unexpected(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Unexpected server error.",
            ),
            GetUsersError::Unauthenticated() => {
                (StatusCode::UNAUTHORIZED, "You are not logged in.")
            }
            GetUsersError::Unauthorized() => (StatusCode::FORBIDDEN, "You are not administrator."),
        };
        HttpResponse::build(code).json(&error(msg))
    }
}
