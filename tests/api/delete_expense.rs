use crate::test_app::TestApp;
use claim::{assert_none, assert_some};
use reqwest::header;
use test_context::test_context;

#[test_context(TestApp)]
#[tokio::test]
async fn delete_expense_returns_204_and_deletes_the_expense_when_user_is_admin(
    app: &TestApp,
) -> anyhow::Result<()> {
    // Arrange
    let group = app
        .create_user_and_group("rbiland", "r@r.com", "201", "my group")
        .await?;
    let other_user = app
        .create_user_and_login_and_device("r", "r1@r.com", "123")
        .await?;
    app.join_group(&group, other_user.cookie.as_str()).await?;
    let expense_id = app
        .create_expense(&group.id, other_user.cookie.as_str(), "expense", 12.0)
        .await?;

    // Act
    let response = app
        .client
        .delete(&format!(
            "{}/groups/{}/expenses/{}",
            &app.address, &group.id, &expense_id
        ))
        .header(header::COOKIE, group.admin.cookie)
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 204);
    assert_none!(app.get_expense_by_id(expense_id).await);
    assert_eq!(
        app.get_event_type().await,
        Some("ExpenseDeleted".to_string())
    );
    Ok(())
}

#[test_context(TestApp)]
#[tokio::test]
async fn delete_expense_returns_204_and_deletes_the_expense_when_user_is_author(
    app: &TestApp,
) -> anyhow::Result<()> {
    // Arrange
    let group = app
        .create_user_and_group("rbiland", "r@r.com", "201", "my group")
        .await?;
    let other_user = app
        .create_user_and_login_and_device("r", "r1@r.com", "123")
        .await?;
    app.join_group(&group, other_user.cookie.as_str()).await?;
    let expense_id = app
        .create_expense(&group.id, other_user.cookie.as_str(), "expense", 12.0)
        .await?;
    // Act
    let response = app
        .client
        .delete(&format!(
            "{}/groups/{}/expenses/{}",
            &app.address, &group.id, &expense_id
        ))
        .header(header::COOKIE, other_user.cookie)
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 204);
    assert_none!(app.get_expense_by_id(expense_id).await);
    assert_eq!(
        app.get_event_type().await,
        Some("ExpenseDeleted".to_string())
    );
    Ok(())
}

#[test_context(TestApp)]
#[tokio::test]
async fn delete_expense_returns_403_when_user_is_not_author(app: &TestApp) -> anyhow::Result<()> {
    // Arrange
    let group = app
        .create_user_and_group("rbiland", "r@r.com", "201", "my group")
        .await?;
    let other_user = app
        .create_user_and_login_and_device("r", "r1@r.com", "123")
        .await?;
    app.join_group(&group, other_user.cookie.as_str()).await?;
    let expense_id = app
        .create_expense(&group.id, other_user.cookie.as_str(), "expense", 12.0)
        .await?;
    let other_user_2 = app
        .create_user_and_login_and_device("r", "r2@r.com", "123")
        .await?;
    app.join_group(&group, other_user_2.cookie.as_str()).await?;
    // Act
    let response = app
        .client
        .delete(&format!(
            "{}/groups/{}/expenses/{}",
            &app.address, &group.id, &expense_id
        ))
        .header(header::COOKIE, other_user_2.cookie)
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 403);
    assert_some!(app.get_expense_by_id(expense_id).await);
    match app.get_event_type().await {
        None => {}
        Some(event_type) => assert_ne!(event_type, "ExpenseDeleted".to_string()),
    }
    Ok(())
}

#[test_context(TestApp)]
#[tokio::test]
async fn delete_expense_returns_403_when_user_is_not_member(app: &TestApp) -> anyhow::Result<()> {
    // Arrange
    let group = app
        .create_user_and_group("rbiland", "r@r.com", "201", "my group")
        .await?;
    let other_user = app
        .create_user_and_login_and_device("r", "r1@r.com", "123")
        .await?;
    app.join_group(&group, other_user.cookie.as_str()).await?;
    let expense_id = app
        .create_expense(&group.id, other_user.cookie.as_str(), "expense", 12.0)
        .await?;
    let other_user_2 = app
        .create_user_and_login_and_device("r", "r2@r.com", "123")
        .await?;
    // Act
    let response = app
        .client
        .delete(&format!(
            "{}/groups/{}/expenses/{}",
            &app.address, &group.id, &expense_id
        ))
        .header(header::COOKIE, other_user_2.cookie)
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 403);
    assert_some!(app.get_expense_by_id(expense_id).await);
    match app.get_event_type().await {
        None => {}
        Some(event_type) => assert_ne!(event_type, "ExpenseDeleted".to_string()),
    }
    Ok(())
}

#[test_context(TestApp)]
#[tokio::test]
async fn delete_expense_returns_404_when_expense_does_not_exist(
    app: &TestApp,
) -> anyhow::Result<()> {
    // Arrange
    let group = app
        .create_user_and_group("rbiland", "r@r.com", "201", "my group")
        .await?;
    let other_user = app
        .create_user_and_login_and_device("r", "r1@r.com", "123")
        .await?;
    app.join_group(&group, other_user.cookie.as_str()).await?;
    let expense_id = app
        .create_expense(&group.id, other_user.cookie.as_str(), "expense", 12.0)
        .await?;
    // Act
    let response = app
        .client
        .delete(&format!(
            "{}/groups/{}/expenses/{}",
            &app.address, &group.id, "e6f9b275-3df9-4012-9fbe-47826275bc30"
        ))
        .header(header::COOKIE, other_user.cookie)
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 404);
    assert_some!(app.get_expense_by_id(expense_id).await);
    match app.get_event_type().await {
        None => {}
        Some(event_type) => assert_ne!(event_type, "ExpenseDeleted".to_string()),
    }
    Ok(())
}

