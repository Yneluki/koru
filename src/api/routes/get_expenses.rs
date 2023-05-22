use crate::api::response::{error, ok};
use crate::api::routes::middleware::user_session::UserId;
use crate::application::app::Application;
use crate::application::store::MultiRepository;
use crate::domain::errors::GetExpensesError;
use crate::domain::usecases::dto::dtos::ExpenseDto;
use crate::domain::usecases::group::{GetExpensesRequest, GroupUseCase};
use actix_web::body::BoxBody;
use actix_web::http::StatusCode;
use actix_web::{web, HttpResponse, ResponseError};
use chrono::{LocalResult, TimeZone, Utc};
#[cfg(feature = "openapi")]
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

/// Fetches expenses of a group.
///     - If no filters are provided: un-settled expenses.
///     - If date filters (`from` and/or `to`) are provided: expenses within the date filters.
///     - If `settlement_id` filter is provided: expenses of a given settlement, ignoring any provided date filters.
///
/// Date filters should be Unix timestamps.
/// Settlement id should be a valid settlement Uuid of the group.
///
/// This action can only be performed by a group member.
///
/// Requires the auth cookie from `/login` to be attached to the request.
///
/// Examples (replace GROUP_ID ith the group UUID):
/// ```
/// curl -i -b cookie "http://localhost:8000/groups/GROUP_ID/expenses"
/// curl -i -b cookie "http://localhost:8000/groups/GROUP_ID/expenses?settlement_id=SETTLEMENT_ID"
/// curl -i -b cookie "http://localhost:8000/groups/GROUP_ID/expenses?from=1676869911768"
/// curl -i -b cookie "http://localhost:8000/groups/GROUP_ID/expenses?to=1676869911768"
/// curl -i -b cookie "http://localhost:8000/groups/GROUP_ID/expenses?from=1676869911768&to=1676869945455"
/// ```
///
#[cfg_attr(feature = "openapi", utoipa::path(
    get,
    path = "/groups/{group_id}/expenses",
    params(
        ("group_id" = Uuid, Path, description = "Group Uuid"),
        GetExpensesFilter,
    ),
    responses(
        (status = 200, description = "List of expenses matching the filters", body = GetExpensesResponse),
        (status = 400, description = "Invalid group id or filters", body = ErrorResponse),
        (status = 401, description = "User is not logged in", body = ErrorResponse),
        (status = 403, description = "User is not allowed", body = ErrorResponse),
        (status = 404, description = "Group or settlement not found (if settlement filter provided)", body = ErrorResponse),
        (status = 500, description = "Unexpected server error", body = ErrorResponse),
    ),
    security(
        ("cookieAuth" = [])
    ),
    tag = "Expenses",
))]
#[tracing::instrument(
    name = "Get expenses",
    skip(path_param, req_param, app, user_id),
    fields(
        user_id = %user_id.0,
        group_id = tracing::field::Empty,
        filters = tracing::field::Empty,
    )
)]
pub async fn get_expenses<Store: MultiRepository>(
    path_param: Option<web::Path<Uuid>>,
    req_param: Option<web::Query<GetExpensesFilter>>,
    app: web::Data<Application<Store>>,
    user_id: web::ReqData<UserId>,
) -> Result<HttpResponse, GetExpensesError> {
    match (path_param, req_param) {
        (Some(group_id), Some(filters)) => {
            let group_id = group_id.into_inner();
            let user_id = user_id.into_inner().0;
            let filters = filters.into_inner();
            tracing::Span::current().record("group_id", &tracing::field::display(&group_id));
            tracing::Span::current().record("filters", &tracing::field::debug(&filters));

            let from = match filters.from {
                Some(e) => match Utc.timestamp_millis_opt(e) {
                    LocalResult::Single(date) => Some(date),
                    LocalResult::None => None,
                    LocalResult::Ambiguous(_, _) => None,
                },
                None => None,
            };
            let to = match filters.to {
                Some(e) => match Utc.timestamp_millis_opt(e) {
                    LocalResult::Single(date) => Some(date),
                    LocalResult::None => None,
                    LocalResult::Ambiguous(_, _) => None,
                },
                None => None,
            };
            let settlement_id = filters.settlement_id;

            let data = GetExpensesRequest {
                group_id,
                user_id,
                settlement_id,
                from,
                to,
            };
            let expenses = app.groups().get_expenses(data).await?;

            Ok(HttpResponse::Ok().json(&ok(ExpensesResponse { expenses })))
        }
        _ => Ok(HttpResponse::BadRequest().json(&error("Group id or filters are invalid."))),
    }
}

#[derive(serde::Serialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct ExpensesResponse {
    pub expenses: Vec<ExpenseDto>,
}

#[derive(serde::Deserialize, Debug)]
#[cfg_attr(feature = "openapi", derive(IntoParams))]
#[cfg_attr(feature = "openapi", into_params(parameter_in=Query))]
pub struct GetExpensesFilter {
    from: Option<i64>,
    to: Option<i64>,
    settlement_id: Option<Uuid>,
}

impl ResponseError for GetExpensesError {
    fn error_response(&self) -> HttpResponse<BoxBody> {
        let (code, msg) = match self {
            GetExpensesError::Unexpected(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Unexpected server error.",
            ),
            GetExpensesError::NotFound(msg) => (StatusCode::NOT_FOUND, *msg),
            GetExpensesError::Unauthorized(_) => (
                StatusCode::FORBIDDEN,
                "You are not authorized to perform this action.",
            ),
            GetExpensesError::Unauthenticated() => {
                (StatusCode::UNAUTHORIZED, "You are not logged in.")
            }
        };
        HttpResponse::build(code).json(&error(msg))
    }
}
