use crate::api::response::{error, ok};
use crate::api::routes::middleware::user_session::UserId;
use crate::application::app::Application;
use crate::application::store::MultiRepository;
use crate::domain::errors::GetSettlementsError;
use crate::domain::usecases::dto::dtos::SettlementDto;
use crate::domain::usecases::group::{GetSettlementsRequest, GroupUseCase};
use actix_web::body::BoxBody;
use actix_web::http::StatusCode;
use actix_web::{web, HttpResponse, ResponseError};
#[cfg(feature = "openapi")]
use utoipa::ToSchema;
use uuid::Uuid;

/// Fetches all settlements of the group.
///
/// This action can only be performed by a group member.
///
/// Requires the auth cookie from `/login` to be attached to the request.
///
/// Examples (replace GROUP_ID ith the group UUID):
/// ```
/// curl -i -b cookie "http://localhost:8000/groups/GROUP_ID/settlements"
/// ```
///
#[cfg_attr(feature = "openapi", utoipa::path(
    get,
    path = "/groups/{group_id}/settlements",
    params(
        ("group_id" = Uuid, Path, description = "Group Uuid"),
    ),
    responses(
        (status = 200, description = "List of settlements of the group", body = GetSettlementsResponse),
        (status = 400, description = "Invalid group id", body = ErrorResponse),
        (status = 401, description = "User is not logged in", body = ErrorResponse),
        (status = 403, description = "User is not allowed", body = ErrorResponse),
        (status = 404, description = "Group not found", body = ErrorResponse),
        (status = 500, description = "Unexpected server error", body = ErrorResponse),
    ),
    security(
        ("cookieAuth" = [])
    ),
    tag = "Settlements",
))]
#[tracing::instrument(
    name = "Get settlements",
    skip(app, user_id),
    fields(
        user_id = %user_id.0,
        group_id = tracing::field::Empty,
    )
)]
pub async fn get_settlements<Store: MultiRepository>(
    path_param: Option<web::Path<Uuid>>,
    app: web::Data<Application<Store>>,
    user_id: web::ReqData<UserId>,
) -> Result<HttpResponse, GetSettlementsError> {
    match path_param {
        Some(group_id) => {
            let group_id = group_id.into_inner();
            let user_id = *user_id.into_inner();
            tracing::Span::current().record("group_id", &tracing::field::display(&group_id));
            let data = GetSettlementsRequest { group_id, user_id };
            let settlements = app.groups().get_settlements(data).await?;
            Ok(HttpResponse::Ok().json(&ok(SettlementsResponse { settlements })))
        }
        None => Ok(HttpResponse::BadRequest().json(&error("Group id is invalid."))),
    }
}

#[derive(serde::Serialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct SettlementsResponse {
    settlements: Vec<SettlementDto>,
}

impl ResponseError for GetSettlementsError {
    fn error_response(&self) -> HttpResponse<BoxBody> {
        let (code, msg) = match self {
            GetSettlementsError::Unexpected(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Unexpected server error.",
            ),
            GetSettlementsError::NotFound(msg) => (StatusCode::NOT_FOUND, *msg),
            GetSettlementsError::Unauthorized(_) => (
                StatusCode::FORBIDDEN,
                "You are not authorized to perform this action.",
            ),
            GetSettlementsError::Unauthenticated() => {
                (StatusCode::UNAUTHORIZED, "You are not logged in.")
            }
        };
        HttpResponse::build(code).json(&error(msg))
    }
}
