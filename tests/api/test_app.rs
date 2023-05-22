use crate::mem_test_db::MemTestDb;
#[cfg(feature = "postgres")]
use crate::pg_test_db::PgTestDb;
use anyhow::Result;
use chrono::{DateTime, Utc};
use koru::api::RestApi;
use koru::application::app::Application;
use koru::configuration::application::SessionStoreSettings;
use koru::configuration::event_bus::EventBusSettings;
use koru::configuration::store::DatabaseSettings;
use koru::configuration::{get_configuration, Settings};
use koru::infrastructure::event_bus::EventBusImpl;
use koru::infrastructure::store::StoreImpl;
use koru::utils::telemetry::{get_subscriber, init_subscriber};
use once_cell::sync::Lazy;
use reqwest::header;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use test_context::AsyncTestContext;
use uuid::Uuid;

pub enum Infra {
    Memory,
    #[cfg(any(feature = "production", feature = "development"))]
    External,
}

impl TryFrom<String> for Infra {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "local" => Ok(Self::Memory),
            #[cfg(any(feature = "production", feature = "development"))]
            "integration" => Ok(Self::External),
            other => Err(format!(
                "`{}` is not a supported infra. Use either `local` or `integration` (requires `--features full`).",
                other
            )),
        }
    }
}

pub struct TestApp {
    pub address: String,
    pub db: TestDb,
    pub client: reqwest::Client,
}

pub enum TestDb {
    #[cfg(feature = "postgres")]
    Postgres(PgTestDb),
    Memory(MemTestDb),
}

// Ensure that the `tracing` stack is only initialised once
static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "sqlx=error,info".to_string();
    let subscriber_name = "test".to_string();
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    };
});

fn in_memory_config() -> Settings {
    let mut c = get_configuration().expect("Failed to read configuration.");
    c.database = DatabaseSettings::Memory;
    c.event_bus = EventBusSettings::Memory;
    c.api.session.store = SessionStoreSettings::Memory;
    // Use a random OS port
    c.api.host = "127.0.0.1".to_string();
    c.api.port = 0;
    c
}

#[cfg(any(feature = "production", feature = "development"))]
fn ext_config() -> Settings {
    let mut c = get_configuration().expect("Failed to read configuration.");
    // Use a different database for each test case
    c.database = match c.database {
        DatabaseSettings::Postgres(mut postgres) => {
            postgres.name = Uuid::new_v4().to_string();
            DatabaseSettings::Postgres(postgres)
        }
        DatabaseSettings::Memory => DatabaseSettings::Memory,
    };
    // Use a different channel for each test case
    c.event_bus = match c.event_bus {
        EventBusSettings::Redis(mut redis) => {
            redis.event_channel = Uuid::new_v4().to_string();
            EventBusSettings::Redis(redis)
        }
        EventBusSettings::Memory => EventBusSettings::Memory,
    };
    // Use a random OS port
    c.api.host = "127.0.0.1".to_string();
    c.api.port = 0;
    c
}

