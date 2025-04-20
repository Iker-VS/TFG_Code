use crate::entities::item::{delete_item, Item};
use actix_web::{delete, get, post, put, web, HttpMessage, HttpRequest, HttpResponse, Responder};
use futures_util::stream::TryStreamExt;
use mongodb::{
    bson::{doc, oid::ObjectId},
    Database,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
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

impl Zone {
    fn new(
        name: String,
        property_id: ObjectId,
        user_id: Option<ObjectId>,
        parent_zone_id: Option<ObjectId>,
    ) -> Zone {
        Self {
            id: None,
            name,
            property_id,
            user_id,
            parent_zone_id,
        }
    }
}

#[get("/zones")]
async fn get_zones_handler(db: web::Data<Database>) -> impl Responder {
    let collection = db.collection::<Zone>("zones");
    let cursor = match collection.find(doc! {}).await {
        Ok(cursor) => cursor,
        Err(_) => return HttpResponse::BadRequest().body("Error inesperado, intentelo nuevamente"),
    };
    let zones: Vec<Zone> = match cursor.try_collect().await {
        Ok(zones) => zones,
        Err(_) => return HttpResponse::BadRequest().body("Error inesperado, intentelo nuevamente"),
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
    path: web::Path<String>,
    req: HttpRequest,
) -> impl Responder {
    // Recupera las claims ya inyectadas por el middleware
    let claims = match req.extensions().get::<crate::middleware::auth::Claims>().cloned() {
        Some(c) => c,
        None => return HttpResponse::Unauthorized().body("Token no encontrado"),
    };

    let parent_id = match ObjectId::parse_str(&path.into_inner()) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("ID inválido"),
    };

    let token_user_obj_id = match ObjectId::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("Token user ID inválido"),
    };

    // Filtro: parentZoneId igual al proporcionado y userId no existente o igual al del token.
    let zone_collection = db.collection::<Zone>("zones");
    let zone_cursor = match zone_collection
        .find(doc! {
            "parentZoneId": parent_id,
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
    let zones: Vec<Zone> = match zone_cursor.try_collect().await {
        Ok(zones) => zones,
        Err(_) => return HttpResponse::BadRequest().body("Error inesperado, intentelo nuevamente"),
    };
    HttpResponse::Ok().json(zones)
}

#[post("/zones")]
async fn create_zone_handler(db: web::Data<Database>, new_zone: web::Json<Zone>) -> impl Responder {
    let collection = db.collection::<Zone>("zones");
    let mut zone = new_zone.into_inner();
    zone.id = None;
    match collection.insert_one(zone).await {
        Ok(result) => HttpResponse::Ok().json(result.inserted_id),
        Err(_) => HttpResponse::BadRequest().body("Error inesperado, intentelo nuevamente"),
    }
}

#[put("/zones/{id}")]
async fn update_zones_handler(
    db: web::Data<Database>,
    path: web::Path<String>,
    updated_zone: web::Json<Zone>,
) -> impl Responder {
    let collection = db.collection::<Zone>("zones");
    let obj_id = match ObjectId::parse_str(&path.into_inner()) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("ID inválido"),
    };
    let mut update_doc = doc! {
        "$set": {
            "name": updated_zone.name.clone(),
            "propertyId": updated_zone.property_id.clone(),
        }
    };
    if let Some(user_id) = &updated_zone.user_id {
        update_doc
            .get_mut("$set")
            .unwrap()
            .as_document_mut()
            .unwrap()
            .insert("userId", user_id.clone());
    } else {
        update_doc.insert("$unset", doc! {"userId": ""});
    }
    if let Some(parent_zone_id) = &updated_zone.parent_zone_id {
        update_doc
            .get_mut("$set")
            .unwrap()
            .as_document_mut()
            .unwrap()
            .insert("parentZoneId", parent_zone_id.clone());
    } else {
        update_doc.insert("$unset", doc! {"parentZoneId": ""});
    }
    match collection
        .update_one(doc! {"_id": obj_id}, update_doc)
        .await
    {
        Ok(result) if result.matched_count == 1 => HttpResponse::Ok().body("Zona actualizada"),
        Ok(_) => HttpResponse::NotFound().body("Zona no encontrada"),
        Err(_) => HttpResponse::BadRequest().body("Error inesperado, intentelo nuevamente"),
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
        Err(_) => return HttpResponse::BadRequest().body("Error inesperado, intentelo nuevamente"),
    };
    let items: Vec<Item> = match item_cursor.try_collect().await {
        Ok(items) => items,
        Err(_) => return HttpResponse::BadRequest().body("Error inesperado, intentelo nuevamente"),
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
        Err(_) => HttpResponse::BadRequest().body("Error inesperado, intentelo nuevamente"),
    }
}

#[delete("/zones/{id}")]
async fn delete_zone_handler(db: web::Data<Database>, path: web::Path<String>) -> impl Responder {
    let client = db.client();
    let mut session = match client.start_session().await {
        Ok(s) => s,
        Err(_) => return HttpResponse::BadRequest().body("Error inesperado, intentelo nuevamente"),
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
        .service(update_zones_handler)
        .service(delete_zone_handler);
}
