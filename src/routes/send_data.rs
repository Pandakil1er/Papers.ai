use crate::entities::image;
use crate::helper::decrypt;
use crate::services::gemini::send_image_to_gemini_api;
use actix_multipart::form::{json::Json as MpJson, tempfile::TempFile, MultipartForm};
use actix_web::{post, web::Data, HttpResponse, Responder};
use base64::{engine::general_purpose, Engine};
use elasticsearch::{Elasticsearch, IndexParts};
use infer;
use sea_orm::{ActiveModelTrait, DatabaseConnection, Set};
use serde::Deserialize;
use serde::Serialize;
use serde_json::json;
use std::env;
use tokio::fs;
use uuid::Uuid; // replace with the correct path to your `image.rs` // Add Elasticsearch imports

#[derive(Debug, Deserialize)]
struct Metadata {
    name: String,
}

#[derive(Debug, MultipartForm)]
struct UploadForm {
    #[multipart(limit = "10MB")]
    file: TempFile,
    json: MpJson<Metadata>,
}

#[derive(Serialize)]
struct ImageIndex {
    uuid: String,
    name: String,
    summary: String,
    keywords: Vec<String>,
}

#[post("/upload")]
pub async fn upload(
    db: Data<DatabaseConnection>,
    es: Data<Elasticsearch>,
    MultipartForm(form): MultipartForm<UploadForm>,
) -> impl Responder {
    if form.json.name.trim().is_empty() {
        return HttpResponse::BadRequest().body("File name is missing");
    }

    let file_name = form
        .file
        .file_name
        .clone()
        .unwrap_or_else(|| "file.bin".to_string());

    let initial_path = form.file.file.path();

    // 🔧 Read file content asynchronously and handle error
    let file_bytes = match fs::read(initial_path).await {
        Ok(bytes) => bytes,
        Err(e) => {
            return HttpResponse::InternalServerError().body(format!(
                "Failed to read file '{}': {}",
                initial_path.display(),
                e
            ));
        }
    };

    // ✅ Encode to Base64
    let encoded_image = general_purpose::STANDARD.encode(&file_bytes);

    // 📁 Prepare target path
    let uuid = Uuid::new_v4();
    let mut target_path = env::current_dir().unwrap();
    target_path.push("uploads");
    target_path.push(format!("{}-{}", uuid, file_name));

    // 🔧 Ensure "uploads" dir exists
    if let Some(parent) = target_path.parent() {
        if let Err(e) = fs::create_dir_all(parent).await {
            return HttpResponse::InternalServerError()
                .body(format!("Failed to create upload directory: {}", e));
        }
    }

    // 📤 Copy the file
    if let Err(err) = fs::copy(initial_path, &target_path).await {
        return HttpResponse::InternalServerError().body(format!("File copy failed: {}", err));
    }

    let mime_type = infer::get(&file_bytes)
        .map(|t| t.mime_type())
        .unwrap_or("application/octet-stream");

    // HttpResponse::Ok().body(format!(
    //     "File saved as {}\nBase64 Preview: {}",
    //     target_path.display(),
    //     &encoded[..40]
    // ))

    let mut summary = String::new();
    let mut keywords = Vec::new(); // Assuming keywords is a Vec or similar
                                   //
                                   //
                                   // Loop until a non-empty summary is received
    loop {
        match send_image_to_gemini_api(&encoded_image, mime_type).await {
            Ok((s, k)) => {
                if !s.trim().is_empty() {
                    summary = s;
                    keywords = k; // Assign keywords once summary is valid
                    println!("Gemini returned a non-empty summary: {}", summary);
                    break; // Exit the loop as summary is not empty
                } else {
                    println!("Gemini returned an empty summary. Retrying...");
                    // Optionally, add a delay here to prevent hammering the API
                    // tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                }
            }
            Err(e) => {
                println!("Gemini API error during retry: {}", e);
                // Decide how to handle persistent errors:
                // - You could return an InternalServerError immediately
                // - You could continue retrying a limited number of times
                return HttpResponse::InternalServerError()
                    .body(format!("Gemini API error: {}", e));
            }
        }
    }

    println!("Final Gemini summary: {}", summary);
    println!("Final Gemini keywords: {:?}", keywords);

    // Now `summary` and `keywords` contain the valid data

    let new_image = image::ActiveModel {
        uuid: Set(uuid),
        name: Set(form.json.name.clone()),
        path: Set(target_path.to_string_lossy().to_string()),
        summary: Set(summary.clone()), // Use the obtained summary
        ..Default::default()
    };

    match new_image.insert(db.get_ref()).await {
        Ok(model) => {
            println!("Inserted image with UUID: {}", model.uuid);

            // 🔍 Index in Elasticsearch
            let doc = ImageIndex {
                uuid: model.uuid.to_string(),
                name: model.name.clone(),
                summary: model.summary.clone(),
                keywords: keywords.clone(),
            };

            let index_res = es
                .index(IndexParts::IndexId("papers", &model.uuid.to_string()))
                .body(json!(doc))
                .send()
                .await;

            if let Err(e) = index_res {
                println!("Elasticsearch indexing failed: {}", e);
                // Optionally continue, or return error
            }
            HttpResponse::Ok().json(json!({
                "summary": summary,
                "uuid": uuid.to_string()
            }))
        }
        Err(e) => {
            println!("Database insertion failed: {}", e);
            HttpResponse::InternalServerError().body(format!("Database insertion failed: {}", e))
        }
    }
}
