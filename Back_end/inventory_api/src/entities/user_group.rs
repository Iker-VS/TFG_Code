use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use futures_util::stream::TryStreamExt;
use mongodb::{
    bson::{doc, oid::ObjectId, DateTime},
    Database,
};
use serde::{Deserialize, Serialize};

use crate::entities::user_group;

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

#[get("/user-groups")]
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
#[get("/user-groups/{id}")]
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


/*#[get("/users/group/{id}")]
async fn get_users_group(db: web::Data<Database>,path: web::Path<String>)-> impl Responder{
    let collection_user_group =db.collection::<UserGroup>("userGroup");
    let obj_id= match ObjectId::parse_str(&path.into_inner()) {
        Ok(id)=>id,
        Err(_)=> return HttpResponse::BadRequest().body("ID inválido"),
    };
    let cursor_user_group =match collection_user_group.find(doc! {"groupId":obj_id}).await {
        Ok(cursor_user_group)=>cursor_user_group,
        Err(e)=>return HttpResponse::InternalServerError().body(e.to_string()),
    };
    let users:Vec<UserGroup>=match collection_user_group.try_collect().await {
        
    };



}
*/ 