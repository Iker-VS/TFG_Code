use actix_web::{delete, get, post, patch, web, HttpMessage, HttpRequest, HttpResponse, Responder};
use futures_util::stream::TryStreamExt;
use mongodb::{
    bson::{self, doc, oid::ObjectId},
    Database,
};
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::entities::user_group::UserGroup;
use crate::log::write_log;

use super::{
    property::{delete_property, Property},
    user_group::delete_user_group,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateGroup {
    pub name: String,
    #[serde(rename = "userMax")]
    pub user_max: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]

pub struct Group {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    #[serde(rename = "userMax", skip_serializing_if = "Option::is_none")]
    pub user_max: Option<i32>,
    #[serde(rename = "userCount")]
    pub user_count: i32,
    #[serde(rename = "groupCode")]
    pub group_code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
}

impl Group {
    // fn new(
    //     name: String,
    //     user_max: Option<i32>,
    //     user_count: i32,
    //     group_code: Option<String>,
    //     tags: Option<Vec<String>>,
    // ) -> Group {
    //     Self {
    //         id: None,
    //         name,
    //         user_max,
    //         user_count,
    //         group_code: group_code
    //             .or_else(|| Some(Self::create_group_code()))
    //             .unwrap(),
    //         tags,
    //     }
    // }
}

async fn generate_unique_group_code(collection: &mongodb::Collection<Group>) -> String {
    loop {
        let mut rand = rand::rng();
        let characters: Vec<char> = ('0'..='9').chain('a'..='z').chain('A'..='Z').collect();

        let group_code = (0..8)
            .map(|_| characters[rand.random_range(0..characters.len())])
            .collect::<String>();

        if collection
            .find_one(doc! {"groupCode": &group_code})
            .await
            .unwrap_or(None)
            .is_none() 
        {
            return group_code;
        }
    }
}

#[get("/groups")]
async fn get_groups_handler(db: web::Data<Database>, req: HttpRequest) -> impl Responder {
    let claims = match req
        .extensions()
        .get::<crate::middleware::auth::Claims>()
        .cloned()
    {
        Some(c) => c,
        None => {
            write_log("GET /groups - Token no encontrado").ok();
            return HttpResponse::Unauthorized().body("Token no encontrado");
        },
    };
    if claims.role != "admin" {
        write_log(&format!("GET /groups - Acceso denegado para usuario {}", claims.sub)).ok();
        return HttpResponse::Unauthorized().body("Acceso no autorizado");
    }
    let collection = db.collection::<Group>("groups");
    let cursor = match collection.find(doc! {}).await {
        Ok(cursor) => cursor,
        Err(_) => {
            write_log(&format!("GET /groups - Error en find para usuario {}", claims.sub)).ok();
            return HttpResponse::BadRequest().body("Error inesperado, inténtelo  nuevamente");
        },
    };
    let groups: Vec<Group> = match cursor.try_collect().await {
        Ok(groups) => groups,
        Err(_) => {
            write_log(&format!("GET /groups - Error en try_collect para usuario {}", claims.sub)).ok();
            return HttpResponse::BadRequest().body("Error inesperado, inténtelo  nuevamente");
        },
    };
    write_log(&format!("GET /groups - usuario {} obtuvo {} grupos", claims.sub, groups.len())).ok();
    HttpResponse::Ok().json(groups)
}

#[get("/groups/{id}")]
async fn get_group_handler(db: web::Data<Database>, path: web::Path<String>) -> impl Responder {
    let collection = db.collection::<Group>("groups");
    let id_str = path.into_inner();
    let obj_id = match ObjectId::parse_str(&id_str) {
        Ok(obj_id) => obj_id,
        Err(_) => {
            write_log(&format!("GET /groups/{{id}} - ID inválido: {}", id_str)).ok();
            return HttpResponse::BadRequest().body("ID inválido");
        },
    };
    match collection.find_one(doc! {"_id":obj_id}).await {
        Ok(Some(group)) => {
            write_log(&format!("GET /groups/{{id}} - Grupo encontrado: {:?}", group)).ok();
            HttpResponse::Ok().json(group)
        },
        Ok(None) => {
            write_log(&format!("GET /groups/{{id}} - Grupo no encontrado: {}", obj_id)).ok();
            HttpResponse::NotFound().body("Grupo no encontrado")
        },
        Err(e) => {
            write_log(&format!("GET /groups/{{id}} - Error: {}", e)).ok();
            HttpResponse::BadRequest().body(e.to_string())
        },
    }
}
#[get("/groups/code/{code}")]
async fn get_group_by_code_handler(
    db: web::Data<Database>,
    path: web::Path<String>,
) -> impl Responder {
    let collection = db.collection::<Group>("groups");
    let group_code = path.into_inner();
    match collection.find_one(doc! {"groupCode": group_code.clone()}).await {
        Ok(Some(group)) => {
            write_log(&format!("GET /groups/code/{{code}} - Grupo encontrado: code={:?}, grupo={:?}", group_code, group)).ok();
            HttpResponse::Ok().json(group)
        },
        Ok(None) => {
            write_log(&format!("GET /groups/code/{{code}} - Grupo no encontrado: code={:?}", group_code)).ok();
            HttpResponse::NotFound().body("Grupo no encontrado")
        },
        Err(e) => {
            write_log(&format!("GET /groups/code/{{code}} - Error: {}", e)).ok();
            HttpResponse::BadRequest().body(e.to_string())
        },
    }
}

#[post("/groups")]
async fn create_group_handler(
    db: web::Data<Database>,
    new_group: web::Json<CreateGroup>,
    req: HttpRequest,
) -> impl Responder {
    let claims = match req.extensions().get::<crate::middleware::auth::Claims>().cloned() {
        Some(claims) => claims,
        None => {
            write_log("POST /groups - Token no encontrado").ok();
            return HttpResponse::Unauthorized().body("Token no encontrado");
        },
    };
    let user_id = match ObjectId::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => {
            write_log(&format!("POST /groups - ID de usuario inválido: {}", claims.sub)).ok();
            return HttpResponse::BadRequest().body("ID de usuario inválido");
        },
    };
    let client = db.client();
    let mut session = match client.start_session().await {
        Ok(s) => s,
        Err(_) => {
            write_log("POST /groups - Error al iniciar sesión").ok();
            return HttpResponse::BadRequest().body("Error inesperado, inténtelo  nuevamente");
        },
    };
    session.start_transaction().await.ok();
    let collection = db.collection::<Group>("groups");
    let group_code = generate_unique_group_code(&collection).await;
    let group = Group {
        id: None,
        name: new_group.name.clone(),
        user_max: new_group.user_max,
        user_count: 1,
        group_code: group_code.clone(),
        tags: None,
    };
    let group_result = match collection.insert_one(group).await {
        Ok(result) => result,
        Err(_) => {
            session.abort_transaction().await.ok();
            write_log(&format!("POST /groups - Error al insertar grupo para usuario {}", user_id)).ok();
            return HttpResponse::BadRequest().body("Error inesperado, inténtelo  nuevamente");
        }
    };
    let group_id = match group_result.inserted_id.as_object_id() {
        Some(id) => id,
        None => {
            session.abort_transaction().await.ok();
            write_log("POST /groups - Error al obtener ID del grupo").ok();
            return HttpResponse::BadRequest().body("Error al obtener ID del grupo");
        }
    };
    let user_group = UserGroup {
        id: None,
        group_id,
        user_id,
    };
    let user_group_collection = db.collection::<UserGroup>("userGroup");
    if let Err(_) = user_group_collection.insert_one(&user_group).await {
        session.abort_transaction().await.ok();
        write_log(&format!("POST /groups - Error al crear relación usuario-grupo para usuario {} y grupo {}", user_id, group_id)).ok();
        return HttpResponse::BadRequest().body("Error al crear la relación usuario-grupo");
    }
    session.commit_transaction().await.ok();
    write_log(&format!("POST /groups - Grupo creado correctamente: id={}, code={}, usuario={}", group_id, group_code, user_id)).ok();
    HttpResponse::Ok().json(group_result.inserted_id)
}


