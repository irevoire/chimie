use actix_web::{App, HttpServer, middleware::Logger, web};

mod api;
mod static_assets;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port = 8080;
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    println!("Staring server on port {port}");
    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .service(web::scope("api").configure(api::configure))
            .route("/{filename:.*}", web::get().to(static_assets::handle_files))
    })
    .bind(("127.0.0.1", port))?
    .run()
    .await
}
