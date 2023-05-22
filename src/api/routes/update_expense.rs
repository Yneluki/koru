use crate::api::response::{error, ok_message};
use crate::api::routes::middleware::user_session::UserId;
use crate::application::app::Application;
use crate::application::store::MultiRepository;
use crate::domain::errors::UpdateExpenseError;
use crate::domain::usecases::group::GroupUseCase;
use crate::domain::usecases::group::UpdateExpenseRequest;
use actix_web::body::BoxBody;
use actix_web::http::StatusCode;
use actix_web::{web, HttpResponse, ResponseError};
#[cfg(feature = "openapi")]
use utoipa::ToSchema;
use uuid::Uuid;

/// Updates and expense within the provided group id.
///
/// This action can only be performed by the member himself or the group administrator.
///
/// Requires the auth cookie from `/login` to be attached to the request.
///
/// Example (replace GROUP_ID with the group UUID and EXPENSE_ID with the expense UUID):
/// ```
/// curl -i -H 'Content-Type: application/json' -d '{"description":"my expense 2", "amount": 20}' -b cookie -X PUT "http://localhost:8000/groups/GROUP_ID/expenses/EXPENSE_ID"
/// ```
///
#[cfg_attr(feature = "openapi", utoipa::path(
    put,
    path = "/groups/{group_id}/expenses/{expense_id}",
    params(
        ("group_id" = Uuid, Path, description = "Group Uuid"),
        ("expense_id" = Uuid, Path, description = "Expense Uuid"),
    ),
    request_body = UpdateExpensePayload,
    responses(
        (status = 200, description = "Expense updated successfully", body = MessageResponse),
        (status = 400, description = "Invalid payload, group id or expense id", body = ErrorResponse),
        (status = 401, description = "User is not logged in", body = ErrorResponse),
        (status = 403, description = "User is not allowed", body = ErrorResponse),
        (status = 404, description = "Group or expense not found", body = ErrorResponse),
        (status = 500, description = "Unexpected server error", body = ErrorResponse),
    ),
    security(
        ("cookieAuth" = [])
    ),
    tag = "Expenses",
))]
#[tracing::instrument(
    name = "Update expense",
    skip(payload, path_param, app, user_id),
    fields(
        user_id = %user_id.0,
        group_id = tracing::field::Empty,
        expense_id = tracing::field::Empty,
    )
)]
pub async fn update_expense<Store: MultiRepository>(
    path_param: Option<web::Path<(Uuid, Uuid)>>,
    payload: web::Json<UpdateExpensePayload>,
    app: web::Data<Application<Store>>,
    user_id: web::ReqData<UserId>,
) -> Result<HttpResponse, UpdateExpenseError> {
    match path_param {
        Some(path_param) => {
            let (group_id, expense_id) = path_param.into_inner();
            let user_id = *user_id.into_inner();
            let description = payload.0.description;
            let amount = payload.0.amount;
            tracing::Span::current().record("group_id", &tracing::field::display(&group_id));
            tracing::Span::current().record("expense_id", &tracing::field::display(&expense_id));
            let data = UpdateExpenseRequest {
                group_id,
                expense_id,
                user_id,
                description,
                amount,
            };
            app.groups().update_expense(data).await?;

            Ok(HttpResponse::Ok().json(&ok_message("Expense updated.")))
        }
        None => Ok(HttpResponse::BadRequest().json(&error("Group or Expense id are invalid."))),
    }
}

#[derive(serde::Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct UpdateExpensePayload {
    description: String,
    amount: f32,
}

impl ResponseError for UpdateExpenseError {
    fn error_response(&self) -> HttpResponse<BoxBody> {
        let (code, msg) = match self {
            UpdateExpenseError::Validation(msg) => (StatusCode::BAD_REQUEST, *msg),
            UpdateExpenseError::NotFound(msg) => (StatusCode::NOT_FOUND, *msg),
            UpdateExpenseError::Unauthorized(_) => (
                StatusCode::FORBIDDEN,
                "You are not authorized to update this expense.",
            ),
            UpdateExpenseError::Unexpected(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Unexpected server error.",
            ),
            UpdateExpenseError::Unauthenticated() => {
                (StatusCode::UNAUTHORIZED, "You are not logged in.")
            }
        };
        HttpResponse::build(code).json(&error(msg))
    }
}
