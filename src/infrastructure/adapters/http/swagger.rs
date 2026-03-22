use crate::presentation::api::rest::v1::schema::auth::{
    LoginSchema, RecoverySchema, RegisterSchema, SendVerificationCodeSchema,
    SubmitVerificationCodeSchema, UpdateSettingsSchema, UserProfileSchema, VerifyByLinkSchema,
};
use actix_web::{HttpResponse, Responder, get, web};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::presentation::api::rest::v1::handlers::register::register,
        crate::presentation::api::rest::v1::handlers::login::login,
        crate::presentation::api::rest::v1::handlers::logout::logout,
        crate::presentation::api::rest::v1::handlers::current_user::current_user,
        crate::presentation::api::rest::v1::handlers::recovery::recovery,
        crate::presentation::api::rest::v1::handlers::settings::update_settings,
        crate::presentation::api::rest::v1::handlers::verification::verify_by_link,
        crate::presentation::api::rest::v1::handlers::verification::send_verification_code,
        crate::presentation::api::rest::v1::handlers::verification::submit_verification_code,
    ),
    components(schemas(
        RegisterSchema,
        LoginSchema,
        RecoverySchema,
        UpdateSettingsSchema,
        VerifyByLinkSchema,
        SendVerificationCodeSchema,
        SubmitVerificationCodeSchema,
        UserProfileSchema,
    )),
    tags(
        (name = "auth", description = "Authentication endpoints")
    )
)]
pub struct ApiDoc;

#[get("/api-docs/openapi.json")]
async fn openapi_json() -> impl Responder {
    HttpResponse::Ok()
        .content_type("application/json")
        .body(ApiDoc::openapi().to_json().unwrap())
}

#[get("/docs")]
async fn swagger_ui_html() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("swagger-dark.html"))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-docs/openapi.json", ApiDoc::openapi()),
    );
    cfg.service(openapi_json);
    cfg.service(swagger_ui_html);
}
