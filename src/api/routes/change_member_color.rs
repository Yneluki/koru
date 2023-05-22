use crate::api::response::{error, ok_message};
use crate::api::routes::middleware::user_session::UserId;
use crate::application::app::Application;
use crate::application::store::MultiRepository;
use crate::domain::errors::ChangeMemberColorError;
use crate::domain::usecases::dto::dtos::ColorDto;
use crate::domain::usecases::group::{ChangeMemberColorRequest, GroupUseCase};
use actix_web::body::BoxBody;
use actix_web::http::StatusCode;
use actix_web::{web, HttpResponse, ResponseError};
#[cfg(feature = "openapi")]
use utoipa::ToSchema;
use uuid::Uuid;

/// Changes the color of the member making the request, within the provided group id.
///
/// This action can only be performed by the member himself.
///
/// Requires the auth cookie from `/login` to be attached to the request.
///
/// Example (replace GROUP_ID ith the group UUID):
/// ```
/// curl -i -H 'Content-Type: application/json' -d '{"color":{"red":255,"green":255,"blue":255}}' -b cookie -X PATCH "http://localhost:8000/groups/GROUP_ID/members"
/// ```
///
#[cfg_attr(feature = "openapi", utoipa::path(
    patch,
    path = "/groups/{group_id}/members",
    params(
        ("group_id" = Uuid, Path, description = "Group Uuid"),
    ),
    request_body = ChangeColorPayload,
    responses(
        (status = 200, description = "Color changed successfully", body = MessageResponse),
        (status = 400, description = "Invalid payload or group id", body = ErrorResponse),
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
    name = "Change color",
    skip(payload, path_param, app, user_id),
    fields(
        user_id = %user_id.0,
        group_id = tracing::field::Empty,
    )
)]
pub async fn change_color<Store: MultiRepository>(
    payload: web::Json<ChangeColorPayload>,
    path_param: Option<web::Path<Uuid>>,
    app: web::Data<Application<Store>>,
    user_id: web::ReqData<UserId>,
) -> Result<HttpResponse, ChangeMemberColorError> {
    match path_param {
        Some(path_param) => {
            let group_id = path_param.into_inner();
            let user_id = *user_id.into_inner();
            let payload = payload.into_inner();
            tracing::Span::current().record("group_id", &tracing::field::display(group_id));
            let data = ChangeMemberColorRequest {
                group_id,
                user_id,
                color: payload.color.clone(),
            };
            app.groups().change_member_color(data).await?;
            Ok(HttpResponse::Ok().json(&ok_message("Color changed.")))
        }
        None => Ok(HttpResponse::BadRequest().json(&error("Group id is invalid."))),
    }
}

#[derive(serde::Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct ChangeColorPayload {
    color: ColorDto,
}

impl ResponseError for ChangeMemberColorError {
    fn error_response(&self) -> HttpResponse<BoxBody> {
        let (code, msg) = match self {
            ChangeMemberColorError::NotFound(msg) => (StatusCode::NOT_FOUND, *msg),
            ChangeMemberColorError::Unauthorized(msg) => (StatusCode::FORBIDDEN, *msg),
            ChangeMemberColorError::Unexpected(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Unexpected server error.",
            ),
            ChangeMemberColorError::Unauthenticated() => {
                (StatusCode::UNAUTHORIZED, "You are not logged in.")
            }
        };
        HttpResponse::build(code).json(&error(msg))
    }
}