#[post("/groups/join/{code}")]
async fn join_group_handler(
    db: web::Data<Database>,
    path: web::Path<String>,
    req: HttpRequest,
) -> impl Responder {
    let claims = match req.extensions().get::<crate::middleware::auth::Claims>().cloned() {
        Some(claims) => claims,
        None => {
            write_log("POST /groups/join/{code} - Token no encontrado").ok();
            return HttpResponse::Unauthorized().body("Token no encontrado");
        },
    };
    let user_id = match ObjectId::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => {
            write_log(&format!("POST /groups/join/{{code}} - ID de usuario inválido: {}", claims.sub)).ok();
            return HttpResponse::BadRequest().body("ID de usuario inválido");
        },
    };
    let group_code = path.into_inner();
    let group_collection = db.collection::<Group>("groups");
    let group = match group_collection.find_one(doc! {"groupCode": &group_code}).await {
        Ok(Some(group)) => group,
        Ok(None) => {
            write_log(&format!("POST /groups/join/{{code}} - Grupo no encontrado: {}", group_code)).ok();
            return HttpResponse::NotFound().body("Grupo no encontrado");
        },
        Err(_) => {
            write_log(&format!("POST /groups/join/{{code}} - Error al buscar grupo: {}", group_code)).ok();
            return HttpResponse::InternalServerError().body("Error al buscar el grupo");
        },
    };
    let user_group_collection = db.collection::<UserGroup>("userGroup");
    if let Ok(Some(_)) = user_group_collection
        .find_one(doc! {
            "groupId": group.id.unwrap(),
            "userId": user_id
        })
        .await
    {
        write_log(&format!("POST /groups/join/{{code}} - Usuario {} ya es miembro del grupo {}", user_id, group_code)).ok();
        return HttpResponse::BadRequest().body("Ya eres miembro de este grupo");
    }
    if let Some(max_users) = group.user_max {
        if group.user_count >= max_users {
            write_log(&format!("POST /groups/join/{{code}} - Grupo {} lleno", group_code)).ok();
            return HttpResponse::BadRequest().body("El grupo ha alcanzado su límite de usuarios");
        }
    }
    let client = db.client();
    let mut session = match client.start_session().await {
        Ok(s) => s,
        Err(_) => {
            write_log("POST /groups/join/{code} - Error al iniciar la transacción").ok();
            return HttpResponse::InternalServerError().body("Error al iniciar la transacción");
        },
    };
    session.start_transaction().await.ok();
    let user_group = UserGroup {
        id: None,
        group_id: group.id.unwrap(),
        user_id,
    };
    if let Err(_) = user_group_collection.insert_one(&user_group).await {
        session.abort_transaction().await.ok();
        write_log(&format!("POST /groups/join/{{code}} - Error al unirse al grupo {} para usuario {}", group_code, user_id)).ok();
        return HttpResponse::InternalServerError().body("Error al unirse al grupo");
    }
    match group_collection
        .update_one(
            doc! {"_id": group.id.unwrap()},
            doc! {"$inc": {"userCount": 1}},
        )
        .await
    {
        Ok(_) => {
            session.commit_transaction().await.ok();
            write_log(&format!("POST /groups/join/{{code}} - Usuario {} se unió al grupo {}", user_id, group_code)).ok();
            HttpResponse::Ok().body("Te has unido al grupo exitosamente")
        }
        Err(_) => {
            session.abort_transaction().await.ok();
            write_log(&format!("POST /groups/join/{{code}} - Error al actualizar contador de usuarios para grupo {}", group_code)).ok();
            HttpResponse::InternalServerError().body("Error al actualizar el contador de usuarios")
        }
    }
}

