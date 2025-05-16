use actix_web::{get, App, HttpResponse, HttpServer, Responder};
use config::{Config, ConfigError, File};
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
        .add_source(File::with_name("config.json"))
        .build()?;

    settings.try_deserialize()
}

// Handler function that returns "Hello, world!
#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello, world!")
}

// Health check endpoint
#[get("/health")]
async fn health_check() -> impl Responder {
    HttpResponse::Ok().body("Tutto Bene!")
}

// Shutdown endpoint
#[get("/quit")]
async fn quit(server: actix_web::web::Data<mpsc::Sender<()>>) -> impl Responder {
    println!("Shutdown requested, stopping server...");
    let _ = server.send(());
    HttpResponse::Ok().body("Shutting down server...")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load configuration
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