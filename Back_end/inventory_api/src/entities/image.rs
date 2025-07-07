use actix_multipart::Multipart;
use actix_web::{delete, get, patch, post, web, HttpResponse, Responder};
use futures_util::StreamExt;
use mongodb::bson::{doc, oid::ObjectId};
use mongodb::Database;
use std::fs;
use std::path::Path;

use crate::log::write_log;

#[get("/image/{filename}")]
pub async fn get_image_by_name_handler(path: web::Path<String>) -> impl Responder {
    let filename = path.into_inner();
    let file_path = Path::new("images").join(&filename);
    if !file_path.exists() {
        write_log(&format!(
            "GET /image/{{filename}} - Imagen no encontrada: {}",
            filename
        ))
        .ok();
        return HttpResponse::NotFound().body("Imagen no encontrada");
    }
    let file_data = match fs::read(&file_path) {
        Ok(data) => data,
        Err(_) => {
            write_log(&format!(
                "GET /image/{{filename}} - Error al leer la imagen: {}",
                filename
            ))
            .ok();
            return HttpResponse::InternalServerError().body("Error al leer la imagen");
        }
    };
    write_log(&format!(
        "GET /image/{{filename}} - Imagen servida correctamente: {} ({} bytes)",
        filename,
        file_data.len()
    ))
    .ok();
    HttpResponse::Ok()
        .content_type("image/png")
        .append_header((
            "Content-Disposition",
            format!("inline; filename=\"{}\"", filename),
        ))
        .body(file_data)
}

#[post("/image")]
pub async fn post_image_handler(mut payload: Multipart, db: web::Data<Database>) -> impl Responder {
    let mut object_id: Option<String> = None;
    let mut file_bytes: Option<bytes::BytesMut> = None;
    while let Some(item) = payload.next().await {
        let mut field = match item {
            Ok(f) => f,
            Err(_) => {
                write_log("POST /image - Error procesando multipart").ok();
                return HttpResponse::BadRequest().body("Error procesando multipart");
            }
        };
        match field.name() {
            Some("objectID") => {
                let mut data = Vec::new();
                while let Some(chunk) = field.next().await {
                    let bytes = match chunk {
                        Ok(d) => d,
                        Err(_) => {
                            write_log("POST /image - Error leyendo objectID").ok();
                            return HttpResponse::BadRequest().body("Error leyendo objectID");
                        }
                    };
                    data.extend_from_slice(&bytes);
                }
                object_id = Some(String::from_utf8_lossy(&data).to_string());
            }
            Some("file") => {
                let mut bytes_mut = bytes::BytesMut::new();
                while let Some(chunk) = field.next().await {
                    let data = match chunk {
                        Ok(d) => d,
                        Err(_) => {
                            write_log("POST /image - Error leyendo archivo").ok();
                            return HttpResponse::BadRequest().body("Error leyendo archivo");
                        }
                    };
                    bytes_mut.extend_from_slice(&data);
                }
                file_bytes = Some(bytes_mut);
            }
            Some(_) => continue,
            None => continue,
        }
    }
    let file_data = match file_bytes {
        Some(data) => data,
        None => {
            write_log("POST /image - No se recibió archivo").ok();
            return HttpResponse::BadRequest().body("No se recibió archivo");
        }
    };
    let oid_str = match object_id {
        Some(id) => id,
        None => {
            write_log("POST /image - No se recibió objectID").ok();
            return HttpResponse::BadRequest().body("No se recibió objectID");
        }
    };
    let item_obj_id = match ObjectId::parse_str(&oid_str) {
        Ok(oid) => oid,
        Err(_) => {
            write_log(&format!("POST /image - objectID inválido: {}", oid_str)).ok();
            return HttpResponse::BadRequest().body("objectID inválido");
        }
    };
    let items_collection = db.collection::<mongodb::bson::Document>("items");
    let existing_item = match items_collection
        .find_one(doc! {"_id": item_obj_id.clone()})
        .await
    {
        Ok(Some(doc)) => doc,
        _ => {
            write_log(&format!("POST /image - Item no encontrado: {}", oid_str)).ok();
            return HttpResponse::BadRequest().body("Item no encontrado");
        }
    };
    // Eliminar imagen anterior si existe
    if let Some(old_pic) = existing_item.get_str("pictureUrl").ok() {
        let old_file_path = Path::new("images").join(old_pic);
        if old_file_path.exists() {
            let _ = fs::remove_file(old_file_path);
        }
    }
    let images_dir = Path::new("images");
    if !images_dir.exists() {
        if let Err(_) = fs::create_dir_all(&images_dir) {
            write_log("POST /image - Error creando directorio").ok();
            return HttpResponse::InternalServerError().body("Error creando directorio");
        }
    }
    let file_name = format!("{}.png", oid_str);
    let file_path = images_dir.join(&file_name);
    if let Err(_) = fs::write(&file_path, &file_data) {
        write_log(&format!(
            "POST /image - Error guardando archivo: {}",
            file_name
        ))
        .ok();
        return HttpResponse::InternalServerError().body("Error guardando archivo");
    }
    if let Err(_) = items_collection
        .update_one(
            doc! {"_id": item_obj_id},
            doc! { "$set": { "pictureUrl": file_name.clone() } },
        )
        .await
    {
        write_log(&format!(
            "POST /image - Error actualizando item: {}",
            file_name
        ))
        .ok();
        return HttpResponse::InternalServerError().body("Error actualizando item");
    }
    write_log(&format!(
        "POST /image - Imagen actualizada correctamente: {} ({} bytes)",
        file_name,
        file_data.len()
    ))
    .ok();
    HttpResponse::Ok().body("Imagen actualizada")
}

