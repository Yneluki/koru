use crate::test_app::{SettlementResponse, TestApp, TransactionData, UserData};
use claim::{assert_none, assert_some};
use reqwest::header;
use test_context::test_context;

#[test_context(TestApp)]
#[tokio::test]
async fn settle_returns_201_when_user_is_admin(app: &TestApp) -> anyhow::Result<()> {
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
        .create_expense(&group.id, cookie_1, "expense", 10.0)
        .await?;
    let exp2 = app
        .create_expense(&group.id, cookie_1, "expense", 10.0)
        .await?;
    let exp3 = app
        .create_expense(&group.id, cookie_2, "expense", 5.0)
        .await?;
    let exp4 = app
        .create_expense(&group.id, cookie_2, "expense", 2.5)
        .await?;
    let exp5 = app
        .create_expense(&group.id, cookie_2, "expense", 2.5)
        .await?;
    let exp6 = app
        .create_expense(&group.id, cookie_3, "expense", 50.0)
        .await?;
    let exp7 = app
        .create_expense(&group.id, cookie_3, "expense", 15.0)
        .await?;
    let exp8 = app
        .create_expense(&group.id, cookie_3, "expense", 35.0)
        .await?;
    let exp9 = app
        .create_expense(&group.id, cookie_adm, "expense", 70.0)
        .await?;
    let exp10 = app
        .create_expense(&group.id, cookie_adm, "expense", 50.0)
        .await?;
    let exp_ids = vec![exp1, exp2, exp3, exp4, exp5, exp6, exp7, exp8, exp9, exp10];
    // Act
    let response = app
        .client
        .post(&format!(
            "{}/groups/{}/settlements",
            &app.address, &group.id
        ))
        .header(header::COOKIE, cookie_adm)
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 201);
    let saved = assert_some!(app.get_settlement().await);
    assert_eq!(saved.group_id, group.id);
    let expenses = app.get_expenses_status(&exp_ids).await;
    assert_eq!(expenses.len(), exp_ids.len());
    assert!(expenses.iter().all(|r| r.1)); // settled
    let settled_expenses = app.settled_expenses().await;
    assert_eq!(settled_expenses.len(), exp_ids.len());
    assert!(settled_expenses
        .iter()
        .all(|r| r.0 == saved.id && exp_ids.contains(&r.1)));
    let transactions = app.get_transactions().await;
    assert_eq!(transactions.len(), 3);

    assert_eq!(transactions.get(0).unwrap().settlement_id, saved.id);
    assert_eq!(transactions.get(0).unwrap().from_user_id, user_1.id);
    assert_eq!(transactions.get(0).unwrap().to_user_id, group.admin.id);
    assert_eq!(transactions.get(0).unwrap().amount, 5.0);

    assert_eq!(transactions.get(1).unwrap().settlement_id, saved.id);
    assert_eq!(transactions.get(1).unwrap().from_user_id, user_1.id);
    assert_eq!(transactions.get(1).unwrap().to_user_id, user_3.id);
    assert_eq!(transactions.get(1).unwrap().amount, 37.5);

    assert_eq!(transactions.get(2).unwrap().settlement_id, saved.id);
    assert_eq!(transactions.get(2).unwrap().from_user_id, user_2.id);
    assert_eq!(transactions.get(2).unwrap().to_user_id, group.admin.id);
    assert_eq!(transactions.get(2).unwrap().amount, 52.5);

    let body = response.json::<SettlementResponse>().await?;
    assert_eq!(body.success, true);
    assert_eq!(body.data.id, saved.id);
    assert_eq!(body.data.transactions.len(), 3);
    assert_eq!(
        body.data.transactions.get(0).unwrap(),
        &TransactionData {
            from: UserData {
                id: user_2.id,
                name: String::from("r2")
            },
            to: UserData {
                id: group.admin.id,
                name: String::from("rbiland")
            },
            amount: 52.5
        }
    );
    assert_eq!(
        body.data.transactions.get(1).unwrap(),
        &TransactionData {
            from: UserData {
                id: user_1.id,
                name: String::from("r1")
            },
            to: UserData {
                id: user_3.id,
                name: String::from("r3")
            },
            amount: 37.5
        }
    );
    assert_eq!(
        body.data.transactions.get(2).unwrap(),
        &TransactionData {
            from: UserData {
                id: user_1.id,
                name: String::from("r1")
            },
            to: UserData {
                id: group.admin.id,
                name: String::from("rbiland")
            },
            amount: 5.0
        }
    );
    assert_eq!(app.get_event_type().await, Some("Settled".to_string()));
    Ok(())
}

#[test_context(TestApp)]
#[tokio::test]
async fn settle_returns_403_when_user_is_not_admin(app: &TestApp) -> anyhow::Result<()> {
    // Arrange
    let group = app
        .create_user_and_group("rbiland", "r@r.com", "201", "my group")
        .await?;
    let user_1 = app
        .create_user_and_login_and_device("r1", "r1@r.com", "201")
        .await?;
    app.join_group(&group, user_1.cookie.as_str()).await?;
    let exp1 = app
        .create_expense(&group.id, user_1.cookie.as_str(), "expense", 10.0)
        .await?;
    // Act
    let response = app
        .client
        .post(&format!(
            "{}/groups/{}/settlements",
            &app.address, &group.id
        ))
        .header(header::COOKIE, user_1.cookie.as_str())
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 403);
    assert_none!(app.get_settlement().await);
    let expenses = app.get_expenses_status(&[exp1]).await;
    assert_eq!(expenses.len(), 1);
    assert!(expenses.iter().all(|r| !r.1)); // not settled
    let transactions = app.get_transactions().await;
    assert_eq!(transactions.len(), 0);
    match app.get_event_type().await {
        None => {}
        Some(event_type) => assert_ne!(event_type, "Settled".to_string()),
    }
    Ok(())
}

#[test_context(TestApp)]
#[tokio::test]
async fn settle_returns_401_when_user_is_not_logged_in(app: &TestApp) -> anyhow::Result<()> {
    // Arrange
    let group = app
        .create_user_and_group("rbiland", "r@r.com", "201", "my group")
        .await?;
    let user_1 = app
        .create_user_and_login_and_device("r1", "r1@r.com", "201")
        .await?;
    app.join_group(&group, user_1.cookie.as_str()).await?;
    let exp1 = app
        .create_expense(&group.id, user_1.cookie.as_str(), "expense", 10.0)
        .await?;
    // Act
    let response = app
        .client
        .post(&format!(
            "{}/groups/{}/settlements",
            &app.address, &group.id
        ))
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 401);
    assert_none!(app.get_settlement().await);
    let expenses = app.get_expenses_status(&[exp1]).await;
    assert_eq!(expenses.len(), 1);
    assert!(expenses.iter().all(|r| !r.1)); // not settled
    let transactions = app.get_transactions().await;
    assert_eq!(transactions.len(), 0);
    match app.get_event_type().await {
        None => {}
        Some(event_type) => assert_ne!(event_type, "Settled".to_string()),
    }
    Ok(())
}
