use rust_kratos::domain::ports::inbound::login::{AuthenticationPort, LoginCredentials};
use rust_kratos::domain::ports::outbound::session::SessionPort;
use rust_kratos::domain::value_objects::email::Email;
use rust_kratos::domain::value_objects::password::Password;
use rust_kratos::infrastructure::adapters::kratos::http::login::KratosAuthenticationAdapter;
use rust_kratos::infrastructure::adapters::kratos::http::logout::KratosSessionAdapter;
use std::sync::Arc;

#[path = "../common/mod.rs"]
mod common;
use common::TestContext;

fn make_auth_adapter(ctx: &TestContext) -> KratosAuthenticationAdapter {
    let session: Arc<dyn SessionPort> =
        Arc::new(KratosSessionAdapter::new(ctx.client.clone(), None));
    KratosAuthenticationAdapter::new(ctx.client.clone(), session)
}

#[tokio::test]
async fn test_initiate_login_returns_flow_id() {
    let ctx = TestContext::new();
    let adapter = make_auth_adapter(&ctx);
    let result = adapter.initiate_login(None).await;
    assert!(result.is_ok());
    let flow_id = result.unwrap();
    assert!(!flow_id.is_empty());
}

#[tokio::test]
async fn test_complete_login_with_invalid_credentials() {
    let ctx = TestContext::new();
    let adapter = make_auth_adapter(&ctx);
    let flow_id = adapter.initiate_login(None).await.unwrap();
    let credentials = LoginCredentials {
        identifier: Email::new("nonexistent@example.com").unwrap(),
        password: Password::new("wrongpassword").unwrap(),
        address: None,
        code: None,
        resend: None,
    };
    let result = adapter.complete_login(&flow_id, credentials).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_register_then_login() {
    let ctx = TestContext::new();
    let email = TestContext::random_email();
    let password = "Test1234!@#$";
    register_user(&ctx, &email, password).await;
    let adapter = make_auth_adapter(&ctx);
    let flow_id = adapter.initiate_login(None).await.unwrap();
    let credentials = LoginCredentials {
        identifier: Email::new(&email).unwrap(),
        password: Password::new(password).unwrap(),
        address: None,
        code: None,
        resend: None,
    };
    let result = adapter.complete_login(&flow_id, credentials).await;
    assert!(result.is_ok());
    let cookie = result.unwrap();
    assert!(!cookie.is_empty());
}

#[tokio::test]
async fn test_initiate_login_with_active_session_returns_error() {
    let ctx = TestContext::new();
    let email = TestContext::random_email();
    let password = "Test1234!@#$";
    let session_cookie = register_and_login(&ctx, &email, password).await;
    let adapter = make_auth_adapter(&ctx);
    let result = adapter.initiate_login(Some(&session_cookie)).await;
    assert!(result.is_err());
}

async fn register_user(ctx: &TestContext, email: &str, password: &str) {
    use rust_kratos::domain::value_objects::auth_method::AuthMethod;
    use rust_kratos::infrastructure::adapters::kratos::http::flows::{fetch_flow, post_flow};

    #[derive(serde::Serialize)]
    struct RegistrationTraits {
        email: String,
        username: String,
    }

    #[derive(serde::Serialize)]
    struct RegistrationPayload {
        method: AuthMethod,
        password: String,
        traits: RegistrationTraits,
        csrf_token: String,
    }

    let flow = fetch_flow(
        &ctx.client.client,
        &ctx.client.public_url,
        "registration",
        None,
    )
    .await
    .unwrap();

    let payload = RegistrationPayload {
        method: AuthMethod::Password,
        password: password.to_string(),
        traits: RegistrationTraits {
            email: email.to_string(),
            username: format!("user_{}", uuid::Uuid::new_v4()),
        },
        csrf_token: flow.csrf_token.clone(),
    };

    post_flow(
        &ctx.client.client,
        &ctx.client.public_url,
        "registration",
        &flow.flow_id,
        serde_json::to_value(payload).unwrap(),
        &flow.cookies,
    )
    .await
    .unwrap();
}

async fn register_and_login(ctx: &TestContext, email: &str, password: &str) -> String {
    register_user(ctx, email, password).await;
    let adapter = make_auth_adapter(ctx);
    let flow_id = adapter.initiate_login(None).await.unwrap();
    let credentials = LoginCredentials {
        identifier: Email::new(email).unwrap(),
        password: Password::new(password).unwrap(),
        address: None,
        code: None,
        resend: None,
    };
    adapter.complete_login(&flow_id, credentials).await.unwrap()
}