// #[patch("/image/{id}")]
// pub async fn patch_image_handler(
//     path: web::Path<String>,
//     mut payload: Multipart,
//     db: web::Data<Database>,
// ) -> impl Responder {
//     let oid_str = path.into_inner();
//     let item_obj_id = match ObjectId::parse_str(&oid_str) {
//         Ok(oid) => oid,
//         Err(_) => {
//             write_log(&format!("PATCH /image/{{id}} - ID inválido: {}", oid_str)).ok();
//             return HttpResponse::BadRequest().body("ID inválido");
//         },
//     };
//     let mut file_bytes: Option<bytes::BytesMut> = None;
//     while let Some(item) = payload.next().await {
//         let mut field = match item {
//             Ok(f) => f,
//             Err(_) => {
//                 write_log("PATCH /image/{id} - Error procesando multipart").ok();
//                 return HttpResponse::BadRequest().body("Error procesando multipart");
//             },
//         };
//         if let Some("file") = field.name() {
//             let mut bytes_mut = bytes::BytesMut::new();
//             while let Some(chunk) = field.next().await {
//                 let data = match chunk {
//                     Ok(d) => d,
//                     Err(_) => {
//                         write_log("PATCH /image/{id} - Error leyendo archivo").ok();
//                         return HttpResponse::BadRequest().body("Error leyendo archivo");
//                     },
//                 };
//                 bytes_mut.extend_from_slice(&data);
//             }
//             file_bytes = Some(bytes_mut);
//         }
//     }
//     let file_data = match file_bytes {
//         Some(data) => data,
//         None => {
//             write_log("PATCH /image/{id} - No se recibió archivo").ok();
//             return HttpResponse::BadRequest().body("No se recibió archivo");
//         },
//     };
//     let items_collection = db.collection::<mongodb::bson::Document>("items");
//     let existing_item = match items_collection
//         .find_one(doc! {"_id": item_obj_id.clone()})
//         .await
//     {
//         Ok(Some(doc)) => doc,
//         _ => {
//             write_log(&format!("PATCH /image/{{id}} - Item no encontrado: {}", oid_str)).ok();
//             return HttpResponse::BadRequest().body("Item no encontrado");
//         },
//     };
//     if let Some(old_pic) = existing_item.get_str("pictureUrl").ok() {
//         let old_file_path = Path::new("images").join(old_pic);
//         if old_file_path.exists() {
//             let _ = fs::remove_file(old_file_path);
//         }
//     }
//     let images_dir = Path::new("images");
//     if !images_dir.exists() {
//         if let Err(_) = fs::create_dir_all(&images_dir) {
//             write_log("PATCH /image/{id} - Error creando directorio").ok();
//             return HttpResponse::InternalServerError().body("Error creando directorio");
//         }
//     }
//     let file_name = format!("{}.png", oid_str);
//     let file_path = images_dir.join(&file_name);
//     if let Err(_) = fs::write(&file_path, &file_data) {
//         write_log(&format!("PATCH /image/{{id}} - Error guardando archivo: {}", file_name)).ok();
//         return HttpResponse::InternalServerError().body("Error guardando archivo");
//     }
//     if let Err(_) = items_collection
//         .update_one(
//             doc! {"_id": item_obj_id},
//             doc! { "$set": { "pictureUrl": file_name.clone() } },
//         )
//         .await
//     {
//         write_log(&format!("PATCH /image/{{id}} - Error actualizando item: {}", file_name)).ok();
//         return HttpResponse::InternalServerError().body("Error actualizando item");
//     }
//     write_log(&format!("PATCH /image/{{id}} - Imagen actualizada correctamente: {} ({} bytes)", file_name, file_data.len())).ok();
//     HttpResponse::Ok().body("Imagen actualizada")
// }

// #[delete("/image/{id}")]
// pub async fn delete_image_handler(path: web::Path<String>, db: web::Data<Database>) -> impl Responder {
//     let oid_str = path.into_inner();
//     let item_obj_id = match ObjectId::parse_str(&oid_str) {
//         Ok(oid) => oid,
//         Err(_) => {
//             write_log(&format!("DELETE /image/{{id}} - ID inválido: {}", oid_str)).ok();
//             return HttpResponse::BadRequest().body("ID inválido");
//         },
//     };
//     let items_collection = db.collection::<mongodb::bson::Document>("items");
//     let existing_item = match items_collection.find_one(doc! {"_id": item_obj_id.clone()}).await {
//         Ok(Some(doc)) => doc,
//         _ => {
//             write_log(&format!("DELETE /image/{{id}} - Item no encontrado: {}", oid_str)).ok();
//             return HttpResponse::BadRequest().body("Item no encontrado");
//         },
//     };
//     if let Some(old_pic) = existing_item.get_str("pictureUrl").ok() {
//         let old_file_path = Path::new("images").join(old_pic);
//         if old_file_path.exists() {
//             let _ = fs::remove_file(old_file_path);
//         }
//     }
//     if let Err(_) = items_collection.update_one(
//         doc! {"_id": item_obj_id},
//         doc! { "$unset": { "pictureUrl": "" } }
//     ).await {
//         write_log(&format!("DELETE /image/{{id}} - Error actualizando item: {}", oid_str)).ok();
//         return HttpResponse::InternalServerError().body("Error actualizando item");
//     }
//     write_log(&format!("DELETE /image/{{id}} - Imagen eliminada correctamente para item {}", oid_str)).ok();
//     HttpResponse::Ok().body("Imagen eliminada")
// }

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_image_by_name_handler);
    cfg.service(post_image_handler);
    //cfg.service(patch_image_handler);
    //cfg.service(delete_image_handler);
}
