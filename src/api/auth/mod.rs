use actix_web::{HttpRequest, HttpResponse, http::header::ContentType, web};
use facet_actix::Json;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("login", web::post().to(login));
}

#[derive(facet::Facet)]
#[facet(rename_all = "camelCase", deny_unknown_fields)]
pub struct LoginRequest {
    email: String,
    #[facet(sensitive)]
    password: String,
}

#[derive(facet::Facet)]
#[facet(rename_all = "camelCase", deny_unknown_fields)]
pub struct LoginResponse {
    #[facet(sensitive)]
    access_token: String,
    user_id: String,
    user_email: String,
    name: String,
    is_admin: bool,
    profile_image_path: String,
    should_change_password: bool,
    is_onboarded: bool,
}

pub async fn login(_req: HttpRequest, login: Json<LoginRequest>) -> HttpResponse {
    if login.email != "demo@immich.app" && login.password != "demo" {
        return HttpResponse::Unauthorized()
            .content_type(ContentType::json())
            .body("{ \"error\": \"Only user with email demo@immich.app and password demo is authorized\"}");
    }
    let ret = LoginResponse {
        access_token: String::from("F3dVtaMX4ET2i6Uugs98kQEMhQaaUrU7UOsw1QtWM"),
        user_id: String::from("6bbe2767-7851-461a-aa2d-afbd3460aa85"),
        user_email: String::from("demo@immich.app"),
        name: String::from("Jane Doe"),
        is_admin: true,
        profile_image_path: String::from(""),
        should_change_password: true,
        is_onboarded: true,
    };
    let ret = facet_json::to_vec(&ret).unwrap();
    HttpResponse::Created()
        .content_type(ContentType::json())
        .body(ret)
}
