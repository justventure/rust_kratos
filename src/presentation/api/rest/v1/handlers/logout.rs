use crate::application::commands::CommandHandler;
use crate::application::commands::auth::logout::LogoutCommand;
use crate::domain::errors::{AuthError, DomainError};
use crate::infrastructure::di::container::UseCases;
use crate::presentation::api::rest::v1::handlers::utils::extract_cookie;
use actix_web::{HttpRequest, HttpResponse, web};
use std::sync::Arc;

#[utoipa::path(
    get,
    path = "/api/v1/auth/logout",
    tag = "auth",
    responses(
        (status = 204, description = "Logged out"),
        (status = 401, description = "Not authenticated"),
    )
)]
pub async fn logout(req: HttpRequest, use_cases: web::Data<Arc<UseCases>>) -> HttpResponse {
    let command = LogoutCommand {
        cookie: extract_cookie(&req),
    };
    match use_cases.commands.logout.handle(command).await {
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(DomainError::Auth(AuthError::NotAuthenticated)) => {
            HttpResponse::Unauthorized().body("Not authenticated")
        }
        Err(DomainError::Auth(AuthError::SessionExpired)) => {
            HttpResponse::Unauthorized().body("Session expired")
        }
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}
