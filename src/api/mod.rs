use actix_web::{HttpRequest, HttpResponse, http::header::ContentType, web};

mod auth;
mod config;
mod features;
mod media_types;
mod notifications;
mod server;
mod timeline;
mod users;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("server").configure(server::configure))
        .service(web::scope("notifications").configure(notifications::configure))
        .service(web::scope("users").configure(users::configure))
        .service(web::scope("auth").configure(auth::configure))
        .service(web::scope("timeline").configure(timeline::configure))
        .route("memories", web::get().to(memories))
        .route("albums", web::get().to(albums));
}

#[derive(facet::Facet)]
#[facet(transparent)]
struct Memories(Vec<Memory>);

#[derive(facet::Facet)]
#[facet(rename_all = "camelCase", deny_unknown_fields)]
struct Memory {
    id: String,
    created_at: String,
    updated_at: String,
    memory_at: String,
    show_at: String,
    hide_at: String,
    owner_id: String,
    #[facet(rename = "type")]
    kind: String,
    data: MemoryData,
    is_saved: bool,

    assets: Vec<Asset>,
}

#[derive(facet::Facet)]
#[facet(rename_all = "camelCase", deny_unknown_fields)]
struct MemoryData {
    year: usize,
}

#[derive(facet::Facet)]
#[facet(rename_all = "camelCase", deny_unknown_fields)]
struct Asset {
    id: String,
    created_at: String,
    device_asset_id: String,
    owner_id: String,
    device_id: String,
    library_id: Option<String>,
    #[facet(rename = "type")]
    kind: String,
    original_path: String,
    original_file_name: String,
    original_mime_type: String,
    thumbhash: String,
    file_created_at: String,
    file_modified_at: String,
    local_date_time: String,
    updated_at: String,
    is_favorite: bool,
    is_archived: bool,
    is_trashed: bool,
    visibility: String,
    duration: String,
    live_photo_video_id: Option<String>,
    // TODO: What is a people
    people: Vec<()>,
    checksum: String,
    is_offline: bool,
    has_metadata: bool,
    duplicate_id: Option<String>,
    resized: bool,
    width: usize,
    height: usize,
    is_edited: bool,
}

pub async fn memories(_req: HttpRequest) -> HttpResponse {
    let ret = Memories(Vec::new());
    let ret = facet_json::to_vec(&ret).unwrap();
    HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(ret)
}

#[derive(facet::Facet)]
#[facet(transparent)]
struct Albums(Vec<Album>);

#[derive(facet::Facet)]
#[facet(rename_all = "camelCase", deny_unknown_fields)]
struct Album {
    album_name: String,
    description: String,
    album_thumbnail_asset_id: String,
    created_at: String,
    updated_at: String,
    id: String,
    owner_id: String,
    owner: AlbumOwner,
    // TODO: what is this
    album_users: Vec<()>,
    shared: bool,
    has_shared_link: bool,
    start_date: String,
    end_date: String,
    // TODO: not sure about this one
    assets: Vec<Asset>,
    asset_count: usize,
    is_activity_enabled: bool,
    order: String,
    last_modified_asset_timestamp: String,
}

#[derive(facet::Facet)]
#[facet(rename_all = "camelCase", deny_unknown_fields)]
struct AlbumOwner {
    id: String,
    email: String,
    name: String,
    profile_image_path: String,
    avatar_color: String,
    profile_changed_at: String,
}

pub async fn albums(_req: HttpRequest) -> HttpResponse {
    let ret = Memories(Vec::new());
    let ret = facet_json::to_vec(&ret).unwrap();
    HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(ret)
}
