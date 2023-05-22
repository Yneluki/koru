use crate::api::routes::middleware::user_session::{UserId, UserSession};
use crate::application::app::Application;
use crate::application::store::MultiRepository;
use crate::domain::usecases::user::{LogoutRequest, UserUseCase};
use actix_web::{web, HttpResponse};
use log::warn;

/// Logs out the user making the request.
///
/// Requires the auth cookie from `/login` to be attached to the request.
///
/// Example:
/// ```
/// curl -i -H 'Content-Type: application/json' -c cookie -X POST "http://localhost:8000/logout"
///```
#[cfg_attr(feature = "openapi", utoipa::path(
    post,
    path = "/logout",
    responses(
        (status = 204, description = "Successful logout"),
        (status = 401, description = "User is not logged in"),
    ),
    security(
        ("cookieAuth" = [])
    ),
    tag = "Authentication",
))]
#[tracing::instrument(
    name = "Logging out",
    skip(session, app, user_id),
    fields(
        user_id = %user_id.0,
    )
)]
pub async fn logout<Store: MultiRepository>(
    app: web::Data<Application<Store>>,
    session: UserSession,
    user_id: web::ReqData<UserId>,
) -> HttpResponse {
    app.users()
        .logout(LogoutRequest {
            user_id: *user_id.into_inner(),
        })
        .await
        .unwrap_or_else(|failure| {
            warn!("{:?}", failure);
        });
    session.purge();
    HttpResponse::NoContent().finish()
}
