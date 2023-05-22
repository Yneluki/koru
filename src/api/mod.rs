mod response;
mod routes;

use crate::api::routes::{
    change_color, create_expense, create_group, delete_expense, delete_group, generate_group_token,
    get_all_groups, get_all_users, get_expenses, get_group, get_groups, get_settlements,
    health_check, join_group, login, logout, middleware, register, register_device, remove_device,
    settle, update_expense,
};
use crate::application::app::Application;
use crate::application::store::MultiRepository;
use crate::configuration::application::{ApiSettings, SessionSettings};
use crate::infrastructure::session_store::SessionStoreImpl;
use actix_session::config::PersistentSession;
use actix_session::SessionMiddleware;
use actix_web::cookie::time::Duration;
use actix_web::cookie::Key;
use actix_web::dev::Server;
use actix_web::{web, App, HttpServer};
use actix_web_lab::middleware::from_fn;
use secrecy::ExposeSecret;
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;
#[cfg(feature = "openapi")]
use utoipa::openapi::security::{ApiKey, ApiKeyValue, SecurityScheme};
#[cfg(feature = "openapi")]
use utoipa::{Modify, OpenApi};
#[cfg(feature = "openapi")]
use utoipa_swagger_ui::SwaggerUi;

pub struct RestApi {
    port: u16,
    server: Server,
}

