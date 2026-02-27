use std::{
    borrow::Cow,
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
use argon2::{
    Argon2, PasswordHasher,
    password_hash::{SaltString, rand_core::OsRng},
};
use fjall::{Database, Keyspace, KeyspaceCreateOptions};
use jiff::Timestamp;
use uuid::Uuid;

use crate::api::{
    assets::AssetUpload,
    auth::{AdminSignUpRequest, AdminSignUpResponse, UserColor, UserLabel, UserStatus},
    config::Config,
};

mod api;
mod cli;
mod error;
mod static_assets;

/// The database storing all the data you upload
pub struct MainDatabase {
    base_path: PathBuf,
    db: Database,
    main_db: Keyspace,
    auth_db: Keyspace,
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
    #[error(
        "Internal error: Couldn't deserialize malformed value for key `{key}` in db `{db_name}`: {error}"
    )]
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
    const AUTH_KEYSPACE: &str = "auth";
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
            db,
        }
    }

    // TODO: Hash+salt the password (use a stable hash)
    pub fn register_admin(&self, req: AdminSignUpRequest) -> Result<User, DbAccessError> {
        let now = jiff::Timestamp::now();

        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(req.password.as_bytes(), &salt)
            .unwrap()
            .to_string();
        let response = User {
            password_salt: salt.to_string(),
            password_hash,
            id: UserId(Uuid::now_v7()),
            email: req.email,
            name: req.name,
            profile_image_path: String::new(),
            avatar_color: UserColor::Yellow,
            profile_changed_at: now,
            storage_label: UserLabel::Admin,
            should_change_password: false,
            is_admin: true,
            created_at: now,
            deleted_at: None,
            updated_at: now,
            oauth_id: String::new(),
            quota_size_in_bytes: None,
            quota_usage_in_bytes: 0,
            status: UserStatus::Active,
            license: None,
        };

        let json = facet_json::to_string(&response).unwrap();
        self.auth_db
            .insert(response.email.as_bytes(), json)
            .map_err(|err| DbAccessError::WritingValue {
                key: response.id.0.to_string().into(),
                // TODO: This can crash and leak the password, we should use the facet pretty print directly
                value: facet_json::to_string_pretty(&response).unwrap(),
                db_name: Self::AUTH_KEYSPACE.into(),
                error: err,
            })?;
        self.update_global_config(|config| Config {
            is_initialized: true,
            ..config
        })?;

        Ok(response)
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
#[facet(rename_all = "camelCase", deny_unknown_fields)]
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

    println!("Staring server on port {port}");
    HttpServer::new(move || {
        App::new()
            .app_data(store.clone())
            .wrap(Logger::default())
            .service(web::scope("api").configure(api::configure))
            .route("/{filename:.*}", web::get().to(static_assets::handle_files))
    })
    .bind(("127.0.0.1", port))?
    .run()
    .await
}
