use std::{
    borrow::Cow,
    collections::HashMap,
    fs::FileTimes,
    io::ErrorKind,
    path::{Path, PathBuf},
    str::FromStr,
};

use actix_web::{
    middleware::Logger,
    web::{self, Data},
    App, HttpServer,
};
use fjall::{Database, KeyspaceCreateOptions};
use fjall_typed::codec::{FacetJson, Str, Unspecified};
use fjall_typed::Keyspace;
use jiff::Timestamp;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::{
    api::{
        assets::AssetUpload,
        auth::{AdminSignUpResponse, UserColor, UserLabel, UserStatus},
        config::Config,
        users::me::{Me, Preferences},
    },
    auth::{middleware::Auth, token_db::AccessTokenDatabase},
    config::SystemConfig,
};

mod api;
mod auth;
mod cli;
mod config;
mod error;
mod static_assets;

/// The database storing all the data you upload
pub struct MainDatabase {
    base_path: PathBuf,
    db: Database,
    main_db: Keyspace<'static, Str, Unspecified>,
    auth_db: Keyspace<'static, Str, Unspecified>,

    user_dbs: RwLock<HashMap<String, Keyspace<'static, Str, Unspecified>>>,
}

#[derive(Debug, thiserror::Error)]
pub enum DbAccessError {
    #[error("While getting value from `{db_name}` db: {error}")]
    ReadingValue {
        db_name: Cow<'static, str>,
        error: Box<dyn std::error::Error>,
    },
    #[error("While inserting key `{key}` and value `{value}` in `{db_name}` db: {error}")]
    WritingValue {
        key: Cow<'static, str>,
        value: String,
        db_name: Cow<'static, str>,
        error: Box<dyn std::error::Error>,
    },
    #[error("Couldn't deserialize malformed value for key `{key}` in db `{db_name}`: {error}")]
    InternalDeserializationError {
        key: Cow<'static, str>,
        db_name: Cow<'static, str>,
        error: facet_json::DeserializeError,
    },
    #[error("User  with email {email} does not exist.")]
    UserDoesNotExist { email: String },
}

#[derive(facet::Facet)]
#[facet(rename_all = "camelCase", deny_unknown_fields)]
pub struct UserMapping {
    #[facet(sensitive)]
    pub password_salt: String,
    // The sum of the actual password and the salt
    #[facet(sensitive)]
    pub password_hash: String,
    pub id: UserId,
}