impl RestApi {
    pub async fn build(
        configuration: &ApiSettings,
        application: Application<impl MultiRepository + 'static>,
    ) -> Result<Self, anyhow::Error> {
        let listener = TcpListener::bind(configuration.address())?;
        let port = listener.local_addr().unwrap().port();
        let server = run(listener, application, &configuration.session).await?;

        Ok(Self { port, server })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

async fn run<Store: MultiRepository + 'static>(
    listener: TcpListener,
    application: Application<Store>,
    session_config: &SessionSettings,
) -> Result<Server, anyhow::Error> {
    let app = web::Data::new(application);
    let session_store = SessionStoreImpl::build(&session_config.store).await?;
    let session_key = Key::from(session_config.hmac.expose_secret().as_bytes());
    let session_duration = Duration::days(session_config.duration as i64);
    #[cfg(feature = "openapi")]
    let openapi = ApiDoc::openapi();
    let server = HttpServer::new(move || {
        let api = App::new()
            .wrap(TracingLogger::default())
            .wrap(
                SessionMiddleware::builder(session_store.clone(), session_key.clone())
                    .session_lifecycle(PersistentSession::default().session_ttl(session_duration))
                    .build(),
            )
            .route("/health_check", web::get().to(health_check))
            .route("/register", web::post().to(register::<Store>))
            .route("/login", web::post().to(login::<Store>))
            .service(
                web::scope("/logout")
                    .wrap(from_fn(middleware::auth::reject_anonymous_users))
                    .route("", web::post().to(logout::<Store>)),
            )
            .service(
                web::scope("/groups")
                    .wrap(from_fn(middleware::auth::reject_anonymous_users))
                    .route("", web::post().to(create_group::<Store>))
                    .route("", web::get().to(get_groups::<Store>))
                    .route("/{group_id}", web::get().to(get_group::<Store>))
                    .route("/{group_id}", web::delete().to(delete_group::<Store>))
                    .route(
                        "/{group_id}/token",
                        web::get().to(generate_group_token::<Store>),
                    )
                    .route("/{group_id}/members", web::post().to(join_group::<Store>))
                    .route(
                        "/{group_id}/members",
                        web::patch().to(change_color::<Store>),
                    )
                    .route("/{group_id}/expenses", web::get().to(get_expenses::<Store>))
                    .route(
                        "/{group_id}/expenses",
                        web::post().to(create_expense::<Store>),
                    )
                    .route("/{group_id}/settlements", web::post().to(settle::<Store>))
                    .route(
                        "/{group_id}/settlements",
                        web::get().to(get_settlements::<Store>),
                    )
                    .route(
                        "/{group_id}/expenses/{expense_id}",
                        web::put().to(update_expense::<Store>),
                    )
                    .route(
                        "/{group_id}/expenses/{expense_id}",
                        web::delete().to(delete_expense::<Store>),
                    ),
            )
            .service(
                web::scope("/admin")
                    .wrap(from_fn(middleware::auth::reject_anonymous_users))
                    .route("/groups", web::get().to(get_all_groups::<Store>))
                    .route("/users", web::get().to(get_all_users::<Store>)),
            )
            .app_data(app.clone());

        #[cfg(feature = "pushy")]
        let api = api.service(
            web::scope("/devices")
                .wrap(from_fn(middleware::auth::reject_anonymous_users))
                .route("", web::post().to(register_device::<Store>))
                .route("", web::delete().to(remove_device::<Store>)),
        );

        #[cfg(feature = "openapi")]
        let api = api.service(
            SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-doc/openapi.json", openapi.clone()),
        );

        api
    })
    .listen(listener)?
    .run();
    Ok(server)
}

#[cfg(feature = "openapi")]
#[derive(OpenApi)]
#[openapi(
    paths(
        crate::api::routes::login,
        crate::api::routes::logout,
        crate::api::routes::register,
        crate::api::routes::change_color,
        crate::api::routes::create_expense,
        crate::api::routes::create_group,
        crate::api::routes::delete_expense,
        crate::api::routes::delete_group,
        crate::api::routes::get_expenses,
        crate::api::routes::get_group,
        crate::api::routes::get_groups,
        crate::api::routes::get_settlements,
        crate::api::routes::join_group,
        crate::api::routes::register_device,
        crate::api::routes::remove_device,
        crate::api::routes::settle,
        crate::api::routes::update_expense,
        crate::api::routes::get_all_groups,
        crate::api::routes::get_all_users,
        crate::api::routes::health_check
    ),
    components(
        schemas(
            crate::api::response::MessageResponse,
            crate::api::response::ErrorResponse,
            crate::api::response::IdResponse,
            crate::api::response::GenerateTokenResponse,
            crate::api::response::GetExpensesResponse,
            crate::api::response::GetGroupResponse,
            crate::api::response::GetGroupsResponse,
            crate::api::response::GetAllGroupsResponse,
            crate::api::response::GetAllUsersResponse,
            crate::api::response::GetSettlementsResponse,
            crate::api::response::SettlementResponse,
            crate::api::response::MessageData,
            crate::api::response::ErrorData,
            crate::api::response::IdData,
            crate::api::routes::LoginPayload,
            crate::api::routes::RegisterPayload,
            crate::api::routes::ChangeColorPayload,
            crate::api::routes::CreateExpensePayload,
            crate::api::routes::CreateGroupPayload,
            crate::api::routes::JoinGroupPayload,
            crate::api::routes::DeviceData,
            crate::api::routes::UpdateExpensePayload,
            crate::api::routes::GroupTokenResponse,
            crate::api::routes::ExpensesResponse,
            crate::api::routes::GroupResponse,
            crate::api::routes::GroupsResponse,
            crate::api::routes::AllGroupsResponse,
            crate::api::routes::AllUsersResponse,
            crate::api::routes::SettlementsResponse,
            crate::domain::usecases::dto::dtos::ColorDto,
            crate::domain::usecases::dto::dtos::GroupDto,
            crate::domain::usecases::dto::dtos::DetailedGroupDto,
            crate::domain::usecases::dto::dtos::ExpenseDto,
            crate::domain::usecases::dto::dtos::MemberDto,
            crate::domain::usecases::dto::dtos::UserDto,
            crate::domain::usecases::dto::dtos::DetailedUserDto,
            crate::domain::usecases::dto::dtos::SettlementDto,
            crate::domain::usecases::dto::dtos::TransactionDto,
        ),
    ),
    tags(
        (name = "Authentication", description = "User & auth endpoints."),
        (name = "Groups", description = "Group management."),
        (name = "Expenses", description = "Expense management."),
        (name = "Settlements", description = "Settlement management."),
        (name = "Admin", description = "Admin endpoints."),
        (name = "Devices", description = "Devices (Pushy) management."),
        (name = "Health", description = "Health check endpoint."),
    ),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

#[cfg(feature = "openapi")]
struct SecurityAddon;

#[cfg(feature = "openapi")]
impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.as_mut().unwrap(); // we can unwrap safely since there already is components registered.
        components.add_security_scheme(
            "cookieAuth",
            SecurityScheme::ApiKey(ApiKey::Cookie(ApiKeyValue::new("id"))),
        )
    }
}
