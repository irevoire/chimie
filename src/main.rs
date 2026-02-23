use std::process::ExitCode;

use actix_web::{
    App, HttpServer,
    middleware::Logger,
    web::{self, Data},
};
use fjall::Database;

mod api;
mod cli;
mod static_assets;

/// The database storing all the data you upload
pub struct MediaStore {
    db: Database,
}

impl MediaStore {
    pub fn new(path: &str) -> Self {
        let db = Database::builder(path).open().unwrap();
        Self { db }
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
