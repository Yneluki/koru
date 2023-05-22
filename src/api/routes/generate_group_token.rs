use crate::api::response::{error, ok};
use crate::api::routes::middleware::user_session::UserId;
use crate::application::app::Application;
use crate::application::store::MultiRepository;
use crate::domain::errors::GenerateGroupTokenError;
use crate::domain::usecases::group::{GenerateGroupTokenRequest, GroupUseCase};
use actix_web::body::BoxBody;
use actix_web::http::StatusCode;
use actix_web::{web, HttpResponse, ResponseError};
#[cfg(feature = "openapi")]
use utoipa::ToSchema;
use uuid::Uuid;

/// Generates a token for joining the group, valid 15 minutes.
///
/// This action can only be performed by the group administrator.
///
/// Requires the auth cookie from `/login` to be attached to the request.
///
/// Example (replace GROUP_ID ith the group UUID):
/// ```
/// curl -i -b cookie "http://localhost:8000/groups/GROUP_ID/token"
/// ```
///
#[cfg_attr(feature = "openapi", utoipa::path(
    get,
    path = "/groups/{group_id}/token",
    params(
        ("group_id" = Uuid, Path, description = "Group Uuid"),
    ),
    responses(
        (status = 200, description = "Group token generated", body = GenerateTokenResponse),
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
    name = "Generate group token",
    skip(path_param, app, user_id),
    fields(
        user_id = %user_id.0,
        group_id = tracing::field::Empty,
    )
)]
pub async fn generate_group_token<Store: MultiRepository>(
    path_param: Option<web::Path<Uuid>>,
    app: web::Data<Application<Store>>,
    user_id: web::ReqData<UserId>,
) -> Result<HttpResponse, GenerateGroupTokenError> {
    match path_param {
        Some(path_param) => {
            let group_id = path_param.into_inner();
            tracing::Span::current().record("group_id", &tracing::field::display(group_id));
            let data = GenerateGroupTokenRequest {
                group_id,
                user_id: *user_id.into_inner(),
            };
            let token = app.groups().generate_token(data).await?;
            Ok(HttpResponse::Ok().json(&ok(GroupTokenResponse { token })))
        }
        None => Ok(HttpResponse::BadRequest().json(&error("Group id is invalid."))),
    }
}

#[derive(serde::Serialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct GroupTokenResponse {
    token: String,
}

impl ResponseError for GenerateGroupTokenError {
    fn error_response(&self) -> HttpResponse<BoxBody> {
        let (code, msg) = match self {
            GenerateGroupTokenError::NotFound(msg) => (StatusCode::NOT_FOUND, *msg),
            GenerateGroupTokenError::Unauthorized() => (
                StatusCode::FORBIDDEN,
                "You are not authorized to perform this action.",
            ),
            GenerateGroupTokenError::Unexpected(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Unexpected server error.",
            ),
            GenerateGroupTokenError::Unauthenticated() => {
                (StatusCode::UNAUTHORIZED, "You are not logged in.")
            }
        };
        HttpResponse::build(code).json(&error(msg))
    }
}