#[async_trait::async_trait]
impl AsyncTestContext for TestApp {
    async fn setup() -> TestApp {
        Lazy::force(&TRACING);

        let infra: Infra = std::env::var("KORU_ENV")
            .unwrap_or_else(|_| "local".into())
            .try_into()
            .expect("Failed to parse KORU_ENV.");

        let client = reqwest::Client::builder().build().unwrap();

        match infra {
            Infra::Memory => {
                let configuration = in_memory_config();
                let db = MemTestDb::build().await;
                let (event_bus, _) = EventBusImpl::build(&configuration.event_bus)
                    .await
                    .expect("Failed to setup event bus.");

                let app = Application::build(
                    &configuration.application,
                    Arc::new(StoreImpl::Memory(db.store.clone())),
                    event_bus,
                    // use a small one to reduce test speed
                    Some(128),
                )
                .expect("Failed to setup application.");

                // Initialize API
                let api = RestApi::build(&configuration.api, app)
                    .await
                    .expect("Failed to initialize API");
                let port = api.port();

                let _ = tokio::spawn(api.run());
                TestApp {
                    address: format!("http://127.0.0.1:{}", port),
                    db: TestDb::Memory(db),
                    client,
                }
            }
            #[cfg(any(feature = "production", feature = "development"))]
            Infra::External => {
                let configuration = ext_config();
                let db = PgTestDb::build(&configuration).await;
                let (event_bus, _) = EventBusImpl::build(&configuration.event_bus)
                    .await
                    .expect("Failed to setup event bus.");

                let app = Application::build(
                    &configuration.application,
                    Arc::new(
                        StoreImpl::build(&configuration.database)
                            .await
                            .expect("Failed to start store."),
                    ),
                    event_bus,
                    // use a small one to reduce test speed
                    Some(128),
                )
                .expect("Failed to setup application.");

                // Initialize API
                let api = RestApi::build(&configuration.api, app)
                    .await
                    .expect("Failed to initialize API");
                let port = api.port();

                let _ = tokio::spawn(api.run());
                TestApp {
                    address: format!("http://127.0.0.1:{}", port),
                    db: TestDb::Postgres(db),
                    client,
                }
            }
        }
    }

    async fn teardown(self) {
        match self.db {
            #[cfg(feature = "postgres")]
            TestDb::Postgres(db) => db.teardown().await,
            TestDb::Memory(_) => {}
        }
    }
}

impl TestApp {
    pub async fn create_user(&self, name: &str, email: &str, password: &str) {
        let _ = self
            .client
            .post(&format!("{}/register", &self.address))
            .json(&json!({"name":name,"email":email,"password":password}))
            .send()
            .await
            .expect("Failed to execute request.");
    }

    pub async fn login(&self, email: &str, password: &str) -> Result<LoginData> {
        let response = self
            .client
            .post(&format!("{}/login", &self.address))
            .json(&json!({"email":email,"password":password}))
            .send()
            .await
            .expect("Failed to execute request.");
        let cookie_value = String::from(
            response
                .cookies()
                .next()
                .expect("Cookie should exist")
                .value(),
        );
        let body = response.json::<LoginResponse>().await?;
        let user_data = LoginData {
            id: body.data.id,
            cookie: format!("id={}", cookie_value),
        };
        println!("logged in user: {:?}", user_data);
        Ok(user_data)
    }

    pub async fn create_user_and_login_and_device(
        &self,
        name: &str,
        email: &str,
        password: &str,
    ) -> Result<LoginData> {
        self.create_user(name, email, password).await;
        let login_data = self.login(email, password).await?;
        self.register_device("my_device_id", &login_data.cookie)
            .await;
        Ok(login_data)
    }

    pub async fn create_admin_and_login_and_device(
        &self,
        name: &str,
        email: &str,
        password: &str,
    ) -> Result<LoginData> {
        let login_data = self
            .create_user_and_login_and_device(name, email, password)
            .await?;
        match &self.db {
            #[cfg(feature = "postgres")]
            TestDb::Postgres(db) => db.make_admin(&login_data.id).await,
            TestDb::Memory(db) => db.make_admin(&login_data.id).await,
        }
        Ok(login_data)
    }

    pub async fn create_user_and_login(
        &self,
        name: &str,
        email: &str,
        password: &str,
    ) -> Result<LoginData> {
        self.create_user(name, email, password).await;
        self.login(email, password).await
    }

    pub async fn register_device(&self, device: &str, cookie: &str) {
        let _ = self
            .client
            .post(&format!("{}/devices", &self.address))
            .header(header::COOKIE, cookie)
            .json(&json!({ "device": device }))
            .send()
            .await
            .expect("Failed to execute request.");
    }

