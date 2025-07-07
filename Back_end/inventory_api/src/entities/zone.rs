use crate::entities::item::{delete_item, Item};
use crate::log::write_log;
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

//deprecated
// #[get("/zones")]
// async fn get_zones_handler(
//     db: web::Data<Database>,
//     req: HttpRequest,
// ) -> impl Responder {
//     // Recupera las claims inyectadas por el middleware
//     let claims = match req.extensions().get::<crate::middleware::auth::Claims>().cloned() {
//         Some(claims) => claims,
//         None => {
//             write_log("GET /zones - Token no encontrado").ok();
//             return HttpResponse::Unauthorized().body("Token no encontrado");
//         },
//     };

//     // Solo el admin puede obtener todas las zonas
//     if claims.role != "admin" {
//         write_log("GET /zones - Acceso no autorizado: se requiere administrador").ok();
//         return HttpResponse::Unauthorized().body("Acceso no autorizado: se requiere administrador");
//     }

//     let collection = db.collection::<Zone>("zones");
//     let cursor = match collection.find(doc! {}).await {
//         Ok(cursor) => cursor,
//         Err(_) => {
//             write_log("GET /zones - Error buscando zonas").ok();
//             return HttpResponse::BadRequest().body("Error inesperado, inténtelo  nuevamente");
//         },
//     };
//     let zones: Vec<Zone> = match cursor.try_collect().await {
//         Ok(zones) => zones,
//         Err(_) => {
//             write_log("GET /zones - Error recogiendo zonas").ok();
//             return HttpResponse::BadRequest().body("Error inesperado, inténtelo  nuevamente");
//         },
//     };
//     write_log(&format!("GET /zones - {} zonas recuperadas", zones.len())).ok();
//     HttpResponse::Ok().json(zones)
// }

#[get("/zones/{id}")]
async fn get_zone_handler(db: web::Data<Database>, path: web::Path<String>) -> impl Responder {
    let collection = db.collection::<Zone>("zones");
    let obj_id = match ObjectId::parse_str(&path.into_inner()) {
        Ok(obj_id) => obj_id,
        Err(_) => {
            write_log("GET /zones/{id} - ID inválido").ok();
            return HttpResponse::BadRequest().body("ID inválido");
        }
    };
    match collection.find_one(doc! {"_id":obj_id}).await {
        Ok(Some(zone)) => {
            write_log("GET /zones/{id} - Zona encontrada").ok();
            HttpResponse::Ok().json(zone)
        }
        Ok(None) => {
            write_log("GET /zones/{id} - Zona no encontrada").ok();
            HttpResponse::NotFound().body("zona no encontrada")
        }
        Err(e) => {
            write_log(&format!("GET /zones/{} - Error: {}", obj_id, e)).ok();
            HttpResponse::BadRequest().body(e.to_string())
        }
    }
}

