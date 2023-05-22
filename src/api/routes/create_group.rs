use crate::api::response::{error, ok_id};
use crate::api::routes::middleware::user_session::UserId;
use crate::application::app::Application;
use crate::application::store::MultiRepository;
use crate::domain::errors::CreateGroupError;
use crate::domain::usecases::dto::dtos::ColorDto;
use crate::domain::usecases::group::{CreateGroupRequest, GroupUseCase};
use actix_web::body::BoxBody;
use actix_web::http::StatusCode;
use actix_web::{web, HttpResponse, ResponseError};
#[cfg(feature = "openapi")]
use utoipa::ToSchema;

/// Creates a new group with the member making the request as admin.
///
/// Requires the auth cookie from `/login` to be attached to the request.
///
/// Example:
/// ```
/// curl -i -H 'Content-Type: application/json' -d '{"name":"my group","color":{"red":0,"green":255,"blue":0}}' -b cookie "http://localhost:8000/groups"
/// ```
///
#[cfg_attr(feature = "openapi", utoipa::path(
    post,
    path = "/groups",
    request_body = CreateGroupPayload,
    responses(
        (status = 201, description = "Group created successfully. Returns the ID of the group created.", body = IdResponse),
        (status = 400, description = "Invalid payload", body = ErrorResponse),
        (status = 401, description = "User is not logged in", body = ErrorResponse),
        (status = 500, description = "Unexpected server error", body = ErrorResponse),
    ),
    security(
        ("cookieAuth" = [])
    ),
    tag = "Groups",
))]
#[tracing::instrument(
    name = "Create a group",
    skip(payload, app, user_id),
    fields(
        user_id = %user_id.0,
    )
)]
pub async fn create_group<Store: MultiRepository>(
    payload: web::Json<CreateGroupPayload>,
    app: web::Data<Application<Store>>,
    user_id: web::ReqData<UserId>,
) -> Result<HttpResponse, CreateGroupError> {
    let group_data = CreateGroupRequest {
        name: payload.0.name,
        admin_id: user_id.into_inner().0,
        admin_color: payload.0.color.clone(),
    };
    let group_id = app.groups().create_group(group_data).await?;
    Ok(HttpResponse::Created().json(&ok_id(group_id)))
}

#[derive(serde::Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct CreateGroupPayload {
    name: String,
    color: ColorDto,
}

impl ResponseError for CreateGroupError {
    fn error_response(&self) -> HttpResponse<BoxBody> {
        let (code, msg) = match self {
            CreateGroupError::Validation(msg) => (StatusCode::BAD_REQUEST, *msg),
            CreateGroupError::Unexpected(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Unexpected server error.",
            ),
            CreateGroupError::Unauthenticated() => {
                (StatusCode::UNAUTHORIZED, "You are not logged in.")
            }
        };
        HttpResponse::build(code).json(&error(msg))
    }
}
