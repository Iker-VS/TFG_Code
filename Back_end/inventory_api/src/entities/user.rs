use super::user_group::UserGroup;
use crate::entities::user_group::delete_user_group;
use crate::log::write_log;
use crate::middleware::auth::{self};
use actix_web::{delete, get, patch, post, web, HttpMessage, HttpRequest, HttpResponse, Responder};
use futures_util::stream::TryStreamExt;
use mongodb::bson::Document;
use mongodb::{
    bson::{doc, oid::ObjectId},
    Database,
};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json;
use sha2::{Digest, Sha256}; // Nuevo import para cifrado

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

// impl User {
//     pub fn new(mail: String, password_hash: String, name: String, admin: Option<bool>) -> Self {
//         Self {
//             id: None,
//             mail,
//             password_hash,
//             name,
//             admin,
//         }
//     }
// }

// Función helper para cifrar contraseñas
fn hash_password(password: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[get("/users")]
async fn get_users_handler(db: web::Data<Database>, req: HttpRequest) -> impl Responder {
    // Recupera las claims inyectadas por el middleware
    let claims = match req
        .extensions()
        .get::<crate::middleware::auth::Claims>()
        .cloned()
    {
        Some(claims) => claims,
        None => {
            write_log("GET /users - Token no encontrado").ok();
            return HttpResponse::Unauthorized().body("Token no encontrado");
        }
    };

    // Solo el admin puede obtener todos los usuarios
    if claims.role != "admin" {
        write_log("GET /users - Acceso no autorizado: se requiere administrador").ok();
        return HttpResponse::Unauthorized()
            .body("Acceso no autorizado: se requiere administrador");
    }

    let collection = db.collection::<User>("users");
    let cursor = match collection.find(doc! {}).await {
        Ok(cursor) => cursor,
        Err(_) => {
            write_log("GET /users - Error buscando usuarios").ok();
            return HttpResponse::BadRequest().body("Error inesperado, inténtelo  nuevamente");
        }
    };
    let users: Vec<User> = match cursor.try_collect().await {
        Ok(users) => users,
        Err(_) => {
            write_log("GET /users - Error recogiendo usuarios").ok();
            return HttpResponse::BadRequest().body("Error inesperado, inténtelo  nuevamente");
        }
    };
    write_log(&format!(
        "GET /users - {} usuarios recuperados",
        users.len()
    ))
    .ok();
    HttpResponse::Ok().json(users)
}

#[get("/users/{id}")]
async fn get_user_handler(
    db: web::Data<Database>,
    path: web::Path<String>,
    req: HttpRequest,
) -> impl Responder {
    // Retrieve claims from the middleware and ensure admin access
    let claims = match req
        .extensions()
        .get::<crate::middleware::auth::Claims>()
        .cloned()
    {
        Some(c) => c,
        None => {
            write_log("GET /users/{id} - Token no encontrado").ok();
            return HttpResponse::Unauthorized().body("Token no encontrado");
        }
    };
    if claims.role != "admin" {
        write_log("GET /users/{id} - Acceso no autorizado: se requiere administrador").ok();
        return HttpResponse::Unauthorized()
            .body("Acceso no autorizado: se requiere administrador");
    }
    // ...existing code...
    let collection = db.collection::<User>("users");
    let obj_id = match ObjectId::parse_str(&path.into_inner()) {
        Ok(id) => id,
        Err(_) => {
            write_log("GET /users/{id} - ID inválido").ok();
            return HttpResponse::BadRequest().body("ID inválido");
        }
    };
    match collection.find_one(doc! {"_id": obj_id}).await {
        Ok(Some(user)) => {
            write_log("GET /users/{id} - Usuario encontrado").ok();
            HttpResponse::Ok().json(user)
        }
        Ok(None) => {
            write_log("GET /users/{id} - Usuario no encontrado").ok();
            HttpResponse::NotFound().body("Usuario no encontrado")
        }
        Err(_) => {
            write_log("GET /users/{id} - Error inesperado").ok();
            HttpResponse::BadRequest().body("Error inesperado, inténtelo  nuevamente")
        }
    }
}

#[get("/users/me/")]
async fn get_my_user_handler(db: web::Data<Database>, req: HttpRequest) -> impl Responder {
    // Retrieve claims from the token
    let claims = match req
        .extensions()
        .get::<crate::middleware::auth::Claims>()
        .cloned()
    {
        Some(c) => c,
        None => {
            write_log("GET /users/me - Token no encontrado").ok();
            return HttpResponse::Unauthorized().body("Token no encontrado");
        }
    };
    // Extract user id from claims and parse as ObjectId
    let obj_id = match ObjectId::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => {
            write_log("GET /users/me - ID inválido").ok();
            return HttpResponse::BadRequest().body("ID inválido");
        }
    };
    // Query the "users" collection for the user's document
    let collection = db.collection::<User>("users");
    match collection.find_one(doc! {"_id": obj_id}).await {
        Ok(Some(user)) => {
            write_log("GET /users/me - Usuario encontrado").ok();
            HttpResponse::Ok().json(user)
        }
        Ok(None) => {
            write_log("GET /users/me - Usuario no encontrado").ok();
            HttpResponse::NotFound().body("Usuario no encontrado")
        }
        Err(_) => {
            write_log("GET /users/me - Error inesperado").ok();
            HttpResponse::BadRequest().body("Error inesperado, inténtelo  nuevamente")
        }
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
        None => {
            write_log("POST /users/login - Falta el campo 'mail'").ok();
            return HttpResponse::BadRequest().body("Falta el campo 'mail'");
        }
    };
    let password = match body.get("password").and_then(|v| v.as_str()) {
        Some(p) => p,
        None => {
            write_log("POST /users/login - Falta el campo 'password'").ok();
            return HttpResponse::BadRequest().body("Falta el campo 'password'");
        }
    };

    // Cifrar la contraseña recibida
    let hashed_password = hash_password(password);

    let collection = db.collection::<User>("users");
    match collection
        .find_one(doc! {"$and": [{"mail": mail}, {"passwordHash": hashed_password}]})
        .await
    {
        Ok(Some(user)) => {
            let token = auth::generate_token(
                user.id.clone().unwrap().to_hex(),
                user.admin.map_or("user".to_string(), |b| {
                    if b {
                        "admin".to_string()
                    } else {
                        "user".to_string()
                    }
                }),
            );
            write_log("POST /users/login - Login correcto").ok();
            HttpResponse::Ok().json(serde_json::json!({
                "token": token,
                "user": user
            }))
        }
        Ok(None) => {
            write_log("POST /users/login - Usuario no encontrado").ok();
            HttpResponse::NotFound().body("Usuario no encontrado")
        }
        Err(_) => {
            write_log("POST /users/login - Usuario o contraseña erronea").ok();
            HttpResponse::NotFound().body("Usuario o contraseña erronea")
        }
    }
}

