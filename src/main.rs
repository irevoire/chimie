use std::{
    borrow::Cow,
    io::ErrorKind,
    path::{Path, PathBuf},
};

use actix_web::{
    App, HttpServer,
    middleware::Logger,
    web::{self, Data},
};
use fjall::{Database, Keyspace, KeyspaceCreateOptions};

use crate::api::{assets::AssetUpload, config::Config};

mod api;
mod cli;
mod error;
mod static_assets;

/// The database storing all the data you upload
pub struct MainDatabase {
    base_path: PathBuf,
    db: Database,
    main_db: Keyspace,
}

#[derive(Debug, thiserror::Error)]
pub enum DbAccessError {
    #[error("While getting value from `{db_name}` db: {error}")]
    ReadingValue {
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
            db,
        }
    }

    pub fn add_media(&self, user: &str, media: AssetUpload) {
        let keyspace = self
            .db
            .keyspace(user, KeyspaceCreateOptions::default)
            .unwrap();
        let file_name = media.asset_data.file_name.unwrap();
        let path = self.media_path().join(file_name);
        let _file = media.asset_data.file.persist(&path).unwrap();
        // TODO: Fix that
        // file.set_times(media.file_created_at).unwrap();
        // file.set_modified(media.file_modified_at).unwrap();

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
