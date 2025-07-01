use actix_web::{delete, get, patch, post, web, HttpMessage, HttpRequest, HttpResponse, Responder};
use futures_util::stream::TryStreamExt;
use mongodb::{
    bson::{doc, oid::ObjectId, Document},
    Database,
};
use serde::{Deserialize, Serialize};
use crate::log::write_log;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "pictureUrl", skip_serializing_if = "Option::is_none")]
    pub picture_url: Option<String>,
    #[serde(rename = "zoneId")]
    pub zone_id: ObjectId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
}

// impl Item {
//     fn new(
//         name: String,
//         description: Option<String>,
//         picture_url: Option<String>,
//         zone_id: ObjectId,
//         tags: Option<Vec<String>>,
//     ) -> Item {
//         Self {
//             id: None,
//             name,
//             description,
//             picture_url,
//             zone_id,
//             tags,
//         }
//     }
// }

#[get("/items")]
async fn get_items_handler(
    db: web::Data<Database>,
    req: HttpRequest,
) -> impl Responder {
    let claims = match req.extensions().get::<crate::middleware::auth::Claims>().cloned() {
        Some(claims) => claims,
        None => {
            write_log("GET /items - Token no encontrado").ok();
            return HttpResponse::Unauthorized().body("Token no encontrado");
        },
    };
    if claims.role != "admin" {
        write_log(&format!("GET /items - Acceso denegado para usuario {}", claims.sub)).ok();
        return HttpResponse::Unauthorized().body("Acceso no autorizado: se requiere administrador");
    }
    let collection = db.collection::<Item>("items");
    let cursor = match collection.find(doc! {}).await {
        Ok(cursor) => cursor,
        Err(e) => {
            write_log(&format!("GET /items - Error en find para usuario {}: {}", claims.sub, e)).ok();
            return HttpResponse::InternalServerError().body(e.to_string());
        },
    };
    let items: Vec<Item> = match cursor.try_collect().await {
        Ok(items) => items,
        Err(e) => {
            write_log(&format!("GET /items - Error en try_collect para usuario {}: {}", claims.sub, e)).ok();
            return HttpResponse::InternalServerError().body(e.to_string());
        },
    };
    write_log(&format!("GET /items - usuario {} obtuvo {} items", claims.sub, items.len())).ok();
    HttpResponse::Ok().json(items)
}

#[get("/items/{id}")]
async fn get_item_handler(db: web::Data<Database>, path: web::Path<String>) -> impl Responder {
    let collection = db.collection::<Item>("items");
    let id_str = path.into_inner();
    let obj_id = match ObjectId::parse_str(&id_str) {
        Ok(obj_id) => obj_id,
        Err(_) => {
            write_log(&format!("GET /items/{{id}} - ID inválido: {}", id_str)).ok();
            return HttpResponse::BadRequest().body("ID inválido");
        },
    };
    match collection.find_one(doc! {"_id":obj_id}).await {
        Ok(Some(item)) => {
            write_log(&format!("GET /items/{{id}} - Item encontrado: {:?}", item)).ok();
            HttpResponse::Ok().json(item)
        },
        Ok(None) => {
            write_log(&format!("GET /items/{{id}} - Objeto no encontrado: {}", obj_id)).ok();
            HttpResponse::NotFound().body("Objeto no encontrado")
        },
        Err(e) => {
            write_log(&format!("GET /items/{{id}} - Error: {}", e)).ok();
            HttpResponse::BadRequest().body(e.to_string())
        },
    }
}

