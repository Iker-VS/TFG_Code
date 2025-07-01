use actix_web::{get, web, HttpMessage, HttpRequest, HttpResponse, Responder};
use mongodb::{
    bson::{doc, oid::ObjectId},
    Database,
};
use futures_util::stream::TryStreamExt;
use serde::Serialize;
use crate::log::write_log;

use crate::entities::{
    group::Group,
    property::Property,
    zone::Zone,
    item::Item,
    user_group::UserGroup,
};

#[derive(Serialize)]
struct SearchResponse {
    groups: Vec<Group>,
    properties: Vec<Property>,
    zones: Vec<Zone>,
    items: Vec<Item>,
}

#[get("/search/{name}")]
pub async fn search_endpoint(db: web::Data<Database>, req: HttpRequest, name: web::Path<String>) -> impl Responder {
    let search_str = name.into_inner().to_lowercase();
    let claims = req.extensions().get::<crate::middleware::auth::Claims>().unwrap().clone();
    let user_id = ObjectId::parse_str(&claims.sub).unwrap();
    let is_admin = claims.role == "admin";

    let mut groups_res = Vec::new();
    let mut properties_res = Vec::new();
    let mut zones_res = Vec::new();
    let mut items_res = Vec::new();

    // Paso 1: Obtener todos los grupos del usuario
    let mut group_ids = Vec::new();
    let user_group_coll = db.collection::<UserGroup>("userGroup");
    let filter = doc! { "userId": user_id.clone() };
    let usergroups_cursor = match user_group_coll.find(filter).await {
        Ok(cursor) => cursor,
        Err(e) => {
            write_log(&format!("GET /search/{{name}} - Error buscando userGroup: {}", e)).ok();
            return HttpResponse::InternalServerError().body(e.to_string());
        },
    };
    let user_groups: Vec<UserGroup> = match usergroups_cursor.try_collect().await {
        Ok(ugs) => ugs,
        Err(e) => {
            write_log(&format!("GET /search/{{name}} - Error recogiendo userGroup: {}", e)).ok();
            return HttpResponse::InternalServerError().body(e.to_string());
        },
    };
    let group_coll = db.collection::<Group>("groups");
    for ug in user_groups {
        if let Ok(Some(group)) = group_coll.find_one(doc! { "_id": &ug.group_id }).await {
            // Guardar group_id para siguiente paso
            if let Some(gid) = group.id.clone() { group_ids.push(gid); }
            if group.name.to_lowercase().contains(&search_str) {
                groups_res.push(group);
            }
        }
    }

    // Paso 2: Por cada grupo, obtener todas las propiedades
    let mut property_ids = Vec::new();
    let property_coll = db.collection::<Property>("properties");
    for gid in group_ids {
        let prop_filter = doc! {
            "groupId": gid.clone(),
            // No filtrar por userId si es admin
        };
        if let Ok(prop_cursor) = property_coll.find(prop_filter).await {
            let props: Vec<Property> = match prop_cursor.try_collect().await {
                Ok(ps) => ps,
                Err(_) => Vec::new(),
            };
            for prop in props {
                // Si NO es admin, omitir privadas ajenas
                if !is_admin {
                    if let Some(prop_owner) = &prop.user_id {
                        if *prop_owner != user_id {
                            continue;
                        }
                    }
                }
                if prop.name.to_lowercase().contains(&search_str) {
                    properties_res.push(prop.clone());
                }
                if let Some(pid) = prop.id {
                    property_ids.push(pid);
                }
            }
        }
    }

    // Paso 3: Por cada propiedad, obtener todas las zonas
    let mut zone_ids = Vec::new();
    let zone_coll = db.collection::<Zone>("zones");
    for pid in property_ids {
        let zone_filter = doc! { "propertyId": pid.clone() };
        if let Ok(zone_cursor) = zone_coll.find(zone_filter).await {
            let zones: Vec<Zone> = match zone_cursor.try_collect().await {
                Ok(zs) => zs,
                Err(_) => Vec::new(),
            };
            for zone in zones {
                // Si NO es admin, omitir privadas ajenas
                if !is_admin {
                    if let Some(zone_owner) = &zone.user_id {
                        if *zone_owner != user_id {
                            continue;
                        }
                    }
                }
                if zone.name.to_lowercase().contains(&search_str) {
                    zones_res.push(zone.clone());
                }
                if let Some(zid) = zone.id {
                    zone_ids.push(zid);
                }
            }
        }
    }

    // Paso 4: Por cada zona, obtener todos los ítems
    let item_coll = db.collection::<Item>("items");
    for zid in zone_ids {
        let item_filter = doc! { "zoneId": zid };
        if let Ok(item_cursor) = item_coll.find(item_filter).await {
            let items: Vec<Item> = match item_cursor.try_collect().await {
                Ok(is) => is,
                Err(_) => Vec::new(),
            };
            for item in items {
                if item.name.to_lowercase().contains(&search_str) {
                    items_res.push(item.clone());
                }
            }
        }
    }

    let response = SearchResponse {
        groups: groups_res,
        properties: properties_res,
        zones: zones_res,
        items: items_res,
    };

    write_log(&format!("GET /search/{{name}} - Búsqueda realizada: grupos={}, propiedades={}, zonas={}, items={}",
        response.groups.len(), response.properties.len(), response.zones.len(), response.items.len())).ok();
    HttpResponse::Ok().json(response)
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(search_endpoint);
}

