use actix_web::{
    HttpRequest, HttpResponse,
    http::header::ContentType,
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
    id: UserId,
    email: String,
    name: String,
    profile_image_path: String,
    avatar_color: UserColor,
    profile_changed_at: Timestamp,
    storage_label: UserLabel,
    should_change_password: bool,
    is_admin: bool,
    created_at: Timestamp,
    deleted_at: Option<Timestamp>,
    updated_at: Timestamp,
    oauth_id: String,
    quota_size_in_bytes: Option<usize>,
    quota_usage_in_bytes: usize,
    status: UserStatus,
    license: Option<String>,
}

pub async fn me(
    db: Data<MainDatabase>,
    user: UserExtractor,
    _req: HttpRequest,
) -> Result<Json<Me>, HttpError> {
    let user = db.get_user(user.0)?;
    let ret = Me {
        id: user.id,
        email: user.email,
        name: user.name,
        profile_image_path: user.profile_image_path,
        avatar_color: user.avatar_color,
        profile_changed_at: user.profile_changed_at,
        storage_label: user.storage_label,
        should_change_password: user.should_change_password,
        is_admin: user.is_admin,
        created_at: user.created_at,
        deleted_at: user.deleted_at,
        updated_at: user.updated_at,
        oauth_id: user.oauth_id,
        quota_size_in_bytes: user.quota_size_in_bytes,
        quota_usage_in_bytes: user.quota_usage_in_bytes,
        status: user.status,
        license: user.license,
    };

    Ok(Json(ret))
}

#[derive(facet::Facet)]
#[facet(rename_all = "camelCase", deny_unknown_fields)]
struct Enabled<T> {
    enabled: bool,
    #[facet(flatten)]
    other: T,
}

#[derive(facet::Facet)]
#[facet(rename_all = "camelCase", deny_unknown_fields)]
struct SidebarWeb {
    sidebar_web: bool,
}

#[derive(facet::Facet)]
#[facet(rename_all = "camelCase", deny_unknown_fields)]
struct Albums {
    default_asset_order: AscDesc,
}

#[derive(facet::Facet)]
#[facet(rename_all = "lowercase")]
#[repr(C)]
enum AscDesc {
    Asc,
    Desc,
}

#[derive(facet::Facet)]
#[facet(rename_all = "camelCase", deny_unknown_fields)]
struct Duration {
    duration: usize,
}

#[derive(facet::Facet)]
#[facet(rename_all = "camelCase", deny_unknown_fields)]
struct EmailNotifications {
    album_invite: bool,
    album_update: bool,
}

#[derive(facet::Facet)]
#[facet(rename_all = "camelCase", deny_unknown_fields)]
struct Download {
    archive_size: usize,
    include_embedded_videos: bool,
}

#[derive(facet::Facet)]
#[facet(rename_all = "camelCase", deny_unknown_fields)]
struct Purchase {
    show_support_badge: bool,
    hide_buy_button_until: String,
}

#[derive(facet::Facet)]
#[facet(rename_all = "camelCase", deny_unknown_fields)]
struct Cast {
    g_cast_enabled: bool,
}

#[derive(facet::Facet)]
#[facet(rename_all = "camelCase", deny_unknown_fields)]
struct Preferences {
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

pub async fn preferences(_req: HttpRequest) -> HttpResponse {
    let ret = Preferences {
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
    };

    let ret = facet_json::to_vec(&ret).unwrap();
    HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(ret)
}
