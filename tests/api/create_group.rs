use crate::test_app::{CreateGroupResponse, TestApp};
use claim::assert_some;
use reqwest::header;
use serde_json::json;
use test_context::test_context;

#[test_context(TestApp)]
#[tokio::test]
async fn group_creation_success_saves_the_group_and_returns_201(
    app: &TestApp,
) -> anyhow::Result<()> {
    // Arrange
    let login_data = app
        .create_user_and_login_and_device("rbiland", "r@r.com", "201")
        .await?;
    // Act
    let response = app
        .client
        .post(&format!("{}/groups", &app.address))
        .header(header::COOKIE, login_data.cookie)
        .json(&json!({"name":"my group", "color":{"red":0,"green":255,"blue":0}}))
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 201);
    let saved = assert_some!(app.get_group().await);
    assert_eq!(saved.name, "my group");
    assert_eq!(saved.admin_id, login_data.id);
    let body = response.json::<CreateGroupResponse>().await?;
    assert_eq!(body.success, true);
    assert_eq!(body.data.id, saved.id);
    assert_eq!(app.get_event_type().await, Some("GroupCreated".to_string()));
    Ok(())
}

#[test_context(TestApp)]
#[tokio::test]
async fn group_creation_without_logging_in_returns_401(app: &TestApp) {
    // Arrange
    // Act
    let response = app
        .client
        .post(&format!("{}/groups", &app.address))
        .json(&json!({"name":"my group", "color":{"red":0,"green":255,"blue":0}}))
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 401);
    match app.get_event_type().await {
        None => {}
        Some(event_type) => assert_ne!(event_type, "GroupCreated".to_string()),
    }
}

#[test_context(TestApp)]
#[tokio::test]
async fn group_creation_with_invalid_user_id_returns_401(app: &TestApp) -> anyhow::Result<()> {
    // Arrange
    let login_data = app
        .create_user_and_login_and_device("rbiland", "r@r.com", "201")
        .await?;
    app.delete_user(login_data.id).await;
    // Act
    let response = app
        .client
        .post(&format!("{}/groups", &app.address))
        .header(header::COOKIE, login_data.cookie)
        .json(&json!({"name":"my group", "color":{"red":0,"green":255,"blue":0}}))
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 401);
    match app.get_event_type().await {
        None => {}
        Some(event_type) => assert_ne!(event_type, "GroupCreated".to_string()),
    }
    Ok(())
}

#[test_context(TestApp)]
#[tokio::test]
async fn group_creation_with_invalid_data_returns_400(app: &TestApp) -> anyhow::Result<()> {
    // Arrange
    let login_data = app
        .create_user_and_login_and_device("rbiland", "r@r.com", "201")
        .await?;
    let cases = vec![
        (
            json!({"name":"","color":{"red":0,"green":255,"blue":0}}),
            "empty name",
        ),
        (json!({"color":{"red":0,"green":255,"blue":0}}), "no name"),
        (json!({"name":"my group"}), "no color"),
        (
            json!({"name":"my group","color":{"red":0}}),
            "missing color fields",
        ),
        (
            json!({"name":"my group","color":{"red":300,"green":255,"blue":0}}),
            "invalid color fields",
        ),
    ];
    // Act
    for (body, description) in cases {
        let response = app
            .client
            .post(&format!("{}/groups", &app.address))
            .header(header::COOKIE, &login_data.cookie)
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
        match app.get_event_type().await {
            None => {}
            Some(event_type) => assert_ne!(event_type, "GroupCreated".to_string()),
        }
    }
    Ok(())
}

#[test_context(TestApp)]
#[tokio::test]
async fn group_creation_on_db_error_returns_500(app: &TestApp) -> anyhow::Result<()> {
    // Arrange
    let login_data = app
        .create_user_and_login_and_device("rbiland", "r@r.com", "201")
        .await?;
    app.break_group_db().await;
    // Act
    let response = app
        .client
        .post(&format!("{}/groups", &app.address))
        .header(header::COOKIE, login_data.cookie)
        .json(&json!({"name":"my group", "color":{"red":0,"green":255,"blue":0}}))
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 500);
    match app.get_event_type().await {
        None => {}
        Some(event_type) => assert_ne!(event_type, "GroupCreated".to_string()),
    }
    Ok(())
}
