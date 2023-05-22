use crate::test_app::{ColorDto, EventKindDto, Notification, PushyInfo, PushyOkResponse, TestApp};
use chrono::Utc;
use claim::assert_some;
use test_context::test_context;
use uuid::Uuid;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

#[test_context(TestApp)]
#[tokio::test]
async fn on_member_joined_sends_notification_to_other_members(app: &TestApp) -> anyhow::Result<()> {
    // setup wiremock
    let _guard = Mock::given(path("/push"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(PushyOkResponse {
            id: Uuid::new_v4().to_string(),
            success: true,
            info: PushyInfo { devices: 1 },
        }))
        .named("Push notification")
        .expect(2)
        .mount_as_scoped(app.notification_server())
        .await;
    // setup database
    let admin = app
        .with_user(
            "admin".to_string(),
            "r@r.com".to_string(),
            "admin-device".to_string(),
        )
        .await;
    let member = app
        .with_user(
            "member".to_string(),
            "r@r1.com".to_string(),
            "member-device".to_string(),
        )
        .await;
    let new_member = app
        .with_user(
            "new member".to_string(),
            "r@r2.com".to_string(),
            "new-member-device".to_string(),
        )
        .await;
    let group = app.with_group("my group".to_string(), admin).await;
    app.with_member(group, member).await;
    app.with_member(group, new_member).await;

    // launch event
    let event = app
        .with_event(EventKindDto::MemberJoined {
            group_id: group,
            member_id: new_member,
            color: ColorDto {
                red: 0,
                green: 255,
                blue: 0,
            },
        })
        .await;
    app.publish_event(event).await;

    // check
    let requests = app
        .notification_server()
        .received_requests()
        .await
        .expect("Pushy to be called");
    let notifications: Vec<Notification> = requests
        .into_iter()
        .map(|r| r.body_json::<Notification>().unwrap())
        .collect();
    assert_eq!(notifications.len(), 2);
    assert_some!(notifications.iter().find(|n| n.to == "admin-device"));
    assert_some!(notifications.iter().find(|n| n.to == "member-device"));
    let expected_message = "new member joined group my group|r@r2.com".to_string();
    for notif in notifications {
        assert_eq!(notif.data.message, expected_message);
    }
    assert_some!(app.get_event_process_date(event).await);
    Ok(())
}

#[test_context(TestApp)]
#[tokio::test]
async fn on_expense_created_sends_notification_to_other_members(
    app: &TestApp,
) -> anyhow::Result<()> {
    // setup wiremock
    let _guard = Mock::given(path("/push"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(PushyOkResponse {
            id: Uuid::new_v4().to_string(),
            success: true,
            info: PushyInfo { devices: 1 },
        }))
        .named("Push notification")
        .expect(2)
        .mount_as_scoped(app.notification_server())
        .await;
    // setup database
    let admin = app
        .with_user(
            "admin".to_string(),
            "r@r.com".to_string(),
            "admin-device".to_string(),
        )
        .await;
    let member_1 = app
        .with_user(
            "member 1".to_string(),
            "r@r1.com".to_string(),
            "member-1-device".to_string(),
        )
        .await;
    let member_2 = app
        .with_user(
            "member 2".to_string(),
            "r@r2.com".to_string(),
            "member-2-device".to_string(),
        )
        .await;
    let group = app.with_group("my group".to_string(), admin).await;
    app.with_member(group, member_1).await;
    app.with_member(group, member_2).await;
    let expense = app
        .with_expense(group, member_2, "stuff".to_string(), 12.0)
        .await;

    // launch event
    let event = app
        .with_event(EventKindDto::ExpenseCreated {
            id: expense,
            group_id: group,
            member_id: member_2,
            description: "stuff".to_string(),
            amount: 12.0,
            date: Utc::now(),
        })
        .await;
    app.publish_event(event).await;

    // check
    let requests = app
        .notification_server()
        .received_requests()
        .await
        .expect("Pushy to be called");
    for r in requests.iter() {
        assert_eq!(r.url.query(), Some("api_key=YOUR_TOKEN"));
    }
    let notifications: Vec<Notification> = requests
        .into_iter()
        .map(|r| r.body_json::<Notification>().unwrap())
        .collect();
    assert_eq!(notifications.len(), 2);
    assert_some!(notifications.iter().find(|n| n.to == "admin-device"));
    assert_some!(notifications.iter().find(|n| n.to == "member-1-device"));
    let expected_message = "Expense from member 2 in my group|stuff: 12".to_string();
    for notif in notifications {
        assert_eq!(notif.data.message, expected_message);
    }
    assert_some!(app.get_event_process_date(event).await);
    Ok(())
}
