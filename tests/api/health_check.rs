use crate::test_app::TestApp;
use test_context::test_context;

#[test_context(TestApp)]
#[tokio::test]
async fn health_check_returns_200(app: &TestApp) {
    // Arrange
    // Act
    let response = app
        .client
        .get(&format!("{}/health_check", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert!(response.status().is_success());
}
