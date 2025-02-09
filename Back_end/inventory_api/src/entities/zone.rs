use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use futures_util::stream::TryStreamExt;
use mongodb::{
    bson::{doc, oid::ObjectId},
    Database,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Zone {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    #[serde(rename = "propertyId")]
    pub property_id: ObjectId,
    #[serde(rename = "userId", skip_serializing_if = "Option::is_none")]
    pub user_id: Option<ObjectId>,
    #[serde(rename = "parentZoneId", skip_serializing_if = "Option::is_none")]
    pub parent_zone_id: Option<ObjectId>,
}
impl Zone {
    fn new(
        name: String,
        property_id: ObjectId,
        user_id: Option<ObjectId>,
        parent_zone_id: Option<ObjectId>,
    ) -> Zone {
        Self {
            id: None,
            name,
            property_id,
            user_id,
            parent_zone_id,
        }
    }
}
