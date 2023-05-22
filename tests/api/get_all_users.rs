use crate::test_app::TestApp;
use chrono::{DateTime, Utc};
use claim::assert_some;
use reqwest::header;
use test_context::test_context;
use uuid::Uuid;

#[test_context(TestApp)]
#[tokio::test]
async fn get_all_users_returns_200_and_all_groups_when_user_is_admin(
    app: &TestApp,
) -> anyhow::Result<()> {
    // Arrange
    let admin = app
        .create_admin_and_login_and_device("rbiland", "r@r.com", "201")
        .await?;
    app.create_user("rbiland1", "r1@r.com", "201").await;
    app.create_user("rbiland2", "r2@r.com", "201").await;
    app.create_user("rbiland3", "r3@r.com", "201").await;
    // Act
    let response = app
        .client
        .get(&format!("{}/admin/users", &app.address))
        .header(header::COOKIE, &admin.cookie)
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 200);
    let body = response.json::<GetUsersResponse>().await?;
    assert_eq!(body.success, true);
    assert_eq!(body.data.users.len(), 4);
    let admin = assert_some!(body.data.users.iter().find(|u| u.name == "rbiland"));
    assert_eq!(admin.email, "r@r.com".to_string());
    assert_eq!(admin.role, "Administrator".to_string());
    let u1 = assert_some!(body.data.users.iter().find(|u| u.name == "rbiland1"));
    assert_eq!(u1.email, "r1@r.com".to_string());
    assert_eq!(u1.role, "User".to_string());
    let u2 = assert_some!(body.data.users.iter().find(|u| u.name == "rbiland2"));
    assert_eq!(u2.email, "r2@r.com".to_string());
    assert_eq!(u2.role, "User".to_string());
    let u3 = assert_some!(body.data.users.iter().find(|u| u.name == "rbiland3"));
    assert_eq!(u3.email, "r3@r.com".to_string());
    assert_eq!(u3.role, "User".to_string());
    Ok(())
}

#[test_context(TestApp)]
#[tokio::test]
async fn get_all_users_returns_401_if_user_is_not_logged_in(app: &TestApp) -> anyhow::Result<()> {
    // Arrange
    let _ = app
        .create_admin_and_login_and_device("rbiland", "r@r.com", "201")
        .await?;
    // Act
    let response = app
        .client
        .get(&format!("{}/admin/users", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 401);
    Ok(())
}

#[test_context(TestApp)]
#[tokio::test]
async fn get_all_users_returns_403_if_user_is_not_admin(app: &TestApp) -> anyhow::Result<()> {
    // Arrange
    let user = app
        .create_user_and_login_and_device("rbiland", "r@r.com", "201")
        .await?;
    // Act
    let response = app
        .client
        .get(&format!("{}/admin/users", &app.address))
        .header(header::COOKIE, user.cookie)
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 403);
    Ok(())
}

#[derive(serde::Deserialize)]
pub struct GetUsersResponse {
    pub success: bool,
    pub data: UsersData,
}

#[derive(serde::Deserialize)]
pub struct UsersData {
    pub users: Vec<UserData>,
}

#[derive(serde::Deserialize)]
pub struct UserData {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub role: String,
    pub created_at: DateTime<Utc>,
}
