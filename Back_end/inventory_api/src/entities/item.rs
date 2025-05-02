use actix_web::{delete, get, patch, post, web, HttpMessage, HttpRequest, HttpResponse, Responder};
use futures_util::stream::TryStreamExt;
use mongodb::{
    bson::{doc, oid::ObjectId, Document},
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
    pub tags: Option<Vec<String>>,
}

impl Item {
    fn new(
        name: String,
        description: Option<String>,
        picture_url: Option<String>,
        zone_id: ObjectId,
        tags: Option<Vec<String>>,
    ) -> Item {
        Self {
            id: None,
            name,
            description,
            picture_url,
            zone_id,
            tags,
        }
    }
}

#[get("/items")]
async fn get_items_handler(
    db: web::Data<Database>,
    req: HttpRequest,
) -> impl Responder {
    // Recupera las claims inyectadas por el middleware
    let claims = match req.extensions().get::<crate::middleware::auth::Claims>().cloned() {
        Some(claims) => claims,
        None => return HttpResponse::Unauthorized().body("Token no encontrado"),
    };

    // Solo el admin puede obtener todos los items
    if claims.role != "admin" {
        return HttpResponse::Unauthorized().body("Acceso no autorizado: se requiere administrador");
    }

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

#[patch("/items/{id}")]
async fn patch_item_handler(
    db: web::Data<Database>,
    path: web::Path<String>,
    updated_item: web::Json<serde_json::Value>,
) -> impl Responder {
    let collection = db.collection::<Item>("items");
    let obj_id = match ObjectId::parse_str(&path.into_inner()) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("ID inválido"),
    };

    let mut set_doc = Document::new();
    let mut unset_doc = Document::new();

    // Campo: name (solo si está presente con valor)
    if let Some(value) = updated_item.get("name") {
        match value {
            serde_json::Value::String(name) => set_doc.insert("name", name.clone()),
            serde_json::Value::Null => return HttpResponse::BadRequest().body("'name' no puede ser null"),
            _ => return HttpResponse::BadRequest().body("Valor inválido para 'name'"),
        };
    }

    // Campo opcional: description
    if let Some(value) = updated_item.get("description") {
        match value {
            serde_json::Value::String(desc) => set_doc.insert("description", desc.clone()),
            serde_json::Value::Null => unset_doc.insert("description", ""),
            _ => return HttpResponse::BadRequest().body("Valor inválido para 'description'"),
        };
    }

    // Campo opcional: pictureUrl
    if let Some(value) = updated_item.get("pictureUrl") {
        match value {
            serde_json::Value::String(url) => set_doc.insert("pictureUrl", url.clone()),
            serde_json::Value::Null => unset_doc.insert("pictureUrl", ""),
            _ => return HttpResponse::BadRequest().body("Valor inválido para 'pictureUrl'"),
        };
    }

    // Campo opcional: tags
    if let Some(value) = updated_item.get("tags") {
        match value {
            serde_json::Value::Array(tags) => {
                let string_tags: Vec<String> = tags
                    .iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect();
                set_doc.insert("tags", string_tags)
            },
            serde_json::Value::Null => unset_doc.insert("tags", ""),
            _ => return HttpResponse::BadRequest().body("Valor inválido para 'tags'"),
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
        .service(patch_item_handler)
        .service(delete_item_handler);
}
