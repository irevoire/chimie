use actix_web::{HttpRequest, HttpResponse, http::header::ContentType, web};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("config").configure(super::config::configure))
        .service(web::scope("features").configure(super::features::configure))
        .service(web::scope("media-types").configure(super::media_types::configure))
        .route("about", web::get().to(about));
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
