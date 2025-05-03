use actix_web::{delete, get, post, patch, web, HttpMessage, HttpRequest, HttpResponse, Responder};
use futures_util::stream::TryStreamExt;
use mongodb::{
    bson::{doc, oid::ObjectId, Document},
    Database,
};
use serde::{Deserialize, Serialize};

use super::zone::{delete_zone, Zone};

#[derive(Debug, Serialize, Deserialize)]
pub struct Property {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction: Option<String>,
    #[serde(rename = "groupId")]
    pub group_id: ObjectId,
    #[serde(rename = "userId", skip_serializing_if = "Option::is_none")]
    pub user_id: Option<ObjectId>,
}
// impl Property {
//     fn new(
//         name: String,
//         direction: Option<String>,
//         group_id: ObjectId,
//         user_id: Option<ObjectId>,
//     ) -> Property {
//         Self {
//             id: None,
//             name,
//             direction,
//             group_id,
//             user_id,
//         }
//     }
// }

#[get("/properties")]
async fn get_properties_handler(
    db: web::Data<Database>,
    req: HttpRequest,
) -> impl Responder {
    // Recupera las claims inyectadas por el middleware
    let claims = match req.extensions().get::<crate::middleware::auth::Claims>().cloned() {
        Some(claims) => claims,
        None => return HttpResponse::Unauthorized().body("Token no encontrado"),
    };

    // Solo el admin puede obtener todas las propiedades
    if claims.role != "admin" {
        return HttpResponse::Unauthorized().body("Acceso no autorizado: se requiere administrador");
    }

    let collection = db.collection::<Property>("properties");
    let cursor = match collection.find(doc! {}).await {
        Ok(cursor) => cursor,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    let properties: Vec<Property> = match cursor.try_collect().await {
        Ok(properties) => properties,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    HttpResponse::Ok().json(properties)
}
#[get("/properties/{id}")]
async fn get_property_handler(db: web::Data<Database>, path: web::Path<String>) -> impl Responder {
    let collection = db.collection::<Property>("properties");
    let obj_id = match ObjectId::parse_str(&path.into_inner()) {
        Ok(obj_id) => obj_id,
        Err(_) => return HttpResponse::BadRequest().body("ID inválido"),
    };
    match collection.find_one(doc! {"_id":obj_id}).await {
        Ok(Some(property)) => return HttpResponse::Ok().json(property),
        Ok(None) => return HttpResponse::NotFound().body("propiedad no encontrada"),
        Err(e) => HttpResponse::BadRequest().body(e.to_string()),
    }
}
#[get("/properties/group/{id}")]
async fn get_properties_from_group_handler(
    db: web::Data<Database>,
    path: web::Path<String>,
    req: HttpRequest,
) -> impl Responder {
    // Recupera las claims inyectadas por el middleware
    let claims = match req
        .extensions()
        .get::<crate::middleware::auth::Claims>()
        .cloned()
    {
        Some(c) => c,
        None => return HttpResponse::Unauthorized().body("Token no encontrado"),
    };

    let group_id = match ObjectId::parse_str(&path.into_inner()) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("ID inválido"),
    };

    let token_user_obj_id = match ObjectId::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("Token user ID inválido"),
    };

    let properties_collection = db.collection::<Property>("properties");
    let cursor = match properties_collection
        .find(doc! {
            "groupId": group_id,
            "$or": [
                { "userId": { "$exists": false } },
                { "userId": token_user_obj_id }
            ]
        })
        .await
    {
        Ok(cursor) => cursor,
        Err(_) => return HttpResponse::BadRequest().body("Error inesperado, intentelo nuevamente"),
    };

    let properties: Vec<Property> = match cursor.try_collect().await {
        Ok(props) => props,
        Err(_) => return HttpResponse::BadRequest().body("Error inesperado, intentelo nuevamente"),
    };

    HttpResponse::Ok().json(properties)
}

