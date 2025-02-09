use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use futures_util::stream::TryStreamExt;
use mongodb::{
    bson::{doc, oid::ObjectId, DateTime},
    Database,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Item {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "pictureUrl", skip_serializing_if = "Option::is_none")]
    pub picture_url: Option<String>,
    #[serde(rename = "zoneId")]
    pub zone_id: ObjectId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub values: Option<Vec<ValueEntry>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ValueEntry {
    name: String,
    value: Value,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Value {
    Text(String),
    Number(i32),
    Date(DateTime),
}
impl Item {
    fn new(
        name: String,
        description: Option<String>,
        picture_url: Option<String>,
        zone_id: ObjectId,
        values: Option<Vec<ValueEntry>>,
        tags: Option<Vec<String>>,
    ) -> Item {
        Self {
            id: None,
            name,
            description,
            picture_url,
            zone_id,
            values,
            tags,
        }
    }
}
