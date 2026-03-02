use actix_web::{
    HttpRequest,
    web::{self, Data},
};
use facet_actix::Json;
use jiff::Timestamp;

use crate::{
    MainDatabase, UserId,
    api::auth::{UserColor, UserLabel, UserStatus},
    auth::UserExtractor,
    error::HttpError,
};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("", web::get().to(me))
        .route("preferences", web::get().to(preferences));
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

#[derive(Debug, facet::Facet)]
#[facet(rename_all = "camelCase", deny_unknown_fields)]
struct Enabled<T> {
    enabled: bool,
    #[facet(flatten)]
    other: T,
}

#[derive(Debug, facet::Facet)]
#[facet(rename_all = "camelCase", deny_unknown_fields)]
struct SidebarWeb {
    sidebar_web: bool,
}

#[derive(Debug, facet::Facet)]
#[facet(rename_all = "camelCase", deny_unknown_fields)]
struct Albums {
    default_asset_order: AscDesc,
}

#[derive(Debug, facet::Facet)]
#[facet(rename_all = "lowercase")]
#[repr(C)]
enum AscDesc {
    Asc,
    Desc,
}

#[derive(Debug, facet::Facet)]
#[facet(rename_all = "camelCase", deny_unknown_fields)]
struct Duration {
    duration: usize,
}

#[derive(Debug, facet::Facet)]
#[facet(rename_all = "camelCase", deny_unknown_fields)]
struct EmailNotifications {
    album_invite: bool,
    album_update: bool,
}

#[derive(Debug, facet::Facet)]
#[facet(rename_all = "camelCase", deny_unknown_fields)]
struct Download {
    archive_size: usize,
    include_embedded_videos: bool,
}

#[derive(Debug, facet::Facet)]
#[facet(rename_all = "camelCase", deny_unknown_fields)]
struct Purchase {
    show_support_badge: bool,
    hide_buy_button_until: String,
}

#[derive(Debug, facet::Facet)]
#[facet(rename_all = "camelCase", deny_unknown_fields)]
struct Cast {
    g_cast_enabled: bool,
}

#[derive(facet::Facet, Debug)]
#[facet(rename_all = "camelCase", deny_unknown_fields)]
pub struct Preferences {
    albums: Albums,
    folders: Enabled<SidebarWeb>,
    memories: Enabled<Duration>,
    people: Enabled<SidebarWeb>,
    shared_links: Enabled<SidebarWeb>,
    ratings: Enabled<()>,
    tags: Enabled<SidebarWeb>,
    email_notifications: Enabled<EmailNotifications>,
    download: Download,
    purchase: Purchase,
    cast: Cast,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            albums: Albums {
                default_asset_order: AscDesc::Desc,
            },
            folders: Enabled {
                enabled: false,
                other: SidebarWeb { sidebar_web: false },
            },
            memories: Enabled {
                enabled: true,
                other: Duration { duration: 5 },
            },
            people: Enabled {
                enabled: true,
                other: SidebarWeb { sidebar_web: false },
            },
            shared_links: Enabled {
                enabled: true,
                other: SidebarWeb { sidebar_web: false },
            },
            ratings: Enabled {
                enabled: false,
                other: (),
            },
            tags: Enabled {
                enabled: true,
                other: SidebarWeb { sidebar_web: true },
            },
            email_notifications: Enabled {
                enabled: true,
                other: EmailNotifications {
                    album_invite: true,
                    album_update: true,
                },
            },
            download: Download {
                archive_size: 4294967296,
                include_embedded_videos: false,
            },
            purchase: Purchase {
                show_support_badge: true,
                hide_buy_button_until: String::from("2124-02-20T23:40:58.100Z"),
            },
            cast: Cast {
                g_cast_enabled: false,
            },
        }
    }
}

pub async fn preferences(
    db: Data<MainDatabase>,
    user: UserExtractor,
) -> Result<Json<Preferences>, HttpError> {
    let user = db.get_user_mapping(user.0)?;
    let db = db.get_or_open_user_db(user.id).await.unwrap();
    let pref = db.preferences()?;
    Ok(Json(pref))
}
