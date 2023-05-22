use crate::api::response::{error, ok};
use crate::api::routes::middleware::user_session::UserId;
use crate::application::app::Application;
use crate::application::store::MultiRepository;
use crate::domain::errors::GetGroupError;
use crate::domain::usecases::dto::dtos::DetailedGroupDto;
use crate::domain::usecases::group::{GetGroupRequest, GroupUseCase};
use actix_web::body::BoxBody;
use actix_web::http::StatusCode;
use actix_web::{web, HttpResponse, ResponseError};
#[cfg(feature = "openapi")]
use utoipa::ToSchema;
use uuid::Uuid;

/// Fetches a group, with it's un-settled expenses.
///
/// This action can only be performed by a group member.
///
/// Requires the auth cookie from `/login` to be attached to the request.
///
/// Examples (replace GROUP_ID ith the group UUID):
/// ```
/// curl -i -b cookie "http://localhost:8000/groups/GROUP_ID"
/// ```
///
#[cfg_attr(feature = "openapi", utoipa::path(
    get,
    path = "/groups/{group_id}",
    params(
        ("group_id" = Uuid, Path, description = "Group Uuid"),
    ),
    responses(
        (status = 200, description = "Group details", body = GetGroupResponse),
        (status = 400, description = "Invalid group id", body = ErrorResponse),
        (status = 401, description = "User is not logged in", body = ErrorResponse),
        (status = 403, description = "User is not allowed", body = ErrorResponse),
        (status = 404, description = "Group not found", body = ErrorResponse),
        (status = 500, description = "Unexpected server error", body = ErrorResponse),
    ),
    security(
        ("cookieAuth" = [])
    ),
    tag = "Groups",
))]
#[tracing::instrument(
    name = "Get group",
    skip(app, user_id),
    fields(
        user_id = %user_id.0,
        group_id = tracing::field::Empty,
    )
)]
pub async fn get_group<Store: MultiRepository>(
    path_param: Option<web::Path<Uuid>>,
    app: web::Data<Application<Store>>,
    user_id: web::ReqData<UserId>,
) -> Result<HttpResponse, GetGroupError> {
    match path_param {
        Some(group_id) => {
            let group_id = group_id.into_inner();
            let user_id = *user_id.into_inner();
            tracing::Span::current().record("group_id", &tracing::field::display(&group_id));
            let data = GetGroupRequest { group_id, user_id };
            let group = app.groups().get_group(data).await?;
            Ok(HttpResponse::Ok().json(&ok(GroupResponse { group })))
        }
        None => Ok(HttpResponse::BadRequest().json(&error("Group id is invalid."))),
    }
}

#[derive(serde::Serialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct GroupResponse {
    group: DetailedGroupDto,
}

impl ResponseError for GetGroupError {
    fn error_response(&self) -> HttpResponse<BoxBody> {
        let (code, msg) = match self {
            GetGroupError::Unexpected(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Unexpected server error.",
            ),
            GetGroupError::NotFound(msg) => (StatusCode::NOT_FOUND, *msg),
            GetGroupError::Unauthorized(_) => (
                StatusCode::FORBIDDEN,
                "You are not authorized to perform this action.",
            ),
            GetGroupError::Unauthenticated() => {
                (StatusCode::UNAUTHORIZED, "You are not logged in.")
            }
        };
        HttpResponse::build(code).json(&error(msg))
    }
}
