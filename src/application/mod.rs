pub mod admin;
pub mod app;
pub mod auth;
pub mod event_bus;
pub mod group;
#[cfg(feature = "notification")]
pub mod notification;
pub mod store;
pub mod user;

#[cfg(test)]
mod tests {
    use crate::application::admin::AdminUsecase;
    use crate::application::auth::{AuthService, CredentialsHasher, Password, UserCredentials};
    use crate::application::event_bus::EventBus;
    use crate::application::group::GroupUsecase;
    use crate::application::notification::DeviceService;
    use crate::application::store::MultiRepository;
    use crate::application::user::UserUsecase;
    #[cfg(feature = "notification")]
    use crate::domain::notification::NotificationService;
    use crate::domain::usecases::admin::AdminUseCase;
    use crate::domain::usecases::group::GroupUseCase;
    use crate::domain::usecases::user::UserUseCase;
    use crate::domain::UserRole::Administrator;
    use crate::domain::{Event, Expense, Group, GroupMember, MemberColor, Settlement, User};
    use crate::infrastructure::event_bus::direct_event_bus::DirectEventBus;
    #[cfg(feature = "notification")]
    use crate::infrastructure::notification_service::{FakeNotificationService, InnerNotification};
    use crate::infrastructure::services::credentials_hasher::FakeCredentialsHasher;
    use crate::infrastructure::store::mem::mem_store::InnerEvent;
    use crate::infrastructure::store::InMemoryStore;
    use crate::infrastructure::token_generator::FakeTokenGenerator;
    use chrono::{DateTime, Utc};
    use itertools::Itertools;
    use rand::random;
    use secrecy::Secret;
    use std::sync::Arc;
    use uuid::Uuid;

    pub struct TestContext {
        store: Arc<InMemoryStore>,
        event_bus: Arc<DirectEventBus>,
        #[cfg(feature = "notification")]
        notification_svc: Arc<FakeNotificationService>,
        group_uc: Arc<GroupUsecase<InMemoryStore>>,
        admin_uc: Arc<AdminUsecase<InMemoryStore>>,
        user_uc: Arc<UserUsecase<InMemoryStore>>,
        user_uc_no_auth: Arc<UserUsecase<InMemoryStore>>,
        token_svc: Arc<FakeTokenGenerator>,
    }

    impl TestContext {
        pub fn new() -> Self {
            let store = Arc::new(InMemoryStore::new());
            let event_bus = Arc::new(DirectEventBus::new());
            let auth_service = AuthService::new(store.clone(), FakeCredentialsHasher::new());
            let token_svc = Arc::new(FakeTokenGenerator::new());
            #[cfg(feature = "pushy")]
            let device_service = Arc::new(DeviceService::new(store.clone()));
            let user_uc = Arc::new(UserUsecase::new(
                store.clone(),
                event_bus.clone(),
                Some(auth_service),
                #[cfg(feature = "pushy")]
                device_service.clone(),
            ));
            let user_uc_no_auth = Arc::new(UserUsecase::new(
                store.clone(),
                event_bus.clone(),
                None,
                #[cfg(feature = "pushy")]
                device_service,
            ));
            let group_uc = Arc::new(GroupUsecase::new(
                store.clone(),
                event_bus.clone(),
                token_svc.clone(),
                user_uc.clone(),
            ));
            let admin_uc = Arc::new(AdminUsecase::new(store.clone(), event_bus.clone()));
            Self {
                store,
                event_bus,
                #[cfg(feature = "notification")]
                notification_svc: Arc::new(FakeNotificationService::new()),
                group_uc,
                admin_uc,
                user_uc,
                user_uc_no_auth,
                token_svc,
            }
        }

        pub fn store(&self) -> Arc<impl MultiRepository> {
            self.store.clone()
        }

        pub fn group(&self) -> Arc<impl GroupUseCase> {
            self.group_uc.clone()
        }

        pub fn admin(&self) -> Arc<impl AdminUseCase> {
            self.admin_uc.clone()
        }

        pub fn user(&self) -> Arc<impl UserUseCase> {
            self.user_uc.clone()
        }

        pub fn user_no_auth(&self) -> Arc<impl UserUseCase> {
            self.user_uc_no_auth.clone()
        }

        #[cfg(feature = "notification")]
        pub fn notification_svc(&self) -> Arc<dyn NotificationService> {
            self.notification_svc.clone()
        }