#[post("/users/register")]
async fn create_user_handler(db: web::Data<Database>, new_user: web::Json<User>) -> impl Responder {
    let collection = db.collection::<User>("users");

    if collection
        .find_one(doc! {"mail": &new_user.mail})
        .await
        .unwrap_or(None)
        .is_some()
    {
        write_log("POST /users/register - El correo está en uso").ok();
        return HttpResponse::BadRequest().body("El correo está en uso");
    }

    // Valida que el correo cumpla con la expresión regular "^.+@.+$"
    let email_regex = Regex::new(r"^.+@.+$").expect("Failed to create regex");
    if !email_regex.is_match(&new_user.mail) {
        write_log("POST /users/register - El correo no es válido").ok();
        return HttpResponse::BadRequest().body("El correo no es válido");
    }

    let mut user = new_user.into_inner();
    // Cifrar la contraseña antes de guardarla
    user.password_hash = hash_password(&user.password_hash);
    user.id = None;
    user.admin = None;
    match collection.insert_one(&user).await {
        Ok(result) => {
            user.id = result.inserted_id.as_object_id();
            //let token = auth::generate_token(result.inserted_id.to_string(), "user".to_string());
            let token = auth::generate_token(
                result.inserted_id.as_object_id().unwrap().to_hex(),
                "user".to_string(),
            );
            write_log("POST /users/register - Usuario registrado correctamente").ok();
            HttpResponse::Ok().json(serde_json::json!({
                "token": token,
                "user": user
            }))
        }
        Err(_) => {
            write_log("POST /users/register - Error inesperado").ok();
            HttpResponse::BadRequest().body("Error inesperado, vuelva a intentarlo")
        }
    }
}

