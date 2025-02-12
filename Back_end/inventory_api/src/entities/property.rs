use std::path;

use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use futures_util::stream::TryStreamExt;
use mongodb::{
    bson::{doc, oid::ObjectId, DateTime},
    Database,
};
use serde::{Deserialize, Serialize};

use crate::entities::property;

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
impl Property {
    fn new(
        name: String,
        direction: Option<String>,
        group_id: ObjectId,
        user_id: Option<ObjectId>,
    ) -> Property {
        Self {
            id: None,
            name,
            direction,
            group_id,
            user_id,
        }
    }
}

#[get("/properties")]
async fn get_properties_handler(db: web::Data<Database>) -> impl Responder {
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
) -> impl Responder {
    let properties_collection = db.collection::<Property>("properties");
    let group_id = match ObjectId::parse_str(&path.into_inner()) {
        Ok(group_id) => group_id,
        Err(_) => return HttpResponse::BadRequest().body("ID inválido"),
    };
    let properties_cursor = match properties_collection.find(doc! {"groupId":group_id}).await {
        Ok(properties_cursor) => properties_cursor,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };
    let properties: Vec<Property> = match properties_cursor.try_collect().await {
        Ok(properties) => properties,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };
    HttpResponse::Ok().json(properties)
}
#[post("/porperties")]
async fn create_property_handler(db: web::Data<Database>, new_property: web::Json<Property>) -> impl Responder {
    let collection = db.collection::<Property>("properties");
    let mut property = new_property.into_inner();
    property.id = None;
    match collection.insert_one(property).await {
        Ok(result) => HttpResponse::Ok().json(result.inserted_id),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_property_handler)
        .service(get_properties_handler)
        .service(get_properties_from_group_handler)
        .service(create_property_handler);
}
