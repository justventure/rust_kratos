use crate::application::commands::CommandHandler;
use crate::application::commands::identity::register::RegisterCommand;
use crate::domain::errors::{AuthError, DomainError};
use crate::domain::ports::inbound::registration::RegistrationData;
use crate::infrastructure::adapters::http::cookies::RequestResponseCookies;
use crate::infrastructure::di::container::UseCases;
use crate::presentation::api::rest::v1::dto::auth::RegisterDto;
use crate::presentation::api::rest::v1::handlers::utils::extract_cookie;
use crate::presentation::api::rest::v1::schema::auth::RegisterSchema;
use actix_web::{HttpMessage, HttpRequest, HttpResponse, web};
use std::sync::Arc;

#[utoipa::path(
    post,
    path = "/api/v1/auth/register",
    tag = "auth",
    request_body = RegisterSchema,
    responses(
        (status = 201, description = "Registered"),
        (status = 400, description = "Invalid data"),
        (status = 409, description = "Already logged in or email exists"),
    )
)]
pub async fn register(
    req: HttpRequest,
    use_cases: web::Data<Arc<UseCases>>,
    dto: web::Json<RegisterDto>,
) -> HttpResponse {
    let data: RegistrationData = match dto.into_inner().try_into() {
        Ok(d) => d,
        Err(e) => return HttpResponse::BadRequest().body(e.to_string()),
    };
    match use_cases
        .commands
        .register
        .handle(RegisterCommand {
            data,
            cookie: extract_cookie(&req),
        })
        .await
    {
        Ok(result) => {
            req.extensions_mut()
                .get_mut::<RequestResponseCookies>()
                .unwrap()
                .add(result.session_cookie);
            HttpResponse::Created().finish()
        }
        Err(DomainError::Auth(AuthError::AlreadyLoggedIn)) => {
            HttpResponse::Conflict().body("Already logged in")
        }
        Err(DomainError::Conflict(msg)) => HttpResponse::Conflict().body(msg),
        Err(DomainError::InvalidData(msg)) => HttpResponse::BadRequest().body(msg),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}
