use std::{
    io::ErrorKind,
    path::{Path, PathBuf},
};

use actix_web::{
    App, HttpServer,
    middleware::Logger,
    web::{self, Data},
};
use fjall::{Database, KeyspaceCreateOptions};

use crate::api::assets::AssetUpload;

mod api;
mod cli;
mod static_assets;

/// The database storing all the data you upload
pub struct MediaStore {
    base_path: PathBuf,
    db: Database,
}

impl MediaStore {
    fn db_path(&self) -> PathBuf {
        self.base_path.join("db")
    }

    fn media_path(&self) -> PathBuf {
        self.base_path.join("media")
    }

    pub fn new(path: &Path) -> Self {
        match std::fs::create_dir_all(path) {
            Ok(_) => (),
            Err(e) if e.kind() == ErrorKind::AlreadyExists => (),
            Err(e) => panic!("{e}"),
        };
        match std::fs::create_dir_all(path.join("media")) {
            Ok(_) => (),
            Err(e) if e.kind() == ErrorKind::AlreadyExists => (),
            Err(e) => panic!("{e}"),
        };

        let db = Database::builder(path.join("db")).open().unwrap();
        Self {
            base_path: path.to_path_buf(),
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
    let store = Data::new(MediaStore::new(&opt.db_path));

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
