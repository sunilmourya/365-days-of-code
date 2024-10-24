use actix_files::{Files, NamedFile};
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use log::{error, info};
use std::env;

pub(crate) fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/", web::get().to(index)) // Serve index.html on "/"
        .service(Files::new("/static", "./static").show_files_listing()); // Serve static files
}

async fn index(req: HttpRequest) -> impl Responder {
    info!("static/index.html requested!");

    // Try to get the current directory and construct the path to index.html
    let path_result =
        env::current_dir().map(|dir| dir.join("static/index.html"));

    match path_result {
        Ok(path) => match NamedFile::open(path) {
            Ok(file) => file.into_response(&req), // Convert NamedFile into HttpResponse
            Err(err) => {
                error!("Failed to open static/index.html: {:?}", err);
                HttpResponse::NotFound().body("File not found") // Return 404 if file is not found
            }
        },
        Err(err) => {
            error!("Failed to get current directory: {:?}", err);
            HttpResponse::InternalServerError().body("Internal server error") // Return 500 if there's a path error
        }
    }
}
