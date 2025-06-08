use crate::entities::item::{delete_item, Item};
use actix_web::{delete, get, patch, post, web, HttpMessage, HttpRequest, HttpResponse, Responder};
use futures_util::stream::TryStreamExt;
use mongodb::{
    bson::{doc, oid::ObjectId, Document},
    Database,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Zone {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    #[serde(rename = "propertyId")]
    pub property_id: ObjectId,
    #[serde(rename = "userId", skip_serializing_if = "Option::is_none")]
    pub user_id: Option<ObjectId>,
    #[serde(rename = "parentZoneId", skip_serializing_if = "Option::is_none")]
    pub parent_zone_id: Option<ObjectId>,
}

// impl Zone {
//     fn new(
//         name: String,
//         property_id: ObjectId,
//         user_id: Option<ObjectId>,
//         parent_zone_id: Option<ObjectId>,
//     ) -> Zone {
//         Self {
//             id: None,
//             name,
//             property_id,
//             user_id,
//             parent_zone_id,
//         }
//     }
// }

#[get("/zones")]
async fn get_zones_handler(
    db: web::Data<Database>,
    req: HttpRequest,
) -> impl Responder {
    // Recupera las claims inyectadas por el middleware
    let claims = match req.extensions().get::<crate::middleware::auth::Claims>().cloned() {
        Some(claims) => claims,
        None => return HttpResponse::Unauthorized().body("Token no encontrado"),
    };

    // Solo el admin puede obtener todas las zonas
    if claims.role != "admin" {
        return HttpResponse::Unauthorized().body("Acceso no autorizado: se requiere administrador");
    }

    let collection = db.collection::<Zone>("zones");
    let cursor = match collection.find(doc! {}).await {
        Ok(cursor) => cursor,
        Err(_) => return HttpResponse::BadRequest().body("Error inesperado, inténtelo  nuevamente"),
    };
    let zones: Vec<Zone> = match cursor.try_collect().await {
        Ok(zones) => zones,
        Err(_) => return HttpResponse::BadRequest().body("Error inesperado, inténtelo  nuevamente"),
    };
    HttpResponse::Ok().json(zones)
}

#[get("/zones/{id}")]
async fn get_zone_handler(db: web::Data<Database>, path: web::Path<String>) -> impl Responder {
    let collection = db.collection::<Zone>("zones");
    let obj_id = match ObjectId::parse_str(&path.into_inner()) {
        Ok(obj_id) => obj_id,
        Err(_) => return HttpResponse::BadRequest().body("ID inválido"),
    };
    match collection.find_one(doc! {"_id":obj_id}).await {
        Ok(Some(zone)) => return HttpResponse::Ok().json(zone),
        Ok(None) => return HttpResponse::NotFound().body("zona no encontrada"),
        Err(e) => HttpResponse::BadRequest().body(e.to_string()),
    }
}

#[get("/zones/parent/{id}")]
async fn get_zone_from_parent_handler(
    db: web::Data<Database>,
    path: web::Path<String>
) -> impl Responder {

    // Parsear parent_id desde la ruta
    let parent_id = match ObjectId::parse_str(&path.into_inner()) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("ID inválido"),
    };

    // Buscar todas las zonas cuyo parentZoneId sea igual a parent_id
    let zone_collection = db.collection::<Zone>("zones");
    let zone_cursor = match zone_collection.find(doc! { "parentZoneId": parent_id }).await {
        Ok(cursor) => cursor,
        Err(_) => return HttpResponse::BadRequest().body("Error al obtener zonas"),
    };
    let zones: Vec<Zone> = match zone_cursor.try_collect().await {
        Ok(zones) => zones,
        Err(_) => return HttpResponse::BadRequest().body("Error al procesar zonas"),
    };

    // Obtener los ítems de todas las zonas encontradas
    let child_zone_ids: Vec<_> = zones.iter().filter_map(|z| z.id.clone()).collect();
    let items: Vec<crate::entities::item::Item> = if child_zone_ids.is_empty() {
        vec![]
    } else {
        let items_collection = db.collection::<crate::entities::item::Item>("items");
        let items_cursor = match items_collection.find(doc! { "zoneId": { "$in": child_zone_ids } }).await {
            Ok(cursor) => cursor,
            Err(_) => return HttpResponse::BadRequest().body("Error al obtener ítems"),
        };
        match items_cursor.try_collect().await {
            Ok(items) => items,
            Err(_) => return HttpResponse::BadRequest().body("Error al procesar ítems"),
        }
    };

    // Retornar zonas e ítems en la respuesta
    HttpResponse::Ok().json(serde_json::json!({
        "zones": zones,
        "items": items
    }))
}

