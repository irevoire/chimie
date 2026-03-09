use actix_web::{
    HttpRequest,
    web::{self, Data},
};
use facet_actix::Json;
use jiff::Timestamp;

use crate::{
    MainDatabase, User, UserId,
    api::auth::{UserColor, UserLabel, UserStatus},
    auth::UserExtractor,
    error::HttpError,
};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("", web::get().to(me))
        .route("preferences", web::get().to(get_preferences))
        .route("preferences", web::put().to(update_preferences))
        .route("onboarding", web::put().to(set_onboarded));
}

#[derive(facet::Facet)]
#[facet(rename_all = "camelCase", deny_unknown_fields)]
pub struct Me {
    pub id: UserId,
    pub email: String,
    pub name: String,
    pub profile_image_path: String,
    pub avatar_color: UserColor,
    pub profile_changed_at: Timestamp,
    pub storage_label: UserLabel,
    pub should_change_password: bool,
    pub is_admin: bool,
    pub created_at: Timestamp,
    pub deleted_at: Option<Timestamp>,
    pub updated_at: Timestamp,
    pub oauth_id: String,
    pub quota_size_in_bytes: Option<usize>,
    pub quota_usage_in_bytes: usize,
    pub status: UserStatus,
    pub license: Option<String>,
}

pub async fn me(
    db: Data<MainDatabase>,
    user: UserExtractor,
    _req: HttpRequest,
) -> Result<Json<Me>, HttpError> {
    let user = db.get_user_mapping(user.0)?;
    let db = db.get_or_open_user_db(user.id).await?;
    let user = db.user()?;
    Ok(Json(user.into()))
}

#[derive(Default, Debug, facet::Facet)]
#[facet(rename_all = "camelCase", deny_unknown_fields)]
struct Enabled<T> {
    enabled: bool,
    #[facet(flatten)]
    other: T,
}

#[derive(Default, Debug, facet::Facet)]
#[facet(rename_all = "camelCase", deny_unknown_fields)]
struct SidebarWeb {
    sidebar_web: bool,
}

#[derive(Default, Debug, facet::Facet)]
#[facet(rename_all = "camelCase", deny_unknown_fields)]
struct Albums {
    default_asset_order: AscDesc,
}

#[derive(Default, Debug, facet::Facet)]
#[facet(rename_all = "lowercase")]
#[repr(C)]
enum AscDesc {
    #[default]
    Asc,
    Desc,
}

#[derive(Default, Debug, facet::Facet)]
#[facet(rename_all = "camelCase", deny_unknown_fields)]
struct Duration {
    duration: usize,
}

#[derive(Default, Debug, facet::Facet)]
#[facet(rename_all = "camelCase", deny_unknown_fields)]
struct EmailNotifications {
    album_invite: bool,
    album_update: bool,
}

#[derive(Default, Debug, facet::Facet)]
#[facet(rename_all = "camelCase", deny_unknown_fields)]
struct Download {
    archive_size: usize,
    include_embedded_videos: bool,
}

#[derive(Default, Debug, facet::Facet)]
#[facet(rename_all = "camelCase", deny_unknown_fields)]
struct Purchase {
    show_support_badge: bool,
    hide_buy_button_until: String,
}

#[derive(Default, Debug, facet::Facet)]
#[facet(rename_all = "camelCase", deny_unknown_fields)]
struct Cast {
    g_cast_enabled: bool,
}

#[derive(facet::Facet, Debug)]
#[facet(rename_all = "camelCase", deny_unknown_fields, default)]
pub struct Preferences {
    albums: Option<Albums>,
    folders: Option<Enabled<SidebarWeb>>,
    memories: Option<Enabled<Duration>>,
    people: Option<Enabled<SidebarWeb>>,
    shared_links: Option<Enabled<SidebarWeb>>,
    ratings: Option<Enabled<()>>,
    tags: Option<Enabled<SidebarWeb>>,
    email_notifications: Option<Enabled<EmailNotifications>>,
    download: Option<Download>,
    purchase: Option<Purchase>,
    cast: Option<Cast>,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            albums: Some(Albums {
                default_asset_order: AscDesc::Desc,
            }),
            folders: Some(Enabled {
                enabled: false,
                other: SidebarWeb { sidebar_web: false },
            }),
            memories: Some(Enabled {
                enabled: true,
                other: Duration { duration: 5 },
            }),
            people: Some(Enabled {
                enabled: true,
                other: SidebarWeb { sidebar_web: false },
            }),
            shared_links: Some(Enabled {
                enabled: true,
                other: SidebarWeb { sidebar_web: false },
            }),
            ratings: Some(Enabled {
                enabled: false,
                other: (),
            }),
            tags: Some(Enabled {
                enabled: true,
                other: SidebarWeb { sidebar_web: true },
            }),
            email_notifications: Some(Enabled {
                enabled: true,
                other: EmailNotifications {
                    album_invite: true,
                    album_update: true,
                },
            }),
            download: Some(Download {
                archive_size: 4294967296,
                include_embedded_videos: false,
            }),
            purchase: Some(Purchase {
                show_support_badge: true,
                hide_buy_button_until: String::from("2124-02-20T23:40:58.100Z"),
            }),
            cast: Some(Cast {
                g_cast_enabled: false,
            }),
        }
    }
}

pub async fn get_preferences(
    db: Data<MainDatabase>,
    user: UserExtractor,
) -> Result<Json<Preferences>, HttpError> {
    let user = db.get_user_mapping(user.0)?;
    let db = db.get_or_open_user_db(user.id).await.unwrap();
    let pref = db.preferences()?;
    Ok(Json(pref))
}

pub async fn update_preferences(
    db: Data<MainDatabase>,
    user: UserExtractor,
    preferences: Json<Preferences>,
) -> Result<(), HttpError> {
    let user = db.get_user_mapping(user.0)?;
    let db = db.get_or_open_user_db(user.id).await.unwrap();
    db.write_preferences(&preferences.0)?;
    Ok(())
}

#[derive(facet::Facet)]
#[facet(rename_all = "camelCase", deny_unknown_fields)]
pub struct Onboarding {
    pub is_onboarded: bool,
}

pub async fn set_onboarded(
    db: Data<MainDatabase>,
    user: UserExtractor,
    onboarding: Json<Onboarding>,
) -> Result<(), HttpError> {
    let user = db.get_user_mapping(user.0)?;
    if !onboarding.0.is_onboarded {
        return Ok(());
    }
    let db = db.get_or_open_user_db(user.id).await.unwrap();
    db.update_user(|user| User {
        is_onboarded: true,
        ..user
    })?;
    Ok(())
}