    pub async fn create_user_and_group(
        &self,
        name: &str,
        email: &str,
        password: &str,
        group: &str,
    ) -> Result<Group> {
        let login_data = self
            .create_user_and_login_and_device(name, email, password)
            .await?;
        let group_id = self.create_group(group, login_data.cookie.as_str()).await?;

        Ok(Group {
            admin: login_data,
            id: group_id,
        })
    }

    pub async fn create_group(&self, group: &str, cookie: &str) -> Result<Uuid> {
        let response = self
            .client
            .post(&format!("{}/groups", &self.address))
            .header(header::COOKIE, cookie)
            .json(&json!({ "name": group, "color":{"red":0,"green":255,"blue":0} }))
            .send()
            .await
            .expect("Failed to execute request.");
        let body = response.json::<CreateGroupResponse>().await?;
        Ok(body.data.id)
    }

    pub async fn group_token(&self, group: &Group) -> Result<String> {
        let response = self
            .client
            .get(&format!("{}/groups/{}/token", &self.address, group.id))
            .header(header::COOKIE, group.admin.cookie.clone())
            .send()
            .await
            .expect("Failed to execute request.");
        let body = response.json::<GenerateTokenResponse>().await?;
        Ok(body.data.token)
    }

    pub async fn add_users_to_group(&self, group: &Group, count: u32) -> Result<()> {
        let group_token = self.group_token(group).await?;
        for i in 0..count {
            let user = self
                .create_user_and_login_and_device(
                    i.to_string().as_str(),
                    (i.to_string() + "@r.com").as_str(),
                    "123",
                )
                .await?;
            let _ = self
                .client
                .post(&format!("{}/groups/{}/members", &self.address, group.id))
                .header(header::COOKIE, user.cookie)
                .json(&json!({ "token": &group_token, "color":{"red":0,"green":255,"blue":0} }))
                .send()
                .await
                .expect("Failed to execute request.");
        }
        Ok(())
    }

    pub async fn join_group(&self, group: &Group, user_cookie: &str) -> Result<()> {
        let group_token = self.group_token(group).await?;
        self.client
            .post(&format!("{}/groups/{}/members", &self.address, group.id))
            .header(header::COOKIE, user_cookie)
            .json(&json!({ "token": &group_token, "color":{"red":0,"green":255,"blue":0} }))
            .send()
            .await
            .expect("Failed to execute request.");
        Ok(())
    }

    pub async fn create_expense(
        &self,
        group_id: &Uuid,
        user_cookie: &str,
        description: &str,
        amount: f32,
    ) -> Result<Uuid> {
        let response = self
            .client
            .post(&format!("{}/groups/{}/expenses", &self.address, group_id))
            .header(header::COOKIE, user_cookie)
            .json(&json!({"description": description, "amount": amount}))
            .send()
            .await
            .expect("Failed to execute request.");
        let body = response.json::<CreateExpenseResponse>().await?;
        Ok(body.data.id)
    }

    pub async fn settle(&self, group: &Group) -> Result<SettlementData> {
        let response = self
            .client
            .post(&format!(
                "{}/groups/{}/settlements",
                &self.address, group.id
            ))
            .header(header::COOKIE, group.admin.cookie.clone())
            .send()
            .await
            .expect("Failed to execute request.");
        let body = response.json::<SettlementResponse>().await?;
        Ok(body.data)
    }

