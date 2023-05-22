use crate::test_app::{
    ExpenseDto, GroupDto, MemberDto, SettlementDto, TransactionDto, UserDevice, UserDto,
};
use itertools::Itertools;
use koru::configuration::store::{DatabaseSettings, PostgresSettings};
use koru::configuration::Settings;
use secrecy::ExposeSecret;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;

pub struct PgTestDb {
    pg_conf: PostgresSettings,
    pg_pool: PgPool,
}

impl PgTestDb {
    pub async fn build(configuration: &Settings) -> Self {
        let pg_conf = if let DatabaseSettings::Postgres(pg_conf) = &configuration.database {
            pg_conf.clone()
        } else {
            panic!("PG conf required")
        };
        let pg_pool = configure_database(&pg_conf).await;
        Self { pg_conf, pg_pool }
    }

    pub async fn teardown(self) {
        self.pg_pool.close().await;
        drop_database(&self.pg_conf).await;
    }

    pub async fn get_member_by_id(&self, id: Uuid) -> Option<MemberDto> {
        let row = sqlx::query!(
            r#"
        SELECT group_id, user_id, color FROM koru_group_members WHERE user_id = $1
        "#,
            id,
        )
        .fetch_optional(&self.pg_pool)
        .await
        .expect("Member not found in DB");
        row.map(|row| MemberDto {
            group_id: row.group_id,
            user_id: row.user_id,
            color: row.color,
        })
    }
    pub async fn get_expense(&self) -> Option<ExpenseDto> {
        let row = sqlx::query!(
            r#"
        SELECT id, group_id, member_id, description, amount FROM koru_expense
        "#
        )
        .fetch_optional(&self.pg_pool)
        .await
        .expect("Expense not found in DB");
        row.map(|row| ExpenseDto {
            id: row.id,
            group_id: row.group_id,
            member_id: row.member_id,
            description: row.description,
            amount: row.amount,
        })
    }
    pub async fn get_expense_by_id(&self, id: Uuid) -> Option<ExpenseDto> {
        let row = sqlx::query!(
            r#"
        SELECT id, group_id, member_id, description, amount FROM koru_expense where id = $1
        "#,
            id
        )
        .fetch_optional(&self.pg_pool)
        .await
        .expect("Expense not found in DB");
        row.map(|row| ExpenseDto {
            id: row.id,
            group_id: row.group_id,
            member_id: row.member_id,
            description: row.description,
            amount: row.amount,
        })
    }
    pub async fn get_event_type(&self) -> Option<String> {
        let event = sqlx::query!(
            r#"
        SELECT event_data FROM koru_event ORDER BY event_date DESC
        "#,
        )
        .fetch_optional(&self.pg_pool)
        .await
        .expect("Event not found in DB");
        event.map(|r| {
            r.event_data
                .as_object()
                .unwrap()
                .keys()
                .next()
                .unwrap()
                .clone()
        })
    }
    pub async fn get_group(&self) -> Option<GroupDto> {
        let row = sqlx::query!(
            r#"
        SELECT id, name, admin_id FROM koru_group
        "#
        )
        .fetch_optional(&self.pg_pool)
        .await
        .expect("Group not found in DB");
        row.map(|row| GroupDto {
            id: row.id,
            name: row.name,
            admin_id: row.admin_id,
        })
    }
    pub async fn get_group_by_id(&self, id: Uuid) -> Option<GroupDto> {
        let row = sqlx::query!(
            r#"
        SELECT id, name, admin_id FROM koru_group WHERE id = $1
        "#,
            id
        )
        .fetch_optional(&self.pg_pool)
        .await
        .expect("Group not found in DB");
        row.map(|row| GroupDto {
            id: row.id,
            name: row.name,
            admin_id: row.admin_id,
        })
    }
    pub async fn get_user_id_by_email(&self, email: String) -> Uuid {
        let saved = sqlx::query!(
            r#"
        SELECT id FROM koru_user WHERE email = $1
        "#,
            email
        )
        .fetch_one(&self.pg_pool)
        .await
        .expect("User not found in DB");
        saved.id
    }
    pub async fn get_user(&self) -> UserDto {
        let saved = sqlx::query!(
            r#"
        SELECT name, koru_user.email, koru_user_credentials.password FROM koru_user JOIN koru_user_credentials ON koru_user.email = koru_user_credentials.email
        "#
        )
        .fetch_one(&self.pg_pool)
        .await
        .expect("User not found in DB");
        UserDto {
            name: saved.name,
            email: saved.email,
            password: saved.password,
        }
    }
    pub async fn get_device(&self) -> Option<UserDevice> {
        let row = sqlx::query!(
            r#"
        SELECT user_id, device FROM koru_user_device
        "#
        )
        .fetch_optional(&self.pg_pool)
        .await
        .expect("Device not found in DB");
        row.map(|r| UserDevice {
            user_id: r.user_id,
            device: r.device,
        })
    }
    pub async fn get_settlement(&self) -> Option<SettlementDto> {
        let row = sqlx::query!(
            r#"
        SELECT id, group_id, end_date FROM koru_settlement
        "#
        )
        .fetch_optional(&self.pg_pool)
        .await
        .expect("Settlement not found in DB");
        row.map(|r| SettlementDto {
            id: r.id,
            group_id: r.group_id,
        })
    }
    pub async fn get_expenses_status(&self, ids: &[Uuid]) -> Vec<(Uuid, bool)> {
        sqlx::query!(
            r#"
        SELECT id, settled
        FROM koru_expense WHERE id = ANY($1)
        "#,
            &ids[..]
        )
        .fetch_all(&self.pg_pool)
        .await
        .expect("Transaction not found in DB")
        .iter()
        .map(|r| (r.id, r.settled))
        .collect_vec()
    }
    pub async fn get_transactions(&self) -> Vec<TransactionDto> {
        sqlx::query!(
            r#"
        SELECT settlement_id, from_user_id, to_user_id, amount
        FROM koru_transaction
        ORDER BY amount ASC
        "#
        )
        .fetch_all(&self.pg_pool)
        .await
        .expect("Transaction not found in DB")
        .iter()
        .map(|r| TransactionDto {
            settlement_id: r.settlement_id,
            from_user_id: r.from_user_id,
            to_user_id: r.to_user_id,
            amount: r.amount,
        })
        .collect_vec()
    }
    pub async fn settled_expenses(&self) -> Vec<(Uuid, Uuid)> {
        sqlx::query!(
            r#"
        SELECT settlement_id, expense_id
        FROM koru_settlement_expenses
        "#,
        )
        .fetch_all(&self.pg_pool)
        .await
        .expect("Transaction not found in DB")
        .iter()
        .map(|r| (r.settlement_id, r.expense_id))
        .collect_vec()
    }

