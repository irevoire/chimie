//! TODO: Understand all the options and see which one should be exposed and how

use actix_web::{
    HttpRequest, HttpResponse,
    http::header::ContentType,
    web::{self, Data},
};

use crate::{MainDatabase, error::HttpError};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("", web::get().to(config));
}

pub async fn config(db: Data<MainDatabase>, _req: HttpRequest) -> Result<HttpResponse, HttpError> {
    let conf = db.global_config()?;
    let ret = facet_json::to_vec(&conf).unwrap();
    Ok(HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(ret))
}

#[derive(facet::Facet)]
#[facet(rename_all = "camelCase", deny_unknown_fields)]
pub struct Config {
    pub login_page_message: String,
    pub trash_days: usize,
    pub user_delete_delay: usize,
    pub oauth_button_text: String,
    pub is_initialized: bool,
    pub is_onboarded: bool,
    pub external_domain: String,
    pub public_users: bool,
    pub map_dark_style_url: String,
    pub map_light_style_url: String,
    pub maintenance_mode: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            login_page_message: String::from(
                "    This is a demo instance of Immich. It is regularly reset. <br>          <b style='color: red'>Due to abuse, uploads to this instance are disabled.</b><br>      <a class=\"underline\" href=\"javascript:       email = document.getElementById('email');       email.value = 'demo|immich.app'.replace('|', '@');       email.dispatchEvent(new Event('input', { bubbles: true }));       password = document.getElementById('password');       password.value = 'demo';       password.dispatchEvent(new Event('input', { bubbles: true }));       document.forms[0].requestSubmit();     \">Login as demo user</a>",
            ),
            trash_days: 30,
            user_delete_delay: 7,
            oauth_button_text: String::from("Login with OAuth"),
            is_initialized: false,
            is_onboarded: false,
            external_domain: String::from(""),
            public_users: true,
            map_dark_style_url: String::from("https://tiles.immich.cloud/v1/style/dark.json"),
            map_light_style_url: String::from("https://tiles.immich.cloud/v1/style/light.json"),
            maintenance_mode: false,
        }
    }
}
