use crate::test_app::{TestApp, UserData};
use chrono::Utc;
use reqwest::header;
use std::time::Duration;
use test_context::test_context;
use uuid::Uuid;

#[test_context(TestApp)]
#[tokio::test]
async fn get_expenses_return_200_and_the_list_of_expenses_when_user_is_admin(
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
    app.settle(&group).await?;

    let exp4 = app
        .create_expense(&group.id, cookie_1, "expense4", 20.0)
        .await?;
    let exp5 = app
        .create_expense(&group.id, cookie_2, "expense5", 15.0)
        .await?;
    let exp6 = app
        .create_expense(&group.id, cookie_2, "expense6", 12.0)
        .await?;
    // Act
    let response = app
        .client
        .get(&format!("{}/groups/{}/expenses", &app.address, &group.id))
        .header(header::COOKIE, cookie_adm)
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 200);

    let body = response.json::<ExpensesResponse>().await?;
    assert_eq!(body.success, true);
    assert_eq!(body.data.expenses.len(), 3);
    assert_eq!(
        body.data.expenses.get(2).unwrap(),
        &ExpenseData {
            id: exp4,
            description: "expense4".to_string(),
            amount: 20.0,
            user: UserData {
                id: user_1.id,
                name: String::from("r1"),
            },
        }
    );
    assert_eq!(
        body.data.expenses.get(1).unwrap(),
        &ExpenseData {
            id: exp5,
            description: "expense5".to_string(),
            amount: 15.0,
            user: UserData {
                id: user_2.id,
                name: String::from("r2"),
            },
        }
    );
    assert_eq!(
        body.data.expenses.get(0).unwrap(),
        &ExpenseData {
            id: exp6,
            description: "expense6".to_string(),
            amount: 12.0,
            user: UserData {
                id: user_2.id,
                name: String::from("r2"),
            },
        }
    );
    Ok(())
}

#[test_context(TestApp)]
#[tokio::test]
async fn get_expenses_return_200_and_the_list_of_expenses_when_user_is_member(
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
    app.settle(&group).await?;

    let exp4 = app
        .create_expense(&group.id, cookie_1, "expense4", 20.0)
        .await?;
    let exp5 = app
        .create_expense(&group.id, cookie_2, "expense5", 15.0)
        .await?;
    let exp6 = app
        .create_expense(&group.id, cookie_2, "expense6", 12.0)
        .await?;
    // Act
    let response = app
        .client
        .get(&format!("{}/groups/{}/expenses", &app.address, &group.id))
        .header(header::COOKIE, cookie_1)
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 200);

    let body = response.json::<ExpensesResponse>().await?;
    assert_eq!(body.success, true);
    assert_eq!(body.data.expenses.len(), 3);
    assert_eq!(
        body.data.expenses.get(2).unwrap(),
        &ExpenseData {
            id: exp4,
            description: "expense4".to_string(),
            amount: 20.0,
            user: UserData {
                id: user_1.id,
                name: String::from("r1"),
            },
        }
    );
    assert_eq!(
        body.data.expenses.get(1).unwrap(),
        &ExpenseData {
            id: exp5,
            description: "expense5".to_string(),
            amount: 15.0,
            user: UserData {
                id: user_2.id,
                name: String::from("r2"),
            },
        }
    );
    assert_eq!(
        body.data.expenses.get(0).unwrap(),
        &ExpenseData {
            id: exp6,
            description: "expense6".to_string(),
            amount: 12.0,
            user: UserData {
                id: user_2.id,
                name: String::from("r2"),
            },
        }
    );
    Ok(())
}

#[test_context(TestApp)]
#[tokio::test]
async fn get_expenses_return_403_when_user_is_not_a_member(app: &TestApp) -> anyhow::Result<()> {
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
    app.join_group(&group, user_1.cookie.as_str()).await?;
    let _ = app
        .create_expense(&group.id, user_1.cookie.as_str(), "expense1", 10.0)
        .await?;
    // Act
    let response = app
        .client
        .get(&format!("{}/groups/{}/expenses", &app.address, &group.id))
        .header(header::COOKIE, user_2.cookie.as_str())
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 403);
    Ok(())
}

#[test_context(TestApp)]
#[tokio::test]
async fn get_expenses_return_401_when_user_is_not_logged_in(app: &TestApp) -> anyhow::Result<()> {
    // Arrange
    let group = app
        .create_user_and_group("rbiland", "r@r.com", "201", "my group")
        .await?;
    let user_1 = app
        .create_user_and_login_and_device("r1", "r1@r.com", "201")
        .await?;
    app.join_group(&group, user_1.cookie.as_str()).await?;
    let _ = app
        .create_expense(&group.id, user_1.cookie.as_str(), "expense1", 10.0)
        .await?;
    // Act
    let response = app
        .client
        .get(&format!("{}/groups/{}/expenses", &app.address, &group.id))
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 401);
    Ok(())
}