async fn patch_group(
    db: &Database,
    group_id: String,
    updated_group: web::Json<serde_json::Value>,
) -> HttpResponse {
    let collection = db.collection::<Group>("groups");
    let obj_id = match ObjectId::parse_str(group_id.clone()) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("ID inválido"),
    };

    // Recuperar grupo actual para validaciones
    let existing_group = match collection.find_one(doc! {"_id": &obj_id}).await {
        Ok(Some(group)) => group,
        Ok(None) => return HttpResponse::NotFound().body("Grupo no encontrado"),
        Err(_) => return HttpResponse::InternalServerError().body("Error al acceder a la base de datos"),
    };

    let name = updated_group.get("name");
    let user_count = updated_group.get("userCount");

    let user_count_val = user_count
        .and_then(|v| v.as_i64())
        .or(Some(existing_group.user_count.into()));

    if let Some(0) = user_count_val {
        return delete_group(db, group_id).await;
    }

    let mut update_set = doc! {};
    let mut update_unset = doc! {};

    if let Some(name_val) = name.and_then(|v| v.as_str()) {
        update_set.insert("name", name_val);
    }

    if let Some(user_count_val) = user_count.and_then(|v| v.as_i64()) {
        update_set.insert("userCount", user_count_val);
    }

    match updated_group.get("userMax") {
        Some(val) if val.is_null() => {
            update_unset.insert("userMax", "");
        }
        Some(val) => {
            if let Some(user_max_val) = val.as_i64() {
                if let Some(current_user_count) = user_count_val {
                    if user_max_val < current_user_count {
                        return HttpResponse::BadRequest()
                            .body("Mayor numero de usuarios de los permitidos");
                    }
                }
                update_set.insert("userMax", user_max_val);
            }
        }
        None => {}
    }

    match updated_group.get("tags") {
        Some(val) if val.is_null() => {
            update_unset.insert("tags", "");
        }
        Some(val) => {
            update_set.insert("tags", bson::to_bson(val).unwrap());
        }
        None => {}
    }

    let mut update_doc = doc! {};
    if !update_set.is_empty() {
        update_doc.insert("$set", update_set);
    }
    if !update_unset.is_empty() {
        update_doc.insert("$unset", update_unset);
    }

    if update_doc.is_empty() {
        return HttpResponse::BadRequest().body("No se especificaron campos a modificar");
    }

    match collection.update_one(doc! {"_id": obj_id}, update_doc).await {
        Ok(result) if result.matched_count == 1 => HttpResponse::Ok().body("Grupo actualizado"),
        Ok(_) => HttpResponse::NotFound().body("Grupo no encontrado"),
        Err(_) => HttpResponse::BadRequest().body("Error inesperado, inténtelo  nuevamente"),
    }
}