#[get("/zones/parent/{id}")]
async fn get_zone_from_parent_handler(
    db: web::Data<Database>,
    path: web::Path<String>,
    req: HttpRequest,
) -> impl Responder {
    // Parsear parent_id desde la ruta
    let parent_id = match ObjectId::parse_str(&path.into_inner()) {
        Ok(id) => id,
        Err(_) => {
            write_log("GET /zones/parent/{id} - ID inválido").ok();
            return HttpResponse::BadRequest().body("ID inválido");
        }
    };

    // Recupera las claims inyectadas por el middleware
    let claims = match req
        .extensions()
        .get::<crate::middleware::auth::Claims>()
        .cloned()
    {
        Some(claims) => claims,
        None => {
            write_log("GET /zones/parent/{id} - Token no encontrado").ok();
            return HttpResponse::Unauthorized().body("Token no encontrado");
        }
    };

    // Convertir claims.sub a ObjectId para comparar correctamente
    let user_obj_id = match ObjectId::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => {
            write_log("GET /zones/parent/{id} - ID de usuario inválido").ok();
            return HttpResponse::Unauthorized().body("ID de usuario inválido");
        }
    };

    // Buscar todas las zonas cuyo parentZoneId sea igual a parent_id
    let zone_collection = db.collection::<Zone>("zones");
    let zone_filter = if claims.role == "admin" {
        doc! { "parentZoneId": parent_id }
    } else {
        doc! { "parentZoneId": parent_id, "$or": [ { "userId": { "$exists": false } }, { "userId": &user_obj_id } ] }
    };
    let zone_cursor = match zone_collection.find(zone_filter).await {
        Ok(cursor) => cursor,
        Err(_) => {
            write_log("GET /zones/parent/{id} - Error al obtener zonas").ok();
            return HttpResponse::BadRequest().body("Error al obtener zonas");
        }
    };
    let zones: Vec<Zone> = match zone_cursor.try_collect().await {
        Ok(zones) => zones,
        Err(_) => {
            write_log("GET /zones/parent/{id} - Error al procesar zonas").ok();
            return HttpResponse::BadRequest().body("Error al procesar zonas");
        }
    };

    // Obtener los ítems de la zona proporcionada (parent_id)
    let items_collection = db.collection::<crate::entities::item::Item>("items");
    let items_filter = if claims.role == "admin" {
        doc! { "zoneId": parent_id.clone() }
    } else {
        doc! { "zoneId": parent_id.clone(), "$or": [ { "userId": { "$exists": false } }, { "userId": &user_obj_id } ] }
    };
    let items_cursor = match items_collection.find(items_filter).await {
        Ok(cursor) => cursor,
        Err(_) => {
            write_log("GET /zones/parent/{id} - Error al obtener ítems").ok();
            return HttpResponse::BadRequest().body("Error al obtener ítems");
        }
    };
    let items: Vec<crate::entities::item::Item> = match items_cursor.try_collect().await {
        Ok(items) => items,
        Err(_) => {
            write_log("GET /zones/parent/{id} - Error al procesar ítems").ok();
            return HttpResponse::BadRequest().body("Error al procesar ítems");
        }
    };

    // Retornar zonas e ítems en la respuesta
    write_log(&format!(
        "GET /zones/parent/{{id}} - {} zonas y {} items recuperados",
        zones.len(),
        items.len()
    ))
    .ok();
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
    let claims = match req
        .extensions()
        .get::<crate::middleware::auth::Claims>()
        .cloned()
    {
        Some(claims) => claims,
        None => {
            write_log("POST /zones - Token no encontrado").ok();
            return HttpResponse::Unauthorized().body("Token no encontrado");
        }
    };

    // Extraer "name"
    let name = match new_zone.get("name") {
        Some(value) => match value.as_str() {
            Some(s) => s.to_string(),
            None => {
                write_log("POST /zones - El nombre debe ser una cadena de texto").ok();
                return HttpResponse::BadRequest().body("El nombre debe ser una cadena de texto");
            }
        },
        None => {
            write_log("POST /zones - El nombre es requerido").ok();
            return HttpResponse::BadRequest().body("El nombre es requerido");
        }
    };

    // Extraer "parentZoneId" (obligatorio)
    let parent_zone_str = match new_zone.get("parentZoneId") {
        Some(value) => match value.as_str() {
            Some(s) => s.to_string(),
            None => {
                write_log("POST /zones - parentZoneId debe ser una cadena de texto").ok();
                return HttpResponse::BadRequest()
                    .body("parentZoneId debe ser una cadena de texto");
            }
        },
        None => {
            write_log("POST /zones - parentZoneId es requerido").ok();
            return HttpResponse::BadRequest().body("parentZoneId es requerido");
        }
    };
    let parent_zone_id = match ObjectId::parse_str(&parent_zone_str) {
        Ok(id) => id,
        Err(_) => {
            write_log("POST /zones - parentZoneId inválido").ok();
            return HttpResponse::BadRequest().body("parentZoneId inválido");
        }
    };

    // Determinar property_id:
    // Primero, buscar en la colección properties
    let property_collection = db.collection::<crate::entities::property::Property>("properties");
    let property = property_collection
        .find_one(doc! {"_id": parent_zone_id.clone()})
        .await
        .ok()
        .flatten();
    let property_id = if let Some(_) = property {
        // Si se encontró, se usa parent_zone_id como property_id
        parent_zone_id.clone()
    } else {
        // Sino, buscar en la colección zones
        let zone_collection = db.collection::<Zone>("zones");
        if let Ok(Some(zone)) = zone_collection
            .find_one(doc! {"_id": parent_zone_id.clone()})
            .await
        {
            zone.property_id
        } else {
            write_log("POST /zones - parentZoneId no corresponde a propiedad ni zona válida").ok();
            return HttpResponse::BadRequest()
                .body("parentZoneId no corresponde a propiedad ni zona válida");
        }
    };

    let is_private = new_zone
        .get("private")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let user_id = if is_private {
        match ObjectId::parse_str(&claims.sub) {
            Ok(id) => Some(id),
            Err(_) => {
                write_log("POST /zones - ID de usuario inválido").ok();
                return HttpResponse::BadRequest().body("ID de usuario inválido");
            }
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
        Ok(result) => {
            write_log("POST /zones - Zona creada correctamente").ok();
            HttpResponse::Ok().json(result.inserted_id)
        }
        Err(_) => {
            write_log("POST /zones - Error inesperado").ok();
            HttpResponse::BadRequest().body("Error inesperado, inténtelo nuevamente")
        }
    }
}

