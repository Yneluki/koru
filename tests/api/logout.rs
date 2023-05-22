use crate::test_app::TestApp;
use reqwest::header;
use test_context::test_context;

#[test_context(TestApp)]
#[tokio::test]
async fn logout_should_return_204_on_success(app: &TestApp) -> anyhow::Result<()> {
    // Arrange
    let user = app
        .create_user_and_login("rbiland", "r@r.com", "201")
        .await?;
    // Act
    let response = app
        .client
        .post(&format!("{}/logout", &app.address))
        .header(header::COOKIE, &user.cookie)
        .send()
        .await
        .expect("Failed to execute request.");
    let response_2 = app
        .client
        .get(&format!("{}/groups", &app.address))
        .header(header::COOKIE, &user.cookie)
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 204);
    assert_eq!(response_2.status().as_u16(), 401);
    Ok(())
}

#[test_context(TestApp)]
#[tokio::test]
async fn login_should_returns_401_when_not_logged_in(app: &TestApp) {
    // Arrange
    // Act
    let response = app
        .client
        .post(&format!("{}/logout", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 401);
}
