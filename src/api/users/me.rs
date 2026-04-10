use actix_web::{
    web::{self, Data},
    HttpRequest,
};
use facet_actix::Json;
use jiff::Timestamp;

use crate::{
    api::auth::{UserColor, UserLabel, UserStatus},
    auth::UserExtractor,
    error::HttpError,
    MainDatabase, User, UserId,
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
    let rtxn = db.read_tx();
    let user = db.get_user_mapping(&rtxn, user.0)?;
    let db = db.get_or_open_user_db(user.id).await?;
    let user = db.user(&rtxn)?;
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
#[facet(rename_all = "camelCase", deny_unknown_fields)]
pub struct Preferences {
    #[facet(default = Some(Albums::default()))]
    albums: Option<Albums>,
    #[facet(default = Some(Enabled::<SidebarWeb>::default()))]
    folders: Option<Enabled<SidebarWeb>>,
    #[facet(default = Some(Enabled::<Duration>::default()))]
    memories: Option<Enabled<Duration>>,
    #[facet(default = Some(Enabled::<SidebarWeb>::default()))]
    people: Option<Enabled<SidebarWeb>>,
    #[facet(default = Some(Enabled::<SidebarWeb>::default()))]
    shared_links: Option<Enabled<SidebarWeb>>,
    #[facet(default = Some(Enabled::<()>::default()))]
    ratings: Option<Enabled<()>>,
    #[facet(default = Some(Enabled::<SidebarWeb>::default()))]
    tags: Option<Enabled<SidebarWeb>>,
    #[facet(default = Some(Enabled::<EmailNotifications>::default()))]
    email_notifications: Option<Enabled<EmailNotifications>>,
    #[facet(default = Some(Download::default()))]
    download: Option<Download>,
    #[facet(default = Some(Purchase::default()))]
    purchase: Option<Purchase>,
    #[facet(default = Some(Cast::default()))]
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
    let rtxn = db.read_tx();
    let user = db.get_user_mapping(&rtxn, user.0)?;
    let db = db.get_or_open_user_db(user.id).await.unwrap();
    let pref = db.preferences(&rtxn)?;
    Ok(Json(pref))
}

pub async fn update_preferences(
    db: Data<MainDatabase>,
    user: UserExtractor,
    preferences: Json<Preferences>,
) -> Result<Json<Preferences>, HttpError> {
    let mut wtxn = db.write_tx()?;
    let user = db.get_user_mapping(&mut wtxn, user.0)?;
    let db = db.get_or_open_user_db(user.id).await.unwrap();
    let pref = db.update_preferences(&mut wtxn, move |pref| Preferences {
        albums: preferences.0.albums.or(pref.albums),
        folders: preferences.0.folders.or(pref.folders),
        memories: preferences.0.memories.or(pref.memories),
        people: preferences.0.people.or(pref.people),
        shared_links: preferences.0.shared_links.or(pref.shared_links),
        ratings: preferences.0.ratings.or(pref.ratings),
        tags: preferences.0.tags.or(pref.tags),
        email_notifications: preferences
            .0
            .email_notifications
            .or(pref.email_notifications),
        download: preferences.0.download.or(pref.download),
        purchase: preferences.0.purchase.or(pref.purchase),
        cast: preferences.0.cast.or(pref.cast),
    })?;
    Ok(Json(pref))
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
) -> Result<Json<Onboarding>, HttpError> {
    let mut wtxn = db.write_tx()?;
    let user = db.get_user_mapping(&wtxn, user.0)?;
    if !onboarding.0.is_onboarded {
        return Ok(onboarding);
    }
    let db = db.get_or_open_user_db(user.id).await.unwrap();
    db.update_user(&mut wtxn, |user| User {
        is_onboarded: true,
        ..user
    })?;
    wtxn.commit()??;
    Ok(onboarding)
}
