use crate::test_app::{SettlementsResponse, TestApp};
use reqwest::header;
use test_context::test_context;
use uuid::Uuid;

#[test_context(TestApp)]
#[tokio::test]
async fn get_settlements_return_200_and_the_list_of_settlements_when_user_is_admin(
    app: &TestApp,
) -> anyhow::Result<()> {
    // Arrange
    let group = app
        .create_user_and_group("rbiland", "r@r.com", "201", "my group")
        .await?;
    let user_1 = app
        .create_user_and_login_and_device("r1", "r1@r.com", "201")
        .await?;
    let user_2 = app
        .create_user_and_login_and_device("r2", "r2@r.com", "201")
        .await?;
    let user_3 = app
        .create_user_and_login_and_device("r3", "r3@r.com", "201")
        .await?;
    let cookie_1 = user_1.cookie.as_str();
    let cookie_2 = user_2.cookie.as_str();
    let cookie_3 = user_3.cookie.as_str();
    let cookie_adm = group.admin.cookie.as_str();
    app.join_group(&group, cookie_1).await?;
    app.join_group(&group, cookie_2).await?;
    app.join_group(&group, cookie_3).await?;
    let _ = app
        .create_expense(&group.id, cookie_1, "expense1", 10.0)
        .await?;
    let _ = app
        .create_expense(&group.id, cookie_1, "expense2", 10.0)
        .await?;
    let _ = app
        .create_expense(&group.id, cookie_2, "expense3", 5.0)
        .await?;
    let _ = app
        .create_expense(&group.id, cookie_3, "expense4", 8.0)
        .await?;
    let stl_1 = app.settle(&group).await?;

    let _ = app
        .create_expense(&group.id, cookie_1, "expense5", 20.0)
        .await?;
    let _ = app
        .create_expense(&group.id, cookie_2, "expense6", 15.0)
        .await?;
    let _ = app
        .create_expense(&group.id, cookie_2, "expense7", 12.0)
        .await?;
    let _ = app
        .create_expense(&group.id, cookie_3, "expense8", 10.0)
        .await?;
    let stl_2 = app.settle(&group).await?;
    // Act
    let response = app
        .client
        .get(&format!(
            "{}/groups/{}/settlements",
            &app.address, &group.id
        ))
        .header(header::COOKIE, cookie_adm)
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 200);

    let body = response.json::<SettlementsResponse>().await?;
    assert_eq!(body.success, true);
    assert_eq!(body.data.settlements.len(), 2);
    assert_eq!(body.data.settlements.get(1).unwrap(), &stl_1);
    assert_eq!(body.data.settlements.get(0).unwrap(), &stl_2);
    Ok(())
}

#[test_context(TestApp)]
#[tokio::test]
async fn get_settlements_return_200_and_the_list_of_settlements_when_user_is_member(
    app: &TestApp,
) -> anyhow::Result<()> {
    // Arrange
    let group = app
        .create_user_and_group("rbiland", "r@r.com", "201", "my group")
        .await?;
    let user_1 = app
        .create_user_and_login_and_device("r1", "r1@r.com", "201")
        .await?;
    let user_2 = app
        .create_user_and_login_and_device("r2", "r2@r.com", "201")
        .await?;
    let user_3 = app
        .create_user_and_login_and_device("r3", "r3@r.com", "201")
        .await?;
    let cookie_1 = user_1.cookie.as_str();
    let cookie_2 = user_2.cookie.as_str();
    let cookie_3 = user_3.cookie.as_str();
    app.join_group(&group, cookie_1).await?;
    app.join_group(&group, cookie_2).await?;
    app.join_group(&group, cookie_3).await?;
    let _ = app
        .create_expense(&group.id, cookie_1, "expense1", 10.0)
        .await?;
    let _ = app
        .create_expense(&group.id, cookie_1, "expense2", 10.0)
        .await?;
    let _ = app
        .create_expense(&group.id, cookie_2, "expense3", 5.0)
        .await?;
    let _ = app
        .create_expense(&group.id, cookie_3, "expense4", 8.0)
        .await?;
    let stl_1 = app.settle(&group).await?;

    let _ = app
        .create_expense(&group.id, cookie_1, "expense5", 20.0)
        .await?;
    let _ = app
        .create_expense(&group.id, cookie_2, "expense6", 15.0)
        .await?;
    let _ = app
        .create_expense(&group.id, cookie_2, "expense7", 12.0)
        .await?;
    let _ = app
        .create_expense(&group.id, cookie_3, "expense8", 10.0)
        .await?;
    let stl_2 = app.settle(&group).await?;
    // Act
    let response = app
        .client
        .get(&format!(
            "{}/groups/{}/settlements",
            &app.address, &group.id
        ))
        .header(header::COOKIE, cookie_1)
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 200);

    let body = response.json::<SettlementsResponse>().await?;
    assert_eq!(body.success, true);
    assert_eq!(body.data.settlements.len(), 2);
    assert_eq!(body.data.settlements.get(1).unwrap(), &stl_1);
    assert_eq!(body.data.settlements.get(0).unwrap(), &stl_2);
    Ok(())
}

