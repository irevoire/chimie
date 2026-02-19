use actix_web::{App, HttpServer, web};

mod config;
mod features;
mod media_types;
mod static_assets;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port = 8080;
    println!("Staring server on port {port}");
    HttpServer::new(|| {
        App::new()
            .route("/api/server/config", web::get().to(config::config))
            .route("/api/server/features", web::get().to(features::features))
            .route(
                "/api/server/media-types",
                web::get().to(media_types::features),
            )
            .route("/{filename:.*}", web::get().to(static_assets::handle_files))
    })
    .bind(("127.0.0.1", port))?
    .run()
    .await
}
