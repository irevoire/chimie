use actix_web::{HttpRequest, HttpResponse, http::header::ContentType, web};
use facet_actix::Json;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("", web::post().to(assets))
        .route("bulk-upload-check", web::post().to(bulk_upload_check));
}

#[derive(Debug, facet::Facet)]
#[facet(rename_all = "camelCase", deny_unknown_fields)]
struct Assets {
    assets: Vec<Asset>,
}

#[derive(Debug, facet::Facet)]
#[facet(rename_all = "camelCase", deny_unknown_fields)]
struct Asset {
    id: String,
    checksum: String,
}

#[derive(Debug, facet::Facet)]
#[facet(rename_all = "camelCase", deny_unknown_fields)]
struct AssetsResult {
    results: Vec<AssetResult>,
}

#[derive(Debug, facet::Facet)]
#[facet(rename_all = "camelCase", deny_unknown_fields)]
struct AssetResult {
    id: String,
    status: String,
}

async fn assets(_req: HttpRequest) -> HttpResponse {
    let ret = AssetsResult {
        results: Vec::new(),
    };
    let ret = facet_json::to_vec(&ret).unwrap();
    HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(ret)
}

async fn bulk_upload_check(assets: Json<Assets>) -> HttpResponse {
    dbg!(&assets);
    let ret = facet_json::to_vec(&true).unwrap();
    HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(ret)
}
