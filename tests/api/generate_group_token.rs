use crate::test_app::{GenerateTokenResponse, TestApp};
use claim::{assert_gt, assert_ok};
use reqwest::header;
use test_context::test_context;

#[test_context(TestApp)]
#[tokio::test]
async fn generate_group_token_returns_200_and_a_valid_token_when_called_by_group_admin(
    app: &TestApp,
) -> anyhow::Result<()> {
    // Arrange
    let group = app
        .create_user_and_group("rbiland", "r@r.com", "201", "my group")
        .await?;
    // Act
    let response = app
        .client
        .get(&format!("{}/groups/{}/token", &app.address, &group.id))
        .header(header::COOKIE, group.admin.cookie)
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 200);
    let body = response.json::<GenerateTokenResponse>().await?;
    assert_eq!(body.success, true);
    assert_gt!(body.data.token.len(), 0);
    assert_ok!(jsonwebtoken::decode_header(&*body.data.token));
    Ok(())
}

#[test_context(TestApp)]
#[tokio::test]
async fn generate_group_token_returns_403_when_not_called_by_group_admin(
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
        .get(&format!("{}/groups/{}/token", &app.address, &group.id))
        .header(header::COOKIE, other_user.cookie)
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 403);
    Ok(())
}

#[test_context(TestApp)]
#[tokio::test]
async fn generate_group_token_returns_404_when_group_does_not_exist(
    app: &TestApp,
) -> anyhow::Result<()> {
    // Arrange
    let user = app
        .create_user_and_login_and_device("r", "r1@r.com", "123")
        .await?;
    // Act
    let response = app
        .client
        .get(&format!(
            "{}/groups/{}/token",
            &app.address, "e6f9b275-3df9-4012-9fbe-47826275bc30"
        ))
        .header(header::COOKIE, user.cookie)
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 404);
    Ok(())
}

#[test_context(TestApp)]
#[tokio::test]
async fn generate_group_token_returns_400_when_group_id_is_invalid(
    app: &TestApp,
) -> anyhow::Result<()> {
    // Arrange
    let user = app
        .create_user_and_login_and_device("r", "r1@r.com", "123")
        .await?;
    // Act
    let response = app
        .client
        .get(&format!("{}/groups/{}/token", &app.address, "bob"))
        .header(header::COOKIE, user.cookie)
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 400);
    Ok(())
}

#[test_context(TestApp)]
#[tokio::test]
async fn generate_group_token_returns_500_on_db_error(app: &TestApp) -> anyhow::Result<()> {
    // Arrange
    let group = app
        .create_user_and_group("rbiland", "r@r.com", "201", "my group")
        .await?;
    app.break_group_db().await;
    // Act
    let response = app
        .client
        .get(&format!("{}/groups/{}/token", &app.address, &group.id))
        .header(header::COOKIE, group.admin.cookie)
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 500);
    Ok(())
}
