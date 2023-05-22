use crate::api::response::{error, ok};
use crate::api::routes::middleware::user_session::UserId;
use crate::application::app::Application;
use crate::application::store::MultiRepository;
use crate::domain::errors::GetGroupsError;
use crate::domain::usecases::dto::dtos::GroupDto;
use crate::domain::usecases::group::{GetGroupsRequest, GroupUseCase};
use actix_web::body::BoxBody;
use actix_web::http::StatusCode;
use actix_web::{web, HttpResponse, ResponseError};
#[cfg(feature = "openapi")]
use utoipa::ToSchema;

/// Fetches the groups of the user making the request.
///
/// Requires the auth cookie from `/login` to be attached to the request.
///
/// Examples:
/// ```
/// curl -i -b cookie "http://localhost:8000/groups"
/// ```
///
#[cfg_attr(feature = "openapi", utoipa::path(
    get,
    path = "/groups",
    responses(
        (status = 200, description = "List of groups of the user", body = GetGroupsResponse),
        (status = 401, description = "User is not logged in", body = ErrorResponse),
        (status = 500, description = "Unexpected server error", body = ErrorResponse),
    ),
    security(
        ("cookieAuth" = [])
    ),
    tag = "Groups",
))]
#[tracing::instrument(
    name = "Get groups",
    skip(app, user_id),
    fields(
        user_id = %user_id.0,
    )
)]
pub async fn get_groups<Store: MultiRepository>(
    app: web::Data<Application<Store>>,
    user_id: web::ReqData<UserId>,
) -> Result<HttpResponse, GetGroupsError> {
    let user_id = *user_id.into_inner();
    let groups = app
        .groups()
        .get_groups(GetGroupsRequest { user_id })
        .await?;

    Ok(HttpResponse::Ok().json(&ok(GroupsResponse { groups })))
}

#[derive(serde::Serialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct GroupsResponse {
    groups: Vec<GroupDto>,
}

impl ResponseError for GetGroupsError {
    fn error_response(&self) -> HttpResponse<BoxBody> {
        let (code, msg) = match self {
            GetGroupsError::Unexpected(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Unexpected server error.",
            ),
            GetGroupsError::Unauthenticated() => {
                (StatusCode::UNAUTHORIZED, "You are not logged in.")
            }
        };
        HttpResponse::build(code).json(&error(msg))
    }
}
