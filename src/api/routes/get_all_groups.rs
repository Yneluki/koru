use crate::api::response::{error, ok};
use crate::api::routes::middleware::user_session::UserId;
use crate::application::app::Application;
use crate::application::store::MultiRepository;
use crate::domain::errors::GetAllGroupsError;
use crate::domain::usecases::admin::AdminUseCase;
use crate::domain::usecases::dto::dtos::GroupDto;
use actix_web::body::BoxBody;
use actix_web::http::StatusCode;
use actix_web::{web, HttpResponse, ResponseError};
#[cfg(feature = "openapi")]
use utoipa::ToSchema;

/// Fetches all groups, only available if user is ADMIN
///
/// Requires the auth cookie from `/login` to be attached to the request.
///
/// Examples:
/// ```
/// curl -i -b cookie "http://localhost:8000/admin/groups"
/// ```
///
#[cfg_attr(feature = "openapi", utoipa::path(
    get,
    path = "/admin/groups",
    responses(
        (status = 200, description = "List of all groups", body = GetAllGroupsResponse),
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
    name = "Get all groups",
    skip(app, user_id),
    fields(
        user_id = %user_id.0,
    )
)]
pub async fn get_all_groups<Store: MultiRepository>(
    app: web::Data<Application<Store>>,
    user_id: web::ReqData<UserId>,
) -> Result<HttpResponse, GetAllGroupsError> {
    let user_id = *user_id.into_inner();
    let groups = app.admin().get_groups(&user_id).await?;

    Ok(HttpResponse::Ok().json(&ok(AllGroupsResponse { groups })))
}

#[derive(serde::Serialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct AllGroupsResponse {
    groups: Vec<GroupDto>,
}

impl ResponseError for GetAllGroupsError {
    fn error_response(&self) -> HttpResponse<BoxBody> {
        let (code, msg) = match self {
            GetAllGroupsError::Unexpected(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Unexpected server error.",
            ),
            GetAllGroupsError::Unauthenticated() => {
                (StatusCode::UNAUTHORIZED, "You are not logged in.")
            }
            GetAllGroupsError::Unauthorized() => {
                (StatusCode::FORBIDDEN, "You are not administrator.")
            }
        };
        HttpResponse::build(code).json(&error(msg))
    }
}
