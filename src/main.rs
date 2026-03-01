use std::{
    borrow::Cow,
    collections::{HashMap, hash_map::Entry},
    fs::FileTimes,
    io::ErrorKind,
    os::macos::fs::FileTimesExt,
    path::{Path, PathBuf},
    str::FromStr,
};

use actix_web::{
    App, HttpServer,
    middleware::Logger,
    web::{self, Data},
};
use fjall::{Database, Keyspace, KeyspaceCreateOptions};
use jiff::Timestamp;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::{
    api::{
        assets::AssetUpload,
        auth::{AdminSignUpResponse, UserColor, UserLabel, UserStatus},
        config::Config,
    },
    auth::{middleware::Auth, token_db::AccessTokenDatabase},
};

mod api;
mod auth;
mod cli;
mod error;
mod static_assets;

/// The database storing all the data you upload
pub struct MainDatabase {
    base_path: PathBuf,
    db: Database,
    main_db: Keyspace,
    auth_db: Keyspace,

    user_dbs: RwLock<HashMap<String, Keyspace>>,
}

#[derive(Debug, thiserror::Error)]
pub enum DbAccessError {
    #[error("While getting value from `{db_name}` db: {error}")]
    ReadingValue {
        db_name: Cow<'static, str>,
        error: fjall::Error,
    },
    #[error("While inserting key `{key}` and value `{value}` in `{db_name}` db: {error}")]
    WritingValue {
        key: Cow<'static, str>,
        value: String,
        db_name: Cow<'static, str>,
        error: fjall::Error,
    },
    #[error("Couldn't deserialize malformed value for key `{key}` in db `{db_name}`: {error}")]
    InternalDeserializationError {
        key: Cow<'static, str>,
        db_name: Cow<'static, str>,
        error: facet_json::DeserializeError,
    },
}

#[derive(facet::Facet)]
#[facet(rename_all = "camelCase", deny_unknown_fields)]
pub struct User {
    pub password_salt: String,
    // The sum of the actual password and the salt
    pub password_hash: String,

    pub id: UserId,
    pub email: String,
    pub name: String,
    pub profile_image_path: String,
    pub avatar_color: UserColor,
    pub profile_changed_at: Timestamp,
    pub storage_label: UserLabel,
    pub should_change_password: bool,
    pub is_admin: bool,
    pub is_onboarded: bool,
    pub created_at: Timestamp,
    pub deleted_at: Option<Timestamp>,
    pub updated_at: Timestamp,
    pub oauth_id: String,
    pub quota_size_in_bytes: Option<usize>,
    pub quota_usage_in_bytes: usize,
    pub status: UserStatus,
    pub license: Option<String>,
}

impl From<User> for AdminSignUpResponse {
    fn from(val: User) -> Self {
        AdminSignUpResponse {
            id: val.id,
            email: val.email,
            name: val.name,
            profile_image_path: val.profile_image_path,
            avatar_color: val.avatar_color,
            profile_changed_at: val.profile_changed_at,
            storage_label: val.storage_label,
            should_change_password: val.should_change_password,
            is_admin: val.is_admin,
            created_at: val.created_at,
            deleted_at: val.deleted_at,
            updated_at: val.updated_at,
            oauth_id: val.oauth_id,
            quota_size_in_bytes: val.quota_size_in_bytes,
            quota_usage_in_bytes: val.quota_usage_in_bytes,
            status: val.status,
            license: val.license,
        }
    }
}

impl MainDatabase {
    const DB_DIR: &str = "db";
    const MEDIA_DIR: &str = "media";
    const MAIN_KEYSPACE: &str = "main";
    const MAIN_GLOBAL_CONFIG_KEY: &str = "global_config";

    fn db_path(&self) -> PathBuf {
        self.base_path.join(Self::DB_DIR)
    }

    fn media_path(&self) -> PathBuf {
        self.base_path.join(Self::MEDIA_DIR)
    }

    fn global_config(&self) -> Result<Config, DbAccessError> {
        self.main_db
            .get(Self::MAIN_GLOBAL_CONFIG_KEY)
            .map_err(|error| DbAccessError::ReadingValue {
                db_name: Self::MAIN_KEYSPACE.into(),
                error,
            })?
            .map(|conf| {
                facet_json::from_slice(conf.as_ref()).map_err(|error| {
                    DbAccessError::InternalDeserializationError {
                        key: Self::MAIN_GLOBAL_CONFIG_KEY.into(),
                        db_name: Self::MAIN_KEYSPACE.into(),
                        error,
                    }
                })
            })
            .unwrap_or_else(|| Ok(Config::default()))
    }

