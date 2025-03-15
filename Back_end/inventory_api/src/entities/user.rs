use super::user_group::UserGroup;
use crate::middleware::auth::{self, Claims};
use crate::{entities::user_group::delete_user_group, middleware::auth::decode_token};
use actix_web::{delete, get, post, put, web, HttpRequest, HttpResponse, Responder};
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

#[post("/users/login/{mail}/{password}")]
async fn login_handler(
    db: web::Data<Database>,
    path: web::Path<(String, String)>,
) -> impl Responder {
    let (mail, password) = path.into_inner();
    let collection = db.collection::<User>("users");
    match collection
        .find_one(doc! {"$and":[{"mail":&mail},{"passwordHash":&password}]})
        .await
    {
        Ok(Some(user)) => {
            return HttpResponse::Ok().body(auth::generate_token(
                user.id.unwrap().to_hex(),
                user.admin
                    .map_or("user", |b| if b { "admin" } else { "user" })
                    .to_string(),
            ))
        }
        Ok(None) => return HttpResponse::NotFound().body("Usuario no encontrado"),
        Err(_) => return HttpResponse::NotFound().body("Usuario o contraseña erronea"),
    };
}

#[post("/users/register")]
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
    user.admin=None;
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
    req:HttpRequest,
) -> impl Responder {
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok());
    let token = auth_header
        .map(|s| s.trim_start_matches("Bearer ").trim())
        .unwrap_or("");
    let claims = match decode_token(token) {
        Ok(claims) => claims,
        Err(e) => return HttpResponse::Unauthorized().body(e.to_string()),
    };
    let user_id = path.into_inner();
    if !(claims.role == "admin" || (claims.role == "user" && claims.sub == user_id)) {
        return HttpResponse::Unauthorized().body("Acceso no autorizado");
    }
    let collection = db.collection::<User>("users");
    let obj_id = match ObjectId::parse_str(&user_id) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("ID inválido"),
    };
    let mut update_doc = doc! {
        "$set": {
            "mail": updated_user.mail.clone(),
            "passwordHash": updated_user.password_hash.clone(),
            "name": updated_user.name.clone(),
        }
    };

    if let Some(admin) = &updated_user.admin  {
        if claims.role =="admin"{
        update_doc
            .get_mut("$set")
            .unwrap()
            .as_document_mut()
            .unwrap()
            .insert("admin", admin.clone());
        }
    } else {
        update_doc.insert("$unset", doc! {"admin": ""});
    }
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
        print!("{:?}", id);
        let res = delete_user_group(db, id.to_string()).await;
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
async fn delete_user_handler(
    db: web::Data<Database>,
    path: web::Path<String>,
    req: HttpRequest,
) -> impl Responder {
    let client = db.client();
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok());
    let token = auth_header
        .map(|s| s.trim_start_matches("Bearer ").trim())
        .unwrap_or("");
    let claims = match decode_token(token) {
        Ok(claims) => claims,
        Err(e) => return HttpResponse::Unauthorized().body(e.to_string()),
    };
    let user_id = path.into_inner();
    if !(claims.role == "admin" || (claims.role == "user" && claims.sub == user_id)) {
        return HttpResponse::Unauthorized().body("Acceso no autorizado");
    }
    let mut session = match client.start_session().await {
        Ok(s) => s,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };
    session.start_transaction().await.ok();
    let response = delete_user(&db, user_id).await;

    if response.status().is_success() {
        session.commit_transaction().await.ok();
    } else {
        session.abort_transaction().await.ok();
    }
    response
}

pub fn configure_private_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_users_handler)
        .service(get_user_handler)
        .service(update_user_handler)
        .service(delete_user_handler);
}
pub fn configure_public_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(login_handler).service(create_user_handler);
}