#[test_context(TestApp)]
#[tokio::test]
async fn get_expenses_return_404_when_group_is_not_found(app: &TestApp) -> anyhow::Result<()> {
    // Arrange
    let group = app
        .create_user_and_group("rbiland", "r@r.com", "201", "my group")
        .await?;
    let user_1 = app
        .create_user_and_login_and_device("r1", "r1@r.com", "201")
        .await?;
    app.join_group(&group, user_1.cookie.as_str()).await?;
    let _ = app
        .create_expense(&group.id, user_1.cookie.as_str(), "expense1", 10.0)
        .await?;
    // Act
    let response = app
        .client
        .get(&format!(
            "{}/groups/{}/expenses",
            &app.address, "e6f9b275-3df9-4012-9fbe-47826275bc30"
        ))
        .header(header::COOKIE, user_1.cookie.as_str())
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 404);
    Ok(())
}

#[test_context(TestApp)]
#[tokio::test]
async fn get_expenses_return_400_when_group_is_invalid(app: &TestApp) -> anyhow::Result<()> {
    // Arrange
    let group = app
        .create_user_and_group("rbiland", "r@r.com", "201", "my group")
        .await?;
    let user_1 = app
        .create_user_and_login_and_device("r1", "r1@r.com", "201")
        .await?;
    app.join_group(&group, user_1.cookie.as_str()).await?;
    let _ = app
        .create_expense(&group.id, user_1.cookie.as_str(), "expense1", 10.0)
        .await?;
    // Act
    let response = app
        .client
        .get(&format!("{}/groups/{}/expenses", &app.address, "bob"))
        .header(header::COOKIE, user_1.cookie.as_str())
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 400);
    Ok(())
}

#[test_context(TestApp)]
#[tokio::test]
async fn get_expenses_return_200_and_an_empty_list_when_there_are_no_expenses(
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
    // Act
    let response = app
        .client
        .get(&format!("{}/groups/{}/expenses", &app.address, &group.id))
        .header(header::COOKIE, cookie_1)
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 200);

    let body = response.json::<ExpensesResponse>().await?;
    assert_eq!(body.success, true);
    assert_eq!(body.data.expenses.len(), 0);
    Ok(())
}

#[test_context(TestApp)]
#[tokio::test]
async fn get_expenses_between_dates_return_200_and_an_empty_list_when_there_are_no_expenses(
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
    // Act
    let response = app
        .client
        .get(&format!(
            "{}/groups/{}/expenses?from={}&to={}",
            &app.address,
            &group.id,
            Utc::now().timestamp_millis(),
            Utc::now().timestamp_millis() + 100
        ))
        .header(header::COOKIE, cookie_1)
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 200);

    let body = response.json::<ExpensesResponse>().await?;
    assert_eq!(body.success, true);
    assert_eq!(body.data.expenses.len(), 0);
    Ok(())
}

#[test_context(TestApp)]
#[tokio::test]
async fn get_expenses_between_dates_return_200_and_expenses_between_those_dates(
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
        .create_expense(&group.id, cookie_1, "expense1", 20.0)
        .await?;
    tokio::time::sleep(Duration::from_millis(1)).await;
    let from = Utc::now().timestamp_millis();
    tokio::time::sleep(Duration::from_millis(1)).await;
    let exp2 = app
        .create_expense(&group.id, cookie_2, "expense2", 15.0)
        .await?;
    let exp3 = app
        .create_expense(&group.id, cookie_2, "expense3", 12.0)
        .await?;
    tokio::time::sleep(Duration::from_millis(1)).await;
    let to = Utc::now().timestamp_millis();
    tokio::time::sleep(Duration::from_millis(1)).await;
    let _ = app
        .create_expense(&group.id, cookie_3, "expense4", 12.0)
        .await?;
    // Act
    let response = app
        .client
        .get(&format!(
            "{}/groups/{}/expenses?from={}&to={}",
            &app.address, &group.id, from, to
        ))
        .header(header::COOKIE, cookie_adm)
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 200);

    let body = response.json::<ExpensesResponse>().await?;
    assert_eq!(body.success, true);
    assert_eq!(body.data.expenses.len(), 2);
    assert_eq!(
        body.data.expenses.get(1).unwrap(),
        &ExpenseData {
            id: exp2,
            description: "expense2".to_string(),
            amount: 15.0,
            user: UserData {
                id: user_2.id,
                name: String::from("r2"),
            },
        }
    );
    assert_eq!(
        body.data.expenses.get(0).unwrap(),
        &ExpenseData {
            id: exp3,
            description: "expense3".to_string(),
            amount: 12.0,
            user: UserData {
                id: user_2.id,
                name: String::from("r2"),
            },
        }
    );
    Ok(())
}

