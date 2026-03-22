use crate::application::commands::CommandHandler;
use crate::application::commands::account::settings::{
    UpdateSettingsCommand, UpdateSettingsResult,
};
use crate::domain::errors::{AuthError, DomainError};
use crate::domain::ports::inbound::settings::SettingsData;
use crate::infrastructure::adapters::http::cookies::RequestResponseCookies;
use crate::infrastructure::di::container::UseCases;
use crate::presentation::api::rest::v1::dto::auth::UpdateSettingsDto;
use crate::presentation::api::rest::v1::handlers::utils::extract_cookie;
use crate::presentation::api::rest::v1::schema::auth::UpdateSettingsSchema;
use actix_web::{HttpMessage, HttpRequest, HttpResponse, web};
use std::sync::Arc;

#[utoipa::path(
    put,
    path = "/api/v1/auth/settings",
    tag = "auth",
    request_body = UpdateSettingsSchema,
    responses(
        (status = 200, description = "Settings updated"),
        (status = 400, description = "Invalid data"),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Privileged session required"),
    )
)]
pub async fn update_settings(
    req: HttpRequest,
    use_cases: web::Data<Arc<UseCases>>,
    dto: web::Json<UpdateSettingsDto>,
) -> HttpResponse {
    let data: SettingsData = match dto.into_inner().try_into() {
        Ok(d) => d,
        Err(e) => return HttpResponse::BadRequest().body(e.to_string()),
    };
    let command = UpdateSettingsCommand {
        data,
        cookie: extract_cookie(&req).unwrap_or_default(),
    };
    match use_cases.commands.update_settings.handle(command).await {
        Ok(UpdateSettingsResult { flow_id, messages }) => {
            let mut ext = req.extensions_mut();
            let response_cookies = ext.get_mut::<RequestResponseCookies>().unwrap();
            for message in messages {
                response_cookies.add(message);
            }
            HttpResponse::Ok().json(serde_json::json!({ "flow_id": flow_id }))
        }
        Err(DomainError::Auth(AuthError::NotAuthenticated)) => {
            HttpResponse::Unauthorized().body("Not authenticated")
        }
        Err(DomainError::Auth(AuthError::PrivilegedSessionRequired)) => {
            HttpResponse::Forbidden().body("Privileged session required")
        }
        Err(DomainError::InvalidData(msg)) => HttpResponse::BadRequest().body(msg),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}