        pub async fn with_group(&self) -> Group {
            let mut tx = self.store.tx().await.unwrap();
            let user = self.with_user().await;
            let mut group = Group::create(
                "My group".to_string(),
                user.id,
                user.name.clone(),
                user.email.clone(),
                MemberColor::default(),
            )
            .unwrap();
            self.store.groups().save(&mut tx, &group).await.unwrap();
            self.store
                .members()
                .save(
                    &mut tx,
                    &group.members.iter().find(|m| m.id == user.id).unwrap(),
                )
                .await
                .unwrap();
            self.store
                .events()
                .save(
                    &mut tx,
                    &group.events.iter().cloned().map(Event::Group).collect_vec(),
                )
                .await
                .unwrap();
            self.store.commit(tx.into_inner()).await.unwrap();
            self.event_bus
                .publish(&group.events.iter().map(|e| e.id).collect_vec())
                .await
                .unwrap();
            group.events.clear();
            group
        }

        pub async fn with_user(&self) -> User {
            let mut tx = self.store.tx().await.unwrap();
            let nb: u32 = random();
            let email = format!("@{}", nb);
            let name = format!("u_{}", nb);
            println!("Created user {}", name);
            let user = User::create(name, email).unwrap();
            let password = FakeCredentialsHasher::new()
                .hash_password(Secret::new(format!("p_{}", nb)))
                .unwrap();
            let user_credentials = UserCredentials {
                email: user.email.clone(),
                password: Password::build_from_hash(password),
            };
            self.store.users().save(&mut tx, &user).await.unwrap();
            self.store
                .credentials()
                .save(&mut tx, &user_credentials)
                .await
                .unwrap();
            self.store.commit(tx.into_inner()).await.unwrap();
            user
        }

        pub async fn with_admin_user(&self) -> User {
            let mut tx = self.store.tx().await.unwrap();
            let nb: u32 = random();
            let email = format!("@{}", nb);
            let name = format!("u_{}", nb);
            println!("Created user {}", name);
            let mut user = User::create(name, email).unwrap();
            user.role = Administrator;
            let password = FakeCredentialsHasher::new()
                .hash_password(Secret::new(format!("p_{}", nb)))
                .unwrap();
            let user_credentials = UserCredentials {
                email: user.email.clone(),
                password: Password::build_from_hash(password),
            };
            self.store.users().save(&mut tx, &user).await.unwrap();
            self.store
                .credentials()
                .save(&mut tx, &user_credentials)
                .await
                .unwrap();
            self.store.commit(tx.into_inner()).await.unwrap();
            user
        }

        pub async fn with_member(&self, group: &mut Group) -> GroupMember {
            let mut tx = self.store.tx().await.unwrap();
            let user = self.with_user().await;
            let member = group
                .add_member(
                    user.id,
                    user.name.clone(),
                    user.email.clone(),
                    MemberColor {
                        red: 0,
                        green: 255,
                        blue: 0,
                    },
                )
                .unwrap();
            self.store.groups().save(&mut tx, &group).await.unwrap();
            self.store.members().save(&mut tx, &member).await.unwrap();
            self.store
                .events()
                .save(
                    &mut tx,
                    &group.events.iter().cloned().map(Event::Group).collect_vec(),
                )
                .await
                .unwrap();
            self.store.commit(tx.into_inner()).await.unwrap();
            self.event_bus
                .publish(&group.events.iter().map(|e| e.id).collect_vec())
                .await
                .unwrap();
            group.events.clear();
            member
        }

        pub async fn with_expense(&self, group: &mut Group, user: Uuid) -> Expense {
            let mut tx = self.store.tx().await.unwrap();
            let expense = group
                .add_expense("my expense".to_string(), 12.0, user)
                .unwrap();
            self.store.groups().save(&mut tx, &group).await.unwrap();
            self.store.expenses().save(&mut tx, &expense).await.unwrap();
            self.store
                .events()
                .save(
                    &mut tx,
                    &group.events.iter().cloned().map(Event::Group).collect_vec(),
                )
                .await
                .unwrap();
            self.store.commit(tx.into_inner()).await.unwrap();
            self.event_bus
                .publish(&group.events.iter().map(|e| e.id).collect_vec())
                .await
                .unwrap();
            group.events.clear();
            expense
        }

