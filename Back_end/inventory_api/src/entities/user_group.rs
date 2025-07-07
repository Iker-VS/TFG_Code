use actix_web::{delete, get, patch, post, web, HttpMessage, HttpRequest, HttpResponse, Responder};
use futures_util::stream::TryStreamExt;
use mongodb::{
    bson::{doc, oid::ObjectId},
    Database,
};
use serde::{Deserialize, Serialize};

use crate::entities::{group::Group, user::User};
use crate::log::write_log;

#[derive(Debug, Serialize, Deserialize)]
pub struct UserGroup {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    #[serde(rename = "groupId")]
    pub group_id: ObjectId,
    #[serde(rename = "userId")]
    pub user_id: ObjectId,
}
// impl UserGroup {
//     fn new(group_id: ObjectId, user_id: ObjectId) -> UserGroup {
//         Self {
//             id: None,
//             group_id,
//             user_id,
//         }
//     }
// }

// #[get("/user-group-relationships")]
// async fn get_users_groups_handler(db: web::Data<Database>) -> impl Responder {
//     let collection = db.collection::<UserGroup>("userGroup");
//     let cursor = match collection.find(doc! {}).await {
//         Ok(cursor) => cursor,
//         Err(_) => {
//             write_log("GET /user-group-relationships - Error buscando relaciones").ok();
//             return HttpResponse::BadRequest().body("Error inesperado, inténtelo  nuevamente");
//         },
//     };
//     let user_groups: Vec<UserGroup> = match cursor.try_collect().await {
//         Ok(users_groups) => users_groups,
//         Err(_) => {
//             write_log("GET /user-group-relationships - Error recogiendo relaciones").ok();
//             return HttpResponse::BadRequest().body("Error inesperado, inténtelo  nuevamente");
//         },
//     };
//     write_log(&format!("GET /user-group-relationships - {} relaciones recuperadas", user_groups.len())).ok();
//     HttpResponse::Ok().json(user_groups)
// }
// #[get("/user-group-relationship/{id}")]
// async fn get_user_group_handler(
//     db: web::Data<Database>,
//     path: web::Path<String>,
// ) -> impl Responder {
//     let collection = db.collection::<UserGroup>("userGroup");
//     let obj_id = match ObjectId::parse_str(&path.into_inner()) {
//         Ok(obj_id) => obj_id,
//         Err(_) => {
//             write_log("GET /user-group-relationship/{id} - ID inválido").ok();
//             return HttpResponse::BadRequest().body("ID inválido");
//         },
//     };
//     match collection.find_one(doc! {"_id":obj_id}).await {
//         Ok(Some(user_group)) => {
//             write_log("GET /user-group-relationship/{id} - Relación encontrada").ok();
//             return HttpResponse::Ok().json(user_group)
//         },
//         Ok(None) => {
//             write_log("GET /user-group-relationship/{id} - Relación no encontrada").ok();
//             return HttpResponse::NotFound().body("grupo-usuario no encontrado")
//         },
//         Err(e) => {
//             write_log(&format!("GET /user-group-relationship/{} - Error: {}", obj_id, e)).ok();
//             HttpResponse::BadRequest().body(e.to_string())
//         },
//     }
// }

