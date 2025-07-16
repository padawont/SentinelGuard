use actix_web::HttpServer;
use sentinel_guard::config::AppConfig;
use sentinel_guard::repositories::register::register_repositories;
use sentinel_guard::routes::register::register_routes;
use sentinel_guard::utils::swagger::get_swagger_ui;
use sqlx::postgres::PgPool;
use std::{sync::Arc, time::Duration};
use tokio::signal;

#[actix_web::main]
async fn main() -> Result<(), anyhow::Error> {
    let config = AppConfig::from_env(Some(true))?;

    let pool = Arc::new(PgPool::connect(&config.database_uri).await?);
    let host = config.host;
    let port = config.port;

    let server = HttpServer::new(move || {
        let app = actix_web::App::new();

        let app = register_repositories(app, pool.clone());
        let app = register_routes(app);
        app.service(get_swagger_ui())
    })
    .bind((host.clone(), port))?
    .shutdown_timeout(30) // 30 seconds graceful shutdown timeout
    .workers(4) // Set number of workers
    .keep_alive(Duration::from_secs(75)) // Keep-alive timeout
    .run();

    println!("Server started at http://{}:{}", host, port);

    let server_handle = server.handle();

    // Wait for the server to finish or for a Ctrl+C signal
    tokio::select! {
        _ = server => {
            println!("Server finished");
        }
        _ = signal::ctrl_c() => {
            println!("Received ctrl+c signal, shutting down gracefully");

            // Stop accepting new connections
            server_handle.stop(true).await;

            // Give some time for cleanup
            tokio::time::sleep(Duration::from_secs(1)).await;
            println!("Cleanup completed, server shutting down");
        }
    }

    Ok(())
}
