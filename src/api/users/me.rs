use actix_web::{HttpRequest, HttpResponse, http::header::ContentType, web};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("", web::get().to(me))
        .route("preferences", web::get().to(preferences));
}

#[derive(facet::Facet)]
#[facet(rename_all = "camelCase", deny_unknown_fields)]
struct Me {
    id: String,
    email: String,
    name: String,
    profile_image_path: String,
    avatar_color: String,
    profile_changed_at: String,
    storage_label: String,
    should_change_password: bool,
    is_admin: bool,
    created_at: String,
    deleted_at: Option<String>,
    updated_at: String,
    oauth_id: String,
    quota_size_in_bytes: Option<usize>,
    quota_usage_in_bytes: usize,
    status: String,
    license: Option<String>,
}

pub async fn me(_req: HttpRequest) -> HttpResponse {
    let ret = Me {
        id: String::from("6bbe2767-7851-461a-aa2d-afbd3460aa85"),
        email: String::from("demo@immich.app"),
        name: String::from("Jane Doe"),
        profile_image_path: String::from(""),
        avatar_color: String::from("yellow"),
        profile_changed_at: String::from("2026-01-20T15:13:30.207Z"),
        storage_label: String::from("admin"),
        should_change_password: true,
        is_admin: true,
        created_at: String::from("2026-01-20T15:13:30.207Z"),
        deleted_at: None,
        updated_at: String::from("2026-02-21T00:00:00.195Z"),
        oauth_id: String::from(""),
        quota_size_in_bytes: None,
        quota_usage_in_bytes: 20846462480,
        status: String::from("active"),
        license: None,
    };

    let ret = facet_json::to_vec(&ret).unwrap();
    HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(ret)
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
