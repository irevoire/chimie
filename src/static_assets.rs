use actix_web::HttpRequest;
use actix_web::HttpResponse;
use include_dir::{Dir, include_dir};

static WEB: Dir = include_dir!("$CARGO_MANIFEST_DIR/assets/web");

pub async fn handle_files(req: HttpRequest) -> HttpResponse {
    let path: String = req.match_info().query("filename").parse().unwrap();
    let file = WEB
        .get_file(&path)
        .unwrap_or_else(|| WEB.get_file("index.html").unwrap());
    let content_type =
        actix_files::file_extension_to_mime(&file.path().extension().map_or("".to_string(), |s| {
            s.to_os_string().to_string_lossy().into_owned()
        }));
    HttpResponse::Ok()
        .content_type(content_type)
        .body(file.contents())
}