#[get("/items/zone/{id}")]
async fn get_items_from_zone_handler(
    db: web::Data<Database>,
    path: web::Path<String>,
    req: HttpRequest,
) -> impl Responder {
    let claims = match req.extensions().get::<crate::middleware::auth::Claims>().cloned() {
        Some(claims) => claims,
        None => {
            write_log("GET /items/zone/{id} - Token no encontrado").ok();
            return HttpResponse::Unauthorized().body("Token no encontrado");
        },
    };

    let zone_id_str = path.into_inner();
    let zone_id = match ObjectId::parse_str(&zone_id_str) {
        Ok(id) => id,
        Err(_) => {
            write_log(&format!("GET /items/zone/{{id}} - ID inválido: {}", zone_id_str)).ok();
            return HttpResponse::BadRequest().body("ID inválido");
        },
    };
    let token_user_obj_id = match ObjectId::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => {
            write_log(&format!("GET /items/zone/{{id}} - Token user ID inválido: {}", claims.sub)).ok();
            return HttpResponse::BadRequest().body("Token user ID inválido");
        },
    };
    let items_collection = db.collection::<Item>("items");
    let filter = if claims.role == "admin" {
        doc! { "zoneId": zone_id }
    } else {
        doc! {
            "zoneId": zone_id,
            "$or": [
                { "userId": { "$exists": false } },
                { "userId": token_user_obj_id }
            ]
        }
    };
    let cursor = match items_collection.find(filter).await {
        Ok(cursor) => cursor,
        Err(_) => {
            write_log(&format!("GET /items/zone/{{id}} - Error inesperado para usuario {} y zona {}", claims.sub, zone_id)).ok();
            return HttpResponse::BadRequest().body("Error inesperado, inténtelo  nuevamente");
        },
    };
    let items: Vec<Item> = match cursor.try_collect().await {
        Ok(items) => items,
        Err(_) => {
            write_log(&format!("GET /items/zone/{{id}} - Error inesperado para usuario {} y zona {}", claims.sub, zone_id)).ok();
            return HttpResponse::BadRequest().body("Error inesperado, inténtelo  nuevamente");
        },
    };
    write_log(&format!("GET /items/zone/{{id}} - usuario {} obtuvo {} items de la zona {}", claims.sub, items.len(), zone_id)).ok();
    HttpResponse::Ok().json(items)
}

#[post("/items")]
async fn create_item_handler(db: web::Data<Database>, new_item: web::Json<Item>) -> impl Responder {
    let collection = db.collection::<Item>("items");
    let mut item = new_item.into_inner();
    item.id = None;
    match collection.insert_one(item).await {
        Ok(result) => {
            write_log(&format!("POST /items - Item creado correctamente: {:?}", result.inserted_id)).ok();
            HttpResponse::Ok().json(result.inserted_id)
        },
        Err(_) => {
            write_log("POST /items - Error inesperado al crear item").ok();
            HttpResponse::BadRequest().body("Error inesperado, inténtelo  nuevamente")
        },
    }
}

#[patch("/items/{id}")]
async fn patch_item_handler(
    db: web::Data<Database>,
    path: web::Path<String>,
    updated_item: web::Json<serde_json::Value>,
) -> impl Responder {
    let collection = db.collection::<Item>("items");
    let id_str = path.into_inner();
    let obj_id = match ObjectId::parse_str(&id_str) {
        Ok(id) => id,
        Err(_) => {
            write_log(&format!("PATCH /items/{{id}} - ID inválido: {}", id_str)).ok();
            return HttpResponse::BadRequest().body("ID inválido");
        },
    };
    let mut set_doc = Document::new();
    let mut unset_doc = Document::new();
    if let Some(value) = updated_item.get("name") {
        match value {
            serde_json::Value::String(name) => set_doc.insert("name", name.clone()),
            serde_json::Value::Null => return HttpResponse::BadRequest().body("'name' no puede ser null"),
            _ => return HttpResponse::BadRequest().body("Valor inválido para 'name'"),
        };
    }
    if let Some(value) = updated_item.get("description") {
        match value {
            serde_json::Value::String(desc) => set_doc.insert("description", desc.clone()),
            serde_json::Value::Null => unset_doc.insert("description", ""),
            _ => return HttpResponse::BadRequest().body("Valor inválido para 'description'"),
        };
    }
    if let Some(value) = updated_item.get("pictureUrl") {
        match value {
            serde_json::Value::String(url) => set_doc.insert("pictureUrl", url.clone()),
            serde_json::Value::Null => unset_doc.insert("pictureUrl", ""),
            _ => return HttpResponse::BadRequest().body("Valor inválido para 'pictureUrl'"),
        };
    }
    if let Some(value) = updated_item.get("tags") {
        match value {
            serde_json::Value::Array(tags) => {
                let string_tags: Vec<String> = tags
                    .iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect();
                set_doc.insert("tags", string_tags)
            },
            serde_json::Value::Null => unset_doc.insert("tags", ""),
            _ => return HttpResponse::BadRequest().body("Valor inválido para 'tags'"),
        };
    }
    if set_doc.is_empty() && unset_doc.is_empty() {
        return HttpResponse::BadRequest().body("No hay campos para actualizar");
    }
    let mut update_doc = Document::new();
    if !set_doc.is_empty() {
        update_doc.insert("$set", set_doc);
    }
    if !unset_doc.is_empty() {
        update_doc.insert("$unset", unset_doc);
    }
    match collection
        .update_one(doc! {"_id": obj_id}, update_doc)
        .await
    {
        Ok(result) if result.matched_count == 1 => {
            write_log(&format!("PATCH /items/{{id}} - Objeto actualizado: {}", obj_id)).ok();
            HttpResponse::Ok().body("Objeto actualizado")
        },
        Ok(_) => {
            write_log(&format!("PATCH /items/{{id}} - Objeto no encontrado: {}", obj_id)).ok();
            HttpResponse::NotFound().body("Objeto no encontrado")
        },
        Err(_) => {
            write_log(&format!("PATCH /items/{{id}} - Error inesperado al actualizar objeto: {}", obj_id)).ok();
            HttpResponse::BadRequest().body("Error inesperado, inténtelo  nuevamente")
        },
    }
}


