use std::sync::Arc;

use rust_kratos::domain::ports::inbound::registration::{RegistrationData, RegistrationPort};
use rust_kratos::domain::value_objects::email::Email;
use rust_kratos::domain::value_objects::password::Password;
use rust_kratos::infrastructure::adapters::kratos::http::logout::KratosSessionAdapter;
use rust_kratos::infrastructure::adapters::kratos::http::register::KratosRegistrationAdapter;

#[path = "../common/mod.rs"]
mod common;
use common::TestContext;

fn make_adapter(ctx: &TestContext) -> KratosRegistrationAdapter {
    let session = Arc::new(KratosSessionAdapter::new(ctx.client.clone(), None));
    KratosRegistrationAdapter::new(ctx.client.clone(), session)
}

#[tokio::test]
async fn test_initiate_registration_returns_flow_id() {
    let ctx = TestContext::new();
    let adapter = make_adapter(&ctx);
    let result = adapter.initiate_registration(None).await;
    assert!(result.is_ok());
    assert!(!result.unwrap().flow_id.is_empty());
}

#[tokio::test]
async fn test_complete_registration_returns_session_cookie() {
    let ctx = TestContext::new();
    let adapter = make_adapter(&ctx);
    let flow = adapter.initiate_registration(None).await.unwrap();
    let data = RegistrationData {
        email: Email::new(&TestContext::random_email()).unwrap(),
        username: format!("user_{}", uuid::Uuid::new_v4()),
        password: Password::new("Test1234!@#$").unwrap(),
        geo_location: None,
    };
    let result = adapter.complete_registration(flow, data).await;
    assert!(result.is_ok());
    assert!(result.unwrap().session_cookie.contains("ory_kratos_session"));
}

#[tokio::test]
async fn test_complete_registration_with_duplicate_email_fails() {
    let ctx = TestContext::new();
    let adapter = make_adapter(&ctx);
    let email = TestContext::random_email();

    let flow = adapter.initiate_registration(None).await.unwrap();
    adapter
        .complete_registration(
            flow,
            RegistrationData {
                email: Email::new(&email).unwrap(),
                username: format!("user_{}", uuid::Uuid::new_v4()),
                password: Password::new("Test1234!@#$").unwrap(),
                geo_location: None,
            },
        )
        .await
        .unwrap();

    let flow = adapter.initiate_registration(None).await.unwrap();
    let result = adapter
        .complete_registration(
            flow,
            RegistrationData {
                email: Email::new(&email).unwrap(),
                username: format!("user_{}", uuid::Uuid::new_v4()),
                password: Password::new("Test1234!@#$").unwrap(),
                geo_location: None,
            },
        )
        .await;
    assert!(result.is_err());
}