#[patch("/groups/{id}")]
async fn patch_group_handler(
    db: web::Data<Database>,
    path: web::Path<String>,
    updated_group: web::Json<serde_json::Value>,
    req: HttpRequest,
) -> impl Responder {
    let claims = match req.extensions().get::<crate::middleware::auth::Claims>().cloned() {
        Some(claims) => claims,
        None => {
            write_log("PATCH /groups/{id} - Token no encontrado").ok();
            return HttpResponse::Unauthorized().body("Token no encontrado");
        },
    };
    let group_id = match ObjectId::parse_str(&path.into_inner()) {
        Ok(group_id) => group_id,
        Err(_) => {
            write_log("PATCH /groups/{id} - ID inválido").ok();
            return HttpResponse::BadRequest().body("ID inválido");
        },
    };
    if claims.role != "admin" {
        let user_group_collection = db.collection::<UserGroup>("userGroup");
        let user_group = match user_group_collection.find_one(doc! {"groupId": &group_id}).await {
            Ok(Some(user_group)) => user_group,
            Ok(None) => {
                write_log(&format!("PATCH /groups/{{id}} - Usuario {} no pertenece al grupo {}", claims.sub, group_id)).ok();
                return HttpResponse::NotFound().body("El Usuario no pertenece a este grupo");
            },
            Err(_) => {
                write_log(&format!("PATCH /groups/{{id}} - Error inesperado para usuario {}", claims.sub)).ok();
                return HttpResponse::BadRequest().body("Error inesperado, inténtelo  nuevamente");
            },
        };
        if claims.sub != user_group.user_id.to_string() || group_id != user_group.group_id {
            write_log(&format!("PATCH /groups/{{id}} - Acceso no autorizado para usuario {}", claims.sub)).ok();
            return HttpResponse::Unauthorized().body("Acceso no autorizado");
        }
    }
    let client = db.client();
    let mut session = match client.start_session().await {
        Ok(s) => s,
        Err(_) => {
            write_log("PATCH /groups/{id} - Error al iniciar sesión").ok();
            return HttpResponse::BadRequest().body("Error inesperado, inténtelo  nuevamente");
        },
    };
    session.start_transaction().await.ok();
    let response = patch_group(&db, group_id.to_string(), updated_group).await;
    if response.status().is_success() {
        session.commit_transaction().await.ok();
        write_log(&format!("PATCH /groups/{{id}} - Grupo {} actualizado por usuario {}", group_id, claims.sub)).ok();
    } else {
        session.abort_transaction().await.ok();
        write_log(&format!("PATCH /groups/{{id}} - Error al actualizar grupo {} por usuario {}", group_id, claims.sub)).ok();
    }
    response
}


