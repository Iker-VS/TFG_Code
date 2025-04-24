use super::user_group::UserGroup;
use crate::middleware::auth::{self};
use crate::{entities::user_group::delete_user_group};
use actix_web::{delete, get, patch, post, web, HttpMessage, HttpRequest, HttpResponse, Responder};
use futures_util::stream::TryStreamExt;
use regex::Regex;
use mongodb::{
    bson::{doc, oid::ObjectId},
    Database,
};
use serde::{Deserialize, Serialize};
use serde_json;

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
async fn get_users_handler(
    db: web::Data<Database>,
    req: HttpRequest,
) -> impl Responder {
    // Recupera las claims inyectadas por el middleware
    let claims = match req.extensions().get::<crate::middleware::auth::Claims>().cloned() {
        Some(claims) => claims,
        None => return HttpResponse::Unauthorized().body("Token no encontrado"),
    };

    // Solo el admin puede obtener todos los usuarios
    if claims.role != "admin" {
        return HttpResponse::Unauthorized().body("Acceso no autorizado: se requiere administrador");
    }

    let collection = db.collection::<User>("users");
    let cursor = match collection.find(doc! {}).await {
        Ok(cursor) => cursor,
        Err(_) => return HttpResponse::BadRequest().body("Error inesperado, intentelo nuevamente"),
    };
    let users: Vec<User> = match cursor.try_collect().await {
        Ok(users) => users,
        Err(_) => return HttpResponse::BadRequest().body("Error inesperado, intentelo nuevamente"),
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
        Err(_) => HttpResponse::BadRequest().body("Error inesperado, intentelo nuevamente"),
    }
}


#[post("/users/login")]
async fn login_handler(
    db: web::Data<Database>,
    body: web::Json<serde_json::Value>,
) -> impl Responder {
    // Extraer "mail" y "password" del body
    let mail = match body.get("mail").and_then(|v| v.as_str()) {
        Some(m) => m,
        None => return HttpResponse::BadRequest().body("Falta el campo 'mail'"),
    };
    let password = match body.get("password").and_then(|v| v.as_str()) {
        Some(p) => p,
        None => return HttpResponse::BadRequest().body("Falta el campo 'password'"),
    };

    let collection = db.collection::<User>("users");
    match collection
        .find_one(doc! {"$and": [{"mail": mail}, {"passwordHash": password}]})
        .await
    {
        Ok(Some(user)) => {
            let token = auth::generate_token(
                user.id.clone().unwrap().to_hex(),
                user.admin.map_or("user".to_string(), |b| if b { "admin".to_string() } else { "user".to_string() }),
            );
            HttpResponse::Ok().json(serde_json::json!({
                "token": token,
                "user": user
            }))
        }
        Ok(None) => HttpResponse::NotFound().body("Usuario no encontrado"),
        Err(_) => HttpResponse::NotFound().body("Usuario o contraseña erronea"),
    }
}

#[post("/users/register")]
async fn create_user_handler(
    db: web::Data<Database>,
    new_user: web::Json<User>
) -> impl Responder {
    let collection = db.collection::<User>("users");
    
    if collection
        .find_one(doc! {"mail": &new_user.mail})
        .await
        .unwrap_or(None)
        .is_some()
    {
        return HttpResponse::BadRequest().body("El correo está en uso");
    }
    
    // Valida que el correo cumpla con la expresión regular "^.+@.+$"
    let email_regex = Regex::new(r"^.+@.+$").expect("Failed to create regex");
    if !email_regex.is_match(&new_user.mail) {
        return HttpResponse::BadRequest().body("El correo no es válido");
    }
    
    let mut user = new_user.into_inner();
    user.id = None;
    user.admin = None;
    match collection.insert_one(&user).await {
        Ok(result) => {
            user.id = result.inserted_id.as_object_id();
            let token = auth::generate_token(result.inserted_id.to_string(), "user".to_string());
            HttpResponse::Ok().json(serde_json::json!({
                "token": token,
                "user": user
            }))
        }
        Err(_) => HttpResponse::BadRequest().body("Error inesperado, vuelva a intentarlo"),
    }
}


#[patch("/users/{id}")]
async fn update_user_handler(
    db: web::Data<Database>,
    path: web::Path<String>,
    updated_user: web::Json<User>,
    req: HttpRequest,
) -> impl Responder {
    // Recupera las claims ya decodificadas del middleware (se asume que Claims implementa Clone)
    let claims = match req.extensions().get::<crate::middleware::auth::Claims>().cloned() {
        Some(claims) => claims,
        None => return HttpResponse::Unauthorized().body("Token no encontrado"),
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
            "passwordHash": updated_user.password_hash.clone(),
            "name": updated_user.name.clone(),
        }
    };

    if let Some(admin) = &updated_user.admin {
        if claims.role == "admin" {
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
        Ok(result) if result.matched_count == 1 => {
            HttpResponse::Ok().body("Usuario actualizado")
        }
        Ok(_) => HttpResponse::NotFound().body("Usuario no encontrado"),
        Err(_) => HttpResponse::BadRequest().body("Error inesperado, intentelo nuevamente"),
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
        Err(_) => return HttpResponse::BadRequest().body("Error inesperado, intentelo nuevamente"),
    };
    let users_groups: Vec<UserGroup> = match user_group_cursor.try_collect().await {
        Ok(user_group) => user_group,
        Err(_) => return HttpResponse::BadRequest().body("Error inesperado, intentelo nuevamente"),
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
        Err(_) => HttpResponse::BadRequest().body("Error inesperado, intentelo nuevamente"),
    }
}

#[delete("/users/{id}")]
async fn delete_user_handler(
    db: web::Data<Database>,
    path: web::Path<String>,
    req: HttpRequest,
) -> impl Responder {
    // Recupera las claims insertadas por el middleware
    let claims = match req.extensions().get::<crate::middleware::auth::Claims>().cloned() {
        Some(claims) => claims,
        None => return HttpResponse::Unauthorized().body("Token no encontrado"),
    };

    let user_id = path.into_inner();
    if !(claims.role == "admin" || (claims.role == "user" && claims.sub == user_id)) {
        return HttpResponse::Unauthorized().body("Acceso no autorizado");
    }
    
    let client = db.client();
    let mut session = match client.start_session().await {
        Ok(s) => s,
        Err(_) => return HttpResponse::BadRequest().body("Error inesperado, intentelo nuevamente"),
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
