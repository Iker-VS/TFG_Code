use std::path;

use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use futures_util::stream::TryStreamExt;
use mongodb::{
    bson::{doc, oid::ObjectId},
    Database,
};
use rand::Rng;
use serde::{Deserialize, Serialize};

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
        Ok(None) => return HttpResponse::NotFound().body("Usuario no encontrado"),
        Err(e) => HttpResponse::BadRequest().body(e.to_string()),
    }
}
