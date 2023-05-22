use crate::test_app::TestApp;
use serde_json::json;
use test_context::test_context;

#[test_context(TestApp)]
#[tokio::test]
async fn registration_success_saves_user_and_returns_201(app: &TestApp) {
    // Arrange
    // Act
    let response = app
        .client
        .post(&format!("{}/register", &app.address))
        .json(&json!({"name":"rbiland","email":"r@r.com","password":"201"}))
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 201);
    let saved = app.get_user().await;
    assert_eq!(saved.name, "rbiland");
    assert_eq!(saved.email, "r@r.com");
    assert_ne!(saved.password, "201"); // verify password is not in clear text
}

#[test_context(TestApp)]
#[tokio::test]
async fn registration_with_bad_params_returns_400(app: &TestApp) {
    // Arrange
    let cases = vec![
        (
            json!({"name":"rbiland","email":"r@r.com","password":""}),
            "empty password",
        ),
        (
            json!({"name":"","email":"r@r.com","password":"201"}),
            "empty name",
        ),
        (
            json!({"name":"rbiland","email":"","password":"201"}),
            "empty email",
        ),
        (
            json!({"name":"rbiland","email":"rrrr","password":"201"}),
            "invalid email",
        ),
    ];
    // Act
    for (body, description) in cases {
        let response = app
            .client
            .post(&format!("{}/register", &app.address))
            .json(&body)
            .send()
            .await
            .expect("Failed to execute request.");
        // Assert
        assert_eq!(
            response.status().as_u16(),
            400,
            "The API did not return 400 when the payload was {}.",
            description
        );
    }
}

#[test_context(TestApp)]
#[tokio::test]
async fn registration_on_db_error_returns_500(app: &TestApp) {
    // Arrange
    app.break_user_db().await;
    // Act
    let response = app
        .client
        .post(&format!("{}/register", &app.address))
        .json(&json!({"name":"rbiland","email":"r@r.com","password":"201"}))
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 500,);
}

#[test_context(TestApp)]
#[tokio::test]
async fn registration_duplicate_returns_500(app: &TestApp) {
    // Arrange
    app.create_user("rbiland", "r@r.com", "201").await;
    // Act
    let response = app
        .client
        .post(&format!("{}/register", &app.address))
        .json(&json!({"name":"bob","email":"r@r.com","password":"123"}))
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 500,);
}
