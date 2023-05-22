use crate::test_app::TestApp;
use claim::{assert_none, assert_some};
use reqwest::header;
use test_context::test_context;

#[test_context(TestApp)]
#[tokio::test]
async fn group_deletion_success_deletes_the_group_and_returns_204(
    app: &TestApp,
) -> anyhow::Result<()> {
    // Arrange
    let group = app
        .create_user_and_group("rbiland", "r@r.com", "201", "my group")
        .await?;
    app.add_users_to_group(&group, 5).await?;
    // Act
    let response = app
        .client
        .delete(&format!("{}/groups/{}", &app.address, &group.id))
        .header(header::COOKIE, group.admin.cookie)
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 204);
    assert_none!(app.get_group_by_id(group.id).await);
    Ok(())
}

#[test_context(TestApp)]
#[tokio::test]
async fn group_deletion_returns_403_if_user_is_not_admin(app: &TestApp) -> anyhow::Result<()> {
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
        .delete(&format!("{}/groups/{}", &app.address, &group.id))
        .header(header::COOKIE, other_user.cookie)
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 403);
    assert_some!(app.get_group_by_id(group.id).await);
    Ok(())
}

#[test_context(TestApp)]
#[tokio::test]
async fn group_deletion_returns_401_if_user_is_not_logged_in(app: &TestApp) -> anyhow::Result<()> {
    // Arrange
    let group = app
        .create_user_and_group("rbiland", "r@r.com", "201", "my group")
        .await?;
    // Act
    let response = app
        .client
        .delete(&format!("{}/groups/{}", &app.address, &group.id))
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 401);
    assert_some!(app.get_group_by_id(group.id).await);
    Ok(())
}

#[test_context(TestApp)]
#[tokio::test]
async fn group_deletion_returns_404_if_group_does_not_exist(app: &TestApp) -> anyhow::Result<()> {
    // Arrange
    let group = app
        .create_user_and_group("rbiland", "r@r.com", "201", "my group")
        .await?;
    app.delete_group(group.id).await;
    // Act
    let response = app
        .client
        .delete(&format!("{}/groups/{}", &app.address, &group.id))
        .header(header::COOKIE, group.admin.cookie)
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 404);
    Ok(())
}

#[test_context(TestApp)]
#[tokio::test]
async fn group_deletion_returns_400_if_group_id_is_invalid(app: &TestApp) -> anyhow::Result<()> {
    // Arrange
    let group = app
        .create_user_and_group("rbiland", "r@r.com", "201", "my group")
        .await?;
    // Act
    let response = app
        .client
        .delete(&format!("{}/groups/{}", &app.address, "bob"))
        .header(header::COOKIE, group.admin.cookie)
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 400);
    assert_some!(app.get_group_by_id(group.id).await);
    Ok(())
}
