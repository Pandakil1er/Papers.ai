use crate::entities::image;
use actix_web::{get, web, HttpResponse, Responder};
use sea_orm::{DatabaseConnection, EntityTrait};

#[get("/images")]
async fn get_all_images(db: web::Data<DatabaseConnection>) -> impl Responder {
    match image::Entity::find().all(db.get_ref()).await {
        Ok(images) => HttpResponse::Ok().json(images),
        Err(e) => HttpResponse::InternalServerError().body(format!("DB error: {}", e)),
    }
}
