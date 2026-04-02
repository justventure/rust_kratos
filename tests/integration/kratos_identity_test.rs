use std::sync::Arc;

use rust_kratos::domain::ports::outbound::identity::IdentityPort;
use rust_kratos::domain::ports::outbound::session::SessionPort;
use rust_kratos::infrastructure::adapters::kratos::http::identity::KratosIdentityAdapter;
use rust_kratos::infrastructure::adapters::kratos::http::logout::KratosSessionAdapter;

use crate::common::TestContext;

#[tokio::test]
async fn test_get_current_user_without_cookie_returns_error() {
    let ctx = TestContext::new();
    let adapter = KratosIdentityAdapter::new(ctx.client.clone(), None, 0);
    let result = adapter.get_current_user("ory_kratos_session=invalid").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_get_current_user_after_login_returns_profile() {
    let ctx = TestContext::new();
    let email = TestContext::random_email();
    let password = "Test1234!@#$";
    let username = format!("user_{}", uuid::Uuid::new_v4());

    let session_token = register_and_login(&ctx, &email, &username, password).await;
    let cookie = format!("ory_kratos_session={}", session_token);

    let adapter = KratosIdentityAdapter::new(ctx.client.clone(), None, 0);
    let result = adapter.get_current_user(&cookie).await;

    assert!(result.is_ok());
    let profile = result.unwrap();
    assert_eq!(profile.traits.email, email);
    assert_eq!(profile.traits.username.unwrap(), username);
}

#[tokio::test]
async fn test_get_current_user_returns_email_and_username() {
    let ctx = TestContext::new();
    let email = TestContext::random_email();
    let password = "Test1234!@#$";
    let username = format!("user_{}", uuid::Uuid::new_v4());

    let session_token = register_and_login(&ctx, &email, &username, password).await;
    let cookie = format!("ory_kratos_session={}", session_token);

    let adapter = KratosIdentityAdapter::new(ctx.client.clone(), None, 0);
    let profile = adapter.get_current_user(&cookie).await.unwrap();

    assert!(!profile.traits.email.is_empty());
    assert!(profile.traits.username.is_some());
}

async fn register_and_login(ctx: &TestContext, email: &str, username: &str, password: &str) -> String {
    use rust_kratos::domain::ports::inbound::login::{AuthenticationPort, LoginCredentials};
    use rust_kratos::domain::value_objects::auth_method::AuthMethod;
    use rust_kratos::domain::value_objects::email::Email;
    use rust_kratos::domain::value_objects::password::Password;
    use rust_kratos::infrastructure::adapters::kratos::http::flows::{fetch_flow, post_flow};
    use rust_kratos::infrastructure::adapters::kratos::http::login::KratosAuthenticationAdapter;

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

    let flow = fetch_flow(&ctx.client.client, &ctx.client.public_url, "registration", None)
        .await
        .unwrap();

    let payload = RegistrationPayload {
        method: AuthMethod::Password,
        password: password.to_string(),
        traits: RegistrationTraits {
            email: email.to_string(),
            username: username.to_string(),
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

    let session: Arc<dyn SessionPort> = Arc::new(KratosSessionAdapter::new(ctx.client.clone(), None));
    let adapter = KratosAuthenticationAdapter::new(ctx.client.clone(), session);
    let login_flow = adapter.initiate_login(None).await.unwrap();

    let credentials = LoginCredentials {
        identifier: Email::new(email).unwrap(),
        password: Password::new(password).unwrap(),
        address: None,
        code: None,
        resend: None,
    };

    let result = adapter.complete_login(login_flow, credentials).await.unwrap();

    result
        .session_cookie
        .split(';')
        .next()
        .and_then(|s| s.trim().strip_prefix("ory_kratos_session="))
        .unwrap_or(&result.session_cookie)
        .to_string()
}