#[test_context(TestApp)]
#[tokio::test]
async fn delete_expense_returns_404_when_group_does_not_exist(app: &TestApp) -> anyhow::Result<()> {
    // Arrange
    let group = app
        .create_user_and_group("rbiland", "r@r.com", "201", "my group")
        .await?;
    let other_user = app
        .create_user_and_login_and_device("r", "r1@r.com", "123")
        .await?;
    app.join_group(&group, other_user.cookie.as_str()).await?;
    let expense_id = app
        .create_expense(&group.id, other_user.cookie.as_str(), "expense", 12.0)
        .await?;
    // Act
    let response = app
        .client
        .delete(&format!(
            "{}/groups/{}/expenses/{}",
            &app.address, "e6f9b275-3df9-4012-9fbe-47826275bc30", &expense_id
        ))
        .header(header::COOKIE, other_user.cookie)
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 404);
    assert_some!(app.get_expense_by_id(expense_id).await);
    match app.get_event_type().await {
        None => {}
        Some(event_type) => assert_ne!(event_type, "ExpenseDeleted".to_string()),
    }
    Ok(())
}

#[test_context(TestApp)]
#[tokio::test]
async fn delete_expense_returns_401_when_user_is_not_logged_in(
    app: &TestApp,
) -> anyhow::Result<()> {
    // Arrange
    let group = app
        .create_user_and_group("rbiland", "r@r.com", "201", "my group")
        .await?;
    let other_user = app
        .create_user_and_login_and_device("r", "r1@r.com", "123")
        .await?;
    app.join_group(&group, other_user.cookie.as_str()).await?;
    let expense_id = app
        .create_expense(&group.id, other_user.cookie.as_str(), "expense", 12.0)
        .await?;
    // Act
    let response = app
        .client
        .delete(&format!(
            "{}/groups/{}/expenses/{}",
            &app.address, &group.id, &expense_id
        ))
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 401);
    assert_some!(app.get_expense_by_id(expense_id).await);
    match app.get_event_type().await {
        None => {}
        Some(event_type) => assert_ne!(event_type, "ExpenseDeleted".to_string()),
    }
    Ok(())
}

#[test_context(TestApp)]
#[tokio::test]
async fn delete_expense_returns_400_when_group_is_not_valid(app: &TestApp) -> anyhow::Result<()> {
    // Arrange
    let group = app
        .create_user_and_group("rbiland", "r@r.com", "201", "my group")
        .await?;
    let other_user = app
        .create_user_and_login_and_device("r", "r1@r.com", "123")
        .await?;
    app.join_group(&group, other_user.cookie.as_str()).await?;
    let expense_id = app
        .create_expense(&group.id, other_user.cookie.as_str(), "expense", 12.0)
        .await?;
    // Act
    let response = app
        .client
        .delete(&format!(
            "{}/groups/{}/expenses/{}",
            &app.address, "bob", &expense_id
        ))
        .header(header::COOKIE, other_user.cookie)
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 400);
    assert_some!(app.get_expense_by_id(expense_id).await);
    match app.get_event_type().await {
        None => {}
        Some(event_type) => assert_ne!(event_type, "ExpenseDeleted".to_string()),
    }
    Ok(())
}

#[test_context(TestApp)]
#[tokio::test]
async fn delete_expense_returns_400_when_expense_is_not_valid(app: &TestApp) -> anyhow::Result<()> {
    // Arrange
    let group = app
        .create_user_and_group("rbiland", "r@r.com", "201", "my group")
        .await?;
    let other_user = app
        .create_user_and_login_and_device("r", "r1@r.com", "123")
        .await?;
    app.join_group(&group, other_user.cookie.as_str()).await?;
    let expense_id = app
        .create_expense(&group.id, other_user.cookie.as_str(), "expense", 12.0)
        .await?;
    // Act
    let response = app
        .client
        .delete(&format!(
            "{}/groups/{}/expenses/{}",
            &app.address, &group.id, "bob"
        ))
        .header(header::COOKIE, other_user.cookie)
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 400);
    assert_some!(app.get_expense_by_id(expense_id).await);
    match app.get_event_type().await {
        None => {}
        Some(event_type) => assert_ne!(event_type, "ExpenseDeleted".to_string()),
    }
    Ok(())
}

#[test_context(TestApp)]
#[tokio::test]
async fn delete_expense_returns_404_when_expense_is_settled(app: &TestApp) -> anyhow::Result<()> {
    // Arrange
    let group = app
        .create_user_and_group("rbiland", "r@r.com", "201", "my group")
        .await?;
    let other_user = app
        .create_user_and_login_and_device("r", "r1@r.com", "123")
        .await?;
    app.join_group(&group, other_user.cookie.as_str()).await?;
    let expense_id = app
        .create_expense(&group.id, other_user.cookie.as_str(), "expense", 12.0)
        .await?;
    app.settle(&group).await?;

    // Act
    let response = app
        .client
        .delete(&format!(
            "{}/groups/{}/expenses/{}",
            &app.address, &group.id, &expense_id
        ))
        .header(header::COOKIE, other_user.cookie)
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 404);
    assert_some!(app.get_expense_by_id(expense_id).await);
    match app.get_event_type().await {
        None => {}
        Some(event_type) => assert_ne!(event_type, "ExpenseDeleted".to_string()),
    }
    Ok(())
}
