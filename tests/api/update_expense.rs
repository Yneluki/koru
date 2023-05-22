use crate::test_app::TestApp;
use claim::assert_some;
use reqwest::header;
use serde_json::json;
use test_context::test_context;

#[test_context(TestApp)]
#[tokio::test]
async fn update_expense_returns_200_and_updates_the_expense_when_user_is_admin(
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
        .put(&format!(
            "{}/groups/{}/expenses/{}",
            &app.address, &group.id, &expense_id
        ))
        .header(header::COOKIE, group.admin.cookie)
        .json(&json!({"description":"new name", "amount": 12.95}))
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 200);
    let saved = assert_some!(app.get_expense_by_id(expense_id).await);
    assert_eq!(saved.description, "new name");
    assert_eq!(saved.amount, 12.95);
    assert_eq!(
        app.get_event_type().await,
        Some("ExpenseModified".to_string())
    );
    Ok(())
}

#[test_context(TestApp)]
#[tokio::test]
async fn update_expense_returns_200_and_updates_the_expense_when_user_is_author(
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
        .put(&format!(
            "{}/groups/{}/expenses/{}",
            &app.address, &group.id, &expense_id
        ))
        .header(header::COOKIE, other_user.cookie)
        .json(&json!({"description":"new name", "amount": 12.95}))
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 200);
    let saved = assert_some!(app.get_expense_by_id(expense_id).await);
    assert_eq!(saved.description, "new name");
    assert_eq!(saved.amount, 12.95);
    assert_eq!(
        app.get_event_type().await,
        Some("ExpenseModified".to_string())
    );
    Ok(())
}

#[test_context(TestApp)]
#[tokio::test]
async fn update_expense_returns_403_when_user_is_not_author(app: &TestApp) -> anyhow::Result<()> {
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
        .put(&format!(
            "{}/groups/{}/expenses/{}",
            &app.address, &group.id, &expense_id
        ))
        .header(header::COOKIE, other_user_2.cookie)
        .json(&json!({"description":"new name", "amount": 12.95}))
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 403);
    let saved = assert_some!(app.get_expense_by_id(expense_id).await);
    assert_eq!(saved.description, "expense");
    assert_eq!(saved.amount, 12.0);
    match app.get_event_type().await {
        None => {}
        Some(event_type) => assert_ne!(event_type, "ExpenseModified".to_string()),
    }
    Ok(())
}

#[test_context(TestApp)]
#[tokio::test]
async fn update_expense_returns_403_when_user_is_not_member(app: &TestApp) -> anyhow::Result<()> {
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
        .put(&format!(
            "{}/groups/{}/expenses/{}",
            &app.address, &group.id, &expense_id
        ))
        .header(header::COOKIE, other_user_2.cookie)
        .json(&json!({"description":"new name", "amount": 12.95}))
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 403);
    let saved = assert_some!(app.get_expense_by_id(expense_id).await);
    assert_eq!(saved.description, "expense");
    assert_eq!(saved.amount, 12.0);
    match app.get_event_type().await {
        None => {}
        Some(event_type) => assert_ne!(event_type, "ExpenseModified".to_string()),
    }
    Ok(())
}

#[test_context(TestApp)]
#[tokio::test]
async fn update_expense_returns_404_when_expense_does_not_exist(
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
        .put(&format!(
            "{}/groups/{}/expenses/{}",
            &app.address, &group.id, "e6f9b275-3df9-4012-9fbe-47826275bc30"
        ))
        .header(header::COOKIE, other_user.cookie)
        .json(&json!({"description":"new name", "amount": 12.95}))
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 404);
    let saved = assert_some!(app.get_expense_by_id(expense_id).await);
    assert_eq!(saved.description, "expense");
    assert_eq!(saved.amount, 12.0);
    match app.get_event_type().await {
        None => {}
        Some(event_type) => assert_ne!(event_type, "ExpenseModified".to_string()),
    }
    Ok(())
}

#[test_context(TestApp)]
#[tokio::test]
async fn update_expense_returns_404_when_group_does_not_exist(app: &TestApp) -> anyhow::Result<()> {
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
        .put(&format!(
            "{}/groups/{}/expenses/{}",
            &app.address, "e6f9b275-3df9-4012-9fbe-47826275bc30", &expense_id
        ))
        .header(header::COOKIE, other_user.cookie)
        .json(&json!({"description":"new name", "amount": 12.95}))
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 404);
    let saved = assert_some!(app.get_expense_by_id(expense_id).await);
    assert_eq!(saved.description, "expense");
    assert_eq!(saved.amount, 12.0);
    match app.get_event_type().await {
        None => {}
        Some(event_type) => assert_ne!(event_type, "ExpenseModified".to_string()),
    }
    Ok(())
}

