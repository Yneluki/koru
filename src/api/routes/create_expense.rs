use crate::api::response::{error, ok_id};
use crate::api::routes::middleware::user_session::UserId;
use crate::application::app::Application;
use crate::application::store::MultiRepository;
use crate::domain::errors::CreateExpenseError;
use crate::domain::usecases::group::{CreateExpenseRequest, GroupUseCase};
use actix_web::body::BoxBody;
use actix_web::http::StatusCode;
use actix_web::{web, HttpResponse, ResponseError};
#[cfg(feature = "openapi")]
use utoipa::ToSchema;
use uuid::Uuid;

/// Creates a new expense for the member making the request, within the provided group id.
///
/// This action can only be performed by the member himself.
///
/// Requires the auth cookie from `/login` to be attached to the request.
///
/// Example (replace GROUP_ID ith the group UUID):
/// ```
/// curl -i -H 'Content-Type: application/json' -d '{"description":"my expense", "amount": 12}' -b cookie "http://localhost:8000/groups/GROUP_ID/expenses"
/// ```
///
#[cfg_attr(feature = "openapi", utoipa::path(
    post,
    path = "/groups/{group_id}/expenses",
    params(
        ("group_id" = Uuid, Path, description = "Group Uuid"),
    ),
    request_body = CreateExpensePayload,
    responses(
        (status = 201, description = "Expense created successfully. Returns the ID of the expense created.", body = IdResponse),
        (status = 400, description = "Invalid payload or group id", body = ErrorResponse),
        (status = 401, description = "User is not logged in", body = ErrorResponse),
        (status = 403, description = "User is not allowed", body = ErrorResponse),
        (status = 404, description = "Group not found", body = ErrorResponse),
        (status = 500, description = "Unexpected server error", body = ErrorResponse),
    ),
    security(
        ("cookieAuth" = [])
    ),
    tag = "Expenses",
))]
#[tracing::instrument(
    name = "Create expense",
    skip(payload, path_param, app, user_id),
    fields(
        user_id = %user_id.0,
        group_id = tracing::field::Empty,
    )
)]
pub async fn create_expense<Store: MultiRepository>(
    payload: web::Json<CreateExpensePayload>,
    path_param: Option<web::Path<Uuid>>,
    app: web::Data<Application<Store>>,
    user_id: web::ReqData<UserId>,
) -> Result<HttpResponse, CreateExpenseError> {
    match path_param {
        Some(group_id) => {
            let group_id = group_id.into_inner();
            tracing::Span::current().record("group_id", &tracing::field::display(&group_id));
            let expense_data = CreateExpenseRequest {
                group_id,
                member_id: user_id.into_inner().0,
                title: payload.0.description,
                amount: payload.0.amount,
            };
            let expense_id = app.groups().create_expense(expense_data).await?;
            Ok(HttpResponse::Created().json(&ok_id(expense_id)))
        }
        None => Ok(HttpResponse::BadRequest().json(&error("Group id is invalid."))),
    }
}

#[derive(serde::Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct CreateExpensePayload {
    description: String,
    amount: f32,
}

impl ResponseError for CreateExpenseError {
    fn error_response(&self) -> HttpResponse<BoxBody> {
        let (code, msg) = match self {
            CreateExpenseError::Validation(msg) => (StatusCode::BAD_REQUEST, *msg),
            CreateExpenseError::Unexpected(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Unexpected server error.",
            ),
            CreateExpenseError::GroupNotFound() => (StatusCode::NOT_FOUND, "Group not found."),
            CreateExpenseError::Unauthorized() => (
                StatusCode::FORBIDDEN,
                "You are not authorized to perform this action.",
            ),
            CreateExpenseError::Unauthenticated() => {
                (StatusCode::UNAUTHORIZED, "You are not logged in.")
            }
        };
        HttpResponse::build(code).json(&error(msg))
    }
}