pub async fn delete_group(db: &Database, group_id: String) -> HttpResponse {
    let group_collection = db.collection::<Group>("groups");
    let property_collection = db.collection::<Property>("properties");
    let user_group_collection = db.collection::<UserGroup>("userGroup");
    let obj_id = match ObjectId::parse_str(group_id) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("Id incorrecto"),
    };

    let property_cursor = match property_collection.find(doc! {"groupId":obj_id}).await {
        Ok(zones) => zones,
        Err(_) => return HttpResponse::BadRequest().body("Error inesperado, inténtelo  nuevamente"),
    };
    let properties: Vec<Property> = match property_cursor.try_collect().await {
        Ok(properties) => properties,
        Err(_) => return HttpResponse::BadRequest().body("Error inesperado, inténtelo  nuevamente"),
    };
    for property in properties {
        let id = match property.id {
            Some(id) => id,
            None => return HttpResponse::BadRequest().body("No hay ID"),
        };
        let res = delete_property(db, id.to_string()).await;
        if !res.status().is_success() {
            return res; // Si falla, detenemos la ejecución y devolvemos el error
        }
    }
    let user_group_cursor = match user_group_collection.find(doc! {"groupId":obj_id}).await {
        Ok(user_group) => user_group,
        Err(_) => return HttpResponse::BadRequest().body("Error inesperado, inténtelo  nuevamente"),
    };
    let users_groups: Vec<UserGroup> = match user_group_cursor.try_collect().await {
        Ok(user_group) => user_group,
        Err(_) => return HttpResponse::BadRequest().body("Error inesperado, inténtelo  nuevamente"),
    };
    for user_group in users_groups {
        let id = match user_group.id {
            Some(id) => id,
            None => return HttpResponse::BadRequest().body("No hay ID"),
        };
        let res = delete_user_group(db, id.to_string()).await;
        if !res.status().is_success() {
            return res; // Si falla, detenemos la ejecución y devolvemos el error
        }
    }

    match group_collection.delete_one(doc! {"_id": obj_id}).await {
        Ok(_) => HttpResponse::Ok().body("Grupo Eliminado"),
        Err(_) => HttpResponse::BadRequest().body("Error inesperado, inténtelo  nuevamente"),
    }
}

#[delete("/groups/{id}")]
pub async fn delete_group_handler(
    db: web::Data<Database>,
    path: web::Path<String>,
) -> impl Responder {
    let group_id = path.into_inner();
    let client = db.client();
    let mut session = match client.start_session().await {
        Ok(s) => s,
        Err(_) => {
            write_log("DELETE /groups/{id} - Error al iniciar sesión").ok();
            return HttpResponse::BadRequest().body("Error inesperado, inténtelo  nuevamente");
        },
    };
    session.start_transaction().await.ok();
    let response = delete_group(&db, group_id.clone()).await;
    if response.status().is_success() {
        session.commit_transaction().await.ok();
        write_log(&format!("DELETE /groups/{{id}} - Grupo {} eliminado correctamente", group_id)).ok();
    } else {
        session.abort_transaction().await.ok();
        write_log(&format!("DELETE /groups/{{id}} - Error al eliminar grupo {}", group_id)).ok();
    }
    response
}

