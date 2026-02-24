use actix_web::{
    HttpRequest, HttpResponse,
    http::header::ContentType,
    web::{self, Data},
};

use crate::MediaStore;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("buckets", web::get().to(buckets));
}

#[derive(Default, facet::Facet)]
#[facet(rename_all = "camelCase", deny_unknown_fields)]
struct Buckets {
    id: Vec<String>,
    city: Vec<Option<String>>,
    country: Vec<Option<String>>,
    duration: Vec<Option<String>>,
    visibility: Vec<Option<String>>,
    is_favorite: Vec<bool>,
    is_image: Vec<bool>,
    is_trashed: Vec<bool>,
    live_photo_video_id: Vec<Option<String>>,
    file_created_at: Vec<String>,
    local_offset_hours: Vec<usize>,
    owner_id: Vec<String>,
    projection_type: Vec<Option<String>>,
    ratio: Vec<f64>,
    active: Vec<String>,
    thumbhash: Vec<String>,
    stack: Vec<Option<Vec<String>>>,
}

pub async fn buckets(_req: HttpRequest, store: Data<MediaStore>) -> HttpResponse {
    let ids = store.query("demo");
    let mut ret = Buckets::default();
    ret.id = ids;
    let ret = facet_json::to_vec(&ret).unwrap();
    HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(ret)
}
