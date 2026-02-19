use actix_web::web;

mod auth;
mod config;
mod features;
mod media_types;
mod server;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("server").configure(server::configure))
        .service(web::scope("auth").configure(auth::configure));
}
