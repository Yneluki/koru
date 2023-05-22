use crate::api::response::ok_message;
use crate::api::routes::middleware::user_session::UserId;
use crate::application::app::Application;
use crate::application::store::MultiRepository;
use actix_web::{web, HttpResponse};

/// Un-links the device (Pushy) from the user making the request.
///
/// Requires the auth cookie from `/login` to be attached to the request.
///
/// Example:
/// ```
/// curl -i -H 'Content-Type: application/json' -c cookie -X DELETE "http://localhost:8000/devices"
///```
#[cfg_attr(feature = "openapi", utoipa::path(
    delete,
    path = "/devices",
    responses(
        (status = 204, description = "Device un-linked from the user.", body = MessageResponse),
    ),
    security(
        ("cookieAuth" = [])
    ),
    tag = "Devices",
))]
#[tracing::instrument(
    name = "Removing user device",
    skip(app, user_id),
    fields(
        user_id = %user_id.0
    )
)]
pub async fn remove_device<Store: MultiRepository>(
    app: web::Data<Application<Store>>,
    user_id: web::ReqData<UserId>,
) -> HttpResponse {
    app.devices().delete(&user_id.0).await;
    HttpResponse::NoContent().json(&ok_message("Device removed."))
}
