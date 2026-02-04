use actix_web::{web, HttpResponse};

#[derive(serde::Deserialize)]
pub struct SubscriptionFormData {
    email: String,
    name: String,
}

pub async fn subscribe(_form: web::Form<SubscriptionFormData>) -> HttpResponse {
    HttpResponse::Ok().finish()
}
