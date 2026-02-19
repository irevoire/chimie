//! TODO: Understand all the options and see which one should be exposed and how

use actix_web::{HttpRequest, HttpResponse, http::header::ContentType};

pub async fn features(_req: HttpRequest) -> HttpResponse {
    let conf = Features::default();
    let ret = facet_json::to_vec(&conf).unwrap();
    HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(ret)
}

#[derive(facet::Facet)]
#[facet(rename_all = "camelCase", deny_unknown_fields)]
pub struct Features {
    smart_search: bool,
    facial_recognition: bool,
    duplicate_detection: bool,
    map: bool,
    reverse_geocoding: bool,
    import_faces: bool,
    sidecar: bool,
    search: bool,
    trash: bool,
    oauth: bool,
    oauth_auto_launch: bool,
    ocr: bool,
    password_login: bool,
    config_file: bool,
    email: bool,
}

impl Default for Features {
    fn default() -> Self {
        Self {
            smart_search: true,
            facial_recognition: true,
            duplicate_detection: true,
            map: true,
            reverse_geocoding: true,
            import_faces: false,
            sidecar: true,
            search: true,
            trash: true,
            oauth: false,
            oauth_auto_launch: false,
            ocr: true,
            password_login: true,
            config_file: false,
            email: false,
        }
    }
}
