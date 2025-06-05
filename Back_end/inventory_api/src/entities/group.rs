use actix_web::{delete, get, post, patch, web, HttpMessage, HttpRequest, HttpResponse, Responder};
use futures_util::stream::TryStreamExt;
use mongodb::{
    bson::{self, doc, oid::ObjectId},
    Database,
};
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::entities::user_group::UserGroup;

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
    // Recupera las claims ya inyectadas por el middleware
    let claims = match req
        .extensions()
        .get::<crate::middleware::auth::Claims>()
        .cloned()
    {
        Some(c) => c,
        None => return HttpResponse::Unauthorized().body("Token no encontrado"),
    };

    // Solo un admin puede obtener todos los grupos
    if claims.role != "admin" {
        return HttpResponse::Unauthorized().body("Acceso no autorizado");
    }

    let collection = db.collection::<Group>("groups");
    let cursor = match collection.find(doc! {}).await {
        Ok(cursor) => cursor,
        Err(_) => return HttpResponse::BadRequest().body("Error inesperado, inténtelo  nuevamente"),
    };

    let groups: Vec<Group> = match cursor.try_collect().await {
        Ok(groups) => groups,
        Err(_) => return HttpResponse::BadRequest().body("Error inesperado, inténtelo  nuevamente"),
    };

    HttpResponse::Ok().json(groups)
}

#[get("/groups/{id}")]
async fn get_group_handler(db: web::Data<Database>, path: web::Path<String>) -> impl Responder {
    let collection = db.collection::<Group>("groups");
    let obj_id = match ObjectId::parse_str(&path.into_inner()) {
        Ok(obj_id) => obj_id,
        Err(_) => return HttpResponse::BadRequest().body("ID inválido"),
    };
    match collection.find_one(doc! {"_id":obj_id}).await {
        Ok(Some(group)) => return HttpResponse::Ok().json(group),
        Ok(None) => return HttpResponse::NotFound().body("Grupo no encontrado"),
        Err(e) => HttpResponse::BadRequest().body(e.to_string()),
    }
}
#[get("/groups/code/{code}")]
async fn get_group_by_code_handler(
    db: web::Data<Database>,
    path: web::Path<String>,
) -> impl Responder {
    let collection = db.collection::<Group>("groups");
    let group_code = path.into_inner();
    match collection.find_one(doc! {"groupCode": group_code}).await {
        Ok(Some(group)) => HttpResponse::Ok().json(group),
        Ok(None) => HttpResponse::NotFound().body("Grupo no encontrado"),
        Err(e) => HttpResponse::BadRequest().body(e.to_string()),
    }
}

#[post("/groups")]
async fn create_group_handler(
    db: web::Data<Database>,
    new_group: web::Json<CreateGroup>,
    req: HttpRequest,
) -> impl Responder {
    // Obtener claims del token
    let claims = match req.extensions().get::<crate::middleware::auth::Claims>().cloned() {
        Some(claims) => claims,
        None => return HttpResponse::Unauthorized().body("Token no encontrado"),
    };

    let user_id = match ObjectId::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("ID de usuario inválido"),
    };

    let client = db.client();
    let mut session = match client.start_session().await {
        Ok(s) => s,
        Err(_) => return HttpResponse::BadRequest().body("Error inesperado, inténtelo  nuevamente"),
    };

    session.start_transaction().await.ok();

    let collection = db.collection::<Group>("groups");
    let group_code = generate_unique_group_code(&collection).await;

    let group = Group {
        id: None,
        name: new_group.name.clone(),
        user_max: new_group.user_max,
        user_count: 1,
        group_code,
        tags: None,
    };

    let group_result = match collection.insert_one(group).await {
    // Insertar usando el documento BSON explícito

        Ok(result) => result,
        Err(_) => {
            session.abort_transaction().await.ok();
            return HttpResponse::BadRequest().body("Error inesperado, inténtelo  nuevamente");
        }
    };

    let group_id = match group_result.inserted_id.as_object_id() {
        Some(id) => id,
        None => {
            session.abort_transaction().await.ok();
            return HttpResponse::BadRequest().body("Error al obtener ID del grupo");
        }
    };

    // Crear la relación usuario-grupo
    let user_group = UserGroup {
        id: None,
        group_id,
        user_id,
    };

    let user_group_collection = db.collection::<UserGroup>("userGroup");
    if let Err(_) = user_group_collection.insert_one(&user_group).await {
        session.abort_transaction().await.ok();
        return HttpResponse::BadRequest().body("Error al crear la relación usuario-grupo");
    }

    session.commit_transaction().await.ok();
    HttpResponse::Ok().json(group_result.inserted_id)
}