// #[get("/user-group-relationship/find/{userId}/{groupId}")]
// async fn get_user_group_id_handler(
//     db: web::Data<Database>,
//     path: web::Path<(String, String)>,
// ) -> impl Responder {
//     let (user_id_str, group_id_str) = path.into_inner();
//     let user_id = match ObjectId::parse_str(&user_id_str) {
//         Ok(id) => id,
//         Err(_) => {
//             write_log("GET /user-group-relationship/find/{userId}/{groupId} - ID de usuario inválido").ok();
//             return HttpResponse::BadRequest().body("ID de usuario inválido");
//         },
//     };
//     let group_id = match ObjectId::parse_str(&group_id_str) {
//         Ok(id) => id,
//         Err(_) => {
//             write_log("GET /user-group-relationship/find/{userId}/{groupId} - ID de grupo inválido").ok();
//             return HttpResponse::BadRequest().body("ID de grupo inválido");
//         },
//     };
//     let collection = db.collection::<UserGroup>("userGroup");
//     match collection
//         .find_one(doc! {"userId": user_id, "groupId": group_id})
//         .await
//     {
//         Ok(Some(user_group)) => {
//             write_log("GET /user-group-relationship/find/{userId}/{groupId} - Relación encontrada").ok();
//             HttpResponse::Ok().json(user_group.id.unwrap())
//         },
//         Ok(None) => {
//             write_log("GET /user-group-relationship/find/{userId}/{groupId} - Relación no encontrada").ok();
//             HttpResponse::NotFound().body("No se encontró registro para el user-group")
//         },
//         Err(_) => {
//             write_log("GET /user-group-relationship/find/{userId}/{groupId} - Error inesperado").ok();
//             HttpResponse::BadRequest().body("Error inesperado, inténtelo  nuevamente")
//         },
//     }
// }

// #[get("/groups/{id}/users")]
// async fn get_users_from_group_handler(
//     db: web::Data<Database>,
//     path: web::Path<String>,
// ) -> impl Responder {
//     let user_group_collection = db.collection::<UserGroup>("userGroup");
//     let obj_id = match ObjectId::parse_str(&path.into_inner()) {
//         Ok(id) => id,
//         Err(_) => {
//             write_log("GET /groups/{id}/users - ID inválido").ok();
//             return HttpResponse::BadRequest().body("ID inválido");
//         },
//     };
//     let user_group_cursor = match user_group_collection.find(doc! {"groupId":obj_id}).await {
//         Ok(cursor_user_group) => cursor_user_group,
//         Err(_) => {
//             write_log("GET /groups/{id}/users - Error buscando relaciones").ok();
//             return HttpResponse::BadRequest().body("Error inesperado, inténtelo  nuevamente");
//         },
//     };
//     let user_group: Vec<UserGroup> = match user_group_cursor.try_collect().await {
//         Ok(users) => users,
//         Err(_) => {
//             write_log("GET /groups/{id}/users - Error recogiendo relaciones").ok();
//             return HttpResponse::BadRequest().body("Error inesperado, inténtelo  nuevamente");
//         },
//     };

//     let users_id: Vec<ObjectId> = user_group.iter().map(|u| u.user_id).collect();
//     let users_collection = db.collection::<User>("users");
//     let mut users: Vec<User> = Vec::new();
//     for id in users_id {
//         match users_collection.find_one(doc! {"_id": id}).await {
//             Ok(Some(user)) => users.push(user),
//             Ok(None) => continue,
//             Err(_) => {
//                 write_log("GET /groups/{id}/users - Error buscando usuario").ok();
//                 return HttpResponse::BadRequest().body("Error inesperado, inténtelo  nuevamente")
//             }
//         }
//     }
//     if users.is_empty() {
//         write_log("GET /groups/{id}/users - No hay usuarios asociados a ese grupo").ok();
//         return HttpResponse::BadRequest().body("no hay usarios asociados a ese grupo");
//     }
//     write_log(&format!("GET /groups/{{id}}/users - {} usuarios recuperados", users.len())).ok();
//     HttpResponse::Ok().json(users)
// }