#[test_context(TestApp)]
#[tokio::test]
async fn get_settlements_return_403_when_user_is_not_a_member(app: &TestApp) -> anyhow::Result<()> {
    // Arrange
    let group = app
        .create_user_and_group("rbiland", "r@r.com", "201", "my group")
        .await?;
    let user_1 = app
        .create_user_and_login_and_device("r1", "r1@r.com", "201")
        .await?;
    let user_2 = app
        .create_user_and_login_and_device("r2", "r2@r.com", "201")
        .await?;
    let user_3 = app
        .create_user_and_login_and_device("r3", "r3@r.com", "201")
        .await?;
    let cookie_1 = user_1.cookie.as_str();
    let cookie_2 = user_2.cookie.as_str();
    let cookie_3 = user_3.cookie.as_str();
    app.join_group(&group, cookie_1).await?;
    app.join_group(&group, cookie_2).await?;
    let _ = app
        .create_expense(&group.id, cookie_1, "expense1", 10.0)
        .await?;
    let _ = app
        .create_expense(&group.id, cookie_1, "expense2", 10.0)
        .await?;
    let _ = app
        .create_expense(&group.id, cookie_2, "expense3", 5.0)
        .await?;
    let _ = app.settle(&group).await?;

    let _ = app
        .create_expense(&group.id, cookie_1, "expense4", 20.0)
        .await?;
    let _ = app
        .create_expense(&group.id, cookie_2, "expense5", 15.0)
        .await?;
    let _ = app
        .create_expense(&group.id, cookie_2, "expense6", 12.0)
        .await?;
    let _ = app.settle(&group).await?;
    // Act
    let response = app
        .client
        .get(&format!(
            "{}/groups/{}/settlements",
            &app.address, &group.id
        ))
        .header(header::COOKIE, cookie_3)
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 403);
    Ok(())
}

#[test_context(TestApp)]
#[tokio::test]
async fn get_settlements_return_401_when_user_is_not_logged_in(
    app: &TestApp,
) -> anyhow::Result<()> {
    // Arrange
    let group = app
        .create_user_and_group("rbiland", "r@r.com", "201", "my group")
        .await?;
    let user_1 = app
        .create_user_and_login_and_device("r1", "r1@r.com", "201")
        .await?;
    let user_2 = app
        .create_user_and_login_and_device("r2", "r2@r.com", "201")
        .await?;
    let user_3 = app
        .create_user_and_login_and_device("r3", "r3@r.com", "201")
        .await?;
    let cookie_1 = user_1.cookie.as_str();
    let cookie_2 = user_2.cookie.as_str();
    let cookie_3 = user_3.cookie.as_str();
    app.join_group(&group, cookie_1).await?;
    app.join_group(&group, cookie_2).await?;
    app.join_group(&group, cookie_3).await?;
    let _ = app
        .create_expense(&group.id, cookie_1, "expense1", 10.0)
        .await?;
    let _ = app
        .create_expense(&group.id, cookie_1, "expense2", 10.0)
        .await?;
    let _ = app
        .create_expense(&group.id, cookie_2, "expense3", 5.0)
        .await?;
    let _ = app.settle(&group).await?;

    let _ = app
        .create_expense(&group.id, cookie_1, "expense4", 20.0)
        .await?;
    let _ = app
        .create_expense(&group.id, cookie_2, "expense5", 15.0)
        .await?;
    let _ = app
        .create_expense(&group.id, cookie_2, "expense6", 12.0)
        .await?;
    let _ = app.settle(&group).await?;
    // Act
    let response = app
        .client
        .get(&format!(
            "{}/groups/{}/settlements",
            &app.address, &group.id
        ))
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 401);
    Ok(())
}

#[test_context(TestApp)]
#[tokio::test]
async fn get_settlements_return_404_when_group_does_not_exist(app: &TestApp) -> anyhow::Result<()> {
    // Arrange
    let group = app
        .create_user_and_group("rbiland", "r@r.com", "201", "my group")
        .await?;
    let user_1 = app
        .create_user_and_login_and_device("r1", "r1@r.com", "201")
        .await?;
    let user_2 = app
        .create_user_and_login_and_device("r2", "r2@r.com", "201")
        .await?;
    let user_3 = app
        .create_user_and_login_and_device("r3", "r3@r.com", "201")
        .await?;
    let cookie_1 = user_1.cookie.as_str();
    let cookie_2 = user_2.cookie.as_str();
    let cookie_3 = user_3.cookie.as_str();
    app.join_group(&group, cookie_1).await?;
    app.join_group(&group, cookie_2).await?;
    app.join_group(&group, cookie_3).await?;
    let _ = app
        .create_expense(&group.id, cookie_1, "expense1", 10.0)
        .await?;
    let _ = app
        .create_expense(&group.id, cookie_1, "expense2", 10.0)
        .await?;
    let _ = app
        .create_expense(&group.id, cookie_2, "expense3", 5.0)
        .await?;
    let _ = app.settle(&group).await?;

    let _ = app
        .create_expense(&group.id, cookie_1, "expense4", 20.0)
        .await?;
    let _ = app
        .create_expense(&group.id, cookie_2, "expense5", 15.0)
        .await?;
    let _ = app
        .create_expense(&group.id, cookie_2, "expense6", 12.0)
        .await?;
    let _ = app.settle(&group).await?;
    // Act
    let response = app
        .client
        .get(&format!(
            "{}/groups/{}/settlements",
            &app.address,
            Uuid::new_v4()
        ))
        .header(header::COOKIE, cookie_1)
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 404);
    Ok(())
}
