use actix_web::web;

mod auth;
mod config;
mod features;
mod me;
mod media_types;
mod notifications;
mod server;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("server").configure(server::configure))
        .service(web::scope("notifications").configure(notifications::configure))
        .service(web::scope("me").configure(me::configure))
        .service(web::scope("auth").configure(auth::configure));
}
