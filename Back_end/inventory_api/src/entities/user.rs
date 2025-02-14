use super::user_group::{delete_user_group_from_user, UserGroup};
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

#[post("/users")]
async fn create_user_handler(db: web::Data<Database>, new_user: web::Json<User>) -> impl Responder {
    let collection = db.collection::<User>("users");
    if collection
        .find_one(doc! {"mail":&new_user.mail})
        .await
        .unwrap_or(None)
        .is_some()
    {
        return HttpResponse::BadRequest().body("El correo está en uso");
    }
    let mut user = new_user.into_inner();
    user.id = None;
    match collection.insert_one(user).await {
        Ok(result) => HttpResponse::Ok().json(result.inserted_id),
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
            "admin": updated_user.admin.clone(),
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

pub async fn delete_user(db: &Database, user_id: String) -> HttpResponse {
    let item_collection = db.collection::<User>("users");
    let user_group_collection = db.collection::<UserGroup>("userGroup");
    let obj_id = match ObjectId::parse_str(user_id) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("ID inválido"),
    };

    let user_group_cursor = match user_group_collection.find(doc! {"userId":obj_id}).await {
        Ok(user_group) => user_group,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };
    let users_groups: Vec<UserGroup> = match user_group_cursor.try_collect().await {
        Ok(user_group) => user_group,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };
    for user_group in users_groups {
        let id = match user_group.id {
            Some(id) => id,
            None => return HttpResponse::BadRequest().body("No hay ID"),
        };
        let res = delete_user_group_from_user(db, id.to_string()).await;
        if !res.status().is_success() {
            return res; // Si falla, detenemos la ejecución y devolvemos el error
        }
    }

    match item_collection.delete_one(doc! {"_id": obj_id}).await {
        Ok(result) if result.deleted_count == 1 => HttpResponse::Ok().body("Usuario eliminado"),
        Ok(_) => HttpResponse::NotFound().body("Usuario no encontrado"),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[delete("/users/{id}")]
async fn delete_user_handler(db: web::Data<Database>, path: web::Path<String>) -> impl Responder {
    let client = db.client();
    let mut session = match client.start_session().await {
        Ok(s) => s,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };
    session.start_transaction().await.ok();
    let response = delete_user(&db, path.into_inner()).await;

    if response.status().is_success() {
        session.commit_transaction().await.ok();
    } else {
        session.abort_transaction().await.ok();
    }
    response
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_users_handler)
        .service(create_user_handler)
        .service(get_user_handler)
        .service(update_user_handler)
        .service(delete_user_handler);
}
