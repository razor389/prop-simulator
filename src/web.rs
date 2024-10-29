// src/web.rs
#[allow(unused_imports)]
use actix_web::{post, web, App, HttpServer, Responder, HttpResponse, middleware::Logger};
use prop_simulator::simulator::{SimulationConfig, run_simulation};
use env_logger::Env;
use log::info;

use actix_multipart::Multipart;
use futures_util::stream::StreamExt as _;

#[post("/simulate")]
async fn simulate(mut payload: Multipart) -> impl Responder {
    // Initialize variables to hold the configuration and CSV data
    let mut config: Option<SimulationConfig> = None;
    let mut csv_data: Option<String> = None;

    // Iterate over multipart form data
    while let Some(item) = payload.next().await {
        let mut field = item.unwrap();

        let content_disposition = field.content_disposition();
        let name = match content_disposition.get_name() {
            Some(name) => name,
            None => {
                return HttpResponse::BadRequest().body("Missing field name in content disposition");
            }
        };

        if name == "config" {
            // Read the JSON config data
            let mut data = Vec::new();
            while let Some(chunk) = field.next().await {
                data.extend_from_slice(&chunk.unwrap());
            }
            let config_json = String::from_utf8(data).unwrap();
            config = Some(serde_json::from_str(&config_json).unwrap());
        } else if name == "csv_file" {
            // Read the CSV file data
            let mut data = Vec::new();
            while let Some(chunk) = field.next().await {
                data.extend_from_slice(&chunk.unwrap());
            }
            csv_data = Some(String::from_utf8(data).unwrap());
        }
    }

    // Ensure config is present
    let mut config = match config {
        Some(c) => c,
        None => {
            return HttpResponse::BadRequest().body("Missing simulation configuration");
        }
    };

    // Set csv_data in config if provided
    if let Some(data) = csv_data {
        config.csv_data = Some(data);
    }

    // Run the simulation with the provided parameters
    match run_simulation(config) {
        Ok(result) => {
            // Return the result as JSON
            HttpResponse::Ok().json(result)
        }
        Err(e) => {
            // Return an error response
            HttpResponse::BadRequest().body(format!("Error: {}", e))
        }
    }
}


#[actix_web::main]
pub async fn main() -> std::io::Result<()> {
    // Initialize the logger
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    info!("Starting the Prop Simulator Web Server");

    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .service(simulate)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
