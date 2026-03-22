use std::sync::Arc;

use actix_web::{HttpRequest, HttpResponse, web};

use crate::application::queries::QueryHandler;
use crate::application::queries::get_current_user::GetCurrentUserQuery;
use crate::domain::errors::{AuthError, DomainError};
use crate::infrastructure::di::container::UseCases;
use crate::presentation::api::rest::v1::dto::auth::UserProfileResponse;
use crate::presentation::api::rest::v1::handlers::utils::extract_cookie;
use crate::presentation::api::rest::v1::schema::auth::UserProfileSchema;

#[utoipa::path(
    get,
    path = "/api/v1/auth/me",
    tag = "auth",
    responses(
        (status = 200, body = UserProfileSchema),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 500, description = "Internal server error"),
    )
)]
pub async fn current_user(req: HttpRequest, use_cases: web::Data<Arc<UseCases>>) -> HttpResponse {
    let query = GetCurrentUserQuery {
        cookie: extract_cookie(&req),
    };
    match use_cases.queries.get_current_user.handle(query).await {
        Ok(profile) => HttpResponse::Ok().json(UserProfileResponse::from(profile)),
        Err(DomainError::Auth(AuthError::NotAuthenticated)) => HttpResponse::Unauthorized().body("Not authenticated"),
        Err(DomainError::Auth(AuthError::SessionExpired)) => HttpResponse::Unauthorized().body("Session expired"),
        Err(DomainError::Auth(AuthError::Forbidden)) => HttpResponse::Forbidden().body("Forbidden"),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}
