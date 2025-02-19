use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use futures_util::stream::TryStreamExt;
use mongodb::{
    bson::{doc, oid::ObjectId},
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
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}


#[put("/properties/{id}")]
async fn update_property_handler(
    db: web::Data<Database>,
    path: web::Path<String>,
    updated_property: web::Json<Property>,
) -> impl Responder {
    let collection = db.collection::<Property>("properties");
    let obj_id = match ObjectId::parse_str(&path.into_inner()) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("ID inválido"),
    };
    let mut update_doc = doc! {
        "$set": {
            "name": updated_property.name.clone(),
            "groupId": updated_property.group_id.clone(),
        }
    };

    if let Some(direction) = &updated_property.direction {
        update_doc.get_mut("$set").unwrap().as_document_mut().unwrap().insert("direction", direction.clone());
    } else {
        update_doc.insert("$unset", doc! {"direction": ""});
    }
    
    if let Some(user_id) = &updated_property.user_id {
        update_doc.get_mut("$set").unwrap().as_document_mut().unwrap().insert("userId", user_id.clone());
    } else {
        update_doc.insert("$unset", doc! {"userId": ""});
    }


    match collection
        .update_one(doc! {"_id": obj_id}, update_doc)
        .await
    {
        Ok(result) if result.matched_count == 1 => HttpResponse::Ok().body("Propiedad actualizada"),
        Ok(_) => HttpResponse::NotFound().body("Propiedad no encontrada"),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
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
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };
    let zones: Vec<Zone> = match zone_cursor.try_collect().await {
        Ok(zones) => zones,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
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
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
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
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
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
        .service(update_property_handler)
        .service(delete_property_handler);
}
