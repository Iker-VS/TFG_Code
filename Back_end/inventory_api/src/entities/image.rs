use actix_web::{get, post, web, HttpResponse, Responder};
use std::fs;
use std::path::Path;
use actix_multipart::Multipart;
use futures_util::StreamExt;
use mongodb::Database;
use mongodb::bson::{doc, oid::ObjectId};

use crate::entities::item::Item;


#[get("/image/{filename}")]
pub async fn get_image_by_name_handler(path: web::Path<String>) -> impl Responder {
    let filename = path.into_inner();
    // La carpeta "images" está al mismo nivel que "src"
    let file_path = Path::new("images").join(&filename);
    if !file_path.exists() {
        return HttpResponse::NotFound().body("Imagen no encontrada");
    }
    let file_data = match fs::read(&file_path) {
        Ok(data) => data,
        Err(_) => return HttpResponse::InternalServerError().body("Error al leer la imagen"),
    };
    HttpResponse::Ok()
        .content_type("image/png")
        .append_header(("Content-Disposition", format!("inline; filename=\"{}\"", filename)))
        .body(file_data)
}

#[post("/image")]
pub async fn post_image_handler(mut payload: Multipart, db: web::Data<Database>) -> impl Responder {
    // Declaramos variables para objectID y archivo
    let mut object_id: Option<String> = None;
    let mut file_bytes: Option<bytes::BytesMut> = None;
    while let Some(item) = payload.next().await {
        let mut field = match item {
            Ok(f) => f,
            Err(_) => return HttpResponse::BadRequest().body("Error procesando multipart"),
        };
        match field.name() {
            Some("objectID") => {
                let mut data = Vec::new();
                while let Some(chunk) = field.next().await {
                    let bytes = match chunk {
                        Ok(d) => d,
                        Err(_) => return HttpResponse::BadRequest().body("Error leyendo objectID"),
                    };
                    data.extend_from_slice(&bytes);
                }
                object_id = Some(String::from_utf8_lossy(&data).to_string());
            },
            Some("file") => {
                let mut bytes_mut = bytes::BytesMut::new();
                while let Some(chunk) = field.next().await {
                    let data = match chunk {
                        Ok(d) => d,
                        Err(_) => return HttpResponse::BadRequest().body("Error leyendo archivo"),
                    };
                    bytes_mut.extend_from_slice(&data);
                }
                file_bytes = Some(bytes_mut);
            },
            _ => {
                // ...existing code...
            }
        }
    }
    // Validar recepción de archivo y objectID
    let file_data = match file_bytes {
        Some(data) => data,
        None => return HttpResponse::BadRequest().body("No se recibió archivo"),
    };
    let oid_str = match object_id {
        Some(id) => id,
        None => return HttpResponse::BadRequest().body("No se recibió objectID"),
    };
    let item_obj_id = match ObjectId::parse_str(&oid_str) {
        Ok(oid) => oid,
        Err(_) => return HttpResponse::BadRequest().body("objectID inválido"),
    };

    // Comprobar que el item existe
    let items_collection = db.collection::<Item>("items");
    match items_collection.find_one(doc! {"_id": item_obj_id.clone()}).await {
        Ok(Some(_)) => { /* Item encontrado */ },
        _ => return HttpResponse::BadRequest().body("Item no encontrado"),
    }

    // Guardar la imagen
    let images_dir = Path::new("images");
    if !images_dir.exists() {
        if let Err(_) = fs::create_dir_all(&images_dir) {
            return HttpResponse::InternalServerError().body("Error creando directorio");
        }
    }
    let file_name = format!("{}.png", oid_str);
    let file_path = images_dir.join(&file_name);
    if let Err(_) = fs::write(&file_path, &file_data) {
        return HttpResponse::InternalServerError().body("Error guardando archivo");
    }

    // Actualizar el campo pictureUrl del item
    if let Err(_) = items_collection.update_one(
        doc! {"_id": item_obj_id},
        doc! { "$set": { "pictureUrl": file_name } }
    ).await {
        return HttpResponse::InternalServerError().body("Error actualizando item");
    }
    
    HttpResponse::Ok().body("Imagen guardada y item actualizado exitosamente")
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_image_by_name_handler);
    cfg.service(post_image_handler);
}