#[patch("/users/{id}")]
async fn patch_user_admin_handler(
    db: web::Data<Database>,
    path: web::Path<String>,
    updated_user: web::Json<serde_json::Value>,
    req: HttpRequest,
) -> impl Responder {
    // Recupera las claims ya decodificadas
    let claims = match req
        .extensions()
        .get::<crate::middleware::auth::Claims>()
        .cloned()
    {
        Some(c) => c,
        None => {
            write_log("PATCH /users/{id} - Token no encontrado").ok();
            return HttpResponse::Unauthorized().body("Token no encontrado");
        }
    };

    // Verificar que es admin
    if claims.role != "admin" {
        write_log("PATCH /users/{id} - Acceso no autorizado: se requiere administrador").ok();
        return HttpResponse::Unauthorized()
            .body("Acceso no autorizado: se requiere administrador");
    }

    let user_id = path.into_inner();
    let collection = db.collection::<User>("users");
    let obj_id = match ObjectId::parse_str(&user_id) {
        Ok(id) => id,
        Err(_) => {
            write_log("PATCH /users/{id} - ID inválido").ok();
            return HttpResponse::BadRequest().body("ID inválido");
        }
    };

    let mut set_doc = Document::new();
    let mut unset_doc = Document::new();

    // Campo: name (solo si está presente con valor)
    if let Some(value) = updated_user.get("name") {
        match value {
            serde_json::Value::String(name) => set_doc.insert("name", name.clone()),
            serde_json::Value::Null => {
                write_log("PATCH /users/{id} - 'name' no puede ser null").ok();
                return HttpResponse::BadRequest().body("'name' no puede ser null");
            }
            _ => {
                write_log("PATCH /users/{id} - Valor inválido para 'name'").ok();
                return HttpResponse::BadRequest().body("Valor inválido para 'name'");
            }
        };
    }

    // Campo: passwordHash (solo si está presente con valor)
    if let Some(value) = updated_user.get("passwordHash") {
        match value {
            serde_json::Value::String(pass) => {
                let hashed = hash_password(pass);
                set_doc.insert("passwordHash", hashed);
            }
            serde_json::Value::Null => {
                write_log("PATCH /users/{id} - 'passwordHash' no puede ser null").ok();
                return HttpResponse::BadRequest().body("'passwordHash' no puede ser null");
            }
            _ => {
                write_log("PATCH /users/{id} - Valor inválido para 'passwordHash'").ok();
                return HttpResponse::BadRequest().body("Valor inválido para 'passwordHash'");
            }
        };
    }

    // Campo admin (solo admin puede modificar)
    if let Some(value) = updated_user.get("admin") {
        match value {
            serde_json::Value::Bool(b) => set_doc.insert("admin", *b),
            serde_json::Value::Null => unset_doc.insert("admin", ""),
            _ => {
                write_log("PATCH /users/{id} - Valor inválido para 'admin'").ok();
                return HttpResponse::BadRequest().body("Valor inválido para 'admin'");
            }
        };
    }

    // Validar que haya algo que actualizar
    if set_doc.is_empty() && unset_doc.is_empty() {
        write_log("PATCH /users/{id} - No hay campos para actualizar").ok();
        return HttpResponse::BadRequest().body("No hay campos para actualizar");
    }

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
        Ok(result) if result.matched_count == 1 => {
            write_log("PATCH /users/{id} - Usuario actualizado").ok();
            HttpResponse::Ok().body("Usuario actualizado")
        }
        Ok(_) => {
            write_log("PATCH /users/{id} - Usuario no encontrado").ok();
            HttpResponse::NotFound().body("Usuario no encontrado")
        }
        Err(_) => {
            write_log("PATCH /users/{id} - Error inesperado").ok();
            HttpResponse::BadRequest().body("Error inesperado, inténtelo nuevamente")
        }
    }
}

