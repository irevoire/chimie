use actix_web::web;

mod me;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("me").configure(me::configure));
}
