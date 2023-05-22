use crate::test_app::EventKindDto;
use chrono::{DateTime, Utc};
use koru::configuration::event_bus::{EventBusSettings, RedisSettings};
use koru::configuration::store::{DatabaseSettings, PostgresSettings};
use koru::configuration::Settings;
use koru::infrastructure::event_bus::EventBusImpl;
use koru::infrastructure::store::StoreImpl;
use koru::worker::Worker;
use log::warn;
use redis::{AsyncCommands, Client};
use secrecy::ExposeSecret;
use sqlx::types::Json;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;
use wiremock::MockServer;

pub struct ExtTestApp {
    pub notification_server: MockServer,
    redis_client: Client,
    redis_conf: RedisSettings,
    pg_pool: PgPool,
    pg_conf: PostgresSettings,
}
impl ExtTestApp {
    pub async fn build(notification_server: MockServer, configuration: &Settings) -> Self {
        let (_, event_listener) = EventBusImpl::build(&configuration.event_bus)
            .await
            .expect("Failed to setup event bus.");
        let pg_conf = if let DatabaseSettings::Postgres(pg_conf) = &configuration.database {
            pg_conf.clone()
        } else {
            panic!("PG conf required")
        };
        let redis_conf = if let EventBusSettings::Redis(redis_conf) = &configuration.event_bus {
            redis_conf.clone()
        } else {
            panic!("Redis conf required")
        };
        let redis_client = Client::open(redis_conf.connection_string().expose_secret().clone())
            .expect("Failed to init redis client.");
        let pg_pool = configure_database(&pg_conf).await;
        let store = Arc::new(
            StoreImpl::build(&configuration.database)
                .await
                .expect("Failed to setup store."),
        );
        let worker = Worker::build(&configuration.application, event_listener, store)
            .await
            .expect("Failed to setup worker.");
        let _ = tokio::spawn(worker.run());
        ExtTestApp {
            notification_server,
            redis_client,
            redis_conf,
            pg_pool,
            pg_conf,
        }
    }

    pub async fn teardown(self) {
        warn!("Closing pool");
        self.pg_pool.close().await;
        drop_database(&self.pg_conf).await;
    }

    pub async fn publish_event(&self, event_id: Uuid) {
        let mut con = self.redis_client.get_async_connection().await.unwrap();
        let mess = event_id.to_string();
        con.publish::<String, String, ()>(self.redis_conf.event_channel.clone(), mess)
            .await
            .unwrap();
        // As the event is published asynchronously, we need to wait a while for the event to be processed.
        // otherwise all tests will fail as they will directly return after having sent the event.
        sleep(Duration::from_millis(500)).await
    }

    pub async fn with_user(&self, name: String, email: String, device: String) -> Uuid {
        let id = Uuid::new_v4();
        sqlx::query!(
            r#"
        INSERT INTO koru_user (id, email, name, created_at) VALUES ($1, $2, $3, $4)
        "#,
            id,
            email,
            name,
            Utc::now()
        )
        .execute(&self.pg_pool)
        .await
        .unwrap();
        sqlx::query!(
            r#"
        INSERT INTO koru_user_device (user_id, device) VALUES ($1, $2)
        "#,
            id,
            device,
        )
        .execute(&self.pg_pool)
        .await
        .unwrap();
        id
    }

    pub async fn with_group(&self, name: String, admin: Uuid) -> Uuid {
        let id = Uuid::new_v4();
        sqlx::query!(
            r#"
        INSERT INTO koru_group (id, name, admin_id, created_at) VALUES ($1, $2, $3, $4)
        ON CONFLICT DO NOTHING
        "#,
            id,
            name,
            admin,
            Utc::now(),
        )
        .execute(&self.pg_pool)
        .await
        .unwrap();
        sqlx::query!(
            r#"
        INSERT INTO koru_group_members (group_id, user_id, joined_at, color) VALUES ($1, $2, $3, $4)
        "#,
            id,
            admin,
            Utc::now(),
            "0,255,0",
        )
        .execute(&self.pg_pool)
        .await
        .unwrap();

        id
    }

    pub async fn with_member(&self, group: Uuid, user: Uuid) {
        sqlx::query!(
            r#"
        INSERT INTO koru_group_members (group_id, user_id, joined_at, color) VALUES ($1, $2, $3, $4)
        "#,
            group,
            user,
            Utc::now(),
            "0,255,0",
        )
        .execute(&self.pg_pool)
        .await
        .unwrap();
    }

    pub async fn with_expense(&self, group: Uuid, user: Uuid, desc: String, amount: f32) -> Uuid {
        let id = Uuid::new_v4();
        sqlx::query!(
            r#"
        INSERT INTO koru_expense (id, group_id, member_id, description, amount, created_at, modified_at, settled)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#,
            id,
            group,
            user,
            desc,
            amount,
            Utc::now(),
            None::<DateTime<Utc>>,
            false,
        )
                    .execute(&self.pg_pool)
                    .await.unwrap();
        id
    }

    pub async fn with_event(&self, event: EventKindDto) -> Uuid {
        let id = Uuid::new_v4();
        sqlx::query(
            r#"
        INSERT INTO koru_event (id, event_date, event_data)
        VALUES ($1, $2, $3)
        "#,
        )
        .bind(id)
        .bind(Utc::now())
        .bind(Json(event))
        .execute(&self.pg_pool)
        .await
        .unwrap();
        id
    }

    pub async fn get_event_process_date(&self, event: Uuid) -> Option<DateTime<Utc>> {
        sqlx::query!(
            r#"
        SELECT process_date FROM koru_event WHERE id = $1
        "#,
            event,
        )
        .fetch_one(&self.pg_pool)
        .await
        .map(|r| r.process_date)
        .unwrap()
    }
}

pub async fn drop_database(config: &PostgresSettings) {
    let mut connection =
        PgConnection::connect(&config.connection_string_without_db().expose_secret())
            .await
            .expect("Failed to connect to Postgres");
    connection
        .execute(format!(r#"DROP DATABASE "{}" WITH (FORCE);"#, config.name).as_str())
        .await
        .expect("Failed to drop database.");
}

pub async fn configure_database(config: &PostgresSettings) -> PgPool {
    // Create database
    let mut connection =
        PgConnection::connect(&config.connection_string_without_db().expose_secret())
            .await
            .expect("Failed to connect to Postgres");
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.name).as_str())
        .await
        .expect("Failed to create database.");
    // Migrate database
    let connection_pool = PgPool::connect(&config.connection_string().expose_secret())
        .await
        .expect("Failed to connect to Postgres.");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");
    connection_pool
}
