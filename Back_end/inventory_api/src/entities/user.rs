use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use futures_util::stream::TryStreamExt;
use mongodb::{
    bson::{doc, oid::ObjectId},
    Database,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub mail: String,
    #[serde(rename = "passwordHash")]
    pub password_hash: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub admin: Option<bool>,
}

impl User {
    pub fn new(mail: String, password_hash: String, name: String, admin: Option<bool>) -> Self {
        Self {
            id: None,
            mail,
            password_hash,
            name,
            admin,
        }
    }
}

#[get("/users")]
async fn get_users_handler(db: web::Data<Database>) -> impl Responder {
    let collection = db.collection::<User>("users");
    let cursor = match collection.find(doc! {}).await {
        Ok(cursor) => cursor,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };
    let users: Vec<User> = match cursor.try_collect().await {
        Ok(users) => users,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };
    HttpResponse::Ok().json(users)
}

#[post("/users")]
async fn create_user_handler(db: web::Data<Database>, new_user: web::Json<User>) -> impl Responder {
    let collection = db.collection::<User>("users");
    let mut user = new_user.into_inner();
    user.id = None;
    match collection.insert_one(user).await {
        Ok(result) => HttpResponse::Ok().json(result.inserted_id),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[get("/users/{id}")]
async fn get_user_handler(db: web::Data<Database>, path: web::Path<String>) -> impl Responder {
    let collection = db.collection::<User>("users");
    let obj_id = match ObjectId::parse_str(&path.into_inner()) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("ID inválido"),
    };
    match collection.find_one(doc! {"_id": obj_id}).await {
        Ok(Some(user)) => HttpResponse::Ok().json(user),
        Ok(None) => HttpResponse::NotFound().body("Usuario no encontrado"),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[put("/users/{id}")]
async fn update_user_handler(
    db: web::Data<Database>,
    path: web::Path<String>,
    updated_user: web::Json<User>,
) -> impl Responder {
    let collection = db.collection::<User>("users");
    let obj_id = match ObjectId::parse_str(&path.into_inner()) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("ID inválido"),
    };
    let update_doc = doc! {
        "$set": {
            "mail": updated_user.mail.clone(),
            "passwordHash": updated_user.password_hash.clone(),
            "name": updated_user.name.clone(),
            "admin": updated_user.admin,
        }
    };
    match collection
        .update_one(doc! {"_id": obj_id}, update_doc)
        .await
    {
        Ok(result) if result.matched_count == 1 => HttpResponse::Ok().body("Usuario actualizado"),
        Ok(_) => HttpResponse::NotFound().body("Usuario no encontrado"),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[delete("/users/{id}")]
async fn delete_user_handler(db: web::Data<Database>, path: web::Path<String>) -> impl Responder {
    let collection = db.collection::<User>("users");
    let obj_id = match ObjectId::parse_str(&path.into_inner()) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("ID inválido"),
    };
    match collection.delete_one(doc! {"_id": obj_id}).await {
        Ok(result) if result.deleted_count == 1 => HttpResponse::Ok().body("Usuario eliminado"),
        Ok(_) => HttpResponse::NotFound().body("Usuario no encontrado"),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_users_handler)
        .service(create_user_handler)
        .service(get_user_handler)
        .service(update_user_handler)
        .service(delete_user_handler);
}
