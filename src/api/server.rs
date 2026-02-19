use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("config").configure(super::config::configure))
        .service(web::scope("features").configure(super::features::configure))
        .service(web::scope("media-types").configure(super::media_types::configure));
}
