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
    let collection = db.collection::<Zone>("zone");
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
