use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use futures_util::{future::ok, stream::TryStreamExt};
use mongodb::{
    bson::{doc, oid::ObjectId, DateTime},
    Database,
};
use serde::{Deserialize, Serialize};

use crate::entities::{group::Group, user::User, user_group};

#[derive(Debug, Serialize, Deserialize)]
pub struct UserGroup {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    #[serde(rename = "groupId")]
    pub group_id: ObjectId,
    #[serde(rename = "userId")]
    pub user_id: ObjectId,
}
impl UserGroup {
    fn new(group_id: ObjectId, user_id: ObjectId) -> UserGroup {
        Self {
            id: None,
            group_id,
            user_id,
        }
    }
}

#[get("/user-group")]
async fn get_users_groups_handler(db: web::Data<Database>) -> impl Responder {
    let collection = db.collection::<UserGroup>("userGroup");
    let cursor = match collection.find(doc! {}).await {
        Ok(cursor) => cursor,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };
    let user_groups: Vec<UserGroup> = match cursor.try_collect().await {
        Ok(users_groups) => users_groups,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };
    HttpResponse::Ok().json(user_groups)
}
#[get("/user-group/{id}")]
async fn get_user_group_handler(db: web::Data<Database>, path: web::Path<String>) -> impl Responder {
    let collection = db.collection::<UserGroup>("userGroup");
    let obj_id = match ObjectId::parse_str(&path.into_inner()) {
        Ok(obj_id) => obj_id,
        Err(_) => return HttpResponse::BadRequest().body("ID inválido"),
    };
    match collection.find_one(doc! {"_id":obj_id}).await {
        Ok(Some(user_group)) => return HttpResponse::Ok().json(user_group),
        Ok(None) => return HttpResponse::NotFound().body("grupo-usuario no encontrado"),
        Err(e) => HttpResponse::BadRequest().body(e.to_string()),
    }
}


#[get("/user-group/group/{id}")]
async fn get_users_from_group(db: web::Data<Database>,path: web::Path<String>)-> impl Responder{
    let user_group_collection =db.collection::<UserGroup>("userGroup");
    let obj_id= match ObjectId::parse_str(&path.into_inner()) {
        Ok(id)=>id,
        Err(_)=> return HttpResponse::BadRequest().body("ID inválido"),
    };
    let user_group_cursor =match user_group_collection.find(doc! {"groupId":obj_id}).await {
        Ok(cursor_user_group)=>cursor_user_group,
        Err(e)=>return HttpResponse::InternalServerError().body(e.to_string()),
    };
    let user_group:Vec<UserGroup>=match user_group_cursor.try_collect().await {
        Ok(users)=>users,
        Err(e)=>return HttpResponse::InternalServerError().body(e.to_string()),
    };
    let users_id:Vec<ObjectId>= user_group.iter().map(|u| u.user_id).collect();
    let users_collection=db.collection::<User>("users");
    let user_cursor = match users_collection.find(doc! {"_id":{ "$in": users_id }}).await {
        Ok(cursor) => cursor,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    let users: Vec<User> = match user_cursor.try_collect().await {
        Ok(users) => users,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };
    HttpResponse::Ok().json(users)
}


#[get("/user-group/user/{id}")]
async fn get_groups_from_user(db: web::Data<Database>, path: web::Path<String>) -> impl Responder {
    let user_group_collection = db.collection::<UserGroup>("userGroup");
    
    let obj_id = match ObjectId::parse_str(&path.into_inner()) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("ID inválido"),
    };
    let user_group_cursor = match user_group_collection.find(doc! {"groupId": obj_id}).await {
        Ok(cursor) => cursor,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    let user_groups: Vec<UserGroup> = match user_group_cursor.try_collect().await {
        Ok(groups) => groups,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    let group_ids: Vec<ObjectId> = user_groups.iter().map(|ug| ug.group_id).collect();
    
    let group_collection = db.collection::<Group>("groups");
    
    let group_cursor = match group_collection.find(doc! {"_id": { "$in": &group_ids }}).await {
        Ok(cursor) => cursor,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    let groups: Vec<Group> = match group_cursor.try_collect().await {
        Ok(groups) => groups,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };
    HttpResponse::Ok().json(groups)
}
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_users_from_group)
    .service(get_users_groups_handler)
    .service(get_groups_from_user)
    .service(get_users_from_group);
    
}
