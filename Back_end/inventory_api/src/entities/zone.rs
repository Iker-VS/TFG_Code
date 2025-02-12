use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
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
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };
    let zones: Vec<Zone> = match cursor.try_collect().await {
        Ok(zones) => zones,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
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
) -> impl Responder {
    let zone_collection = db.collection::<Zone>("zones");
    let parent_id = match ObjectId::parse_str(&path.into_inner()) {
        Ok(parent_id) => parent_id,
        Err(_) => return HttpResponse::BadRequest().body("ID inválido"),
    };
    let zone_cursor = match zone_collection.find(doc! {"parentZoneId":parent_id}).await {
        Ok(zone_cursor) => zone_cursor,
        Err(e) => return HttpResponse::BadRequest().body(e.to_string()),
    };
    let zones: Vec<Zone> = match zone_cursor.try_collect().await {
        Ok(zones) => zones,
        Err(e) => return HttpResponse::BadRequest().body(e.to_string()),
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
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
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
    let update_doc = doc! {
        "$set": {
            "name": updated_zone.name.clone(),
            "propertyId": updated_zone.property_id.clone(),
            "parentZoneId": updated_zone.parent_zone_id.clone(),
            "UserId": updated_zone.user_id.clone(),
        }
    };
    match collection
        .update_one(doc! {"_id": obj_id}, update_doc)
        .await
    {
        Ok(result) if result.matched_count == 1 => HttpResponse::Ok().body("Zona actualizada"),
        Ok(_) => HttpResponse::NotFound().body("Zona no encontrada"),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_zone_handler)
        .service(get_zones_handler)
        .service(get_zone_from_parent_handler)
        .service(create_zone_handler)
        .service(update_zones_handler);
}
