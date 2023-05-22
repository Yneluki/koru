use crate::api::response::{error, ok_message};
use crate::api::routes::middleware::user_session::UserId;
use crate::application::app::Application;
use crate::application::store::MultiRepository;
use crate::domain::errors::DeleteExpenseError;
use crate::domain::usecases::group::{DeleteExpenseRequest, GroupUseCase};
use actix_web::body::BoxBody;
use actix_web::http::StatusCode;
use actix_web::{web, HttpResponse, ResponseError};
use uuid::Uuid;

/// Deletes an expense.
///
/// This action can only be performed by the member himself, or the group administrator.
///
/// Requires the auth cookie from `/login` to be attached to the request.
///
/// Example (replace GROUP_ID ith the group UUID and EXPENSE_ID with the expense Uuid):
/// ```
/// curl -i -b cookie -X DELETE "http://localhost:8000/groups/GROUP_ID/expenses/EXPENSE_ID"
/// ```
///
#[cfg_attr(feature = "openapi", utoipa::path(
    delete,
    path = "/groups/{group_id}/expenses/{expense_id}",
    params(
        ("group_id" = Uuid, Path, description = "Group Uuid"),
        ("expense_id" = Uuid, Path, description = "Expense Uuid"),
    ),
    responses(
        (status = 204, description = "Expense deleted successfully.", body = MessageResponse),
        (status = 400, description = "Invalid expense or group id", body = ErrorResponse),
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
    name = "Delete expense",
    skip(path_param, app, user_id),
    fields(
        user_id = %user_id.0,
        group_id = tracing::field::Empty,
        expense_id = tracing::field::Empty,
    )
)]
pub async fn delete_expense<Store: MultiRepository>(
    path_param: Option<web::Path<(Uuid, Uuid)>>,
    app: web::Data<Application<Store>>,
    user_id: web::ReqData<UserId>,
) -> Result<HttpResponse, DeleteExpenseError> {
    match path_param {
        Some(path_param) => {
            let (group_id, expense_id) = path_param.into_inner();
            let user_id = *user_id.into_inner();
            tracing::Span::current().record("group_id", &tracing::field::display(&group_id));
            tracing::Span::current().record("expense_id", &tracing::field::display(&expense_id));
            let delete_expense_data = DeleteExpenseRequest {
                group_id,
                expense_id,
                user_id,
            };
            app.groups().delete_expense(delete_expense_data).await?;
            Ok(HttpResponse::NoContent().json(&ok_message("Expense deleted.")))
        }
        None => Ok(HttpResponse::BadRequest().json(&error("Group or Expense id are invalid."))),
    }
}

impl ResponseError for DeleteExpenseError {
    fn error_response(&self) -> HttpResponse<BoxBody> {
        let (code, msg) = match self {
            DeleteExpenseError::NotFound(msg) => (StatusCode::NOT_FOUND, *msg),
            DeleteExpenseError::Unauthorized(_) => (
                StatusCode::FORBIDDEN,
                "You are not authorized to delete this expense.",
            ),
            DeleteExpenseError::Unexpected(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Unexpected server error.",
            ),
            DeleteExpenseError::Unauthenticated() => {
                (StatusCode::UNAUTHORIZED, "You are not logged in.")
            }
        };
        HttpResponse::build(code).json(&error(msg))
    }
}