#[delete("/groups/leave/{id}")]
async fn leave_group_handler(
    db: web::Data<Database>,
    path: web::Path<String>,
    req: HttpRequest,
) -> impl Responder {
    let claims = match req.extensions().get::<crate::middleware::auth::Claims>().cloned() {
        Some(claims) => claims,
        None => {
            write_log("DELETE /groups/leave/{id} - Token no encontrado").ok();
            return HttpResponse::Unauthorized().body("Token no encontrado");
        },
    };
    let user_id = match ObjectId::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => {
            write_log(&format!("DELETE /groups/leave/{{id}} - ID de usuario inválido: {}", claims.sub)).ok();
            return HttpResponse::BadRequest().body("ID de usuario inválido");
        },
    };
    let group_id = match ObjectId::parse_str(&path.into_inner()) {
        Ok(id) => id,
        Err(_) => {
            write_log("DELETE /groups/leave/{id} - ID de grupo inválido").ok();
            return HttpResponse::BadRequest().body("ID de grupo inválido");
        },
    };
    let user_group_collection = db.collection::<UserGroup>("userGroup");
    let filter = doc! {
        "groupId": group_id,
        "userId": user_id
    };
    if user_group_collection.find_one(filter.clone()).await.unwrap_or(None).is_none() {
        write_log(&format!("DELETE /groups/leave/{{id}} - Usuario {} no es miembro del grupo {}", user_id, group_id)).ok();
        return HttpResponse::NotFound().body("No eres miembro de este grupo");
    }
    let client = db.client();
    let mut session = match client.start_session().await {
        Ok(s) => s,
        Err(_) => {
            write_log("DELETE /groups/leave/{id} - Error al iniciar la sesión").ok();
            return HttpResponse::BadRequest().body("Error al iniciar la sesión");
        },
    };
    session.start_transaction().await.ok();
    let delete_result = user_group_collection.delete_one(filter).await;
    if delete_result.is_err() {
        session.abort_transaction().await.ok();
        write_log(&format!("DELETE /groups/leave/{{id}} - Error al salir del grupo {} para usuario {}", group_id, user_id)).ok();
        return HttpResponse::BadRequest().body("Error al salir del grupo");
    }
    let group_collection = db.collection::<Group>("groups");
    let update_result = group_collection.update_one(
        doc! {"_id": group_id},
        doc! {"$inc": {"userCount": -1}}
    ).await;
    if update_result.is_err() {
        session.abort_transaction().await.ok();
        write_log(&format!("DELETE /groups/leave/{{id}} - Error al actualizar contador de usuarios para grupo {}", group_id)).ok();
        return HttpResponse::BadRequest().body("Error al actualizar el contador de usuarios");
    }
    let updated_group = group_collection.find_one(doc! {"_id": group_id}).await.unwrap_or(None);
    if let Some(group) = updated_group {
        if group.user_count <= 1 {
            let del_grp = group_collection.delete_one(doc! {"_id": group_id}).await;
            if del_grp.is_err() {
                session.abort_transaction().await.ok();
                write_log(&format!("DELETE /groups/leave/{{id}} - Error al eliminar grupo {}", group_id)).ok();
                return HttpResponse::BadRequest().body("Error al eliminar el grupo");
            }
            session.commit_transaction().await.ok();
            write_log(&format!("DELETE /groups/leave/{{id}} - Usuario {} salió y grupo {} eliminado por no tener miembros", user_id, group_id)).ok();
            return HttpResponse::Ok().body("Has salido del grupo y el grupo ha sido eliminado por no tener miembros");
        }
    }
    session.commit_transaction().await.ok();
    write_log(&format!("DELETE /groups/leave/{{id}} - Usuario {} salió del grupo {} exitosamente", user_id, group_id)).ok();
    HttpResponse::Ok().body("Has salido del grupo exitosamente")
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_group_handler)
        .service(get_groups_handler)
        .service(get_group_by_code_handler)
        .service(create_group_handler)
        .service(patch_group_handler)
        .service(delete_group_handler)
        .service(join_group_handler)
        .service(leave_group_handler);
}
