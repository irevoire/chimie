use actix_web::{
    web::{self, Data},
    HttpRequest,
};
use facet_actix::Json;

use crate::{
    auth::UserExtractor,
    config::{StorageTemplate, SystemConfig},
    error::HttpError,
    MainDatabase,
};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("", web::get().to(get_or_create_system_config))
        .route("defaults", web::get().to(default))
        .route(
            "storage-template-options",
            web::get().to(get_storage_template_options),
        );
}

pub async fn default(_req: HttpRequest) -> Json<SystemConfig> {
    Json(SystemConfig::default())
}

pub async fn get_or_create_system_config(
    db: Data<MainDatabase>,
    user: UserExtractor,
    _req: HttpRequest,
) -> Result<Json<SystemConfig>, HttpError> {
    let rtxn = db.read_tx();
    let user = db.get_user_mapping(&rtxn, user.0)?;
    let db = db.get_or_open_user_db(user.id).await?;
    let config = db.system_config(&rtxn)?;
    Ok(Json(config))
}

pub async fn get_storage_template_options(
    db: Data<MainDatabase>,
    user: UserExtractor,
    req: HttpRequest,
) -> Result<Json<StorageTemplate>, HttpError> {
    let config = get_or_create_system_config(db, user, req).await?;
    let template = config.into_inner().storage_template;
    Ok(Json(template))
}
