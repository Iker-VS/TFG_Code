use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
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
impl Log {
    fn new(description: String, time: DateTime, group_id: ObjectId, user_id: ObjectId) -> Log {
        Self {
            id: None,
            description,
            time,
            group_id,
            user_id,
        }
    }
}

#[get("/logs")]
async fn get_logs_handler(db: web::Data<Database>) -> impl Responder {
    let collection = db.collection::<Log>("logs");
    let cursor = match collection.find(doc! {}).await {
        Ok(cursor) => cursor,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };
    let logs: Vec<Log> = match cursor.try_collect().await {
        Ok(logs) => logs,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };
    HttpResponse::Ok().json(logs)
}
#[get("/logs/{id}")]
async fn get_log_handler(db: web::Data<Database>, path: web::Path<String>) -> impl Responder {
    let collection = db.collection::<Log>("log");
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