#[patch("/users/me/")]
async fn patch_user_me_handler(
    db: web::Data<Database>,
    updated_user: web::Json<serde_json::Value>,
    req: HttpRequest,
) -> impl Responder {
    // Recupera las claims ya decodificadas
    let claims = match req
        .extensions()
        .get::<crate::middleware::auth::Claims>()
        .cloned()
    {
        Some(c) => c,
        None => {
            write_log("PATCH /users/me - Token no encontrado").ok();
            return HttpResponse::Unauthorized().body("Token no encontrado");
        }
    };

    let collection = db.collection::<User>("users");
    let obj_id = match ObjectId::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => {
            write_log("PATCH /users/me - ID inválido").ok();
            return HttpResponse::BadRequest().body("ID inválido");
        }
    };

    let mut set_doc = Document::new();

    // Campo: name (solo si está presente con valor)
    if let Some(value) = updated_user.get("name") {
        match value {
            serde_json::Value::String(name) => set_doc.insert("name", name.clone()),
            serde_json::Value::Null => {
                write_log("PATCH /users/me - 'name' no puede ser null").ok();
                return HttpResponse::BadRequest().body("'name' no puede ser null");
            }
            _ => {
                write_log("PATCH /users/me - Valor inválido para 'name'").ok();
                return HttpResponse::BadRequest().body("Valor inválido para 'name'");
            }
        };
    }

    // Campo: passwordHash (solo si está presente con valor)
    if let Some(value) = updated_user.get("passwordHash") {
        match value {
            serde_json::Value::String(pass) => {
                let hashed = hash_password(pass);
                set_doc.insert("passwordHash", hashed);
            }
            serde_json::Value::Null => {
                write_log("PATCH /users/me - 'passwordHash' no puede ser null").ok();
                return HttpResponse::BadRequest().body("'passwordHash' no puede ser null");
            }
            _ => {
                write_log("PATCH /users/me - Valor inválido para 'passwordHash'").ok();
                return HttpResponse::BadRequest().body("Valor inválido para 'passwordHash'");
            }
        };
    }

    // Validar que haya algo que actualizar
    if set_doc.is_empty() {
        write_log("PATCH /users/me - No hay campos para actualizar").ok();
        return HttpResponse::BadRequest().body("No hay campos para actualizar");
    }

    let update_doc = doc! {"$set": set_doc};

    match collection
        .update_one(doc! {"_id": obj_id}, update_doc)
        .await
    {
        Ok(result) if result.matched_count == 1 => {
            write_log("PATCH /users/me - Usuario actualizado").ok();
            HttpResponse::Ok().body("Usuario actualizado")
        }
        Ok(_) => {
            write_log("PATCH /users/me - Usuario no encontrado").ok();
            HttpResponse::NotFound().body("Usuario no encontrado")
        }
        Err(_) => {
            write_log("PATCH /users/me - Error inesperado").ok();
            HttpResponse::BadRequest().body("Error inesperado, inténtelo nuevamente")
        }
    }
}

