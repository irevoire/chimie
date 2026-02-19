//! TODO: Understand all the options and see which one should be exposed and how

use actix_web::{HttpRequest, HttpResponse, http::header::ContentType, web};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("", web::get().to(media_types));
}

pub async fn media_types(_req: HttpRequest) -> HttpResponse {
    let conf = Features::default();
    let ret = facet_json::to_vec(&conf).unwrap();
    HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(ret)
}

#[derive(facet::Facet)]
#[facet(rename_all = "camelCase", deny_unknown_fields)]
pub struct Features {
    video: Vec<String>,
    image: Vec<String>,
    sidecar: Vec<String>,
}

impl Default for Features {
    fn default() -> Self {
        Self {
            video: vec![
                String::from(".3gp"),
                String::from(".3gpp"),
                String::from(".avi"),
                String::from(".flv"),
                String::from(".insv"),
                String::from(".m2t"),
                String::from(".m2ts"),
                String::from(".m4v"),
                String::from(".mkv"),
                String::from(".mov"),
                String::from(".mp4"),
                String::from(".mpe"),
                String::from(".mpeg"),
                String::from(".mpg"),
                String::from(".mts"),
                String::from(".vob"),
                String::from(".webm"),
                String::from(".wmv"),
            ],
            image: vec![
                String::from(".avif"),
                String::from(".bmp"),
                String::from(".gif"),
                String::from(".jpeg"),
                String::from(".jpg"),
                String::from(".png"),
                String::from(".webp"),
                String::from(".3fr"),
                String::from(".ari"),
                String::from(".arw"),
                String::from(".cap"),
                String::from(".cin"),
                String::from(".cr2"),
                String::from(".cr3"),
                String::from(".crw"),
                String::from(".dcr"),
                String::from(".dng"),
                String::from(".erf"),
                String::from(".fff"),
                String::from(".iiq"),
                String::from(".k25"),
                String::from(".kdc"),
                String::from(".mrw"),
                String::from(".nef"),
                String::from(".nrw"),
                String::from(".orf"),
                String::from(".ori"),
                String::from(".pef"),
                String::from(".psd"),
                String::from(".raf"),
                String::from(".raw"),
                String::from(".rw2"),
                String::from(".rwl"),
                String::from(".sr2"),
                String::from(".srf"),
                String::from(".srw"),
                String::from(".x3f"),
                String::from(".heic"),
                String::from(".heif"),
                String::from(".hif"),
                String::from(".insp"),
                String::from(".jp2"),
                String::from(".jpe"),
                String::from(".jxl"),
                String::from(".svg"),
                String::from(".tif"),
                String::from(".tiff"),
            ],
            sidecar: vec![String::from(".xmp")],
        }
    }
}
