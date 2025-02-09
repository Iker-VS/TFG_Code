use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use futures_util::stream::TryStreamExt;
use mongodb::{
    bson::{doc, oid::ObjectId, DateTime},
    Database,
};
use serde::{Deserialize, Serialize};

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
