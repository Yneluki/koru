use crate::test_app::TestApp;
use claim::assert_some;
use reqwest::header;
use serde_json::json;
use test_context::test_context;

#[test_context(TestApp)]
#[tokio::test]
async fn register_device_returns_200(app: &TestApp) -> anyhow::Result<()> {
    // Arrange
    let login_data = app
        .create_user_and_login("rbiland", "r@r.com", "201")
        .await?;
    // Act
    let response = app
        .client
        .post(&format!("{}/devices", &app.address))
        .header(header::COOKIE, login_data.cookie)
        .json(&json!({"device":"my_device_id"}))
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 200);
    let saved = assert_some!(app.get_device().await);
    assert_eq!(saved.user_id, login_data.id);
    assert_eq!(saved.device, Some(String::from("my_device_id")));
    Ok(())
}

#[test_context(TestApp)]
#[tokio::test]
async fn register_new_device_returns_200(app: &TestApp) -> anyhow::Result<()> {
    // Arrange
    let login_data = app
        .create_user_and_login_and_device("rbiland", "r@r.com", "201")
        .await?;
    // Act
    let response = app
        .client
        .post(&format!("{}/devices", &app.address))
        .header(header::COOKIE, login_data.cookie)
        .json(&json!({"device":"new_device"}))
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 200);
    let saved = assert_some!(app.get_device().await);
    assert_eq!(saved.user_id, login_data.id);
    assert_eq!(saved.device, Some(String::from("new_device")));
    Ok(())
}

#[test_context(TestApp)]
#[tokio::test]
async fn register_device_without_logging_in_returns_401(app: &TestApp) {
    // Arrange
    // Act
    let response = app
        .client
        .post(&format!("{}/devices", &app.address))
        .json(&json!({"device":"my_device_id"}))
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 401);
}

#[test_context(TestApp)]
#[tokio::test]
async fn register_device_without_device_in_returns_400(app: &TestApp) -> anyhow::Result<()> {
    // Arrange
    let login_data = app
        .create_user_and_login("rbiland", "r@r.com", "201")
        .await?;
    // Act
    let response = app
        .client
        .post(&format!("{}/devices", &app.address))
        .header(header::COOKIE, login_data.cookie)
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 400);
    Ok(())
}