    pub async fn delete_group(&self, id: Uuid) {
        sqlx::query!("DELETE FROM koru_group WHERE id= $1", id)
            .execute(&self.pg_pool)
            .await
            .unwrap();
    }

    pub async fn delete_user(&self, id: Uuid) {
        sqlx::query!("DELETE FROM koru_user WHERE id = $1;", id)
            .execute(&self.pg_pool)
            .await
            .unwrap();
    }

    pub async fn break_group_db(&self) {
        sqlx::query!("ALTER TABLE koru_group DROP COLUMN name;",)
            .execute(&self.pg_pool)
            .await
            .unwrap();
    }

    pub async fn break_user_db(&self) {
        sqlx::query!("ALTER TABLE koru_user DROP COLUMN email;",)
            .execute(&self.pg_pool)
            .await
            .unwrap();
    }

    pub async fn break_credentials_db(&self) {
        sqlx::query!("ALTER TABLE koru_user_credentials DROP COLUMN email;",)
            .execute(&self.pg_pool)
            .await
            .unwrap();
    }

    pub async fn make_admin(&self, id: &Uuid) {
        sqlx::query!(
            r#"
        UPDATE koru_user_roles SET role = 'admin' WHERE user_id = $1
        "#,
            id,
        )
        .execute(&self.pg_pool)
        .await
        .unwrap();
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
