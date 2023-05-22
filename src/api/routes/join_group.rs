use crate::api::response::{error, ok_message};
use crate::api::routes::middleware::user_session::UserId;
use crate::application::app::Application;
use crate::application::store::MultiRepository;
use crate::domain::errors::JoinGroupError;
use crate::domain::usecases::dto::dtos::ColorDto;
use crate::domain::usecases::group::{GroupUseCase, JoinGroupRequest};
use actix_web::body::BoxBody;
use actix_web::http::StatusCode;
use actix_web::{web, HttpResponse, ResponseError};
#[cfg(feature = "openapi")]
use utoipa::ToSchema;
use uuid::Uuid;

/// Adds the user making the request to the group whose id is provided, given a valid token from `/groups/{group_id}/token`.
///
/// This action can only be performed by the member himself.
///
/// Requires the auth cookie from `/login` to be attached to the request.
///
/// Example (replace GROUP_ID ith the group UUID):
/// ```
/// curl -i -H 'Content-Type: application/json' -d '{"token":"TOKEN","color":{"red":0,"green":255,"blue":0}}' -b cookie2 "http://localhost:8000/groups/GROUP_ID/members"
/// ```
///
#[cfg_attr(feature = "openapi", utoipa::path(
    post,
    path = "/groups/{group_id}/members",
    params(
        ("group_id" = Uuid, Path, description = "Group Uuid"),
    ),
    request_body = JoinGroupPayload,
    responses(
        (status = 201, description = "Member joined successfully", body = MessageResponse),
        (status = 400, description = "Invalid payload or group id", body = ErrorResponse),
        (status = 401, description = "User is not logged in", body = ErrorResponse),
        (status = 403, description = "Token is invalid", body = ErrorResponse),
        (status = 404, description = "Group not found", body = ErrorResponse),
        (status = 409, description = "User is already a member", body = ErrorResponse),
        (status = 500, description = "Unexpected server error", body = ErrorResponse),
    ),
    security(
        ("cookieAuth" = [])
    ),
    tag = "Groups",
))]
#[tracing::instrument(
    name = "Join group",
    skip(payload, path_param, app, user_id),
    fields(
        user_id = %user_id.0,
        group_id = tracing::field::Empty,
    )
)]
pub async fn join_group<Store: MultiRepository>(
    payload: web::Json<JoinGroupPayload>,
    path_param: Option<web::Path<Uuid>>,
    app: web::Data<Application<Store>>,
    user_id: web::ReqData<UserId>,
) -> Result<HttpResponse, JoinGroupError> {
    match path_param {
        Some(path_param) => {
            let group_id = path_param.into_inner();
            let user_id = *user_id.into_inner();
            let payload = payload.into_inner();
            tracing::Span::current().record("group_id", &tracing::field::display(group_id));
            let data = JoinGroupRequest {
                group_id,
                user_id,
                color: payload.color.clone(),
                token: payload.token,
            };
            app.groups().join_group(data).await?;
            Ok(HttpResponse::Created().json(&ok_message("Group joined.")))
        }
        None => Ok(HttpResponse::BadRequest().json(&error("Group id is invalid."))),
    }
}

#[derive(serde::Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct JoinGroupPayload {
    token: String,
    color: ColorDto,
}

impl ResponseError for JoinGroupError {
    fn error_response(&self) -> HttpResponse<BoxBody> {
        let (code, msg) = match self {
            JoinGroupError::NotFound(msg) => (StatusCode::NOT_FOUND, *msg),
            JoinGroupError::Unauthorized(_) => (StatusCode::FORBIDDEN, "Invalid token."),
            JoinGroupError::Conflict() => (
                StatusCode::CONFLICT,
                "You are already a member of this group.",
            ),
            JoinGroupError::Unexpected(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Unexpected server error.",
            ),
            JoinGroupError::Unauthenticated() => {
                (StatusCode::UNAUTHORIZED, "You are not logged in.")
            }
        };
        HttpResponse::build(code).json(&error(msg))
    }
}