#[get("/users/me/groups")]
async fn get_groups_from_user_handler(db: web::Data<Database>, req: HttpRequest) -> impl Responder {
    let claims = match req
        .extensions()
        .get::<crate::middleware::auth::Claims>()
        .cloned()
    {
        Some(claims) => claims,
        None => {
            write_log("GET /users/me/groups - Token no encontrado").ok();
            return HttpResponse::Unauthorized().body("Token no encontrado");
        }
    };

    // Si es admin, devolver todos los grupos
    let group_collection = db.collection::<Group>("groups");
    if claims.role == "admin" {
        // Admin: devolver todos los grupos
        let cursor = match group_collection.find(doc! {}).await {
            Ok(cursor) => cursor,
            Err(_) => {
                write_log("GET /users/me/groups - Error buscando todos los grupos").ok();
                return HttpResponse::BadRequest().body("Error inesperado, inténtelo  nuevamente");
            }
        };
        let groups: Vec<Group> = match cursor.try_collect().await {
            Ok(groups) => groups,
            Err(_) => {
                write_log("GET /users/me/groups - Error recogiendo todos los grupos").ok();
                return HttpResponse::BadRequest().body("Error inesperado, inténtelo  nuevamente");
            }
        };
        if groups.is_empty() {
            write_log("GET /users/me/groups - No hay grupos en la base de datos").ok();
            return HttpResponse::BadRequest().body("No hay grupos en la base de datos");
        }
        write_log(&format!(
            "GET /users/me/groups - {} grupos recuperados (admin)",
            groups.len()
        ))
        .ok();
        return HttpResponse::Ok().json(groups);
    }

    // Si no es admin, devolver solo los grupos del usuario
    let user_id = match ObjectId::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => {
            write_log("GET /users/me/groups - ID de usuario inválido").ok();
            return HttpResponse::BadRequest().body("ID de usuario inválido");
        }
    };
    let user_group_collection = db.collection::<UserGroup>("userGroup");
    let user_group_cursor = match user_group_collection.find(doc! {"userId": user_id}).await {
        Ok(cursor) => cursor,
        Err(_) => {
            write_log("GET /users/me/groups - Error buscando relaciones").ok();
            return HttpResponse::BadRequest().body("Error inesperado, inténtelo  nuevamente");
        }
    };
    let user_groups: Vec<UserGroup> = match user_group_cursor.try_collect().await {
        Ok(groups) => groups,
        Err(_) => {
            write_log("GET /users/me/groups - Error recogiendo relaciones").ok();
            return HttpResponse::BadRequest().body("Error inesperado, inténtelo  nuevamente");
        }
    };
    let group_ids: Vec<ObjectId> = user_groups.iter().map(|ug| ug.group_id).collect();
    let mut groups: Vec<Group> = Vec::new();
    for id in group_ids {
        match group_collection.find_one(doc! {"_id": id}).await {
            Ok(Some(group)) => groups.push(group),
            Ok(None) => continue,
            Err(_) => {
                write_log("GET /users/me/groups - Error buscando grupo").ok();
                return HttpResponse::BadRequest().body("Error inesperado, inténtelo  nuevamente");
            }
        }
    }
    if groups.is_empty() {
        write_log("GET /users/me/groups - No hay grupos asociados a ese usuario").ok();
        return HttpResponse::BadRequest().body("No hay grupos asociados a ese usuario");
    }
    write_log(&format!(
        "GET /users/me/groups - {} grupos recuperados",
        groups.len()
    ))
    .ok();
    HttpResponse::Ok().json(groups)
}