#[derive(facet::Facet)]
#[facet(rename_all = "camelCase", deny_unknown_fields)]
pub struct User {
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

impl From<User> for Me {
    fn from(user: User) -> Self {
        Self {
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
        }
    }
}

#[derive(Clone)]
pub struct UserDatabase(Keyspace<'static, Str, Unspecified>);

macro_rules! crud_on {
    ($key:ident, $ty:ty) => {
        paste::paste! {
            impl UserDatabase {
                pub const [<$key:upper>]: &str = stringify!($key);

                pub fn $key(&self) -> Result<$ty, DbAccessError> {
                    match self
                        .0
                        .remap_value::<FacetJson<$ty>>()
                        .get(Self::[<$key:upper>])
                        .map_err(|error| DbAccessError::ReadingValue {
                            db_name: Self::[<$key:upper>].into(),
                            error: Box::new(error),
                        })? {
                        Some(pref) => Ok(pref),
                        None => {
                            let pref = $ty::default();
                            self.0
                                .remap_value::<FacetJson<$ty>>()
                                .insert(Self::[<$key:upper>], &pref)
                                .map_err(|error| DbAccessError::WritingValue {
                                    key: Self::[<$key:upper>].into(),
                                    value: format!("{pref:?}"),
                                    db_name: "".into(),
                                    error: Box::new(error),
                                })?;
                            Ok(pref)
                        }
                    }
                }

                pub fn [<write_ $key>] (
                    &self,
                    preferences: &$ty,
                ) -> Result<(), DbAccessError> {
                    self.0.remap_value::<FacetJson<$ty>>().insert(Self::[<$key:upper>], preferences).map_err(|error| {
                        DbAccessError::ReadingValue {
                            db_name: Self::[<$key:upper>].into(),
                            error: Box::new(error),
                        }
                    })?;
                    Ok(())
                }

                pub fn [<update_ $key>] (
                    &self,
                    update: impl FnOnce($ty) -> $ty,
                ) -> Result<$ty, DbAccessError> {
                    let pref = self.$key()?;
                    let pref = (update)(pref);
                    self.[<write_ $key>](&pref)?;
                    Ok(pref)
                }
            }
        }
    };
}

crud_on!(preferences, Preferences);
crud_on!(system_config, SystemConfig);

impl UserDatabase {
    pub const ASSET_PREFIX: &str = "asset";

    // Can't use the crud macro on user because a user doesn't have a default value
    pub const USER: &str = "user";
    pub fn user(&self) -> Result<User, DbAccessError> {
        self.0
            .remap_value::<FacetJson<User>>()
            .get(Self::USER)
            .map_err(|error| DbAccessError::ReadingValue {
                db_name: Self::USER.into(),
                error: Box::new(error),
            })
            .map(|user| user.expect("User MUST contains a user definition"))
    }

    pub fn write_user(&self, user: User) -> Result<(), DbAccessError> {
        self.0
            .remap_value::<FacetJson<User>>()
            .insert(Self::USER, &user)
            .map_err(|error| DbAccessError::ReadingValue {
                db_name: Self::USER.into(),
                error: Box::new(error),
            })?;
        Ok(())
    }

    pub fn update_user(&self, update: impl Fn(User) -> User) -> Result<(), DbAccessError> {
        self.write_user((update)(self.user()?))
    }
}

impl MainDatabase {
    const DB_DIR: &str = "db";
    const MEDIA_DIR: &str = "media";
    const MAIN_KEYSPACE: &str = "main";
    const MAIN_GLOBAL_CONFIG_KEY: &str = "global_config";

    fn user_mapping_prefix(email: &str) -> String {
        format!("user: {email}")
    }

    fn db_path(&self) -> PathBuf {
        self.base_path.join(Self::DB_DIR)
    }

    fn media_path(&self) -> PathBuf {
        self.base_path.join(Self::MEDIA_DIR)
    }

    fn global_config(&self) -> Result<Config, DbAccessError> {
        Ok(self
            .main_db
            .remap_value::<FacetJson<Config>>()
            .get(Self::MAIN_GLOBAL_CONFIG_KEY)
            .map_err(|error| DbAccessError::ReadingValue {
                db_name: Self::MAIN_KEYSPACE.into(),
                error: Box::new(error),
            })?
            .unwrap_or_else(|| Config::default()))
    }

    pub fn write_global_config(&self, config: Config) -> Result<(), DbAccessError> {
        self.main_db
            .remap_value::<FacetJson<Config>>()
            .insert(Self::MAIN_GLOBAL_CONFIG_KEY, &config)
            .map_err(|error| DbAccessError::ReadingValue {
                db_name: Self::MAIN_KEYSPACE.into(),
                error: Box::new(error),
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
            main_db: Keyspace::new(
                db.keyspace(Self::MAIN_KEYSPACE, KeyspaceCreateOptions::default)
                    .unwrap(),
            ),
            auth_db: Keyspace::new(
                db.keyspace(Self::AUTH_KEYSPACE, KeyspaceCreateOptions::default)
                    .unwrap(),
            ),
            user_dbs: Default::default(),
            db,
        }
    }

    pub async fn create_user_db(
        &self,
        id: UserId,
        user: &User,
    ) -> Result<UserDatabase, fjall::Error> {
        let keyspace = self
            .db
            .keyspace(&id.0.to_string(), KeyspaceCreateOptions::default)?;
        let keyspace: Keyspace<'static, Str, Unspecified> = Keyspace::new(keyspace);

        keyspace
            .remap_value::<FacetJson<User>>()
            .insert(UserDatabase::USER, user)
            .map_err(|err| err.unwrap_fjall())?;

        let mut user_dbs = self.user_dbs.write().await;
        user_dbs.insert(user.email.to_string(), keyspace.clone());
        user_dbs.insert(id.0.to_string(), keyspace.clone());

        Ok(UserDatabase(keyspace))
    }

    pub async fn get_or_open_user_db(&self, user_id: UserId) -> Result<UserDatabase, fjall::Error> {
        // fast path
        let keyspace = self
            .user_dbs
            .read()
            .await
            .get(&user_id.0.to_string())
            .cloned();
        match keyspace {
            Some(keyspace) => Ok(UserDatabase(keyspace.clone())),
            None => {
                let keyspace = self
                    .db
                    .keyspace(&user_id.0.to_string(), KeyspaceCreateOptions::default)?;
                let keyspace = Keyspace::new(keyspace);
                self.user_dbs
                    .write()
                    .await
                    .entry(user_id.0.to_string())
                    .or_insert(keyspace.clone());
                Ok(UserDatabase(keyspace))
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

        let updated_at = Timestamp::from_str(&media.file_modified_at.0).unwrap();
        let ft = FileTimes::new()
            .set_modified(updated_at.into())
            .set_accessed(updated_at.into());

        // We can't do the same this on linux (at least ext4)
        #[cfg(target_os = "macos")]
        let ft = {
            use std::os::macos::fs::FileTimesExt;
            let created_at = Timestamp::from_str(&media.file_created_at.0).unwrap();
            ft.set_created(created_at.into())
        };
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

#[derive(Clone, Copy, PartialEq, Eq, facet::Facet)]
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
            // .service(web::resource("api/socket.io"))
            .route("/{filename:.*}", web::get().to(static_assets::handle_files))
    })
    .bind(("127.0.0.1", port))?
    .run()
    .await
}