#[post("/zones")]
async fn create_zone_handler(
    db: web::Data<Database>,
    new_zone: web::Json<serde_json::Value>,
    req: HttpRequest,
) -> impl Responder {
    // Extraer claims del request
    let claims = match req.extensions().get::<crate::middleware::auth::Claims>().cloned() {
        Some(claims) => claims,
        None => return HttpResponse::Unauthorized().body("Token no encontrado"),
    };

    // Extraer "name"
    let name = match new_zone.get("name") {
        Some(value) => match value.as_str() {
            Some(s) => s.to_string(),
            None => return HttpResponse::BadRequest().body("El nombre debe ser una cadena de texto"),
        },
        None => return HttpResponse::BadRequest().body("El nombre es requerido"),
    };

    // Extraer "parentZoneId" (obligatorio)
    let parent_zone_str = match new_zone.get("parentZoneId") {
        Some(value) => match value.as_str() {
            Some(s) => s.to_string(),
            None => return HttpResponse::BadRequest().body("parentZoneId debe ser una cadena de texto"),
        },
        None => return HttpResponse::BadRequest().body("parentZoneId es requerido"),
    };
    let parent_zone_id = match ObjectId::parse_str(&parent_zone_str) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("parentZoneId inválido"),
    };

    // Determinar property_id:
    // Primero, buscar en la colección properties
    let property_collection = db.collection::<crate::entities::property::Property>("properties");
    let property = property_collection.find_one(doc! {"_id": parent_zone_id.clone()}).await.ok().flatten();
    let property_id = if let Some(_) = property {
        // Si se encontró, se usa parent_zone_id como property_id
        parent_zone_id.clone()
    } else {
        // Sino, buscar en la colección zones
        let zone_collection = db.collection::<Zone>("zones");
        if let Ok(Some(zone)) = zone_collection.find_one(doc! {"_id": parent_zone_id.clone()}).await {
            zone.property_id
        } else {
            return HttpResponse::BadRequest().body("parentZoneId no corresponde a propiedad ni zona válida");
        }
    };

    let is_private = new_zone.get("private")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let user_id = if is_private {
        match ObjectId::parse_str(&claims.sub) {
            Ok(id) => Some(id),
            Err(_) => return HttpResponse::BadRequest().body("ID de usuario inválido"),
        }
    } else {
        None
    };

    let zone = Zone {
        id: None,
        name,
        property_id,
        parent_zone_id: Some(parent_zone_id),
        user_id,
    };

    let collection = db.collection::<Zone>("zones");
    match collection.insert_one(zone).await {
        Ok(result) => HttpResponse::Ok().json(result.inserted_id),
        Err(_) => HttpResponse::BadRequest().body("Error inesperado, inténtelo nuevamente"),
    }
}

#[patch("/zones/{id}")]
async fn patch_zones_handler(
    db: web::Data<Database>,
    path: web::Path<String>,
    updated_zone: web::Json<serde_json::Value>,
    req: HttpRequest,
) -> impl Responder {
    let claims = match req.extensions().get::<crate::middleware::auth::Claims>().cloned() {
        Some(claims) => claims,
        None => return HttpResponse::Unauthorized().body("Token no encontrado"),
    };

    let collection = db.collection::<Zone>("zones");
    let obj_id = match ObjectId::parse_str(&path.into_inner()) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("ID inválido"),
    };

    let mut set_doc = doc! {};
    let mut unset_doc = doc! {};

    // Campo name
    if let Some(value) = updated_zone.get("name") {
        match value {
            serde_json::Value::String(name) => { set_doc.insert("name", name.clone()); },
            _ => return HttpResponse::BadRequest().body("Valor inválido para 'name'"),
        }
    }

    // Campo private
    if let Some(value) = updated_zone.get("private") {
        match value {
            serde_json::Value::Bool(is_private) => {
                if *is_private {
                    match ObjectId::parse_str(&claims.sub) {
                        Ok(user_id) => { set_doc.insert("userId", user_id); },
                        Err(_) => return HttpResponse::BadRequest().body("ID de usuario inválido"),
                    }
                } else {
                    unset_doc.insert("userId", "");
                }
            },
            _ => return HttpResponse::BadRequest().body("Valor inválido para 'private'"),
        }
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

    match collection.update_one(doc! {"_id": obj_id}, update_doc).await {
        Ok(result) if result.matched_count == 1 => HttpResponse::Ok().body("Zona actualizada"),
        Ok(_) => HttpResponse::NotFound().body("Zona no encontrada"),
        Err(_) => HttpResponse::BadRequest().body("Error inesperado, inténtelo  nuevamente"),
    }
}

pub async fn delete_zone(db: &Database, zone_id: String) -> HttpResponse {
    let zone_collection = db.collection::<Zone>("zones");
    let item_collection = db.collection::<Item>("items");
    let obj_id = match ObjectId::parse_str(zone_id) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("Id incorrecto"),
    };
    let item_cursor = match item_collection.find(doc! {"zoneId":obj_id}).await {
        Ok(items) => items,
        Err(_) => return HttpResponse::BadRequest().body("Error inesperado, inténtelo  nuevamente"),
    };
    let items: Vec<Item> = match item_cursor.try_collect().await {
        Ok(items) => items,
        Err(_) => return HttpResponse::BadRequest().body("Error inesperado, inténtelo  nuevamente"),
    };
    for item in items {
        let id = match item.id {
            Some(id) => id,
            None => return HttpResponse::BadRequest().body("No hay ID"),
        };
        let res = delete_item(db, id.to_string()).await;
        if !res.status().is_success() {
            return res; // Si falla, detenemos la ejecución y devolvemos el error
        }
    }
    match zone_collection.delete_one(doc! {"_id": obj_id}).await {
        Ok(_) => HttpResponse::Ok().body("Zona eliminada"),
        Err(_) => HttpResponse::BadRequest().body("Error inesperado, inténtelo  nuevamente"),
    }
}

#[delete("/zones/{id}")]
async fn delete_zone_handler(db: web::Data<Database>, path: web::Path<String>) -> impl Responder {
    let client = db.client();
    let mut session = match client.start_session().await {
        Ok(s) => s,
        Err(_) => return HttpResponse::BadRequest().body("Error inesperado, inténtelo  nuevamente"),
    };
    session.start_transaction().await.ok();
    let response = delete_zone(&db, path.into_inner()).await;

    if response.status().is_success() {
        session.commit_transaction().await.ok();
    } else {
        session.abort_transaction().await.ok();
    }

    response
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_zone_handler)
        .service(get_zones_handler)
        .service(get_zone_from_parent_handler)
        .service(create_zone_handler)
        .service(patch_zones_handler)
        .service(delete_zone_handler);
}
