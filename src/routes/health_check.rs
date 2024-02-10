use actix_web::HttpResponse;

// health check
pub async fn health_check() -> HttpResponse {
    HttpResponse::Ok().finish()
}