pub async fn delete_item(db: &Database, item_id: String) -> HttpResponse {
    use std::path::Path;
    use std::fs;
    let collection = db.collection::<Item>("items");
    let obj_id = match ObjectId::parse_str(item_id.clone()) {
        Ok(id) => id,
        Err(_) => {
            write_log(&format!("DELETE /items/{{id}} - Id incorrecto: {}", item_id)).ok();
            return HttpResponse::BadRequest().body("Id incorrecto");
        },
    };
    // Buscar el item antes de eliminarlo
    let item = match collection.find_one(doc! {"_id": obj_id.clone()}).await {
        Ok(Some(item)) => item,
        Ok(None) => {
            write_log(&format!("DELETE /items/{{id}} - Item no encontrado: {}", item_id)).ok();
            return HttpResponse::NotFound().body("Item no encontrado");
        },
        Err(e) => {
            write_log(&format!("DELETE /items/{{id}} - Error buscando item: {}: {}", item_id, e)).ok();
            return HttpResponse::InternalServerError().body("Error buscando item");
        },
    };
    // Si tiene imagen, eliminar el archivo
    if let Some(picture_url) = &item.picture_url {
        let image_path = Path::new("images").join(picture_url);
        if image_path.exists() {
            if let Err(e) = fs::remove_file(&image_path) {
                write_log(&format!("DELETE /items/{{id}} - Error eliminando imagen '{}': {}", picture_url, e)).ok();
                // No retornamos error, solo lo registramos
            } else {
                write_log(&format!("DELETE /items/{{id}} - Imagen '{}' eliminada", picture_url)).ok();
            }
        }
    }
    // Eliminar el item de la base de datos
    match collection.delete_one(doc! {"_id": obj_id}).await {
        Ok(_) => {
            write_log(&format!("DELETE /items/{{id}} - Item eliminado: {}", item_id)).ok();
            HttpResponse::Ok().body("Item Eliminado")
        },
        Err(_) => {
            write_log(&format!("DELETE /items/{{id}} - Error inesperado al eliminar item: {}", item_id)).ok();
            HttpResponse::BadRequest().body("Error inesperado, inténtelo  nuevamente")
        },
    }
}

#[delete("/items/{id}")]
async fn delete_item_handler(db: web::Data<Database>, path: web::Path<String>) -> impl Responder {
    let item_id = path.into_inner();
    let client = db.client();
    let mut session = match client.start_session().await {
        Ok(s) => s,
        Err(_) => {
            write_log("DELETE /items/{id} - Error inesperado al iniciar sesión").ok();
            return HttpResponse::BadRequest().body("Error inesperado, inténtelo  nuevamente");
        },
    };
    session.start_transaction().await.ok();
    let response = delete_item(&db, item_id.clone()).await;
    if response.status().is_success() {
        session.commit_transaction().await.ok();
        write_log(&format!("DELETE /items/{{id}} - Item {} eliminado correctamente", item_id)).ok();
    } else {
        session.abort_transaction().await.ok();
        write_log(&format!("DELETE /items/{{id}} - Error al eliminar item {}", item_id)).ok();
    }
    response
}
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_item_handler)
        .service(get_items_handler)
        .service(get_items_from_zone_handler)
        .service(create_item_handler)
        .service(patch_item_handler)
        .service(delete_item_handler);
}