pub async fn delete_user(db: &Database, user_id: String) -> HttpResponse {
    let item_collection = db.collection::<User>("users");
    let user_group_collection = db.collection::<UserGroup>("userGroup");
    let obj_id = match ObjectId::parse_str(user_id) {
        Ok(id) => id,
        Err(_) => {
            write_log("DELETE /users/{id} - ID inválido").ok();
            return HttpResponse::BadRequest().body("ID inválido");
        }
    };

    let user_group_cursor = match user_group_collection.find(doc! {"userId":obj_id}).await {
        Ok(user_group) => user_group,
        Err(_) => {
            write_log("DELETE /users/{id} - Error buscando relaciones").ok();
            return HttpResponse::BadRequest().body("Error inesperado, inténtelo  nuevamente");
        }
    };
    let users_groups: Vec<UserGroup> = match user_group_cursor.try_collect().await {
        Ok(user_group) => user_group,
        Err(_) => {
            write_log("DELETE /users/{id} - Error recogiendo relaciones").ok();
            return HttpResponse::BadRequest().body("Error inesperado, inténtelo  nuevamente");
        }
    };
    for user_group in users_groups {
        let id = match user_group.id {
            Some(id) => id,
            None => {
                write_log("DELETE /users/{id} - Relación sin ID").ok();
                return HttpResponse::BadRequest().body("No hay ID");
            }
        };
        print!("{:?}", id);
        let res = delete_user_group(db, id.to_string()).await;
        if !res.status().is_success() {
            write_log("DELETE /users/{id} - Error eliminando relación").ok();
            return res; // Si falla, detenemos la ejecución y devolvemos el error
        }
    }

    match item_collection.delete_one(doc! {"_id": obj_id}).await {
        Ok(result) if result.deleted_count == 1 => {
            write_log("DELETE /users/{id} - Usuario eliminado correctamente").ok();
            HttpResponse::Ok().body("Usuario eliminado")
        }
        Ok(_) => {
            write_log("DELETE /users/{id} - Usuario no encontrado").ok();
            HttpResponse::NotFound().body("Usuario no encontrado")
        }
        Err(_) => {
            write_log("DELETE /users/{id} - Error inesperado").ok();
            return HttpResponse::BadRequest().body("Error inesperado, inténtelo  nuevamente");
        }
    }
}

#[delete("/users/{id}")]
async fn delete_user_admin_handler(
    db: web::Data<Database>,
    path: web::Path<String>,
    req: HttpRequest,
) -> impl Responder {
    let claims = match req
        .extensions()
        .get::<crate::middleware::auth::Claims>()
        .cloned()
    {
        Some(claims) => claims,
        None => {
            write_log("DELETE /users/{id} - Token no encontrado").ok();
            return HttpResponse::Unauthorized().body("Token no encontrado");
        }
    };

    // Verificar que es admin
    if claims.role != "admin" {
        write_log("DELETE /users/{id} - Acceso no autorizado: se requiere administrador").ok();
        return HttpResponse::Unauthorized()
            .body("Acceso no autorizado: se requiere administrador");
    }

    let user_id = path.into_inner();
    let client = db.client();
    let mut session = match client.start_session().await {
        Ok(s) => s,
        Err(_) => {
            write_log("DELETE /users/{id} - Error iniciando sesión").ok();
            return HttpResponse::BadRequest().body("Error inesperado, inténtelo  nuevamente");
        }
    };

    session.start_transaction().await.ok();
    let response = delete_user(&db, user_id).await;

    if response.status().is_success() {
        session.commit_transaction().await.ok();
        write_log("DELETE /users/{id} - Transacción confirmada").ok();
    } else {
        session.abort_transaction().await.ok();
        write_log("DELETE /users/{id} - Transacción abortada").ok();
    }
    response
}

#[delete("/users/me/")]
async fn delete_user_me_handler(db: web::Data<Database>, req: HttpRequest) -> impl Responder {
    let claims = match req
        .extensions()
        .get::<crate::middleware::auth::Claims>()
        .cloned()
    {
        Some(claims) => claims,
        None => {
            write_log("DELETE /users/me - Token no encontrado").ok();
            return HttpResponse::Unauthorized().body("Token no encontrado");
        }
    };

    let client = db.client();
    let mut session = match client.start_session().await {
        Ok(s) => s,
        Err(_) => {
            write_log("DELETE /users/me - Error iniciando sesión").ok();
            return HttpResponse::BadRequest().body("Error inesperado, inténtelo  nuevamente");
        }
    };

    session.start_transaction().await.ok();
    let response = delete_user(&db, claims.sub).await;

    if response.status().is_success() {
        session.commit_transaction().await.ok();
        write_log("DELETE /users/me - Transacción confirmada").ok();
    } else {
        session.abort_transaction().await.ok();
        write_log("DELETE /users/me - Transacción abortada").ok();
    }
    response
}

pub fn configure_private_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_users_handler)
        .service(get_user_handler)
        .service(get_my_user_handler)
        .service(patch_user_admin_handler)
        .service(patch_user_me_handler)
        .service(delete_user_admin_handler)
        .service(delete_user_me_handler);
}
pub fn configure_public_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(login_handler).service(create_user_handler);
}