#[post("/groups/join/{code}")]
async fn join_group_handler(
    db: web::Data<Database>,
    path: web::Path<String>,
    req: HttpRequest,
) -> impl Responder {
    // Obtener claims del token
    let claims = match req.extensions().get::<crate::middleware::auth::Claims>().cloned() {
        Some(claims) => claims,
        None => return HttpResponse::Unauthorized().body("Token no encontrado"),
    };

    let user_id = match ObjectId::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("ID de usuario inválido"),
    };

    let group_code = path.into_inner();
    let group_collection = db.collection::<Group>("groups");
    
    // Buscar el grupo por código
    let group = match group_collection.find_one(doc! {"groupCode": &group_code}).await {
        Ok(Some(group)) => group,
        Ok(None) => return HttpResponse::NotFound().body("Grupo no encontrado"),
        Err(_) => return HttpResponse::InternalServerError().body("Error al buscar el grupo"),
    };

    // Verificar si el usuario ya está en el grupo
    let user_group_collection = db.collection::<UserGroup>("userGroup");
    if let Ok(Some(_)) = user_group_collection
        .find_one(doc! {
            "groupId": group.id.unwrap(),
            "userId": user_id
        })
        .await
    {
        return HttpResponse::BadRequest().body("Ya eres miembro de este grupo");
    }

    // Verificar límite de usuarios
    if let Some(max_users) = group.user_max {
        if group.user_count >= max_users {
            return HttpResponse::BadRequest().body("El grupo ha alcanzado su límite de usuarios");
        }
    }

    // Iniciar transacción
    let client = db.client();
    let mut session = match client.start_session().await {
        Ok(s) => s,
        Err(_) => return HttpResponse::InternalServerError().body("Error al iniciar la transacción"),
    };
    session.start_transaction().await.ok();

    // Crear relación usuario-grupo
    let user_group = UserGroup {
        id: None,
        group_id: group.id.unwrap(),
        user_id,
    };

    if let Err(_) = user_group_collection.insert_one(&user_group).await {
        session.abort_transaction().await.ok();
        return HttpResponse::InternalServerError().body("Error al unirse al grupo");
    }

    // Incrementar user_count
    match group_collection
        .update_one(
            doc! {"_id": group.id.unwrap()},
            doc! {"$inc": {"userCount": 1}},
        )
        .await
    {
        Ok(_) => {
            session.commit_transaction().await.ok();
            HttpResponse::Ok().body("Te has unido al grupo exitosamente")
        }
        Err(_) => {
            session.abort_transaction().await.ok();
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
        None => return HttpResponse::Unauthorized().body("Token no encontrado"),
    };

    let group_id = match ObjectId::parse_str(&path.into_inner()) {
        Ok(group_id) => group_id,
        Err(_) => return HttpResponse::BadRequest().body("ID inválido"),
    };

    if claims.role != "admin" {
        let user_group_collection = db.collection::<UserGroup>("userGroup");
        let user_group = match user_group_collection.find_one(doc! {"groupId": &group_id}).await {
            Ok(Some(user_group)) => user_group,
            Ok(None) => return HttpResponse::NotFound().body("El Usuario no pertenece a este grupo"),
            Err(_) => return HttpResponse::BadRequest().body("Error inesperado, inténtelo  nuevamente"),
        };

        if claims.sub != user_group.user_id.to_string() || group_id != user_group.group_id {
            return HttpResponse::Unauthorized().body("Acceso no autorizado");
        }
    }

    let client = db.client();
    let mut session = match client.start_session().await {
        Ok(s) => s,
        Err(_) => return HttpResponse::BadRequest().body("Error inesperado, inténtelo  nuevamente"),
    };
    session.start_transaction().await.ok();

    let response = patch_group(&db, group_id.to_string(), updated_group).await;

    if response.status().is_success() {
        session.commit_transaction().await.ok();
    } else {
        session.abort_transaction().await.ok();
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
    let client = db.client();
    let mut session = match client.start_session().await {
        Ok(s) => s,
        Err(_) => return HttpResponse::BadRequest().body("Error inesperado, inténtelo  nuevamente"),
    };
    session.start_transaction().await.ok();
    let response = delete_group(&db, path.into_inner()).await;

    if response.status().is_success() {
        session.commit_transaction().await.ok();
    } else {
        session.abort_transaction().await.ok();
    }

    response
}

#[delete("/groups/leave/{id}")]
async fn leave_group_handler(
    db: web::Data<Database>,
    path: web::Path<String>,
    req: HttpRequest,
) -> impl Responder {
    // Recuperar claims del token
    let claims = match req.extensions().get::<crate::middleware::auth::Claims>().cloned() {
        Some(claims) => claims,
        None => return HttpResponse::Unauthorized().body("Token no encontrado"),
    };
    // Parsear id del usuario y del grupo
    let user_id = match ObjectId::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("ID de usuario inválido"),
    };
    let group_id = match ObjectId::parse_str(&path.into_inner()) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("ID de grupo inválido"),
    };

    // Obtener colección de userGroup
    let user_group_collection = db.collection::<UserGroup>("userGroup");
    let filter = doc! {
        "groupId": group_id,
        "userId": user_id
    };
    // Verificar que exista la relación
    if user_group_collection.find_one(filter.clone()).await.unwrap_or(None).is_none() {
        return HttpResponse::NotFound().body("No eres miembro de este grupo");
    }

    // Iniciar transacción
    let client = db.client();
    let mut session = match client.start_session().await {
        Ok(s) => s,
        Err(_) => return HttpResponse::BadRequest().body("Error al iniciar la sesión"),
    };
    session.start_transaction().await.ok();

    // Eliminar la relación usuario-grupo
    let delete_result = user_group_collection.delete_one(filter).await;
    if delete_result.is_err() {
        session.abort_transaction().await.ok();
        return HttpResponse::BadRequest().body("Error al salir del grupo");
    }

    // Decrementar userCount en el grupo
    let group_collection = db.collection::<Group>("groups");
    let update_result = group_collection.update_one(
        doc! {"_id": group_id},
        doc! {"$inc": {"userCount": -1}}
    ).await;
    if update_result.is_err() {
        session.abort_transaction().await.ok();
        return HttpResponse::BadRequest().body("Error al actualizar el contador de usuarios");
    }

    // Obtener grupo actualizado
    let updated_group = group_collection.find_one(doc! {"_id": group_id}).await.unwrap_or(None);
    if let Some(group) = updated_group {
        if group.user_count <= 1 { // antes de la actualización era 1 o 0
            // Eliminar el grupo si ya no hay usuarios
            let del_grp = group_collection.delete_one(doc! {"_id": group_id}).await;
            if del_grp.is_err() {
                session.abort_transaction().await.ok();
                return HttpResponse::BadRequest().body("Error al eliminar el grupo");
            }
            session.commit_transaction().await.ok();
            return HttpResponse::Ok().body("Has salido del grupo y el grupo ha sido eliminado por no tener miembros");
        }
    }
    session.commit_transaction().await.ok();
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
