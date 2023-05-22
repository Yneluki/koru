use crate::test_app::TestApp;
use claim::assert_none;
use reqwest::header;
use test_context::test_context;

#[test_context(TestApp)]
#[tokio::test]
async fn remove_device_returns_204(app: &TestApp) -> anyhow::Result<()> {
    // Arrange
    let login_data = app
        .create_user_and_login_and_device("rbiland", "r@r.com", "201")
        .await?;

    // Act
    let response = app
        .client
        .delete(&format!("{}/devices", &app.address))
        .header(header::COOKIE, login_data.cookie)
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 204);
    assert_none!(app.get_device().await);
    Ok(())
}

#[test_context(TestApp)]
#[tokio::test]
async fn remove_device_without_logging_in_returns_401(app: &TestApp) {
    // Arrange
    // Act
    let response = app
        .client
        .delete(&format!("{}/devices", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 401);
}
