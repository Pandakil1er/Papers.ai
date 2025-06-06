use crate::services::crypto;
use actix_web::{web, App, HttpServer};
use dotenv::dotenv;
use elasticsearch::{
    http::{
        headers::{HeaderMap, HeaderValue, AUTHORIZATION},
        transport::{SingleNodeConnectionPool, TransportBuilder},
    },
    Elasticsearch,
};
use routes::delete::delete_image_by_uuid;
use routes::get_all::get_all_images;
use routes::get_one::get_image_by_uuid;
use routes::hello::{echo, hello, manual_hello};
use routes::search::search;
use routes::send_data::upload;
use sea_orm::Database;
use std::env;
use url::Url;

mod entities;
mod helper;
mod routes;
mod services;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db = Database::connect(&database_url)
        .await
        .expect("Failed to connect to database");

    // Elasticsearch config
    let elasticsearch_url = env::var("ELASTICSEARCH_URL").expect("ELASTICSEARCH_URL must be set");
    let api_key = env::var("ELASTICSEARCH_API_KEY").expect("ELASTICSEARCH_API_KEY must be set");

    let url = Url::parse(&elasticsearch_url).expect("Invalid Elasticsearch URL");

    let conn_pool = SingleNodeConnectionPool::new(url);

    // Add Authorization header
    let mut headers = HeaderMap::new();
    let auth_value = format!("ApiKey {}", api_key);
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&auth_value).expect("Invalid API Key format"),
    );

    let transport = TransportBuilder::new(conn_pool)
        .headers(headers)
        .disable_proxy()
        .build()
        .expect("Failed to build Elasticsearch transport");

    let es_client = Elasticsearch::new(transport);

    println!("Connecting to DB: {}", database_url);
    let (host, port) = if let Ok(port_str) = env::var("PORT") {
        // PORT environment variable is set, likely on a server like Render
        let port: u16 = port_str.parse().expect("PORT must be a number");
        ("0.0.0.0", port)
    } else {
        // PORT environment variable is NOT set, assume local development
        ("127.0.0.1", 8080) // Default local port
    };
    let address = format!("{}:{}", host, port);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(db.clone()))
            .app_data(web::Data::new(es_client.clone()))
            .service(hello)
            .service(echo)
            .service(upload)
            .service(get_image_by_uuid)
            .service(search)
            .service(delete_image_by_uuid)
            .service(get_all_images)
            .route("/hey", web::get().to(manual_hello))
    })
    .bind(address)? // Bind to the dynamic address
    .run()
    .await
}
