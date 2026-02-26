use actix_web::{
    HttpRequest, HttpResponse, Responder,
    http::{StatusCode, header::ContentType},
    web::{self, Data},
};
use facet_actix::Json;
use jiff::Timestamp;

use crate::{MainDatabase, UserId, error::HttpError};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("login", web::post().to(login))
        .route("admin-sign-up", web::post().to(admin_sign_up));
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

#[derive(facet::Facet)]
#[facet(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdminSignUpRequest {
    pub email: String,
    pub name: String,
    #[facet(sensitive)]
    pub password: String,
}

#[derive(facet::Facet)]
#[facet(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdminSignUpResponse {
    pub id: UserId,
    pub email: String,
    pub name: String,
    pub profile_image_path: String,
    pub avatar_color: UserColor,
    pub profile_changed_at: Timestamp,
    pub storage_label: UserLabel,
    pub should_change_password: bool,
    pub is_admin: bool,
    pub created_at: Timestamp,
    pub deleted_at: Option<Timestamp>,
    pub updated_at: Timestamp,
    pub oauth_id: String,
    pub quota_size_in_bytes: Option<usize>,
    pub quota_usage_in_bytes: usize,
    pub status: UserStatus,
    pub license: Option<String>,
}

#[derive(facet::Facet)]
#[facet(rename_all = "camelCase", deny_unknown_fields)]
#[repr(u8)]
pub enum UserStatus {
    Active,
}

#[derive(facet::Facet)]
#[facet(rename_all = "camelCase", deny_unknown_fields)]
#[repr(u8)]
pub enum UserColor {
    Yellow,
}

#[derive(facet::Facet)]
#[facet(rename_all = "camelCase", deny_unknown_fields)]
#[repr(u8)]
pub enum UserLabel {
    Admin,
}

pub async fn admin_sign_up(
    db: Data<MainDatabase>,
    _req: HttpRequest,
    login: Json<AdminSignUpRequest>,
) -> impl Responder {
    db.register_admin(login.0)
        .map(Json)
        .map_err(HttpError::from)
        .customize()
        .with_status(StatusCode::CREATED)
}
