use crate::test_app::TestApp;
use claim::assert_some;
use reqwest::header;
use serde_json::json;
use test_context::test_context;

#[test_context(TestApp)]
#[tokio::test]
async fn join_group_adds_the_user_to_the_group_and_returns_201(
    app: &TestApp,
) -> anyhow::Result<()> {
    // Arrange
    let group = app
        .create_user_and_group("rbiland", "r@r.com", "201", "my group")
        .await?;
    let other_user = app
        .create_user_and_login_and_device("r", "r1@r.com", "123")
        .await?;
    let group_token = app.group_token(&group).await?;
    // Act
    let response = app
        .client
        .post(&format!("{}/groups/{}/members", &app.address, &group.id))
        .header(header::COOKIE, other_user.cookie)
        .json(&json!({ "token": &group_token, "color":{"red":0,"green":255,"blue":0} }))
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 201);
    let saved = assert_some!(app.get_member_by_id(other_user.id).await);
    assert_eq!(saved.group_id, group.id);
    assert_eq!(saved.user_id, other_user.id);
    assert_eq!(saved.color, "0,255,0");
    assert_eq!(app.get_event_type().await, Some("MemberJoined".to_string()));
    Ok(())
}

#[test_context(TestApp)]
#[tokio::test]
async fn join_group_returns_409_if_user_is_group_admin(app: &TestApp) -> anyhow::Result<()> {
    // Arrange
    let group = app
        .create_user_and_group("rbiland", "r@r.com", "201", "my group")
        .await?;
    let group_token = app.group_token(&group).await?;
    // Act
    let response = app
        .client
        .post(&format!("{}/groups/{}/members", &app.address, &group.id))
        .json(&json!({ "token": &group_token, "color":{"red":0,"green":255,"blue":0} }))
        .header(header::COOKIE, group.admin.cookie)
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 409);
    match app.get_event_type().await {
        None => {}
        Some(event_type) => assert_ne!(event_type, "MemberJoined".to_string()),
    }
    Ok(())
}

#[test_context(TestApp)]
#[tokio::test]
async fn join_group_returns_409_if_user_is_already_member(app: &TestApp) -> anyhow::Result<()> {
    // Arrange
    let group = app
        .create_user_and_group("rbiland", "r@r.com", "201", "my group")
        .await?;
    let group_token = app.group_token(&group).await?;
    let other_user = app
        .create_user_and_login_and_device("r", "r1@r.com", "123")
        .await?;
    let _ = app
        .client
        .post(&format!("{}/groups/{}/members", &app.address, &group.id))
        .header(header::COOKIE, other_user.cookie.clone())
        .json(&json!({ "token": &group_token, "color":{"red":0,"green":255,"blue":0} }))
        .send()
        .await
        .expect("Failed to execute request.");
    // Act
    let response = app
        .client
        .post(&format!("{}/groups/{}/members", &app.address, &group.id))
        .header(header::COOKIE, other_user.cookie)
        .json(&json!({ "token": &group_token, "color":{"red":0,"green":255,"blue":0} }))
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 409);
    Ok(())
}

#[test_context(TestApp)]
#[tokio::test]
async fn join_group_returns_500_on_db_error(app: &TestApp) -> anyhow::Result<()> {
    // Arrange
    let group = app
        .create_user_and_group("rbiland", "r@r.com", "201", "my group")
        .await?;
    let group_token = app.group_token(&group).await?;
    let other_user = app
        .create_user_and_login_and_device("r", "r1@r.com", "123")
        .await?;
    app.break_group_db().await;
    // Act
    let response = app
        .client
        .post(&format!("{}/groups/{}/members", &app.address, &group.id))
        .json(&json!({ "token": &group_token, "color":{"red":0,"green":255,"blue":0} }))
        .header(header::COOKIE, other_user.cookie)
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 500);
    match app.get_event_type().await {
        None => {}
        Some(event_type) => assert_ne!(event_type, "MemberJoined".to_string()),
    }
    Ok(())
}

#[test_context(TestApp)]
#[tokio::test]
async fn join_group_returns_404_if_group_does_not_exist(app: &TestApp) -> anyhow::Result<()> {
    // Arrange
    let group = app
        .create_user_and_group("rbiland", "r@r.com", "201", "my group")
        .await?;
    let group_token = app.group_token(&group).await?;
    let other_user = app
        .create_user_and_login_and_device("r", "r1@r.com", "123")
        .await?;
    app.delete_group(group.id).await;
    // Act
    let response = app
        .client
        .post(&format!("{}/groups/{}/members", &app.address, &group.id))
        .header(header::COOKIE, other_user.cookie)
        .json(&json!({ "token": &group_token, "color":{"red":0,"green":255,"blue":0} }))
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 404);
    match app.get_event_type().await {
        None => {}
        Some(event_type) => assert_ne!(event_type, "MemberJoined".to_string()),
    }
    Ok(())
}

