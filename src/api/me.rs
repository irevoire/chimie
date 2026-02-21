use actix_web::{HttpRequest, HttpResponse, http::header::ContentType, web};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("", web::get().to(me))
}

#[derive(facet::Facet)]
#[facet(rename_all = "camelCase", deny_unknown_fields)]
struct Me {
    id: String,
    email: String,
    name: String,
    profile_image_path: String,
    avatar_color: String,
    profile_changed_at: String,
    storage_label: String,
    should_change_password: bool,
    is_admin: bool,
    created_at: String,
    deleted_at: Option<String>,
    updated_at: String,
    oauth_id: String,
    quota_size_in_bytes: Option<usize>,
    quota_usage_in_bytes: usize,
    status: String,
    license: Option<String>,
}

pub async fn me(_req: HttpRequest) -> HttpResponse {
    let ret = Me {
        id: String::from("6bbe2767-7851-461a-aa2d-afbd3460aa85"),
        email: String::from("demo@immich.app"),
        name: String::from("Jane Doe"),
        profile_image_path: String::from(""),
        avatar_color: String::from("yellow"),
        profile_changed_at: String::from("2026-01-20T15:13:30.207Z"),
        storage_label: String::from("admin"),
        should_change_password: true,
        is_admin: true,
        created_at: String::from("2026-01-20T15:13:30.207Z"),
        deleted_at: None,
        updated_at: String::from("2026-02-21T00:00:00.195Z"),
        oauth_id: String::from(""),
        quota_size_in_bytes: None,
        quota_usage_in_bytes: 20846462480,
        status: String::from("active"),
        license: None,
    };

    let ret = facet_json::to_vec(&ret).unwrap();
    HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(ret)
}

