use std::time::Duration;

use actix_web::{
    HttpRequest, HttpResponse,
    http::{StatusCode, header::ContentType},
    rt,
    web::{self, Data},
};
use actix_ws::AggregatedMessage;
use facet_actix::Json;

use crate::{
    MainDatabase,
    api::config::Config,
    auth::{UserExtractor, middleware::Auth},
    error::HttpError,
};

pub mod assets;
pub mod auth;
pub mod config;
pub mod features;
pub mod media_types;
pub mod notifications;
pub mod server;
pub mod system_config;
pub mod timeline;
pub mod users;

pub fn configure(cfg: &mut web::ServiceConfig, auth: Auth) {
    cfg.service(web::scope("auth").configure(auth::configure))
        .service(web::scope("server").configure(server::configure))
        .service(
            web::scope("notifications")
                .wrap(auth.clone())
                .configure(notifications::configure),
        )
        .service(
            web::scope("users")
                .wrap(auth.clone())
                .configure(users::configure),
        )
        .service(
            web::scope("assets")
                .wrap(auth.clone())
                .configure(assets::configure),
        )
        .service(
            web::scope("timeline")
                .wrap(auth.clone())
                .configure(timeline::configure),
        )
        .service(
            web::scope("system-config")
                .wrap(auth.clone())
                .configure(system_config::configure),
        )
        .service(
            web::scope("system-metadata")
                .wrap(auth.clone())
                .route("admin-onboarding", web::post().to(admin_onboarding)),
        )
        .route("socket.io/", web::get().to(socket))
        .route("memories", web::get().wrap(auth.clone()).to(memories))
        .route("albums", web::get().wrap(auth.clone()).to(albums));
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

#[derive(facet::Facet)]
#[facet(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdminOnboarding {
    is_onboarded: bool,
}

pub async fn admin_onboarding(
    db: Data<MainDatabase>,
    user: UserExtractor,
    payload: Json<AdminOnboarding>,
    _req: HttpRequest,
) -> Result<HttpResponse, HttpError> {
    if payload.is_onboarded {
        db.update_global_config(|config| Config {
            is_onboarded: true,
            ..config
        })?;
        let user = db.get_user_mapping(user.0)?;
        let db = db.get_or_open_user_db(user.id).await.unwrap();
        let user = db.user()?;
        if !user.is_admin {
            return Err(HttpError::NonAdminUserTriedToFinalizeSystemOnboarding {
                user: user.email,
            });
        }
    }
    Ok(HttpResponse::NoContent().finish())
}

async fn socket(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, HttpError> {
    let (mut res, mut session, stream) = actix_ws::handle(&req, stream)?;
    *res.status_mut() = StatusCode::SWITCHING_PROTOCOLS;

    let mut stream = stream
        .aggregate_continuations()
        // aggregate continuation frames up to 1MiB
        .max_continuation_size(2_usize.pow(20));

    // start task but don't wait for it
    rt::spawn(async move {
        println!("Hello from stream, sendning a 2");
        session.text("2").await.unwrap();
        // receive messages from websocket
        while let Some(msg) = stream.recv().await {
            match msg {
                Ok(AggregatedMessage::Text(text)) => {
                    println!("Hello from stream, got a {text}");
                    if let Ok(payload) = text.parse::<i32>() {
                        session
                            .text(format!("{}", payload.saturating_sub(1)))
                            .await
                            .unwrap();
                    }
                }

                // Other should never happens
                Ok(AggregatedMessage::Binary(bin)) => {
                    println!("Hello from stream, got binary");
                    // echo binary message
                    session.binary(bin).await.unwrap();
                }

                Ok(AggregatedMessage::Ping(msg)) => {
                    println!("Hello from stream, got a ping");
                    // respond to PING frame with PONG frame
                    session.pong(&msg).await.unwrap();
                }
                Ok(AggregatedMessage::Pong(msg)) => {
                    println!("Hello from stream, got a pong");
                    session.ping(&msg).await.unwrap();
                }
                Ok(AggregatedMessage::Close(_msg)) => {
                    println!("Stream is closed");
                }
                Err(e) => {
                    println!("Got error in the ws: {e:?}");
                }
            }
            println!("Going to sleep for 25s");
            tokio::time::sleep(Duration::from_secs(25)).await;
        }
        println!("connection to ws was closed");
    });
    println!("Returning the ws");

    // respond immediately with response connected to WS session
    Ok(res)
}
