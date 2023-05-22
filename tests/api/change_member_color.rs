use crate::test_app::TestApp;
use claim::assert_some;
use reqwest::header;
use serde_json::json;
use test_context::test_context;

#[test_context(TestApp)]
#[tokio::test]
async fn change_color_updates_the_user_color_and_returns_200_when_user_is_member(
    app: &TestApp,
) -> anyhow::Result<()> {
    // Arrange
    let group = app
        .create_user_and_group("rbiland", "r@r.com", "201", "my group")
        .await?;
    let other_user = app
        .create_user_and_login_and_device("r", "r1@r.com", "123")
        .await?;
    app.join_group(&group, &other_user.cookie).await?;
    // Act
    let response = app
        .client
        .patch(&format!("{}/groups/{}/members", &app.address, &group.id))
        .header(header::COOKIE, &other_user.cookie)
        .json(&json!({"color":{"red":255,"green":255,"blue":255} }))
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 200);
    let saved = assert_some!(app.get_member_by_id(other_user.id).await);
    assert_eq!(saved.group_id, group.id);
    assert_eq!(saved.user_id, other_user.id);
    assert_eq!(saved.color, "255,255,255");
    assert_eq!(
        app.get_event_type().await,
        Some("MemberColorChanged".to_string())
    );
    Ok(())
}

#[test_context(TestApp)]
#[tokio::test]
async fn change_color_updates_the_user_color_and_returns_200_when_user_is_admin(
    app: &TestApp,
) -> anyhow::Result<()> {
    // Arrange
    let group = app
        .create_user_and_group("rbiland", "r@r.com", "201", "my group")
        .await?;
    // Act
    let response = app
        .client
        .patch(&format!("{}/groups/{}/members", &app.address, &group.id))
        .header(header::COOKIE, &group.admin.cookie)
        .json(&json!({"color":{"red":255,"green":255,"blue":255} }))
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 200);
    let saved = assert_some!(app.get_member_by_id(group.admin.id).await);
    assert_eq!(saved.group_id, group.id);
    assert_eq!(saved.user_id, group.admin.id);
    assert_eq!(saved.color, "255,255,255");
    assert_eq!(
        app.get_event_type().await,
        Some("MemberColorChanged".to_string())
    );
    Ok(())
}

#[test_context(TestApp)]
#[tokio::test]
async fn change_color_returns_500_on_db_error(app: &TestApp) -> anyhow::Result<()> {
    // Arrange
    let group = app
        .create_user_and_group("rbiland", "r@r.com", "201", "my group")
        .await?;
    let other_user = app
        .create_user_and_login_and_device("r", "r1@r.com", "123")
        .await?;
    app.join_group(&group, &other_user.cookie).await?;
    app.break_group_db().await;
    // Act
    let response = app
        .client
        .patch(&format!("{}/groups/{}/members", &app.address, &group.id))
        .json(&json!({ "color":{"red":255,"green":255,"blue":255} }))
        .header(header::COOKIE, other_user.cookie)
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 500);
    match app.get_event_type().await {
        None => {}
        Some(event_type) => assert_ne!(event_type, "MemberColorChanged".to_string()),
    }
    Ok(())
}

#[test_context(TestApp)]
#[tokio::test]
async fn change_color_returns_404_if_group_does_not_exist(app: &TestApp) -> anyhow::Result<()> {
    // Arrange
    let group = app
        .create_user_and_group("rbiland", "r@r.com", "201", "my group")
        .await?;
    let other_user = app
        .create_user_and_login_and_device("r", "r1@r.com", "123")
        .await?;
    app.join_group(&group, &other_user.cookie).await?;
    app.delete_group(group.id).await;
    // Act
    let response = app
        .client
        .patch(&format!("{}/groups/{}/members", &app.address, &group.id))
        .header(header::COOKIE, other_user.cookie)
        .json(&json!({"color":{"red":255,"green":255,"blue":255} }))
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 404);
    match app.get_event_type().await {
        None => {}
        Some(event_type) => assert_ne!(event_type, "MemberColorChanged".to_string()),
    }
    Ok(())
}

#[test_context(TestApp)]
#[tokio::test]
async fn change_color_returns_400_if_group_id_is_invalid(app: &TestApp) -> anyhow::Result<()> {
    // Arrange
    let group = app
        .create_user_and_group("rbiland", "r@r.com", "201", "my group")
        .await?;
    let other_user = app
        .create_user_and_login_and_device("r", "r1@r.com", "123")
        .await?;
    app.join_group(&group, &other_user.cookie).await?;
    // Act
    let response = app
        .client
        .patch(&format!("{}/groups/{}/members", &app.address, "bob"))
        .header(header::COOKIE, &other_user.cookie)
        .json(&json!({"color":{"red":255,"green":255,"blue":255} }))
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 400);
    match app.get_event_type().await {
        None => {}
        Some(event_type) => assert_ne!(event_type, "MemberColorChanged".to_string()),
    }
    Ok(())
}

#[test_context(TestApp)]
#[tokio::test]
async fn change_color_returns_400_if_data_is_invalid(app: &TestApp) -> anyhow::Result<()> {
    // Arrange
    let group = app
        .create_user_and_group("rbiland", "r@r.com", "201", "my group")
        .await?;
    let other_user = app
        .create_user_and_login_and_device("r", "r1@r.com", "123")
        .await?;
    app.join_group(&group, &other_user.cookie).await?;
    let cases = vec![
        (json!({}), "no color"),
        (json!({"color":{"red":0}}), "missing color fields"),
        (
            json!({"color":{"red":300,"green":255,"blue":0}}),
            "invalid color fields",
        ),
    ];
    // Act
    for (body, description) in cases {
        let response = app
            .client
            .patch(&format!("{}/groups/{}/members", &app.address, &group.id))
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
            Some(event_type) => assert_ne!(event_type, "MemberColorChanged".to_string()),
        }
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
    // Act
    let response = app
        .client
        .post(&format!("{}/groups/{}/members", &app.address, &group.id))
        .json(&json!({ "color":{"red":255,"green":255,"blue":0} }))
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 401);
    match app.get_event_type().await {
        None => {}
        Some(event_type) => assert_ne!(event_type, "MemberColorChanged".to_string()),
    }
    Ok(())
}
