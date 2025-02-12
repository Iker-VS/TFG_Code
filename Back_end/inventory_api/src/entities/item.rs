use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use futures_util::stream::TryStreamExt;
use mongodb::{
    bson::{doc, oid::ObjectId, DateTime},
    Database,
};
use serde::{Deserialize, Serialize};

use super::zone::{self, Zone};

#[derive(Debug, Serialize, Deserialize)]
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
    pub values: Option<Vec<ValueEntry>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ValueEntry {
    name: String,
    value: Value,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Value {
    Text(String),
    Number(i32),
    Date(DateTime),
}
impl Item {
    fn new(
        name: String,
        description: Option<String>,
        picture_url: Option<String>,
        zone_id: ObjectId,
        values: Option<Vec<ValueEntry>>,
        tags: Option<Vec<String>>,
    ) -> Item {
        Self {
            id: None,
            name,
            description,
            picture_url,
            zone_id,
            values,
            tags,
        }
    }
}
#[get("/items")]
async fn get_items_handler(db: web::Data<Database>) -> impl Responder {
    let collection = db.collection::<Item>("items");
    let cursor = match collection.find(doc! {}).await {
        Ok(cursor) => cursor,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };
    let items: Vec<Item> = match cursor.try_collect().await {
        Ok(items) => items,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };
    HttpResponse::Ok().json(items)
}

#[get("/items/{id}")]
async fn get_item_handler(db: web::Data<Database>, path: web::Path<String>) -> impl Responder {
    let collection = db.collection::<Item>("items");
    let obj_id = match ObjectId::parse_str(&path.into_inner()) {
        Ok(obj_id) => obj_id,
        Err(_) => return HttpResponse::BadRequest().body("ID inválido"),
    };
    match collection.find_one(doc! {"_id":obj_id}).await {
        Ok(Some(item)) => return HttpResponse::Ok().json(item),
        Ok(None) => return HttpResponse::NotFound().body("Objeto no encontrado"),
        Err(e) => HttpResponse::BadRequest().body(e.to_string()),
    }
}
#[get("/items/zone/{id}")]
async fn get_items_from_zone_handler(db:web::Data<Database>, path: web::Path<String>) -> impl Responder {
    let items_collection= db.collection::<Item>("items");
    let zone_id =match ObjectId::parse_str(&path.into_inner()) {
        Ok(zone_id)=>zone_id,
        Err(_) =>return HttpResponse::BadRequest().body("ID inválido"),
    };
    let items_cursor =match items_collection.find(doc! {"zoneId":zone_id}).await {
        Ok(items_cursor)=>items_cursor,
        Err(e)=> return HttpResponse::BadRequest().body(e.to_string()),        
    };
    let items:Vec<Item> =match items_cursor.try_collect().await{
        Ok(items)=>items,
        Err(e)=> return HttpResponse::InternalServerError().body(e.to_string()),
    };
    HttpResponse::Ok().json(items)
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_item_handler)
    .service(get_items_handler)
    .service(get_items_from_zone_handler);

}