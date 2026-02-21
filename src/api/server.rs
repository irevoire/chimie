use actix_web::{HttpRequest, HttpResponse, http::header::ContentType, web};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("config").configure(super::config::configure))
        .service(web::scope("features").configure(super::features::configure))
        .service(web::scope("media-types").configure(super::media_types::configure))
        .route("about", web::get().to(about))
        .route("version-history", web::get().to(version_history))
        .route("storage", web::get().to(storage));
}

#[derive(facet::Facet)]
#[facet(rename_all = "camelCase", deny_unknown_fields)]
struct About {
    version: String,
    version_url: String,
    licensed: bool,
    build: String,
    build_url: String,
    build_image: String,
    build_image_url: String,
    repository: String,
    repository_url: String,
    source_ref: String,
    source_commit: String,
    source_url: String,
    nodejs: String,
    exiftool: String,
    ffmpeg: String,
    libvips: String,
    imagemagick: String,
}

pub async fn about(_req: HttpRequest) -> HttpResponse {
    let ret = About {
        version: String::from("v2.5.6"),
        version_url: String::from("https://github.com/immich-app/immich/releases/tag/v2.5.6"),
        licensed: false,
        build: String::from("21878147967"),
        build_url: String::from("https://github.com/immich-app/immich/actions/runs/21878147967"),
        build_image: String::from("v2.5.6"),
        build_image_url: String::from(
            "https://github.com/immich-app/immich/pkgs/container/immich-server",
        ),
        repository: String::from("immich-app/immich"),
        repository_url: String::from("https://github.com/immich-app/immich"),
        source_ref: String::from("v2.5.6"),
        source_commit: String::from("3be8e265cd5bd6ca921ff9e665e274c5de45caa0"),
        source_url: String::from(
            "https://github.com/immich-app/immich/commit/3be8e265cd5bd6ca921ff9e665e274c5de45caa0",
        ),
        nodejs: String::from("v24.12.0"),
        exiftool: String::from("13.45"),
        ffmpeg: String::from("7.1.3-1"),
        libvips: String::from("8.17.3"),
        imagemagick: String::from("7.1.2-2"),
    };

    let ret = facet_json::to_vec(&ret).unwrap();
    HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(ret)
}

#[derive(facet::Facet)]
#[facet(rename_all = "camelCase", deny_unknown_fields)]
struct Storage {
    disk_size: String,
    disk_use: String,
    disk_available: String,
    disk_size_raw: usize,
    disk_use_raw: usize,
    disk_available_raw: usize,
    disk_usage_percentage: f64,
}

pub async fn storage(_req: HttpRequest) -> HttpResponse {
    let ret = Storage {
        disk_size: String::from("100.0 GiB"),
        disk_use: String::from("25.5 GiB"),
        disk_available: String::from("74.5 GiB"),
        disk_size_raw: 107374182400,
        disk_use_raw: 27362983936,
        disk_available_raw: 80011198464,
        disk_usage_percentage: 25.48,
    };

    let ret = facet_json::to_vec(&ret).unwrap();
    HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(ret)
}

#[derive(facet::Facet)]
#[facet(rename_all = "camelCase", deny_unknown_fields)]
struct Version {
    id: String,
    created_at: String,
    version: String,
}

pub async fn version_history(_req: HttpRequest) -> HttpResponse {
    let ret: Vec<Version> = Vec::new();
    let ret = facet_json::to_vec(&ret).unwrap();
    HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(ret)
}
