use crate::api::response::ok_message;
use actix_web::HttpResponse;

/// Health check
///
/// Examples:
/// ```
/// curl -i "http://localhost:8000/health_check"
/// ```
///
#[cfg_attr(feature = "openapi", utoipa::path(
    get,
    path = "/health_check",
    responses(
        (status = 200, description = "Service health status", body = MessageResponse)
    ),
    tag = "Health",
))]
pub async fn health_check() -> HttpResponse {
    HttpResponse::Ok().json(&ok_message("Koru is healthy."))
}