#[test_context(TestApp)]
#[tokio::test]
async fn get_expenses_between_dates_return_400_when_date_filters_are_invalid(
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
        .create_expense(&group.id, cookie_1, "expense1", 20.0)
        .await?;
    let from = Utc::now().timestamp_millis();
    let _ = app
        .create_expense(&group.id, cookie_2, "expense2", 15.0)
        .await?;
    let _ = app
        .create_expense(&group.id, cookie_2, "expense3", 12.0)
        .await?;
    let to = Utc::now().timestamp_millis();
    let _ = app
        .create_expense(&group.id, cookie_3, "expense4", 12.0)
        .await?;
    let cases = vec![
        (
            format!(
                "{}/groups/{}/expenses?from={}&to={}",
                &app.address, &group.id, "bob", to
            ),
            "Invalid from",
        ),
        (
            format!(
                "{}/groups/{}/expenses?from={}&to={}",
                &app.address, &group.id, from, "bob"
            ),
            "Invalid to",
        ),
        (
            format!(
                "{}/groups/{}/expenses?from={}&to={}",
                &app.address, &group.id, "bob", to
            ),
            "Invalid from & to",
        ),
    ];
    // Act
    for (url, description) in cases {
        let response = app
            .client
            .get(&url)
            .header(header::COOKIE, cookie_adm)
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
    }

    Ok(())
}

#[test_context(TestApp)]
#[tokio::test]
async fn get_expenses_after_dates_return_200_and_expenses_after_dates(
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
        .create_expense(&group.id, cookie_1, "expense1", 20.0)
        .await?;
    tokio::time::sleep(Duration::from_millis(1)).await;
    let from = Utc::now().timestamp_millis();
    tokio::time::sleep(Duration::from_millis(1)).await;
    let exp2 = app
        .create_expense(&group.id, cookie_2, "expense2", 15.0)
        .await?;
    let exp3 = app
        .create_expense(&group.id, cookie_2, "expense3", 12.0)
        .await?;
    let exp4 = app
        .create_expense(&group.id, cookie_3, "expense4", 12.0)
        .await?;
    // Act
    let response = app
        .client
        .get(&format!(
            "{}/groups/{}/expenses?from={}",
            &app.address, &group.id, from,
        ))
        .header(header::COOKIE, cookie_adm)
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 200);

    let body = response.json::<ExpensesResponse>().await?;
    assert_eq!(body.success, true);
    assert_eq!(body.data.expenses.len(), 3);
    assert_eq!(
        body.data.expenses.get(2).unwrap(),
        &ExpenseData {
            id: exp2,
            description: "expense2".to_string(),
            amount: 15.0,
            user: UserData {
                id: user_2.id,
                name: String::from("r2"),
            },
        }
    );
    assert_eq!(
        body.data.expenses.get(1).unwrap(),
        &ExpenseData {
            id: exp3,
            description: "expense3".to_string(),
            amount: 12.0,
            user: UserData {
                id: user_2.id,
                name: String::from("r2"),
            },
        }
    );
    assert_eq!(
        body.data.expenses.get(0).unwrap(),
        &ExpenseData {
            id: exp4,
            description: "expense4".to_string(),
            amount: 12.0,
            user: UserData {
                id: user_3.id,
                name: String::from("r3"),
            },
        }
    );
    Ok(())
}

#[test_context(TestApp)]
#[tokio::test]
async fn get_expenses_before_dates_return_200_and_expenses_before_dates(
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
    let exp1 = app
        .create_expense(&group.id, cookie_1, "expense1", 20.0)
        .await?;
    let exp2 = app
        .create_expense(&group.id, cookie_2, "expense2", 15.0)
        .await?;
    let exp3 = app
        .create_expense(&group.id, cookie_2, "expense3", 12.0)
        .await?;
    tokio::time::sleep(Duration::from_millis(1)).await;
    let to = Utc::now().timestamp_millis();
    tokio::time::sleep(Duration::from_millis(1)).await;
    let _ = app
        .create_expense(&group.id, cookie_3, "expense4", 12.0)
        .await?;
    // Act
    let response = app
        .client
        .get(&format!(
            "{}/groups/{}/expenses?to={}",
            &app.address, &group.id, to,
        ))
        .header(header::COOKIE, cookie_adm)
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 200);

    let body = response.json::<ExpensesResponse>().await?;
    assert_eq!(body.success, true);
    assert_eq!(body.data.expenses.len(), 3);
    assert_eq!(
        body.data.expenses.get(2).unwrap(),
        &ExpenseData {
            id: exp1,
            description: "expense1".to_string(),
            amount: 20.0,
            user: UserData {
                id: user_1.id,
                name: String::from("r1"),
            },
        }
    );
    assert_eq!(
        body.data.expenses.get(1).unwrap(),
        &ExpenseData {
            id: exp2,
            description: "expense2".to_string(),
            amount: 15.0,
            user: UserData {
                id: user_2.id,
                name: String::from("r2"),
            },
        }
    );
    assert_eq!(
        body.data.expenses.get(0).unwrap(),
        &ExpenseData {
            id: exp3,
            description: "expense3".to_string(),
            amount: 12.0,
            user: UserData {
                id: user_2.id,
                name: String::from("r2"),
            },
        }
    );
    Ok(())
}

