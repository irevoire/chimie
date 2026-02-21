use actix_web::{HttpRequest, HttpResponse, http::header::ContentType, web};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("", web::get().to(notifications));
}

pub async fn notifications(_req: HttpRequest) -> HttpResponse {
    HttpResponse::Ok()
        .content_type(ContentType::json())
        .body("[]")
}