    pub async fn get_member_by_id(&self, id: Uuid) -> Option<MemberDto> {
        match &self.db {
            #[cfg(feature = "postgres")]
            TestDb::Postgres(db) => db.get_member_by_id(id).await,
            TestDb::Memory(db) => db.get_member_by_id(id).await,
        }
    }
    pub async fn get_expense(&self) -> Option<ExpenseDto> {
        match &self.db {
            #[cfg(feature = "postgres")]
            TestDb::Postgres(db) => db.get_expense().await,
            TestDb::Memory(db) => db.get_expense().await,
        }
    }
    pub async fn get_expense_by_id(&self, id: Uuid) -> Option<ExpenseDto> {
        match &self.db {
            #[cfg(feature = "postgres")]
            TestDb::Postgres(db) => db.get_expense_by_id(id).await,
            TestDb::Memory(db) => db.get_expense_by_id(id).await,
        }
    }
    pub async fn get_event_type(&self) -> Option<String> {
        match &self.db {
            #[cfg(feature = "postgres")]
            TestDb::Postgres(db) => db.get_event_type().await,
            TestDb::Memory(db) => db.get_event_type().await,
        }
    }
    pub async fn get_group(&self) -> Option<GroupDto> {
        match &self.db {
            #[cfg(feature = "postgres")]
            TestDb::Postgres(db) => db.get_group().await,
            TestDb::Memory(db) => db.get_group().await,
        }
    }
    pub async fn get_group_by_id(&self, id: Uuid) -> Option<GroupDto> {
        match &self.db {
            #[cfg(feature = "postgres")]
            TestDb::Postgres(db) => db.get_group_by_id(id).await,
            TestDb::Memory(db) => db.get_group_by_id(id).await,
        }
    }
    pub async fn get_user_id_by_email(&self, email: String) -> Uuid {
        match &self.db {
            #[cfg(feature = "postgres")]
            TestDb::Postgres(db) => db.get_user_id_by_email(email).await,
            TestDb::Memory(db) => db.get_user_id_by_email(email).await,
        }
    }
    pub async fn get_user(&self) -> UserDto {
        match &self.db {
            #[cfg(feature = "postgres")]
            TestDb::Postgres(db) => db.get_user().await,
            TestDb::Memory(db) => db.get_user().await,
        }
    }
    pub async fn get_device(&self) -> Option<UserDevice> {
        match &self.db {
            #[cfg(feature = "postgres")]
            TestDb::Postgres(db) => db.get_device().await,
            TestDb::Memory(db) => db.get_device().await,
        }
    }
    pub async fn get_settlement(&self) -> Option<SettlementDto> {
        match &self.db {
            #[cfg(feature = "postgres")]
            TestDb::Postgres(db) => db.get_settlement().await,
            TestDb::Memory(db) => db.get_settlement().await,
        }
    }
    pub async fn get_expenses_status(&self, ids: &[Uuid]) -> Vec<(Uuid, bool)> {
        match &self.db {
            #[cfg(feature = "postgres")]
            TestDb::Postgres(db) => db.get_expenses_status(ids).await,
            TestDb::Memory(db) => db.get_expenses_status(ids).await,
        }
    }
    pub async fn get_transactions(&self) -> Vec<TransactionDto> {
        match &self.db {
            #[cfg(feature = "postgres")]
            TestDb::Postgres(db) => db.get_transactions().await,
            TestDb::Memory(db) => db.get_transactions().await,
        }
    }
    pub async fn settled_expenses(&self) -> Vec<(Uuid, Uuid)> {
        match &self.db {
            #[cfg(feature = "postgres")]
            TestDb::Postgres(db) => db.settled_expenses().await,
            TestDb::Memory(db) => db.settled_expenses().await,
        }
    }

    pub async fn delete_group(&self, id: Uuid) {
        match &self.db {
            #[cfg(feature = "postgres")]
            TestDb::Postgres(db) => db.delete_group(id).await,
            TestDb::Memory(db) => db.delete_group(id).await,
        }
    }

    pub async fn delete_user(&self, id: Uuid) {
        match &self.db {
            #[cfg(feature = "postgres")]
            TestDb::Postgres(db) => db.delete_user(id).await,
            TestDb::Memory(db) => db.delete_user(id).await,
        }
    }

    pub async fn break_group_db(&self) {
        match &self.db {
            #[cfg(feature = "postgres")]
            TestDb::Postgres(db) => db.break_group_db().await,
            TestDb::Memory(db) => db.break_group_db().await,
        }
    }

