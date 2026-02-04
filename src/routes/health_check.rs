use actix_web::{HttpRequest, HttpResponse};

pub async fn health_check(_reg: HttpRequest) -> HttpResponse {
    HttpResponse::Ok().finish()
}
