use actix_web::{delete, get, patch, post, web, HttpMessage, HttpRequest, HttpResponse, Responder};
use futures_util::StreamExt;
use mongodb::{Database, bson::doc};
use std::fs;
use std::path::PathBuf;
use crate::middleware::auth::Claims;
use base64; 
// Helper para obtener la ruta absoluta de la carpeta "images"
// Se asume que la carpeta "images" está al mismo nivel que "src".
fn images_directory() -> PathBuf {
    let mut path = std::env::current_dir().unwrap();
    path.push("images");
    path
}

#[get("/images/{id}")]
pub async fn get_image_handler(path: web::Path<String>, req: HttpRequest) -> impl Responder {
    // Verificar autenticación.
    if req.extensions().get::<Claims>().is_none() {
        return HttpResponse::Unauthorized().body("Token no encontrado");
    }
    let image_id = path.into_inner();
    let mut file_path = images_directory();
    file_path.push(&image_id);
    match fs::read(&file_path) {
        Ok(data) => HttpResponse::Ok()
                        .content_type("application/octet-stream")
                        .body(data),
        Err(_) => HttpResponse::NotFound().body("Imagen no encontrada"),
    }
}

#[derive(serde::Deserialize)]
pub struct NewImage {
    pub id: String,
    pub data: String, // imagen en base64
}

#[post("/images")]
pub async fn post_image_handler(
    db: web::Data<Database>, 
    new_image: web::Json<NewImage>, 
    req: HttpRequest
) -> impl Responder {
    // Verificar autenticación.
    if req.extensions().get::<Claims>().is_none() {
        return HttpResponse::Unauthorized().body("Token no encontrado");
    }
    let image_id = new_image.id.clone();
    let decoded_bytes = match base64::decode(&new_image.data) {
        Ok(bytes) => bytes,
        Err(_) => return HttpResponse::BadRequest().body("Error decodificando la imagen"),
    };
    // Asegurar que la carpeta exista.
    let img_dir = images_directory();
    let _ = fs::create_dir_all(&img_dir);
    let mut file_path = img_dir;
    file_path.push(&image_id);
    if let Err(_) = fs::write(&file_path, &decoded_bytes) {
        return HttpResponse::InternalServerError().body("Error al guardar la imagen");
    }
    // Actualizar el campo pictureUrl del item.
    let items = db.collection::<mongodb::bson::Document>("items");
    let image_url = format!("/images/{}", image_id);
    let _ = items
        .update_one(doc! {"_id": image_id.clone()}, doc! {"$set": {"pictureUrl": image_url.clone()}})
        .await;
    HttpResponse::Ok().body("Imagen guardada y item actualizado")
}

#[patch("/images/{id}")]
pub async fn patch_image_handler(db: web::Data<Database>, path: web::Path<String>, mut payload: web::Payload, req: HttpRequest) -> impl Responder {
    // Verificar autenticación.
    if req.extensions().get::<Claims>().is_none() {
        return HttpResponse::Unauthorized().body("Token no encontrado");
    }
    let image_id = path.into_inner();
    let mut bytes = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        match chunk {
            Ok(data) => bytes.extend_from_slice(&data),
            Err(_) => return HttpResponse::BadRequest().body("Error leyendo datos"),
        }
    }
    let mut file_path = images_directory();
    file_path.push(&image_id);
    if !file_path.exists() {
        return HttpResponse::NotFound().body("Imagen no existente para actualizar");
    }
    if let Err(_) = fs::write(&file_path, &bytes) {
        return HttpResponse::InternalServerError().body("Error al actualizar la imagen");
    }
    // Actualizar el campo pictureUrl del item.
    let items = db.collection::<mongodb::bson::Document>("items");
    let image_url = format!("/images/{}", image_id);
    let _ = items.update_one(doc! {"_id": image_id.clone()}, doc! {"$set": {"pictureUrl": image_url.clone()}}).await;
    HttpResponse::Ok().body("Imagen actualizada y item modificado")
}

#[delete("/images/{id}")]
pub async fn delete_image_handler(db: web::Data<Database>, path: web::Path<String>, req: HttpRequest) -> impl Responder {
    // Verificar autenticación.
    if req.extensions().get::<Claims>().is_none() {
        return HttpResponse::Unauthorized().body("Token no encontrado");
    }
    let image_id = path.into_inner();
    let mut file_path = images_directory();
    file_path.push(&image_id);
    if file_path.exists() {
        if let Err(_) = fs::remove_file(&file_path) {
            return HttpResponse::InternalServerError().body("Error al eliminar la imagen");
        }
    } else {
        return HttpResponse::NotFound().body("Imagen no encontrada");
    }
    // Actualizar el item removiendo el campo pictureUrl.
    let items = db.collection::<mongodb::bson::Document>("items");
    let _ = items.update_one(doc! {"_id": image_id.clone()}, doc! {"$unset": {"pictureUrl": ""}}).await;
    HttpResponse::Ok().body("Imagen eliminada y item actualizado")
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_image_handler)
       .service(post_image_handler)
       .service(patch_image_handler)
       .service(delete_image_handler);
}