#[post("/properties")]
async fn create_property_handler(
    db: web::Data<Database>,
    new_property: web::Json<Property>,
) -> impl Responder {
    let collection = db.collection::<Property>("properties");
    let mut property = new_property.into_inner();
    property.id = None;
    match collection.insert_one(property).await {
        Ok(result) => HttpResponse::Ok().json(result.inserted_id),
        Err(_) => HttpResponse::BadRequest().body("Error inesperado, intentelo nuevamente"),
    }
}

#[patch("/properties/{id}")]
async fn patch_property_handler(
    db: web::Data<Database>,
    path: web::Path<String>,
    updated_property: web::Json<serde_json::Value>,
) -> impl Responder {
    let collection = db.collection::<Property>("properties");
    let obj_id = match ObjectId::parse_str(&path.into_inner()) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("ID inválido"),
    };

    let mut set_doc = Document::new();
    let mut unset_doc = Document::new();

    // Campo: name (solo si está presente con valor)
    if let Some(value) = updated_property.get("name") {
        match value {
            serde_json::Value::String(name) => set_doc.insert("name", name.clone()),
            serde_json::Value::Null => return HttpResponse::BadRequest().body("'name' no puede ser null"),
            _ => return HttpResponse::BadRequest().body("Valor inválido para 'name'"),
        };
    }

    // Campo opcional: direction
    if let Some(value) = updated_property.get("direction") {
        match value {
            serde_json::Value::String(dir) => set_doc.insert("direction", dir.clone()),
            serde_json::Value::Null => unset_doc.insert("direction", ""),
            _ => return HttpResponse::BadRequest().body("Valor inválido para 'direction'"),
        };
    }

    // Validar que haya algo que actualizar
    if set_doc.is_empty() && unset_doc.is_empty() {
        return HttpResponse::BadRequest().body("No hay campos para actualizar");
    }

    // Construir update_doc final
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
        Ok(result) if result.matched_count == 1 => HttpResponse::Ok().body("Propiedad actualizada"),
        Ok(_) => HttpResponse::NotFound().body("Propiedad no encontrada"),
        Err(_) => HttpResponse::BadRequest().body("Error inesperado, intentelo nuevamente"),
    }
}



pub async fn delete_property(db: &Database, property_id: String) -> HttpResponse {
    let zone_collection = db.collection::<Zone>("zones");
    let property_collection = db.collection::<Property>("properties");
    let obj_id = match ObjectId::parse_str(property_id) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("Id incorrecto"),
    };

    let zone_cursor = match zone_collection.find(doc! {"propertyId":obj_id}).await {
        Ok(zones) => zones,
        Err(_) => return HttpResponse::BadRequest().body("Error inesperado, intentelo nuevamente"),
    };
    let zones: Vec<Zone> = match zone_cursor.try_collect().await {
        Ok(zones) => zones,
        Err(_) => return HttpResponse::BadRequest().body("Error inesperado, intentelo nuevamente"),
    };
    for zone in zones {
        let id = match zone.id {
            Some(id) => id,
            None => return HttpResponse::BadRequest().body("No hay ID"),
        };
        let res = delete_zone(db, id.to_string()).await;
        if !res.status().is_success() {
            return res; // Si falla, detenemos la ejecución y devolvemos el error
        }
    }

    match property_collection.delete_one(doc! {"_id": obj_id}).await {
        Ok(_) => HttpResponse::Ok().body("Propiedad Eliminada"),
        Err(_) => HttpResponse::BadRequest().body("Error inesperado, intentelo nuevamente"),
    }
}

#[delete("/properties/{id}")]
async fn delete_property_handler(
    db: web::Data<Database>,
    path: web::Path<String>,
) -> impl Responder {
    let client = db.client();
    let mut session = match client.start_session().await {
        Ok(s) => s,
        Err(_) => return HttpResponse::BadRequest().body("Error inesperado, intentelo nuevamente"),
    };
    session.start_transaction().await.ok();
    let response = delete_property(&db, path.into_inner()).await;

    if response.status().is_success() {
        session.commit_transaction().await.ok();
    } else {
        session.abort_transaction().await.ok();
    }

    response
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_property_handler)
        .service(get_properties_handler)
        .service(get_properties_from_group_handler)
        .service(create_property_handler)
        .service(patch_property_handler)
        .service(delete_property_handler);
}
