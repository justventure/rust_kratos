use actix_web::{HttpResponse, Responder, get, web};

#[get("/health")]
async fn health() -> impl Responder {
    HttpResponse::Ok().body("ok")
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(health);
}
