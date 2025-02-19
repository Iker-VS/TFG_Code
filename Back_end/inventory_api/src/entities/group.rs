use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use futures_util::stream::TryStreamExt;
use mongodb::{
    bson::{doc, oid::ObjectId},
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
    fn new(
        name: String,
        user_max: Option<i32>,
        user_count: i32,
        group_code: Option<String>,
        tags: Option<Vec<String>>,
    ) -> Group {
        Self {
            id: None,
            name,
            user_max,
            user_count,
            group_code: group_code
                .or_else(|| Some(Self::create_group_code()))
                .unwrap(),
            tags,
        }
    }

    fn create_group_code() -> String {
        let mut rand = rand::rng();
        let characters: Vec<char> = ('0'..='9').chain('a'..='z').chain('A'..='Z').collect();

        let group_code = (0..8)
            .map(|_| characters[rand.random_range(0..characters.len())])
            .collect::<String>();
        group_code
    }
}

#[get("/groups")]
async fn get_groups_handler(db: web::Data<Database>) -> impl Responder {
    let collection = db.collection::<Group>("groups");
    let cursor = match collection.find(doc! {}).await {
        Ok(cursor) => cursor,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };
    let groups: Vec<Group> = match cursor.try_collect().await {
        Ok(groups) => groups,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
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

#[post("/groups")]
async fn create_group_handler(
    db: web::Data<Database>,
    new_group: web::Json<Group>,
) -> impl Responder {
    let collection = db.collection::<Group>("groups");
    let mut group = new_group.into_inner();
    let mut group_code = Group::create_group_code();
    while collection
        .find_one(doc! {"groupCode":&group_code})
        .await
        .unwrap_or(None)
        .is_some()
    {
        group_code = Group::create_group_code();
    }
    group.group_code = group_code;
    group.user_count = 0;
    match collection.insert_one(group).await {
        Ok(result) => HttpResponse::Ok().json(result.inserted_id),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[put("/groups/{id}")]
async fn update_group_handler(
    db: web::Data<Database>,
    path: web::Path<String>,
    updated_group: web::Json<Group>,
) -> impl Responder {
    let collection = db.collection::<Group>("groups");
    let obj_id = match ObjectId::parse_str(&path.into_inner()) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("ID inválido"),
    };
    let mut update_doc = doc! {
        "$set": {
            "name": updated_group.name.clone(),
            "userCount":updated_group.user_count.clone(),
            "groupCode":updated_group.group_code.clone(),
        }
    };

    if let Some(user_max) = &updated_group.user_max {
        update_doc.get_mut("$set").unwrap().as_document_mut().unwrap().insert("userMax", user_max.clone());
    } else {
        update_doc.insert("$unset", doc! {"user_max": ""});
    }
    
    if let Some(tags) = &updated_group.tags {
        update_doc.get_mut("$set").unwrap().as_document_mut().unwrap().insert("tags", tags.clone());
    } else {
        update_doc.insert("$unset", doc! {"tags": ""});
    }

    match collection
        .update_one(doc! {"_id": obj_id}, update_doc)
        .await
    {
        Ok(result) if result.matched_count == 1 => HttpResponse::Ok().body("Grupo actualizado"),
        Ok(_) => HttpResponse::NotFound().body("Grupo no encontrado"),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
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
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };
    let properties: Vec<Property> = match property_cursor.try_collect().await {
        Ok(properties) => properties,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
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
        let res = delete_user_group(db, id.to_string()).await;
        if !res.status().is_success() {
            return res; // Si falla, detenemos la ejecución y devolvemos el error
        }
    }

    match group_collection.delete_one(doc! {"_id": obj_id}).await {
        Ok(_) => HttpResponse::Ok().body("Grupo Eliminado"),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[delete("/groups/{id}")]
async fn delete_group_handler(db: web::Data<Database>, path: web::Path<String>) -> impl Responder {
    let client = db.client();
    let mut session = match client.start_session().await {
        Ok(s) => s,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
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

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_group_handler)
        .service(get_groups_handler)
        .service(create_group_handler)
        .service(update_group_handler)
        .service(delete_group_handler);
}
