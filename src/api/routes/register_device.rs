use crate::api::response::ok_message;
use crate::api::routes::middleware::user_session::UserId;
use crate::application::app::Application;
use crate::application::store::MultiRepository;
use actix_web::{web, HttpResponse};
#[cfg(feature = "openapi")]
use utoipa::ToSchema;

/// Links a device id (Pushy) to the user making the request.
///
/// Requires the auth cookie from `/login` to be attached to the request.
///
/// Example:
/// ```
/// curl -i -H 'Content-Type: application/json' -d '{"device":"123"}' -c cookie "http://localhost:8000/devices"
///```
#[cfg_attr(feature = "openapi", utoipa::path(
    post,
    path = "/devices",
    request_body = DeviceData,
    responses(
        (status = 200, description = "Device linked to the user.", body = MessageResponse),
    ),
    security(
        ("cookieAuth" = [])
    ),
    tag = "Devices",
))]
#[tracing::instrument(
    name = "Registering user device",
    skip(payload, app, user_id),
    fields(
        user_id = %user_id.0
    )
)]
pub async fn register_device<Store: MultiRepository>(
    payload: web::Json<DeviceData>,
    app: web::Data<Application<Store>>,
    user_id: web::ReqData<UserId>,
) -> HttpResponse {
    app.devices().save(&user_id.0, payload.device.clone()).await;
    HttpResponse::Ok().json(&ok_message("Device registered."))
}

#[derive(serde::Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct DeviceData {
    pub device: String,
}