    pub async fn break_user_db(&self) {
        match &self.db {
            #[cfg(feature = "postgres")]
            TestDb::Postgres(db) => db.break_user_db().await,
            TestDb::Memory(db) => db.break_user_db().await,
        }
    }

    pub async fn break_credentials_db(&self) {
        match &self.db {
            #[cfg(feature = "postgres")]
            TestDb::Postgres(db) => db.break_credentials_db().await,
            TestDb::Memory(db) => db.break_credentials_db().await,
        }
    }
}

#[derive(Debug)]
pub struct TransactionDto {
    pub settlement_id: Uuid,
    pub from_user_id: Uuid,
    pub to_user_id: Uuid,
    pub amount: f32,
}

#[derive(Debug)]
pub struct SettlementDto {
    pub id: Uuid,
    pub group_id: Uuid,
}

#[derive(Debug)]
pub struct UserDevice {
    pub user_id: Uuid,
    pub device: Option<String>,
}

#[derive(Debug)]
pub struct UserDto {
    pub name: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug)]
pub struct GroupDto {
    pub id: Uuid,
    pub name: String,
    pub admin_id: Uuid,
}

#[derive(Debug)]
pub struct ExpenseDto {
    pub id: Uuid,
    pub group_id: Uuid,
    pub member_id: Uuid,
    pub description: String,
    pub amount: f32,
}

#[derive(Debug)]
pub struct MemberDto {
    pub group_id: Uuid,
    pub user_id: Uuid,
    pub color: String,
}

#[derive(Debug)]
pub struct LoginData {
    pub id: Uuid,
    pub cookie: String,
}

pub struct Group {
    pub admin: LoginData,
    pub id: Uuid,
}

#[derive(Deserialize)]
pub struct LoginResponse {
    pub success: bool,
    pub data: LoginResponseData,
}

#[derive(Deserialize)]
pub struct LoginResponseData {
    pub id: Uuid,
}

#[derive(serde::Deserialize)]
pub struct CreateGroupResponse {
    pub success: bool,
    pub data: CreateGroupData,
}

#[derive(serde::Deserialize)]
pub struct CreateGroupData {
    pub id: Uuid,
}

#[derive(serde::Deserialize)]
pub struct GenerateTokenResponse {
    pub success: bool,
    pub data: GenerateTokenData,
}

#[derive(serde::Deserialize)]
pub struct GenerateTokenData {
    pub token: String,
}

#[derive(serde::Deserialize)]
pub struct CreateExpenseResponse {
    pub success: bool,
    pub data: CreateExpenseData,
}

#[derive(serde::Deserialize)]
pub struct CreateExpenseData {
    pub id: Uuid,
}

#[derive(serde::Deserialize)]
pub struct SettlementsResponse {
    pub success: bool,
    pub data: SettlementsData,
}

#[derive(serde::Deserialize, Debug, PartialEq)]
pub struct SettlementsData {
    pub settlements: Vec<SettlementData>,
}

#[derive(serde::Deserialize)]
pub struct SettlementResponse {
    pub success: bool,
    pub data: SettlementData,
}

#[derive(serde::Deserialize, Debug, PartialEq)]
pub struct SettlementData {
    pub id: Uuid,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: DateTime<Utc>,
    pub transactions: Vec<TransactionData>,
}

#[derive(serde::Deserialize, Debug, PartialEq)]
pub struct TransactionData {
    pub from: UserData,
    pub to: UserData,
    pub amount: f32,
}

#[derive(serde::Deserialize, Debug, PartialEq)]
pub struct UserData {
    pub id: Uuid,
    pub name: String,
}

#[derive(serde::Deserialize, Debug, PartialEq)]
pub struct MemberData {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub is_admin: bool,
    pub color: ColorDto,
    pub joined_at: DateTime<Utc>,
}

#[derive(serde::Deserialize, Debug, PartialEq)]
pub struct ColorDto {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}
