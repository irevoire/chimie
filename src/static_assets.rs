use actix_web::HttpResponse;
use actix_web::{HttpRequest, ResponseError};
use include_dir::{Dir, include_dir};

static WEB: Dir = include_dir!("$CARGO_MANIFEST_DIR/assets/web");

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Ressource {0} does not exists")]
    DoesNotExists(String),
}

impl ResponseError for Error {
    fn status_code(&self) -> actix_web::http::StatusCode {
        actix_web::http::StatusCode::NOT_FOUND
    }
}

pub async fn handle_files(req: HttpRequest) -> Result<HttpResponse, Error> {
    let path: String = req.match_info().query("filename").parse().unwrap();
    let path = if path.is_empty() {
        String::from("index.html")
    } else {
        path
    };
    let file = WEB
        .get_file(&path)
        .ok_or_else(|| Error::DoesNotExists(path.clone()))?;
    let content_type =
        actix_files::file_extension_to_mime(&file.path().extension().map_or("".to_string(), |s| {
            s.to_os_string().to_string_lossy().into_owned()
        }));
    Ok(HttpResponse::Ok()
        .content_type(content_type)
        .body(file.contents()))
}