#[test_context(TestApp)]
#[tokio::test]
async fn get_expenses_for_settlement_return_200_and_expenses_of_settlement(
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
    let exp1 = app
        .create_expense(&group.id, cookie_1, "expense1", 20.0)
        .await?;
    let exp2 = app
        .create_expense(&group.id, cookie_2, "expense2", 15.0)
        .await?;
    let exp3 = app
        .create_expense(&group.id, cookie_2, "expense3", 12.0)
        .await?;
    let stl_id = app.settle(&group).await?.id;
    let _ = app
        .create_expense(&group.id, cookie_3, "expense4", 12.0)
        .await?;
    // Act
    let response = app
        .client
        .get(&format!(
            "{}/groups/{}/expenses?settlement_id={}",
            &app.address, &group.id, stl_id,
        ))
        .header(header::COOKIE, cookie_adm)
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 200);

    let body = response.json::<ExpensesResponse>().await?;
    assert_eq!(body.success, true);
    assert_eq!(body.data.expenses.len(), 3);
    assert_eq!(
        body.data.expenses.get(2).unwrap(),
        &ExpenseData {
            id: exp1,
            description: "expense1".to_string(),
            amount: 20.0,
            user: UserData {
                id: user_1.id,
                name: String::from("r1"),
            },
        }
    );
    assert_eq!(
        body.data.expenses.get(1).unwrap(),
        &ExpenseData {
            id: exp2,
            description: "expense2".to_string(),
            amount: 15.0,
            user: UserData {
                id: user_2.id,
                name: String::from("r2"),
            },
        }
    );
    assert_eq!(
        body.data.expenses.get(0).unwrap(),
        &ExpenseData {
            id: exp3,
            description: "expense3".to_string(),
            amount: 12.0,
            user: UserData {
                id: user_2.id,
                name: String::from("r2"),
            },
        }
    );
    Ok(())
}

#[test_context(TestApp)]
#[tokio::test]
async fn get_expenses_for_settlement_return_400_when_settlement_id_is_invalid(
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
        .create_expense(&group.id, cookie_1, "expense1", 20.0)
        .await?;
    let _ = app
        .create_expense(&group.id, cookie_2, "expense2", 15.0)
        .await?;
    let _ = app
        .create_expense(&group.id, cookie_2, "expense3", 12.0)
        .await?;
    let _ = app.settle(&group).await?;
    let _ = app
        .create_expense(&group.id, cookie_3, "expense4", 12.0)
        .await?;
    // Act
    let response = app
        .client
        .get(&format!(
            "{}/groups/{}/expenses?settlement_id={}",
            &app.address, &group.id, "bob",
        ))
        .header(header::COOKIE, cookie_adm)
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 400);
    Ok(())
}

#[test_context(TestApp)]
#[tokio::test]
async fn get_expenses_for_settlement_return_404_when_settlement_does_not_exist(
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
        .create_expense(&group.id, cookie_1, "expense1", 20.0)
        .await?;
    let _ = app
        .create_expense(&group.id, cookie_2, "expense2", 15.0)
        .await?;
    let _ = app
        .create_expense(&group.id, cookie_2, "expense3", 12.0)
        .await?;
    let _ = app.settle(&group).await?;
    let _ = app
        .create_expense(&group.id, cookie_3, "expense4", 12.0)
        .await?;
    // Act
    let response = app
        .client
        .get(&format!(
            "{}/groups/{}/expenses?settlement_id={}",
            &app.address, &group.id, "e6f9b275-3df9-4012-9fbe-47826275bc30",
        ))
        .header(header::COOKIE, cookie_adm)
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 404);
    Ok(())
}

#[derive(serde::Deserialize)]
pub struct ExpensesResponse {
    pub success: bool,
    pub data: ExpensesData,
}

#[derive(serde::Deserialize)]
pub struct ExpensesData {
    pub expenses: Vec<ExpenseData>,
}

#[derive(serde::Deserialize, PartialEq, Debug)]
pub struct ExpenseData {
    pub id: Uuid,
    pub description: String,
    pub amount: f32,
    pub user: UserData,
}
