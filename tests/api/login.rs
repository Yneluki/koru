use crate::test_app::{LoginResponse, TestApp};
use claim::assert_gt;
use serde_json::json;
use test_context::test_context;

#[test_context(TestApp)]
#[tokio::test]
async fn login_should_return_200_and_user_id_on_success(app: &TestApp) -> anyhow::Result<()> {
    // Arrange
    // register user
    app.create_user("rbiland", "r@r.com", "201").await;
    // Act
    let response = app
        .client
        .post(&format!("{}/login", &app.address))
        .json(&json!({"email":"r@r.com","password":"201"}))
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 200);
    let body = response.json::<LoginResponse>().await?;
    assert_eq!(body.success, true);
    assert_gt!(body.data.id.to_string().len(), 0);
    let saved = app.get_user_id_by_email(String::from("r@r.com")).await;
    assert_eq!(saved, body.data.id);
    Ok(())
}

#[test_context(TestApp)]
#[tokio::test]
async fn login_should_returns_400_on_invalid_input(app: &TestApp) {
    // Arrange
    let cases = vec![
        (json!({"email":"r@r.com","password":""}), "empty password"),
        (json!({"email":"","password":"201"}), "empty email"),
    ];
    // Act
    for (body, description) in cases {
        let response = app
            .client
            .post(&format!("{}/login", &app.address))
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
async fn login_should_returns_401_on_unknown_user(app: &TestApp) {
    // Arrange
    // Act
    let response = app
        .client
        .post(&format!("{}/login", &app.address))
        .json(&json!({"email":"r@r.com","password":"201"}))
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 401);
}

#[test_context(TestApp)]
#[tokio::test]
async fn login_should_returns_401_on_invalid_credentials(app: &TestApp) {
    // Arrange
    // register user
    app.create_user("rbiland", "r@r.com", "201").await;
    // Act
    let response = app
        .client
        .post(&format!("{}/login", &app.address))
        .json(&json!({"email":"r@r.com","password":"101"}))
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 401);
}

#[test_context(TestApp)]
#[tokio::test]
async fn login_on_db_error_returns_500(app: &TestApp) {
    // Arrange
    app.create_user("rbiland", "r@r.com", "201").await;
    app.break_credentials_db().await;
    // Act
    let response = app
        .client
        .post(&format!("{}/login", &app.address))
        .json(&json!({"email":"r@r.com","password":"201"}))
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 500,);
}
