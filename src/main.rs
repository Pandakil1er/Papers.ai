use actix_web::{web, App, HttpServer};
use dotenv::dotenv;
use routes::hello::{echo, hello, manual_hello};
use routes::send_data::upload;
use sea_orm::Database;
use std::env;

mod entities;
mod routes;
mod services;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db = Database::connect(&database_url)
        .await
        .expect("Failed to connect to database");
    println!("Connecting to DB: {}", database_url);
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(db.clone()))
            .service(hello)
            .service(echo)
            .service(upload)
            .route("/hey", web::get().to(manual_hello))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
