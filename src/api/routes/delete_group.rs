use crate::api::response::{error, ok_message};
use crate::api::routes::middleware::user_session::UserId;
use crate::application::app::Application;
use crate::application::store::MultiRepository;
use crate::domain::errors::DeleteGroupError;
use crate::domain::usecases::group::{DeleteGroupRequest, GroupUseCase};
use actix_web::body::BoxBody;
use actix_web::http::StatusCode;
use actix_web::{web, HttpResponse, ResponseError};
use uuid::Uuid;

/// Deletes a group.
///
/// This action can only be performed by the group administrator.
///
/// Requires the auth cookie from `/login` to be attached to the request.
///
/// Example (replace GROUP_ID ith the group UUID):
/// ```
/// curl -i -b cookie -X DELETE "http://localhost:8000/groups/GROUP_ID"
/// ```
///
#[cfg_attr(feature = "openapi", utoipa::path(
    delete,
    path = "/groups/{group_id}",
    params(
        ("group_id" = Uuid, Path, description = "Group Uuid"),
    ),
    responses(
        (status = 204, description = "Group deleted successfully.", body = MessageResponse),
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
    name = "Delete group",
    skip(path_param, app, user_id),
    fields(
        user_id = %user_id.0,
        group_id = tracing::field::Empty,
    )
)]
pub async fn delete_group<Store: MultiRepository>(
    path_param: Option<web::Path<Uuid>>,
    app: web::Data<Application<Store>>,
    user_id: web::ReqData<UserId>,
) -> Result<HttpResponse, DeleteGroupError> {
    match path_param {
        Some(path_param) => {
            let group_id = path_param.into_inner();
            tracing::Span::current().record("group_id", &tracing::field::display(group_id));
            let data = DeleteGroupRequest {
                group_id,
                user_id: *user_id.into_inner(),
            };
            app.groups().delete_group(data).await?;
            Ok(HttpResponse::NoContent().json(&ok_message("Group deleted.")))
        }
        None => Ok(HttpResponse::BadRequest().json(&error("Group id is invalid."))),
    }
}

impl ResponseError for DeleteGroupError {
    fn error_response(&self) -> HttpResponse<BoxBody> {
        let (code, msg) = match self {
            DeleteGroupError::NotFound() => (StatusCode::NOT_FOUND, "Group not found."),
            DeleteGroupError::Unauthorized() => (
                StatusCode::FORBIDDEN,
                "You are not authorized to delete this group.",
            ),
            DeleteGroupError::Unexpected(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Unexpected server error.",
            ),
            DeleteGroupError::Unauthenticated() => {
                (StatusCode::UNAUTHORIZED, "You are not logged in.")
            }
        };
        HttpResponse::build(code).json(&error(msg))
    }
}
