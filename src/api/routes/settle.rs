use crate::api::response::{error, ok};
use crate::api::routes::middleware::user_session::UserId;
use crate::application::app::Application;
use crate::application::store::MultiRepository;
use crate::domain::errors::SettlementError;
use crate::domain::usecases::group::GroupUseCase;
use crate::domain::usecases::group::SettleRequest;
use actix_web::body::BoxBody;
use actix_web::http::StatusCode;
use actix_web::{web, HttpResponse, ResponseError};
use uuid::Uuid;

/// Settles the current expenses and returns the settlement summary.
///
/// This action can only be performed by the group administrator.
///
/// Requires the auth cookie from `/login` to be attached to the request.
///
/// Example (replace GROUP_ID ith the group UUID):
/// ```
/// curl -i -H 'Content-Type: application/json' -b cookie -X POST "http://localhost:8000/groups/GROUP_ID/settlements"
/// ```
///
#[cfg_attr(feature = "openapi", utoipa::path(
    post,
    path = "/groups/{group_id}/settlements",
    params(
        ("group_id" = Uuid, Path, description = "Group Uuid"),
    ),
    responses(
        (status = 201, description = "Settlement computed", body = SettlementResponse),
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
    name = "Settle",
    skip(path_param, app, user_id),
    fields(
        user_id = %user_id.0,
        group_id = tracing::field::Empty,
    )
)]
pub async fn settle<Store: MultiRepository>(
    path_param: Option<web::Path<Uuid>>,
    app: web::Data<Application<Store>>,
    user_id: web::ReqData<UserId>,
) -> Result<HttpResponse, SettlementError> {
    match path_param {
        Some(group_id) => {
            let group_id = group_id.into_inner();
            let user_id = *user_id.into_inner();
            tracing::Span::current().record("group_id", &tracing::field::display(&group_id));
            let data = SettleRequest { group_id, user_id };
            let settlement = app.groups().settle(data).await?;
            Ok(HttpResponse::Created().json(&ok(settlement)))
        }
        None => Ok(HttpResponse::BadRequest().json(&error("Group id is invalid."))),
    }
}

impl ResponseError for SettlementError {
    fn error_response(&self) -> HttpResponse<BoxBody> {
        let (code, msg) = match self {
            SettlementError::Unexpected(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Unexpected server error.",
            ),
            SettlementError::NotFound(msg) => (StatusCode::NOT_FOUND, *msg),
            SettlementError::Unauthorized(_) => (
                StatusCode::FORBIDDEN,
                "You are not authorized to perform this action.",
            ),
            SettlementError::Unauthenticated() => {
                (StatusCode::UNAUTHORIZED, "You are not logged in.")
            }
        };
        HttpResponse::build(code).json(&error(msg))
    }
}
