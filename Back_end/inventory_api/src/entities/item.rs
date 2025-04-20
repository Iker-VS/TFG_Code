use actix_web::{delete, get, post, put, web, HttpMessage, HttpRequest, HttpResponse, Responder};
use futures_util::stream::TryStreamExt;
use mongodb::{
    bson::{self, doc, oid::ObjectId, DateTime},
    Database,
};
use serde::{Deserialize, Serialize};

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
        Err(_) => return HttpResponse::BadRequest().body("Error inesperado, intentelo nuevamente"),
    };
    let items: Vec<Item> = match cursor.try_collect().await {
        Ok(items) => items,
        Err(_) => return HttpResponse::BadRequest().body("Error inesperado, intentelo nuevamente"),
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
async fn get_items_from_zone_handler(
    db: web::Data<Database>,
    path: web::Path<String>,
    req: HttpRequest,
) -> impl Responder {
    // Recupera las claims inyectadas por el middleware
    let claims = match req.extensions().get::<crate::middleware::auth::Claims>().cloned() {
        Some(c) => c,
        None => return HttpResponse::Unauthorized().body("Token no encontrado"),
    };

    let zone_id = match ObjectId::parse_str(&path.into_inner()) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("ID inválido"),
    };

    // Convertir el id del usuario (claims.sub) a ObjectId
    let token_user_obj_id = match ObjectId::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("Token user ID inválido"),
    };

    let items_collection = db.collection::<Item>("items");
    let cursor = match items_collection
        .find(doc! {
            "zoneId": zone_id,
            "$or": [
                { "userId": { "$exists": false } },
                { "userId": token_user_obj_id }
            ]
        }).await
    {
        Ok(cursor) => cursor,
        Err(_) => return HttpResponse::BadRequest().body("Error inesperado, intentelo nuevamente"),
    };

    let items: Vec<Item> = match cursor.try_collect().await {
        Ok(items) => items,
        Err(_) => return HttpResponse::BadRequest().body("Error inesperado, intentelo nuevamente"),
    };

    HttpResponse::Ok().json(items)
}

#[post("/items")]
async fn create_item_handler(db: web::Data<Database>, new_item: web::Json<Item>) -> impl Responder {
    let collection = db.collection::<Item>("items");
    let mut item = new_item.into_inner();
    item.id = None;
    match collection.insert_one(item).await {
        Ok(result) => HttpResponse::Ok().json(result.inserted_id),
        Err(_) => HttpResponse::BadRequest().body("Error inesperado, intentelo nuevamente"),
    }
}

#[put("/items/{id}")]
async fn update_item_handler(
    db: web::Data<Database>,
    path: web::Path<String>,
    updated_item: web::Json<Item>,
) -> impl Responder {
    let collection = db.collection::<Item>("items");
    let obj_id = match ObjectId::parse_str(&path.into_inner()) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("ID inválido"),
    };
    let mut update_doc = doc! {
        "$set": {
            "name": updated_item.name.clone(),
            "zoneId": updated_item.zone_id.clone(),
        }
    };
    if let Some(description) = &updated_item.description {
        update_doc
            .get_mut("$set")
            .unwrap()
            .as_document_mut()
            .unwrap()
            .insert("description", description.clone());
    } else {
        update_doc.insert("$unset", doc! {"description": ""});
    }

    if let Some(picture_url) = &updated_item.picture_url {
        update_doc
            .get_mut("$set")
            .unwrap()
            .as_document_mut()
            .unwrap()
            .insert("pictureUrl", picture_url.clone());
    } else {
        update_doc.insert("$unset", doc! {"pictureUrl": ""});
    }
    if let Some(values) = &updated_item.values {
        update_doc
            .get_mut("$set")
            .unwrap()
            .as_document_mut()
            .unwrap()
            .insert(
                "values",
                values
                    .iter()
                    .map(|v| bson::to_bson(v).ok())
                    .collect::<Option<Vec<_>>>(),
            );
    } else {
        update_doc.insert("$unset", doc! {"values": ""});
    }

    if let Some(tags) = &updated_item.tags {
        update_doc
            .get_mut("$set")
            .unwrap()
            .as_document_mut()
            .unwrap()
            .insert("tags", tags.clone());
    } else {
        update_doc.insert("$unset", doc! {"tags": ""});
    }
    match collection
        .update_one(doc! {"_id": obj_id}, update_doc)
        .await
    {
        Ok(result) if result.matched_count == 1 => HttpResponse::Ok().body("Objeto actualizado"),
        Ok(_) => HttpResponse::NotFound().body("Objeto no encontrado"),
        Err(_) => HttpResponse::BadRequest().body("Error inesperado, intentelo nuevamente"),
    }
}

pub async fn delete_item(db: &Database, item_id: String) -> HttpResponse {
    let collection = db.collection::<Item>("items");
    let obj_id = match ObjectId::parse_str(item_id) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("Id incorrecto"),
    };
    match collection.delete_one(doc! {"_id": obj_id}).await {
        Ok(_) => HttpResponse::Ok().body("Item Eliminado"),
        Err(_) => HttpResponse::BadRequest().body("Error inesperado, intentelo nuevamente"),
    }
}

#[delete("/items/{id}")]
async fn delete_item_handler(db: web::Data<Database>, path: web::Path<String>) -> impl Responder {
    let client = db.client();
    let mut session = match client.start_session().await {
        Ok(s) => s,
        Err(_) => return HttpResponse::BadRequest().body("Error inesperado, intentelo nuevamente"),
    };
    session.start_transaction().await.ok();
    let response = delete_item(&db, path.into_inner()).await;

    if response.status().is_success() {
        session.commit_transaction().await.ok();
    } else {
        session.abort_transaction().await.ok();
    }

    response
}
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_item_handler)
        .service(get_items_handler)
        .service(get_items_from_zone_handler)
        .service(create_item_handler)
        .service(update_item_handler)
        .service(delete_item_handler);
}
