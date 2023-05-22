use crate::test_app::{MemberData, TestApp};
use claim::assert_some;
use reqwest::header;
use test_context::test_context;
use uuid::Uuid;

#[test_context(TestApp)]
#[tokio::test]
async fn get_groups_returns_200_and_the_users_group(app: &TestApp) -> anyhow::Result<()> {
    // Arrange
    let group_1 = app
        .create_user_and_group("rbiland", "r@r.com", "201", "my group")
        .await?;
    let group_2 = app
        .create_user_and_group("rbiland", "r2@r.com", "201", "my group 2")
        .await?;
    let group_3 = app
        .create_user_and_group("rbiland2", "r3@r.com", "201", "my group 3")
        .await?;
    app.add_users_to_group(&group_1, 5).await?;
    app.add_users_to_group(&group_2, 2).await?;
    app.add_users_to_group(&group_3, 3).await?;
    app.join_group(&group_3, &group_1.admin.cookie).await?;
    // Act
    let response = app
        .client
        .get(&format!("{}/groups", &app.address))
        .header(header::COOKIE, &group_1.admin.cookie)
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 200);
    let body = response.json::<GetGroupsResponse>().await?;
    assert_eq!(body.success, true);
    assert_eq!(body.data.groups.len(), 2);
    let grp_1 = body.data.groups.iter().find(|grp| grp.id == group_1.id);
    assert_some!(grp_1);
    let grp_3 = body.data.groups.iter().find(|grp| grp.id == group_3.id);
    assert_some!(grp_3);
    assert_eq!(grp_1.unwrap().name, "my group");
    let admin_1 = grp_1
        .unwrap()
        .members
        .iter()
        .find(|user| user.is_admin)
        .unwrap();
    assert_eq!(admin_1.id, group_1.admin.id);
    assert_eq!(admin_1.name, "rbiland");
    assert_eq!(grp_1.unwrap().members.len(), 6);
    assert_eq!(grp_3.unwrap().name, "my group 3");
    let admin_3 = grp_3
        .unwrap()
        .members
        .iter()
        .find(|user| user.is_admin)
        .unwrap();
    assert_eq!(admin_3.id, group_3.admin.id);
    assert_eq!(admin_3.name, "rbiland2");
    assert_eq!(grp_3.unwrap().members.len(), 5);
    assert_some!(grp_3
        .unwrap()
        .members
        .iter()
        .find(|user| user.id == group_1.admin.id));
    Ok(())
}

#[test_context(TestApp)]
#[tokio::test]
async fn get_groups_returns_200_and_an_empty_list_if_users_has_no_groups(
    app: &TestApp,
) -> anyhow::Result<()> {
    // Arrange
    let user = app
        .create_user_and_login_and_device("rbiland", "r@r.com", "201")
        .await?;
    // Act
    let response = app
        .client
        .get(&format!("{}/groups", &app.address))
        .header(header::COOKIE, user.cookie)
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 200);
    let body = response.json::<GetGroupsResponse>().await?;
    assert_eq!(body.success, true);
    assert_eq!(body.data.groups.len(), 0);
    Ok(())
}

#[test_context(TestApp)]
#[tokio::test]
async fn get_groups_returns_401_if_user_is_not_logged_in(app: &TestApp) -> anyhow::Result<()> {
    // Arrange
    let _ = app
        .create_user_and_login_and_device("rbiland", "r@r.com", "201")
        .await?;
    // Act
    let response = app
        .client
        .get(&format!("{}/groups", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 401);
    Ok(())
}

#[derive(serde::Deserialize)]
pub struct GetGroupsResponse {
    pub success: bool,
    pub data: GroupsData,
}

#[derive(serde::Deserialize)]
pub struct GroupsData {
    pub groups: Vec<GroupData>,
}

#[derive(serde::Deserialize)]
pub struct GroupData {
    pub id: Uuid,
    pub name: String,
    pub members: Vec<MemberData>,
}
