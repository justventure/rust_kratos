use std::sync::Arc;

use actix_web::{HttpMessage, HttpRequest, HttpResponse, web};
use tracing::instrument;

use crate::application::commands::CommandHandler;
use crate::application::commands::auth::login::LoginCommand;
use crate::domain::errors::{AuthError, DomainError};
use crate::domain::ports::inbound::login::LoginCredentials;
use crate::infrastructure::adapters::http::cookies::RequestResponseCookies;
use crate::infrastructure::di::container::UseCases;
use crate::presentation::api::rest::v1::dto::auth::{LoginDto, UserProfileResponse};
use crate::presentation::api::rest::v1::handlers::utils::extract_cookie;
use crate::presentation::api::rest::v1::schema::auth::{LoginSchema, UserProfileResponseSchema};

#[utoipa::path(
    post,
    path = "/api/v1/auth/login",
    tag = "auth",
    request_body = LoginSchema,
    responses(
        (status = 200, description = "Logged in", body = UserProfileResponseSchema),
        (status = 400, description = "Invalid data"),
        (status = 401, description = "Invalid credentials"),
        (status = 409, description = "Already logged in"),
    )
)]
#[instrument(skip_all, name = "http.login")]
pub async fn login(req: HttpRequest, use_cases: web::Data<Arc<UseCases>>, dto: web::Json<LoginDto>) -> HttpResponse {
    let credentials: LoginCredentials = match dto.into_inner().try_into() {
        Ok(c) => c,
        Err(e) => return HttpResponse::BadRequest().body(e.to_string()),
    };
    let command = LoginCommand {
        credentials,
        cookie: extract_cookie(&req),
    };
    match use_cases.commands.login.handle(command).await {
        Ok(result) => {
            req.extensions_mut()
                .get_mut::<RequestResponseCookies>()
                .unwrap()
                .add(result.session_cookie);
            HttpResponse::Ok().json(UserProfileResponse::from(result.user))
        }
        Err(DomainError::Auth(AuthError::AlreadyLoggedIn)) => HttpResponse::Conflict().body("Already logged in"),
        Err(DomainError::Auth(AuthError::InvalidCredentials)) => {
            HttpResponse::Unauthorized().body("Invalid credentials")
        }
        Err(DomainError::Auth(AuthError::NotAuthenticated)) => HttpResponse::Unauthorized().body("Not authenticated"),
        Err(DomainError::InvalidData(msg)) => HttpResponse::BadRequest().body(msg),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}
