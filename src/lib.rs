use actix_web::{get, post, web, HttpResponse, Responder};
use std::sync::mpsc;
use std::fs;
use std::path::Path;
pub mod handler;
use handler::{ConstantRequestPayload, ConstantProcessResult, process_constant};
pub mod config;
use config::get_config;

#[post("/constants")]
// This endpoint is used to handle post requests for the constants data
pub async fn request_constants(payload: web::Json<ConstantRequestPayload>) -> impl Responder {
    let mut results: Vec<ConstantProcessResult> = Vec::new();
    for name in &payload.names {
        let result = process_constant(name);
        results.push(result);
    }
    HttpResponse::Ok().json(results)
}

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



// Helper function to ensure the CSV file exists.
// Creates an empty file if it doesn't.
fn ensure_viewable_csv_file_exists() -> std::io::Result<()> {
    let config = get_config();
    let path = Path::new(config.database.path.as_str());
    if !path.exists() {
        // You could initialize with headers or sample data if needed:
        // fs::write(path, "Header1,Header2\nValue1,Value2\n")?;
        fs::File::create(path)?; // Creates an empty file
    }
    Ok(())
}

/// GET endpoint to display the content of a CSV file.
/// Reads the CSV file and presents its content.
#[get("/view-csv")]
pub async fn view_csv_content() -> impl Responder {
    if let Err(e) = ensure_viewable_csv_file_exists() {
        return HttpResponse::InternalServerError().body(format!("Error ensuring CSV file exists: {}", e));
    }
    let config = get_config();
    let csv_file_path = config.database.path.as_str();
    match fs::read_to_string(csv_file_path) {
        Ok(csv_content) => {
            let processed_content = csv_content
                .lines() // Iterate over each line
                .map(|line| {
                    // For each line, split by comma and join with a tab
                    // This is a simple replacement. For robust CSV parsing,
                    // especially with quoted fields containing commas,
                    // you'd use the `csv` crate's parser.
                    line.split(',')
                        .collect::<Vec<&str>>()
                        .join("\t") // Join parts with a tab
                })
                .collect::<Vec<String>>()
                .join("\n"); // Join lines back with a newline

            // Escape the CSV content before embedding it in HTML to prevent XSS
            let escaped_csv_content = html_escape::encode_text(&processed_content);

            let html_body = format!(
                r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>View CSV File</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 20px; background-color: #f4f4f4; color: #333; }}
        h1 {{ color: #333; }}
        p {{ margin-bottom: 15px; }}
        code {{ background-color: #eee; padding: 2px 4px; border-radius: 3px; }}
        pre {{
            background-color: #fff;
            border: 1px solid #ccc;
            padding: 15px;
            border-radius: 4px;
            white-space: pre-wrap; /* Allows wrapping of long lines */
            word-wrap: break-word; /* Breaks long words if necessary */
            font-family: monospace;
            font-size: 0.9rem;
            box-shadow: 0 0 10px rgba(0,0,0,0.05);
        }}
        .container {{ background-color: white; padding: 20px; border-radius: 5px; box-shadow: 0 0 10px rgba(0,0,0,0.1); }}
    </style>
</head>
<body>
    <div class="container">
        <h1>CSV File Content</h1>
        <p>Displaying content from: <code>{}</code></p>
        <pre>{}</pre>
    </div>
</body>
</html>"#,
                csv_file_path,
                escaped_csv_content
            );
            HttpResponse::Ok().content_type("text/html; charset=utf-8").body(html_body)
        }
        Err(e) => {
            eprintln!("Failed to read CSV file '{}': {}", csv_file_path, e);
            HttpResponse::InternalServerError().body(format!("Could not read CSV file. Error: {}", e))
        }
    }
}