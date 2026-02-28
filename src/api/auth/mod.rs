use actix_web::{
    HttpRequest, Responder,
    cookie::{Cookie, Expiration, time},
    http::StatusCode,
    web::{self, Data},
};
use facet_actix::Json;
use jiff::Timestamp;

use crate::{MainDatabase, UserId, auth::token_db::AccessTokenDatabase, error::HttpError};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("login", web::post().to(login))
        .route("admin-sign-up", web::post().to(admin_sign_up));
}

#[derive(facet::Facet)]
#[facet(rename_all = "camelCase", deny_unknown_fields)]
pub struct LoginRequest {
    pub email: String,
    #[facet(sensitive)]
    pub password: String,
}

#[derive(facet::Facet)]
#[facet(rename_all = "camelCase", deny_unknown_fields)]
pub struct LoginResponse {
    #[facet(sensitive)]
    pub access_token: String,
    pub user_id: UserId,
    pub user_email: String,
    pub name: String,
    pub is_admin: bool,
    pub profile_image_path: String,
    pub should_change_password: bool,
    pub is_onboarded: bool,
}

pub async fn login(
    auth: Data<AccessTokenDatabase>,
    db: Data<MainDatabase>,
    _req: HttpRequest,
    login: Json<LoginRequest>,
) -> Result<impl Responder, HttpError> {
    let ret = db.login(login.0)?;
    auth.register(ret.access_token.clone(), ret.user_email.clone())
        .await;
    let access_token = ret.access_token.clone();

    let expires_in = time::OffsetDateTime::now_utc() + time::Duration::days(1);

    let mut access_cookie = Cookie::new("immich_access_token", access_token);
    access_cookie.set_path("/");
    access_cookie.set_expires(Expiration::DateTime(expires_in));
    let mut auth_type = Cookie::new("immich_auth_type", "password");
    auth_type.set_path("/");
    auth_type.set_expires(Expiration::DateTime(expires_in));
    let mut is_authenticated = Cookie::new("immich_is_authenticated", "true");
    is_authenticated.set_path("/");
    is_authenticated.set_expires(Expiration::DateTime(expires_in));

    Ok(Json(ret)
        .customize()
        .with_status(StatusCode::CREATED)
        .add_cookie(&access_cookie)
        .add_cookie(&auth_type)
        .add_cookie(&is_authenticated))
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
        .map(AdminSignUpResponse::from)
        .map(Json)
        .map_err(HttpError::from)
        .customize()
        .with_status(StatusCode::CREATED)
}