// #[post("/user-group-relationships")]
// async fn create_user_group_handler(
//     db: web::Data<Database>,
//     new_user_group: web::Json<UserGroup>,
//     _req: HttpRequest,
// ) -> impl Responder {
//     let collection = db.collection::<UserGroup>("userGroup");
//     let mut user_group = new_user_group.into_inner();
//     user_group.id = None;
//     match collection.insert_one(user_group).await {
//         Ok(result) => {
//             write_log("POST /user-group-relationships - Relación creada correctamente").ok();
//             HttpResponse::Ok().json(result.inserted_id)
//         },
//         Err(_) => {
//             write_log("POST /user-group-relationships - Error inesperado").ok();
//             HttpResponse::BadRequest().body("Error inesperado, inténtelo  nuevamente")
//         },
//     }
// }
// // seguramente no se use con opción de borrar
// #[patch("/user-group-relationships/{id}")]
// async fn patch_user_group_handler(
//     db: web::Data<Database>,
//     path: web::Path<String>,
//     updated_user_group: web::Json<UserGroup>,
// ) -> impl Responder {
//     let collection = db.collection::<UserGroup>("userGroup");
//     let obj_id = match ObjectId::parse_str(&path.into_inner()) {
//         Ok(id) => id,
//         Err(_) => {
//             write_log("PATCH /user-group-relationships/{id} - ID inválido").ok();
//             return HttpResponse::BadRequest().body("ID inválido");
//         },
//     };
//     let update_doc = doc! {
//         "$set": {
//             "groupId": updated_user_group.group_id.clone(),
//             "UserId": updated_user_group.user_id.clone(),
//         }
//     };
//     match collection
//         .update_one(doc! {"_id": obj_id}, update_doc)
//         .await
//     {
//         Ok(result) if result.matched_count == 1 => {
//             write_log("PATCH /user-group-relationships/{id} - Relación actualizada").ok();
//             HttpResponse::Ok().body("grupo usuario actualizado")
//         }
//         Ok(_) => {
//             write_log("PATCH /user-group-relationships/{id} - Relación no encontrada").ok();
//             HttpResponse::NotFound().body("Grupo usuario no encontrado")
//         },
//         Err(_) => {
//             write_log("PATCH /user-group-relationships/{id} - Error inesperado").ok();
//             HttpResponse::BadRequest().body("Error inesperado, inténtelo  nuevamente")
//         },
//     }
// }

// #[delete("/user-group-relationships/{id}")]
// async fn delete_user_group_handler(
//     db: web::Data<Database>,
//     path: web::Path<String>,
// ) -> impl Responder {
//     let client = db.client();
//     let mut session = match client.start_session().await {
//         Ok(s) => s,
//         Err(_) => {
//             write_log("DELETE /user-group-relationships/{id} - Error iniciando sesión").ok();
//             return HttpResponse::BadRequest().body("Error inesperado, inténtelo  nuevamente");
//         },
//     };
//     session.start_transaction().await.ok();
//     let response = delete_user_group(&db, path.into_inner()).await;

//     if response.status().is_success() {
//         session.commit_transaction().await.ok();
//         write_log("DELETE /user-group-relationships/{id} - Transacción confirmada").ok();
//     } else {
//         session.abort_transaction().await.ok();
//         write_log("DELETE /user-group-relationships/{id} - Transacción abortada").ok();
//     }

//     response
// }

pub async fn delete_user_group(db: &Database, user_group_id: String) -> HttpResponse {
    let collection = db.collection::<UserGroup>("userGroup");
    let obj_id = match ObjectId::parse_str(user_group_id) {
        Ok(id) => id,
        Err(_) => {
            write_log("DELETE /user-group-relationships/{id} - ID inválido").ok();
            return HttpResponse::BadRequest().body("ID inválido");
        }
    };
    match collection.delete_one(doc! {"_id": obj_id}).await {
        Ok(result) if result.deleted_count == 1 => {
            write_log("DELETE /user-group-relationships/{id} - Relación eliminada correctamente")
                .ok();
            HttpResponse::Ok().body("grupo usuario eliminado")
        }
        Ok(_) => {
            write_log("DELETE /user-group-relationships/{id} - Relación no encontrada").ok();
            HttpResponse::NotFound().body("grupo usuario no encontrado")
        }
        Err(_) => {
            write_log("DELETE /user-group-relationships/{id} - Error inesperado").ok();
            HttpResponse::BadRequest().body("Error inesperado, inténtelo  nuevamente")
        }
    }
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    // cfg.service(get_user_group_handler)
    //     .service(get_users_groups_handler)
    //     .service(get_user_group_id_handler)
    cfg.service(get_groups_from_user_handler);
    // .service(get_users_from_group_handler)
    // .service(create_user_group_handler)
    // .service(patch_user_group_handler)
    // .service(delete_user_group_handler);
}
