use crate::entities::image;
use actix_web::{get, web, HttpResponse, Responder};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use uuid::Uuid;

#[get("/images/{uuid}")]
async fn get_image_by_uuid(
    db: web::Data<DatabaseConnection>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let uuid = path.into_inner();

    match image::Entity::find()
        .filter(image::Column::Uuid.eq(uuid)) // <- requires QueryFilter trait
        .one(db.get_ref())
        .await
    {
        Ok(Some(image)) => HttpResponse::Ok().json(image),
        Ok(None) => HttpResponse::NotFound().body("Image not found"),
        Err(e) => HttpResponse::InternalServerError().body(format!("DB error: {}", e)),
    }
}
