use crate::entities::image;
use actix_web::{delete, web, HttpResponse, Responder};
use elasticsearch::{DeleteParts, Elasticsearch};
use sea_orm::ModelTrait;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use uuid::Uuid;

#[delete("/images/{uuid}")]
pub async fn delete_image_by_uuid(
    db: web::Data<DatabaseConnection>,
    es: web::Data<Elasticsearch>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let uuid = path.into_inner();

    // 🔍 Step 1: Find the image first
    match image::Entity::find()
        .filter(image::Column::Uuid.eq(uuid))
        .one(db.get_ref())
        .await
    {
        Ok(Some(model)) => {
            // 🗑 Step 2: Delete from database
            match model.delete(db.get_ref()).await {
                Ok(_) => {
                    // 🔍 Step 3: Delete from Elasticsearch (optional)
                    match es
                        .delete(DeleteParts::IndexId("papers", &uuid.to_string()))
                        .send()
                        .await
                    {
                        Ok(_) => HttpResponse::Ok().body("Image deleted from DB and Elasticsearch"),
                        Err(e) => {
                            println!("Failed to delete from Elasticsearch: {}", e);
                            HttpResponse::Ok().body("Deleted from DB, but failed in Elasticsearch")
                        }
                    }
                }
                Err(e) => {
                    println!("DB deletion failed: {}", e);
                    HttpResponse::InternalServerError().body(format!("DB deletion failed: {}", e))
                }
            }
        }
        Ok(None) => HttpResponse::NotFound().body("Image not found"),
        Err(e) => HttpResponse::InternalServerError().body(format!("DB query error: {}", e)),
    }
}