        pub async fn with_expense_of(&self, group: &mut Group, amount: f32, user: Uuid) -> Expense {
            let mut tx = self.store.tx().await.unwrap();
            let expense = group
                .add_expense("my expense".to_string(), amount, user)
                .unwrap();
            self.store.groups().save(&mut tx, &group).await.unwrap();
            self.store.expenses().save(&mut tx, &expense).await.unwrap();
            self.store
                .events()
                .save(
                    &mut tx,
                    &group.events.iter().cloned().map(Event::Group).collect_vec(),
                )
                .await
                .unwrap();
            self.store.commit(tx.into_inner()).await.unwrap();
            self.event_bus
                .publish(&group.events.iter().map(|e| e.id).collect_vec())
                .await
                .unwrap();
            group.events.clear();
            expense
        }

        pub async fn settle(&self, group: &mut Group, expenses: &mut [Expense]) -> Settlement {
            let mut tx = self.store.tx().await.unwrap();
            let settlement = group.settle(expenses, None, group.admin_id).unwrap();
            self.store
                .settlements()
                .save(&mut tx, &settlement)
                .await
                .unwrap();
            self.store.groups().save(&mut tx, &group).await.unwrap();
            for expense in expenses {
                self.store.expenses().save(&mut tx, expense).await.unwrap();
            }
            self.store
                .events()
                .save(
                    &mut tx,
                    &group.events.iter().cloned().map(Event::Group).collect_vec(),
                )
                .await
                .unwrap();
            self.store.commit(tx.into_inner()).await.unwrap();
            self.event_bus
                .publish(&group.events.iter().map(|e| e.id).collect_vec())
                .await
                .unwrap();
            group.events.clear();
            settlement
        }

        pub async fn get_user(&self, user_id: &Uuid) -> User {
            self.find_user(user_id).await.unwrap()
        }

        pub async fn find_user(&self, user_id: &Uuid) -> Option<User> {
            self.store.users().find(user_id).await.unwrap()
        }

        pub async fn get_credentials(&self, user_id: &Uuid) -> UserCredentials {
            self.find_credentials(user_id).await.unwrap()
        }

        pub async fn find_credentials(&self, user_id: &Uuid) -> Option<UserCredentials> {
            let user = self.get_user(user_id).await;
            self.store
                .credentials()
                .fetch_by_email(&user.email)
                .await
                .unwrap()
        }

        pub async fn get_group(&self, group_id: &Uuid) -> Group {
            self.find_group(group_id).await.unwrap()
        }

        pub async fn find_group(&self, group_id: &Uuid) -> Option<Group> {
            self.store.groups().find(group_id).await.unwrap()
        }

        pub async fn find_settlement(&self, settlement_id: &Uuid) -> Option<Settlement> {
            self.store.settlements().find(settlement_id).await.unwrap()
        }

        pub async fn find_expense(&self, expense_id: &Uuid) -> Option<Expense> {
            self.store.expenses().find(expense_id).await.unwrap()
        }

        pub async fn get_expense(&self, expense_id: &Uuid) -> Expense {
            self.find_expense(expense_id).await.unwrap()
        }

        pub async fn remove_group(&self, group_id: &Uuid) {
            let mut tx = self.store.tx().await.unwrap();
            self.store.groups().delete(&mut tx, group_id).await.unwrap();
            self.store.commit(tx.into_inner()).await.unwrap();
        }

        pub async fn group_token(&self, group: &Group) -> String {
            self.token(&group.admin_id, group).await
        }

        pub async fn token(&self, user_id: &Uuid, group: &Group) -> String {
            group
                .generate_join_token(user_id, self.token_svc.clone())
                .await
                .unwrap()
        }

        pub fn last_stored_event(&self) -> Option<InnerEvent> {
            let evts = self.store.events.lock().unwrap();
            let length = evts.len();
            if length > 0 {
                Some(evts[length - 1].clone())
            } else {
                None
            }
        }

        pub async fn get_event_process_date(&self, event_id: &Uuid) -> Option<DateTime<Utc>> {
            self.store.get_event(event_id).await.processed_date
        }

        pub fn last_published_event(&self) -> Option<Uuid> {
            let evts = self.event_bus.events.lock().unwrap();
            let length = evts.len();
            if length > 0 {
                Some(evts[length - 1])
            } else {
                None
            }
        }

        #[cfg(feature = "notification")]
        pub fn notifications(&self) -> Vec<InnerNotification> {
            let evts = self.notification_svc.notifications.lock().unwrap();
            evts.to_vec()
        }
    }
}
