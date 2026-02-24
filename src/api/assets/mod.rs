use actix_multipart::form::MultipartForm;
use actix_web::{HttpRequest, HttpResponse, http::header::ContentType, web};
use facet_actix::Json;

use crate::{Database, MainDatabase};

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

type MpText<T> = actix_multipart::form::text::Text<T>;

#[derive(Debug, MultipartForm)]
#[multipart(deny_unknown_fields, duplicate_field = "deny")]
pub struct AssetUpload {
    #[multipart(rename = "deviceAssetId")]
    pub device_asset_id: MpText<String>,
    #[multipart(rename = "deviceId")]
    pub device_id: MpText<String>,
    #[multipart(rename = "fileCreatedAt")]
    pub file_created_at: MpText<String>,
    #[multipart(rename = "fileModifiedAt")]
    pub file_modified_at: MpText<String>,
    #[multipart(rename = "isFavorite")]
    pub is_favorite: MpText<String>,
    #[multipart(rename = "duration")]
    pub duration: MpText<String>,
    #[multipart(rename = "assetData")]
    pub asset_data: actix_multipart::form::tempfile::TempFile,
}

async fn assets(
    _req: HttpRequest,
    store: web::Data<MainDatabase>,
    asset: MultipartForm<AssetUpload>,
) -> HttpResponse {
    store.add_media("demo", asset.0);
    let ret = AssetsResult {
        results: Vec::new(),
    };
    let ret = facet_json::to_vec(&ret).unwrap();
    HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(ret)
}

async fn bulk_upload_check(_store: web::Data<Database>, _assets: Json<Assets>) -> HttpResponse {
    let ret = facet_json::to_vec(&true).unwrap();
    HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(ret)
}
