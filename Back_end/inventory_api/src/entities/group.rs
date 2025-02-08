use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use futures_util::stream::TryStreamExt;
use mongodb::{bson::{doc, oid::ObjectId}, Database};
use serde::{Deserialize, Serialize};
use rand::Rng;

#[derive(Debug, Serialize, Deserialize)]

pub struct Group{
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    #[serde(rename = "userMax", skip_serializing_if="Option::is_none")]
    pub user_max: Option<i32>,
    #[serde(rename = "userCount")]
    pub user_count: i32,
    #[serde(rename = "groupCode")]
    pub group_code: String,
    #[serde(skip_serializing_if="Option::is_none")]
    pub tags: Option<Vec<String>>,
}

impl  Group {
    fn new (name:String, user_max:Option<i32>, user_count:i32,group_code:Option<String>,tags:Option<Vec<String>>)->Group{
        Self { id: None,
             name,
             user_max,
             user_count,
             group_code: group_code.or_else(|| Some(Self::create_group_code())).unwrap(),
             tags,
             }


    }
    fn create_group_code()-> String{
        let mut rand =rand::rng();
        let characters:Vec<char>= ('0'..='9')
        .chain('a'..='z')
        .chain('A'..='Z')
        .collect();

    let group_code=(0..8).map(|_| characters[rand.random_range(0..characters.len())])
    .collect::<String>();
    group_code    
    }
    
}