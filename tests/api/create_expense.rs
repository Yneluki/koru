use crate::test_app::{CreateExpenseResponse, TestApp};
use claim::{assert_none, assert_some};
use reqwest::header;
use serde_json::json;
use test_context::test_context;

#[test_context(TestApp)]
#[tokio::test]
async fn create_expense_returns_201_and_saves_the_expense_when_user_is_admin(
    app: &TestApp,
) -> anyhow::Result<()> {
    // Arrange
    let group = app
        .create_user_and_group("rbiland", "r@r.com", "201", "my group")
        .await?;
    // Act
    let response = app
        .client
        .post(&format!("{}/groups/{}/expenses", &app.address, &group.id))
        .header(header::COOKIE, group.admin.cookie)
        .json(&json!({"description":"my expense", "amount": 12.95}))
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 201);
    let saved = assert_some!(app.get_expense().await);
    assert_eq!(saved.group_id, group.id);
    assert_eq!(saved.member_id, group.admin.id);
    assert_eq!(saved.description, "my expense");
    assert_eq!(saved.amount, 12.95);
    let body = response.json::<CreateExpenseResponse>().await?;
    assert_eq!(body.success, true);
    assert_eq!(body.data.id, saved.id);
    assert_eq!(
        app.get_event_type().await,
        Some("ExpenseCreated".to_string())
    );
    Ok(())
}

#[test_context(TestApp)]
#[tokio::test]
async fn create_expense_returns_201_and_saves_the_expense_when_user_is_member(
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
        .post(&format!("{}/groups/{}/expenses", &app.address, &group.id))
        .header(header::COOKIE, other_user.cookie)
        .json(&json!({"description":"my expense", "amount": 12.95}))
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 201);
    let saved = assert_some!(app.get_expense().await);
    assert_eq!(saved.group_id, group.id);
    assert_eq!(saved.member_id, other_user.id);
    assert_eq!(saved.description, "my expense");
    assert_eq!(saved.amount, 12.95);
    let body = response.json::<CreateExpenseResponse>().await?;
    assert_eq!(body.success, true);
    assert_eq!(body.data.id, saved.id);
    assert_eq!(
        app.get_event_type().await,
        Some("ExpenseCreated".to_string())
    );
    Ok(())
}

#[test_context(TestApp)]
#[tokio::test]
async fn create_expense_returns_400_if_the_group_id_is_invalid(
    app: &TestApp,
) -> anyhow::Result<()> {
    // Arrange
    let group = app
        .create_user_and_group("rbiland", "r@r.com", "201", "my group")
        .await?;
    // Act
    let response = app
        .client
        .post(&format!("{}/groups/{}/expenses", &app.address, "bob"))
        .header(header::COOKIE, group.admin.cookie)
        .json(&json!({"description":"my expense", "amount": 12.95}))
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 400);
    assert_none!(app.get_expense().await);
    match app.get_event_type().await {
        None => {}
        Some(event_type) => assert_ne!(event_type, "ExpenseCreated".to_string()),
    }
    Ok(())
}

#[test_context(TestApp)]
#[tokio::test]
async fn create_expense_returns_400_if_the_expense_data_is_invalid(
    app: &TestApp,
) -> anyhow::Result<()> {
    // Arrange
    let group = app
        .create_user_and_group("rbiland", "r@r.com", "201", "my group")
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
            .post(&format!("{}/groups/{}/expenses", &app.address, &group.id))
            .header(header::COOKIE, &group.admin.cookie)
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
        assert_none!(app.get_expense().await);
        match app.get_event_type().await {
            None => {}
            Some(event_type) => assert_ne!(event_type, "ExpenseCreated".to_string()),
        }
    }
    Ok(())
}

#[test_context(TestApp)]
#[tokio::test]
async fn create_expense_returns_404_if_the_group_does_not_exist(
    app: &TestApp,
) -> anyhow::Result<()> {
    // Arrange
    let group = app
        .create_user_and_group("rbiland", "r@r.com", "201", "my group")
        .await?;
    // Act
    let response = app
        .client
        .post(&format!(
            "{}/groups/{}/expenses",
            &app.address, "e6f9b275-3df9-4012-9fbe-47826275bc30"
        ))
        .header(header::COOKIE, group.admin.cookie)
        .json(&json!({"description":"my expense", "amount": 12.95}))
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 404);
    assert_none!(app.get_expense().await);
    match app.get_event_type().await {
        None => {}
        Some(event_type) => assert_ne!(event_type, "ExpenseCreated".to_string()),
    }
    Ok(())
}

#[test_context(TestApp)]
#[tokio::test]
async fn create_expense_returns_403_if_user_is_not_in_the_group(
    app: &TestApp,
) -> anyhow::Result<()> {
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
        .post(&format!("{}/groups/{}/expenses", &app.address, &group.id))
        .header(header::COOKIE, other_user.cookie)
        .json(&json!({"description":"my expense", "amount": 12.95}))
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 403);
    assert_none!(app.get_expense().await);
    match app.get_event_type().await {
        None => {}
        Some(event_type) => assert_ne!(event_type, "ExpenseCreated".to_string()),
    }
    Ok(())
}

#[test_context(TestApp)]
#[tokio::test]
async fn create_expense_returns_401_if_user_is_not_logged_in(app: &TestApp) -> anyhow::Result<()> {
    // Arrange
    let group = app
        .create_user_and_group("rbiland", "r@r.com", "201", "my group")
        .await?;
    // Act
    let response = app
        .client
        .post(&format!("{}/groups/{}/expenses", &app.address, &group.id))
        .json(&json!({"description":"my expense", "amount": 12.95}))
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 401);
    assert_none!(app.get_expense().await);
    match app.get_event_type().await {
        None => {}
        Some(event_type) => assert_ne!(event_type, "ExpenseCreated".to_string()),
    }
    Ok(())
}