#[test_context(TestApp)]
#[tokio::test]
async fn join_group_returns_403_if_token_does_not_match_group(app: &TestApp) -> anyhow::Result<()> {
    // Arrange
    let group = app
        .create_user_and_group("rbiland", "r@r.com", "201", "my group")
        .await?;
    let group_2 = app
        .create_group("my group", group.admin.cookie.as_str())
        .await?;
    let group_token = app.group_token(&group).await?;
    let other_user = app
        .create_user_and_login_and_device("r", "r1@r.com", "123")
        .await?;
    // Act
    let response = app
        .client
        .post(&format!("{}/groups/{}/members", &app.address, &group_2))
        .header(header::COOKIE, other_user.cookie)
        .json(&json!({ "token": &group_token, "color":{"red":0,"green":255,"blue":0} }))
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 403);
    match app.get_event_type().await {
        None => {}
        Some(event_type) => assert_ne!(event_type, "MemberJoined".to_string()),
    }
    Ok(())
}

#[test_context(TestApp)]
#[tokio::test]
async fn join_group_returns_400_if_group_id_is_invalid(app: &TestApp) -> anyhow::Result<()> {
    // Arrange
    let group = app
        .create_user_and_group("rbiland", "r@r.com", "201", "my group")
        .await?;
    let group_token = app.group_token(&group).await?;
    let other_user = app
        .create_user_and_login_and_device("r", "r1@r.com", "123")
        .await?;
    // Act
    let response = app
        .client
        .post(&format!("{}/groups/{}/members", &app.address, "bob"))
        .header(header::COOKIE, other_user.cookie)
        .json(&json!({ "token": &group_token }))
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 400);
    match app.get_event_type().await {
        None => {}
        Some(event_type) => assert_ne!(event_type, "MemberJoined".to_string()),
    }
    Ok(())
}

#[test_context(TestApp)]
#[tokio::test]
async fn join_group_returns_400_if_data_is_invalid(app: &TestApp) -> anyhow::Result<()> {
    // Arrange
    let group = app
        .create_user_and_group("rbiland", "r@r.com", "201", "my group")
        .await?;
    let group_token = app.group_token(&group).await?;
    let other_user = app
        .create_user_and_login_and_device("r", "r1@r.com", "123")
        .await?;
    let cases = vec![
        (json!({ "token": &group_token }), "no color"),
        (
            json!({"token": &group_token,"color":{"red":0}}),
            "missing color fields",
        ),
        (
            json!({"token": &group_token,"color":{"red":300,"green":255,"blue":0}}),
            "invalid color fields",
        ),
    ];
    // Act
    for (body, description) in cases {
        let response = app
            .client
            .post(&format!("{}/groups/{}/members", &app.address, &group.id))
            .header(header::COOKIE, &other_user.cookie)
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
            Some(event_type) => assert_ne!(event_type, "MemberJoined".to_string()),
        }
    }
    Ok(())
}

#[test_context(TestApp)]
#[tokio::test]
async fn join_group_returns_403_if_token_is_invalid(app: &TestApp) -> anyhow::Result<()> {
    // Arrange
    let group = app
        .create_user_and_group("rbiland", "r@r.com", "201", "my group")
        .await?;
    let other_user = app
        .create_user_and_login_and_device("r", "r1@r.com", "123")
        .await?;
    // Act
    let response = app
        .client
        .post(&format!("{}/groups/{}/members", &app.address, &group.id))
        .header(header::COOKIE, other_user.cookie)
        .json(&json!({ "token": "abc", "color":{"red":0,"green":255,"blue":0} }))
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 403);
    match app.get_event_type().await {
        None => {}
        Some(event_type) => assert_ne!(event_type, "MemberJoined".to_string()),
    }
    Ok(())
}

#[test_context(TestApp)]
#[tokio::test]
async fn join_group_returns_401_if_auth_token_is_missing(app: &TestApp) -> anyhow::Result<()> {
    // Arrange
    let group = app
        .create_user_and_group("rbiland", "r@r.com", "201", "my group")
        .await?;
    let group_token = app.group_token(&group).await?;
    // Act
    let response = app
        .client
        .post(&format!("{}/groups/{}/members", &app.address, &group.id))
        .json(&json!({ "token": &group_token, "color":{"red":0,"green":255,"blue":0} }))
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 401);
    match app.get_event_type().await {
        None => {}
        Some(event_type) => assert_ne!(event_type, "MemberJoined".to_string()),
    }
    Ok(())
}