#[patch("/zones/{id}")]
async fn patch_zones_handler(
    db: web::Data<Database>,
    path: web::Path<String>,
    updated_zone: web::Json<serde_json::Value>,
    req: HttpRequest,
) -> impl Responder {
    let claims = match req
        .extensions()
        .get::<crate::middleware::auth::Claims>()
        .cloned()
    {
        Some(claims) => claims,
        None => {
            write_log("PATCH /zones/{id} - Token no encontrado").ok();
            return HttpResponse::Unauthorized().body("Token no encontrado");
        }
    };

    let collection = db.collection::<Zone>("zones");
    let obj_id = match ObjectId::parse_str(&path.into_inner()) {
        Ok(id) => id,
        Err(_) => {
            write_log("PATCH /zones/{id} - ID inválido").ok();
            return HttpResponse::BadRequest().body("ID inválido");
        }
    };

    let mut set_doc = doc! {};
    let mut unset_doc = doc! {};

    // Campo name
    if let Some(value) = updated_zone.get("name") {
        match value {
            serde_json::Value::String(name) => {
                set_doc.insert("name", name.clone());
            }
            _ => {
                write_log("PATCH /zones/{id} - Valor inválido para 'name'").ok();
                return HttpResponse::BadRequest().body("Valor inválido para 'name'");
            }
        }
    }

    // Campo private
    if let Some(value) = updated_zone.get("private") {
        match value {
            serde_json::Value::Bool(is_private) => {
                if *is_private {
                    match ObjectId::parse_str(&claims.sub) {
                        Ok(user_id) => {
                            set_doc.insert("userId", user_id);
                        }
                        Err(_) => {
                            write_log("PATCH /zones/{id} - ID de usuario inválido").ok();
                            return HttpResponse::BadRequest().body("ID de usuario inválido");
                        }
                    }
                } else {
                    unset_doc.insert("userId", "");
                }
            }
            _ => {
                write_log("PATCH /zones/{id} - Valor inválido para 'private'").ok();
                return HttpResponse::BadRequest().body("Valor inválido para 'private'");
            }
        }
    }

    if set_doc.is_empty() && unset_doc.is_empty() {
        write_log("PATCH /zones/{id} - No hay campos para actualizar").ok();
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
            write_log("PATCH /zones/{id} - Zona actualizada").ok();
            HttpResponse::Ok().body("Zona actualizada")
        }
        Ok(_) => {
            write_log("PATCH /zones/{id} - Zona no encontrada").ok();
            HttpResponse::NotFound().body("Zona no encontrada")
        }
        Err(_) => {
            write_log("PATCH /zones/{id} - Error inesperado").ok();
            HttpResponse::BadRequest().body("Error inesperado, inténtelo  nuevamente")
        }
    }
}

