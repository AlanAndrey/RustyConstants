use actix_web::{get, HttpResponse, Responder};
use std::sync::mpsc;

#[get("/")]
// This endpoint is used to return a simple "Hello, world!" message
// It is the default endpoint that is hit when the server is started
pub async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello, world!")
}

#[get("/health")]
// This endpoint is used to check the health of the server
// It returns a simple "Tutto Bene!" message
// indicating that the server is running fine
pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().body("Tutto Bene!")
}

#[get("/quit")]
// This endpoint is used to gracefully shut down the server
// It sends a shutdown signal to the server
// and returns a message indicating that the server is shutting down
async fn quit(server: actix_web::web::Data<mpsc::Sender<()>>) -> impl Responder {
    println!("Shutdown requested, stopping server...");
    let _ = server.send(());
    HttpResponse::Ok().body("Shutting down server...")
}