    pub fn write_global_config(&self, config: Config) -> Result<(), DbAccessError> {
        let json = facet_json::to_string(&config).unwrap();

        self.main_db
            .insert(Self::MAIN_GLOBAL_CONFIG_KEY, json.as_str())
            .map_err(|error| DbAccessError::ReadingValue {
                db_name: Self::MAIN_KEYSPACE.into(),
                error,
            })?;
        Ok(())
    }

    pub fn update_global_config(
        &self,
        update: impl Fn(Config) -> Config,
    ) -> Result<(), DbAccessError> {
        let config = self.global_config()?;
        self.write_global_config((update)(config))
    }

    pub fn new(path: &Path) -> Self {
        match std::fs::create_dir_all(path) {
            Ok(_) => (),
            Err(e) if e.kind() == ErrorKind::AlreadyExists => (),
            Err(e) => panic!("{e}"),
        };
        match std::fs::create_dir_all(path.join(Self::MEDIA_DIR)) {
            Ok(_) => (),
            Err(e) if e.kind() == ErrorKind::AlreadyExists => (),
            Err(e) => panic!("{e}"),
        };

        let db = Database::builder(path.join(Self::DB_DIR)).open().unwrap();
        Self {
            base_path: path.to_path_buf(),
            main_db: db
                .keyspace(Self::MAIN_KEYSPACE, KeyspaceCreateOptions::default)
                .unwrap(),
            auth_db: db
                .keyspace(Self::AUTH_KEYSPACE, KeyspaceCreateOptions::default)
                .unwrap(),
            user_dbs: Default::default(),
            db,
        }
    }

    pub async fn get_or_create_user_db(&self, email: String) -> Result<Keyspace, fjall::Error> {
        // fast path
        let keyspace = self.user_dbs.read().await.get(&email).cloned();
        match keyspace {
            Some(keyspace) => Ok(keyspace.clone()),
            None => {
                let keyspace = self
                    .db
                    .keyspace(&email, || KeyspaceCreateOptions::default())?;
                self.user_dbs
                    .write()
                    .await
                    .entry(email)
                    .or_insert(keyspace.clone());
                Ok(keyspace)
            }
        }
    }

    pub fn add_media(&self, user: &str, media: AssetUpload) {
        let keyspace = self
            .db
            .keyspace(user, KeyspaceCreateOptions::default)
            .unwrap();
        let file_name = media.asset_data.file_name.unwrap();
        let path = self.media_path().join(file_name);
        let file = media.asset_data.file.persist(&path).unwrap();

        let created_at = Timestamp::from_str(&media.file_created_at.0).unwrap();
        let updated_at = Timestamp::from_str(&media.file_modified_at.0).unwrap();
        let ft = FileTimes::new()
            .set_created(created_at.into())
            .set_modified(updated_at.into())
            .set_accessed(updated_at.into());

        file.set_times(ft).unwrap();

        keyspace
            .insert(media.device_asset_id.as_bytes(), path)
            .unwrap();
    }

    pub fn query(&self, user: &str) -> Vec<String> {
        let keyspace = self
            .db
            .keyspace(user, KeyspaceCreateOptions::default)
            .unwrap();
        keyspace
            .iter()
            .map(|guard| String::from_utf8(guard.key().unwrap().to_vec()).unwrap())
            .collect()
    }
}

#[derive(Clone, PartialEq, Eq, facet::Facet)]
#[facet(transparent, rename_all = "camelCase", deny_unknown_fields)]
pub struct UserId(Uuid);

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port = 8080;
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let opt = match figue::from_std_args::<cli::Args>().into_result() {
        Ok(args) => args.value,
        Err(err) if err.is_help() => {
            eprintln!("{}", err.help_text().unwrap_or(""));
            return Ok(());
        }
        Err(err) => {
            eprintln!("{}", err);
            std::process::exit(1);
        }
    };
    let store = Data::new(MainDatabase::new(&opt.db_path));
    let auth = Data::new(AccessTokenDatabase::default());
    let auth_middleware = Auth(auth.clone());

    println!("Staring server on port {port}");
    HttpServer::new(move || {
        App::new()
            .app_data(store.clone())
            .app_data(auth.clone())
            .wrap(Logger::default())
            .service(
                web::scope("api").configure(|cfg| api::configure(cfg, auth_middleware.clone())),
            )
            .route("/{filename:.*}", web::get().to(static_assets::handle_files))
    })
    .bind(("127.0.0.1", port))?
    .run()
    .await
}