pub async fn delete_zone(db: &Database, zone_id: String) -> HttpResponse {
    let zone_collection = db.collection::<Zone>("zones");
    let item_collection = db.collection::<Item>("items");
    let obj_id = match ObjectId::parse_str(zone_id) {
        Ok(id) => id,
        Err(_) => {
            write_log("DELETE /zones/{id} - Id incorrecto").ok();
            return HttpResponse::BadRequest().body("Id incorrecto");
        }
    };
    let item_cursor = match item_collection.find(doc! {"zoneId":obj_id}).await {
        Ok(items) => items,
        Err(_) => {
            write_log("DELETE /zones/{id} - Error buscando items").ok();
            return HttpResponse::BadRequest().body("Error inesperado, inténtelo  nuevamente");
        }
    };
    let items: Vec<Item> = match item_cursor.try_collect().await {
        Ok(items) => items,
        Err(_) => {
            write_log("DELETE /zones/{id} - Error recogiendo items").ok();
            return HttpResponse::BadRequest().body("Error inesperado, inténtelo  nuevamente");
        }
    };
    for item in items {
        let id = match item.id {
            Some(id) => id,
            None => {
                write_log("DELETE /zones/{id} - Item sin ID").ok();
                return HttpResponse::BadRequest().body("No hay ID");
            }
        };
        let res = delete_item(db, id.to_string()).await;
        if !res.status().is_success() {
            write_log("DELETE /zones/{id} - Error eliminando item asociado").ok();
            return res; // Si falla, detenemos la ejecución y devolvemos el error
        }
    }
    match zone_collection.delete_one(doc! {"_id": obj_id}).await {
        Ok(_) => {
            write_log("DELETE /zones/{id} - Zona eliminada correctamente").ok();
            HttpResponse::Ok().body("Zona eliminada")
        }
        Err(_) => {
            write_log("DELETE /zones/{id} - Error eliminando zona").ok();
            HttpResponse::BadRequest().body("Error inesperado, inténtelo  nuevamente")
        }
    }
}

// Función auxiliar para obtener todos los IDs de zonas hijas recursivamente
async fn get_all_child_zone_ids(db: &Database, parent_id: &ObjectId) -> Vec<ObjectId> {
    let mut all_ids = Vec::new();
    let zone_collection = db.collection::<Zone>("zones");
    let mut stack = vec![parent_id.clone()];
    while let Some(current_id) = stack.pop() {
        // Buscar zonas hijas directas
        if let Ok(mut cursor) = zone_collection
            .find(doc! {"parentZoneId": &current_id})
            .await
        {
            while let Some(result) = cursor.try_next().await.transpose() {
                match result {
                    Ok(child_zone) => {
                        if let Some(child_id) = child_zone.id.clone() {
                            stack.push(child_id.clone());
                            all_ids.push(child_id);
                        }
                    }
                    Err(_) => {
                        // Puedes registrar el error si lo deseas
                        continue;
                    }
                }
            }
        }
    }
    all_ids
}

#[delete("/zones/{id}")]
async fn delete_zone_handler(db: web::Data<Database>, path: web::Path<String>) -> impl Responder {
    let client = db.client();
    let mut session = match client.start_session().await {
        Ok(s) => s,
        Err(_) => {
            write_log("DELETE /zones/{id} - Error iniciando sesión").ok();
            return HttpResponse::BadRequest().body("Error inesperado, inténtelo  nuevamente");
        }
    };
    session.start_transaction().await.ok();
    let id_str = path.into_inner();
    let obj_id = match ObjectId::parse_str(&id_str) {
        Ok(id) => id,
        Err(_) => {
            write_log("DELETE /zones/{id} - Id incorrecto").ok();
            session.abort_transaction().await.ok();
            return HttpResponse::BadRequest().body("Id incorrecto");
        }
    };

    // Obtener todos los IDs de zonas hijas recursivamente
    let mut all_zone_ids = get_all_child_zone_ids(&db, &obj_id).await;
    // Incluir la zona original
    all_zone_ids.push(obj_id.clone());

    // Eliminar todas las zonas (y sus ítems) en orden inverso (de hojas a raíz)
    let mut error_response = None;
    for zone_id in all_zone_ids.iter().rev() {
        let res = delete_zone(&db, zone_id.to_hex()).await;
        if !res.status().is_success() {
            error_response = Some(res);
            break;
        }
    }

    let response = if let Some(err) = error_response {
        session.abort_transaction().await.ok();
        write_log("DELETE /zones/{id} - Transacción abortada").ok();
        err
    } else {
        session.commit_transaction().await.ok();
        write_log("DELETE /zones/{id} - Transacción confirmada").ok();
        HttpResponse::Ok().body("Zona y subzonas eliminadas")
    };

    response
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_zone_handler)
        //.service(get_zones_handler)
        .service(get_zone_from_parent_handler)
        .service(create_zone_handler)
        .service(patch_zones_handler)
        .service(delete_zone_handler);
}
