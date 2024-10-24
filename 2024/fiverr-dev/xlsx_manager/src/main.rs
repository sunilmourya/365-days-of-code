use crate::routes::{api, index};
use actix_cors::Cors;
use actix_web::{App, HttpServer};
use log::info;

mod config;
mod routes;
mod xlsx_manager;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize the logger with the default log level as "info"
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info"),
    )
    .init();
    info!("Starting HttpServer..");

    // Load the configuration
    let config =
        config::AppConfig::from_env().expect("Failed to load configuration");
    info!("Using configuration: {:#?}", config);

    let server = HttpServer::new(|| {
        App::new()
            .wrap(
                Cors::default()
                    .allow_any_origin() // Allow any origin
                    .allow_any_method() // Allow any HTTP method
                    .allow_any_header(), // Allow any header
            )
            .configure(index::configure_routes)
            .configure(api::configure_routes)
    })
    .bind((config.server.host, config.server.port))?
    .workers(config.server.workers)
    .shutdown_timeout(config.server.shutdown_timeout)
    .run();
    server.await
}
