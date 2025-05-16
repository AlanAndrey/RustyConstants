use actix_web::{App, HttpServer};
use config::{Config, ConfigError, File};
use rusty_constants::{health_check, hello, quit, request_constants, view_csv_content};
use serde::Deserialize;
use std::sync::mpsc;
use std::thread;

#[derive(Debug, Deserialize)]
struct ServerConfig {
    host: String,
    port: u16,
}

#[derive(Debug, Deserialize)]
struct Settings {
    server: ServerConfig,
}

fn load_config() -> Result<Settings, ConfigError> {
    let settings = Config::builder()
        .add_source(File::with_name("config"))
        .build()?;

    settings.try_deserialize()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load configuration
    // Loads the application configuration from a file or uses a default
    // configuration if loading fails.
    //
    // This block attempts to load the configuration by calling `load_config()`.
    // - If `load_config()` returns `Ok(config)`, the `config` variable is assigned the loaded settings.
    // - If `load_config()` returns `Err(e)`, an error message detailing `e` is printed to the standard error stream.
    //   A subsequent message "Using default configuration" is also printed to `stderr`.
    //   In this case, `config` is initialized with a default `Settings` struct, where the server
    //   is configured to listen on `host: "127.0.0.1"` and `port: 8080`.
    let config = match load_config() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Error loading configuration: {}", e);
            eprintln!("Using default configuration");
            Settings {
                server: ServerConfig {
                    host: "127.0.0.1".to_string(),
                    port: 8080,
                },
            }
        }
    };

    let address = format!("{}:{}", config.server.host, config.server.port);
    println!("Starting server at http://{}", address);

    // Channel for shutdown signal
    let (tx, rx) = mpsc::channel::<()>();
    let server_tx = actix_web::web::Data::new(tx);

    // Start server
    let server = HttpServer::new(move || {
        App::new()
            .app_data(server_tx.clone())
            .service(hello)
            .service(health_check)
            .service(quit)
            .service(view_csv_content)
            .service(request_constants)
    })
    .bind(&address)?
    .run();

    // Get server handle
    let server_handle = server.handle();

    // Spawn thread to wait for shutdown signal
    thread::spawn(move || {
        // Wait for shutdown signal
        rx.recv().ok();
        // Stop server gracefully
        server_handle.stop(true);
    });

    // Run server until stopped
    server.await
}