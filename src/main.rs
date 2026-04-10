use std::{
    borrow::Cow,
    collections::HashMap,
    io::ErrorKind,
    path::{Path, PathBuf},
};

use actix_web::{
    middleware::Logger,
    web::{self, Data},
    App, HttpServer,
};
use fjall::{Database, KeyspaceCreateOptions, OptimisticTxDatabase, Snapshot};
use fjall_typed::{
    codec::{FacetJson, Str, Unspecified},
    OptimisticTxKeyspace,
};
use fjall_typed::{OptimisticWriteTx, TypedReadable};
use jiff::Timestamp;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::{
    api::{
        auth::{AdminSignUpResponse, UserColor, UserLabel, UserStatus},
        config::Config,
        users::me::Me,
    },
    auth::{middleware::Auth, token_db::AccessTokenDatabase},
    user::UserDb,
};

mod api;
mod auth;
mod cli;
mod config;
mod error;
mod static_assets;
mod user;

/// The database storing all the data you upload
pub struct MainDatabase {
    base_path: PathBuf,
    db: OptimisticTxDatabase,
    main_db: OptimisticTxKeyspace<'static, Str, Unspecified>,
    auth_db: OptimisticTxKeyspace<'static, Str, Unspecified>,

    users: RwLock<HashMap<String, UserDb>>,
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

impl MainDatabase {
    const DB_DIR: &str = "db";
    const MEDIA_DIR: &str = "media";
    const MAIN_KEYSPACE: &str = "main";
    const MAIN_GLOBAL_CONFIG_KEY: &str = "global_config";

    pub fn read_tx(&self) -> Snapshot {
        self.db.read_tx()
    }

    pub fn write_tx(&self) -> Result<OptimisticWriteTx, fjall::Error> {
        Ok(OptimisticWriteTx::new(self.db.write_tx()?))
    }

    fn user_mapping_prefix(email: &str) -> String {
        format!("user: {email}")
    }

    fn db_path(&self) -> PathBuf {
        self.base_path.join(Self::DB_DIR)
    }

    fn media_path(&self) -> PathBuf {
        self.base_path.join(Self::MEDIA_DIR)
    }

    fn global_config(&self, rtxn: &impl TypedReadable) -> Result<Config, DbAccessError> {
        Ok(TypedReadable::get(
            rtxn,
            self.main_db
                .remap_value::<FacetJson<Config>>()
                .as_keyspace(),
            Self::MAIN_GLOBAL_CONFIG_KEY,
        )
        .map_err(|error| DbAccessError::ReadingValue {
            db_name: Self::MAIN_KEYSPACE.into(),
            error: Box::new(error),
        })?
        .unwrap_or_else(|| Config::default()))
    }

    pub fn write_global_config(
        &self,
        wtxn: &mut OptimisticWriteTx,
        config: Config,
    ) -> Result<(), DbAccessError> {
        wtxn.insert(
            self.main_db
                .remap_value::<FacetJson<Config>>()
                .as_keyspace(),
            Self::MAIN_GLOBAL_CONFIG_KEY,
            &config,
        )
        .map_err(|error| DbAccessError::ReadingValue {
            db_name: Self::MAIN_KEYSPACE.into(),
            error: Box::new(error),
        })?;
        Ok(())
    }

    pub fn update_global_config(
        &self,
        wtxn: &mut OptimisticWriteTx,
        update: impl Fn(Config) -> Config,
    ) -> Result<(), DbAccessError> {
        let config = self.global_config(wtxn)?;
        self.write_global_config(wtxn, (update)(config))
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

        let db = OptimisticTxDatabase::builder(path.join(Self::DB_DIR))
            .open()
            .unwrap();
        Self {
            base_path: path.to_path_buf(),
            main_db: OptimisticTxKeyspace::new(
                db.keyspace(Self::MAIN_KEYSPACE, KeyspaceCreateOptions::default)
                    .unwrap(),
            ),
            auth_db: OptimisticTxKeyspace::new(
                db.keyspace(Self::AUTH_KEYSPACE, KeyspaceCreateOptions::default)
                    .unwrap(),
            ),
            users: Default::default(),
            db,
        }
    }

    pub async fn create_user_db(&self, id: UserId, user: &User) -> Result<UserDb, fjall::Error> {
        let user_db = UserDb::create(&self.db, id, user)?;

        let mut user_dbs = self.users.write().await;
        user_dbs.insert(user.email.to_string(), user_db.clone());

        Ok(user_db)
    }

    pub async fn get_or_open_user_db(&self, user_id: UserId) -> Result<UserDb, fjall::Error> {
        // fast path
        let db = self.users.read().await.get(&user_id.0.to_string()).cloned();
        match db {
            Some(db) => Ok(db),
            None => {
                let user_db = UserDb::open(&self.db, user_id)?;

                self.users
                    .write()
                    .await
                    .entry(user_id.0.to_string())
                    .or_insert(user_db.clone());
                Ok(user_db)
            }
        }
    }

    pub fn query(&self, _user: &str) -> Vec<String> {
        Vec::new()
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
