use std::fmt::Debug;

use actix_web::{delete, get, patch, post, web, HttpMessage, HttpRequest, HttpResponse, Responder};
use futures_util::stream::TryStreamExt;
use mongodb::{
    bson::{doc, oid::ObjectId, DateTime},
    Database,
};
use serde::{Deserialize, Serialize};



#[derive(Debug, Serialize, Deserialize)]
pub struct Log {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub description: String,
    pub time: DateTime,
    #[serde(rename = "groupId")]
    pub group_id: ObjectId,
    #[serde(rename = "userId")]
    pub user_id: ObjectId,
}

// impl Log {
//     fn new(description: String, time: DateTime, group_id: ObjectId, user_id: ObjectId) -> Log {
//         Self {
//             id: None,
//             description,
//             time,
//             group_id,
//             user_id,
//         }
//     }
// }

#[get("/logs")]
async fn get_logs_handler(db: web::Data<Database>, rep: HttpRequest) -> impl Responder {
    if !check_admin(rep).await {
        return HttpResponse::Unauthorized().body("Acceso no autorizado");
    }
    let collection = db.collection::<Log>("logs");
    let cursor = match collection.find(doc! {}).await {
        Ok(cursor) => cursor,
        Err(_) => return HttpResponse::BadRequest().body("Error inesperado, inténtelo  nuevamente"),
    };
    let logs: Vec<Log> = match cursor.try_collect().await {
        Ok(logs) => logs,
        Err(_) => return HttpResponse::BadRequest().body("Error inesperado, inténtelo  nuevamente"),
    };
    HttpResponse::Ok().json(logs)
}

#[get("/logs/{id}")]
async fn get_log_handler(db: web::Data<Database>, path: web::Path<String>,rep:HttpRequest) -> impl Responder {
    if !check_admin(rep).await {
        return HttpResponse::Unauthorized().body("Acceso no autorizado");
    }
    let collection = db.collection::<Log>("logs");
    let obj_id = match ObjectId::parse_str(&path.into_inner()) {
        Ok(obj_id) => obj_id,
        Err(_) => return HttpResponse::BadRequest().body("ID inválido"),
    };
    match collection.find_one(doc! {"_id":obj_id}).await {
        Ok(Some(log)) => return HttpResponse::Ok().json(log),
        Ok(None) => return HttpResponse::NotFound().body("registro no encontrado"),
        Err(e) => HttpResponse::BadRequest().body(e.to_string()),
    }
}

#[post("/logs")]
async fn create_log_handler(db: web::Data<Database>, new_log: web::Json<Log>,rep: HttpRequest) -> impl Responder {
    if !check_admin(rep).await {
        return HttpResponse::Unauthorized().body("Acceso no autorizado");
    }
    let collection = db.collection::<Log>("logs");
    let mut log = new_log.into_inner();
    log.id = None;
    log.time = DateTime::now();
    match collection.insert_one(log).await {
        Ok(result) => HttpResponse::Ok().json(result.inserted_id),
        Err(_) => HttpResponse::BadRequest().body("Error inesperado, inténtelo  nuevamente"),
    }
}

#[patch("/logs/{id}")]
async fn patch_log_handler(
    db: web::Data<Database>,
    path: web::Path<String>,
    updated_log: web::Json<Log>,
    rep: HttpRequest,
) -> impl Responder {
    if !check_admin(rep).await {
        return HttpResponse::Unauthorized().body("Acceso no autorizado");
    }
    let collection = db.collection::<Log>("logs");
    let obj_id = match ObjectId::parse_str(&path.into_inner()) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("ID inválido"),
    };
    let update_doc = doc! {
        "$set": {
            "description": updated_log.description.clone()
        }
    };
    match collection
        .update_one(doc! {"_id": obj_id}, update_doc)
        .await
    {
        Ok(result) if result.matched_count == 1 => HttpResponse::Ok().body("registro actualizado"),
        Ok(_) => HttpResponse::NotFound().body("registro no encontrado"),
        Err(_) => HttpResponse::BadRequest().body("Error inesperado, inténtelo  nuevamente"),
    }
}

#[delete("/logs/{id}")]
async fn delete_log_handler(db: web::Data<Database>, path: web::Path<String>,rep:HttpRequest) -> impl Responder {
    if !check_admin(rep).await {
        return HttpResponse::Unauthorized().body("Acceso no autorizado");
    }
    let collection = db.collection::<Log>("logs");
    let obj_id = match ObjectId::parse_str(&path.into_inner()) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("ID inválido"),
    };
    match collection.delete_one(doc! {"_id": obj_id}).await {
        Ok(result) if result.deleted_count == 1 => HttpResponse::Ok().body("registro eliminado"),
        Ok(_) => HttpResponse::NotFound().body("registro no encontrado"),
        Err(_) => HttpResponse::BadRequest().body("Error inesperado, inténtelo  nuevamente"),
    }
}

pub async fn check_admin(req: HttpRequest) -> bool {
    if let Some(claims) = req.extensions().get::<crate::middleware::auth::Claims>().cloned() {
        return claims.role == "admin";
    }
    false
}
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_log_handler)
        .service(get_logs_handler)
        .service(create_log_handler)
        .service(patch_log_handler)
        .service(delete_log_handler);
}