#[test_context(TestApp)]
#[tokio::test]
async fn update_expense_returns_401_when_user_is_not_logged_in(
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
        .put(&format!(
            "{}/groups/{}/expenses/{}",
            &app.address, &group.id, &expense_id
        ))
        .json(&json!({"description":"new name", "amount": 12.95}))
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 401);
    let saved = assert_some!(app.get_expense_by_id(expense_id).await);
    assert_eq!(saved.description, "expense");
    assert_eq!(saved.amount, 12.0);
    match app.get_event_type().await {
        None => {}
        Some(event_type) => assert_ne!(event_type, "ExpenseModified".to_string()),
    }
    Ok(())
}

#[test_context(TestApp)]
#[tokio::test]
async fn update_expense_returns_400_when_group_is_not_valid(app: &TestApp) -> anyhow::Result<()> {
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
        .put(&format!(
            "{}/groups/{}/expenses/{}",
            &app.address, "bob", &expense_id
        ))
        .header(header::COOKIE, other_user.cookie)
        .json(&json!({"description":"new name", "amount": 12.95}))
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 400);
    let saved = assert_some!(app.get_expense_by_id(expense_id).await);
    assert_eq!(saved.description, "expense");
    assert_eq!(saved.amount, 12.0);
    match app.get_event_type().await {
        None => {}
        Some(event_type) => assert_ne!(event_type, "ExpenseModified".to_string()),
    }
    Ok(())
}

#[test_context(TestApp)]
#[tokio::test]
async fn update_expense_returns_400_when_expense_id_is_not_valid(
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
        .put(&format!(
            "{}/groups/{}/expenses/{}",
            &app.address, &group.id, "bob"
        ))
        .header(header::COOKIE, other_user.cookie)
        .json(&json!({"description":"new name", "amount": 12.95}))
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 400);
    let saved = assert_some!(app.get_expense_by_id(expense_id).await);
    assert_eq!(saved.description, "expense");
    assert_eq!(saved.amount, 12.0);
    match app.get_event_type().await {
        None => {}
        Some(event_type) => assert_ne!(event_type, "ExpenseModified".to_string()),
    }
    Ok(())
}

#[test_context(TestApp)]
#[tokio::test]
async fn update_expense_returns_404_when_expense_is_settled(app: &TestApp) -> anyhow::Result<()> {
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
        .put(&format!(
            "{}/groups/{}/expenses/{}",
            &app.address, &group.id, &expense_id
        ))
        .header(header::COOKIE, other_user.cookie)
        .json(&json!({"description":"new name", "amount": 12.95}))
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 404);
    let saved = assert_some!(app.get_expense_by_id(expense_id).await);
    assert_eq!(saved.description, "expense");
    assert_eq!(saved.amount, 12.0);
    match app.get_event_type().await {
        None => {}
        Some(event_type) => assert_ne!(event_type, "ExpenseModified".to_string()),
    }
    Ok(())
}

#[test_context(TestApp)]
#[tokio::test]
async fn update_expense_returns_400_if_the_expense_data_is_invalid(
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
    let cases = vec![
        (
            json!({"description":"","amount": 12.95}),
            "empty description",
        ),
        (json!({"amount": 12.95}), "no description"),
        (json!({"description":"my expense"}), "no amount"),
        (json!({"description":"my expense","amount": 0}), "0 amount"),
        (
            json!({"description":"my expense","amount": -10}),
            "negative amount",
        ),
        (
            json!({"description":"my expense","amount": "stuff"}),
            "text amount",
        ),
    ];
    // Act
    for (body, description) in cases {
        let response = app
            .client
            .put(&format!(
                "{}/groups/{}/expenses/{}",
                &app.address, &group.id, &expense_id
            ))
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
        let saved = assert_some!(app.get_expense_by_id(expense_id).await);
        assert_eq!(saved.description, "expense");
        assert_eq!(saved.amount, 12.0);
        match app.get_event_type().await {
            None => {}
            Some(event_type) => assert_ne!(event_type, "ExpenseModified".to_string()),
        }
    }
    Ok(())
}
