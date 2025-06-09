use actix_web::{get, web, HttpMessage, HttpRequest, HttpResponse, Responder};
use mongodb::bson::{doc, oid::ObjectId};
use mongodb::Database;
use serde_json::json;
use futures_util::stream::TryStreamExt;
use std::collections::HashMap;

// Importamos entidades
use crate::entities::group::Group;
use crate::entities::property::Property;
use crate::entities::zone::Zone;
use crate::entities::item::Item;
use crate::entities::user_group::UserGroup;

#[get("/tree")]
pub async fn tree_endpoint(db: web::Data<Database>, req: HttpRequest) -> impl Responder {
    // Extract userId from token in req.extensions()
    let claims = req.extensions().get::<crate::middleware::auth::Claims>().unwrap().clone();
    let user_id = ObjectId::parse_str(&claims.sub).unwrap();
    let db_ref = db.get_ref();

    // --- Step 1: Get userGroup relations ---
    let usergroup_coll = db_ref.collection::<UserGroup>("userGroup");
    let ug_filter = doc! { "userId": user_id.clone() };
    let ug_cursor = usergroup_coll.find(ug_filter).await.unwrap();
    let usergroups: Vec<UserGroup> = ug_cursor.try_collect().await.unwrap_or_else(|_| Vec::new());
    let group_ids: Vec<ObjectId> = usergroups.iter().map(|ug| ug.group_id.clone()).collect();

    // --- Step 2: Concurrently query groups and properties ---
    let groups_coll = db_ref.collection::<Group>("groups");
    let prop_coll = db_ref.collection::<Property>("properties");
    let groups_future = groups_coll.find(doc! { "_id": { "$in": &group_ids } });
    let properties_future = prop_coll.find(doc! { "groupId": { "$in": &group_ids } });
    let groups_cursor = groups_future.await.expect("Failed to fetch groups");
    let prop_cursor = properties_future.await.expect("Failed to fetch properties");
    let groups: Vec<Group> = groups_cursor.try_collect().await.unwrap_or_else(|_| Vec::new());
    let properties: Vec<Property> = prop_cursor.try_collect().await.unwrap_or_else(|_| Vec::new());
    let property_ids: Vec<ObjectId> = properties.iter().filter_map(|p| p.id.clone()).collect();

    // --- Step 3: Concurrently query zones and items ---
    let zones_coll = db_ref.collection::<Zone>("zones");
    let zones_filter = doc! {
        "propertyId": { "$in": &property_ids },
        "$or": [
            { "userId": { "$exists": false } },
            { "userId": user_id.clone() }
        ]
    };
    let items_coll = db_ref.collection::<Item>("items");
    let zones_future = zones_coll.find(zones_filter);
    // We'll later filter items by zoneIds, so query zones first.
    let zones: Vec<Zone> = {
        let cursor = zones_future.await.unwrap();
        cursor.try_collect().await.unwrap_or_else(|_| Vec::new())
    };
    let zone_ids: Vec<ObjectId> = zones.iter().filter_map(|z| z.id.clone()).collect();
    let items: Vec<Item> = {
        let cursor = items_coll.find(doc! { "zoneId": { "$in": &zone_ids } }).await.unwrap();
        cursor.try_collect().await.unwrap_or_else(|_| Vec::new())
    };

    // --- Step 4: Build in-memory maps ---
    let mut items_by_zone: HashMap<ObjectId, Vec<serde_json::Value>> = HashMap::new();
    for item in items {
        if let Some(item_id) = item.id {
            let item_node = json!({
                "_id": item_id.to_hex(),
                "name": item.name,
                "type": "item"
            });
            items_by_zone.entry(item.zone_id.clone())
                .or_insert_with(Vec::new)
                .push(item_node);
        }
    }
    // Group zones by their propertyId (property_id is assumed non-optional)
    let mut zones_by_property: HashMap<ObjectId, Vec<Zone>> = HashMap::new();
    for zone in zones {
        let prop_id = zone.property_id.clone();
        zones_by_property.entry(prop_id)
            .or_insert_with(Vec::new)
            .push(zone);
    }

    // --- Step 5: Build zone tree for each property --- 
    // Synchronous function to build the zone tree from a slice of zones.
    fn build_zone_tree_sync(zones: &[Zone], items_by_zone: &HashMap<ObjectId, Vec<serde_json::Value>>) -> Vec<serde_json::Value> {
        let mut nodes: HashMap<ObjectId, serde_json::Value> = HashMap::new();
        for z in zones {
            if let Some(zid) = z.id.clone() {
                let node = json!({
                    "_id": zid.to_hex(),
                    "name": z.name.clone(),
                    "type": "zone",
                    "children": items_by_zone.get(&zid).cloned().unwrap_or_else(Vec::new)
                });
                nodes.insert(zid, node);
            }
        }
        let mut roots = Vec::new();
        for z in zones {
            if let Some(zid) = z.id.clone() {
                let current_node = nodes.get(&zid).cloned();
                if let Some(parent_id) = z.parent_zone_id {
                    if let Some(parent_node) = nodes.get_mut(&parent_id) {
                        // Clone the child node before borrowing mutably.
                        if let Some(child_node) = current_node {
                            let children = parent_node.get_mut("children").unwrap().as_array_mut().unwrap();
                            children.push(child_node);
                        }
                    } else if let Some(child_node) = current_node {
                        roots.push(child_node);
                    }
                } else if let Some(child_node) = current_node {
                    roots.push(child_node);
                }
            }
        }
        roots
    }
    let mut property_zone_trees: HashMap<ObjectId, Vec<serde_json::Value>> = HashMap::new();
    for (prop_id, zones_vec) in zones_by_property.iter() {
        let tree = build_zone_tree_sync(zones_vec, &items_by_zone);
        property_zone_trees.insert(prop_id.clone(), tree);
    }

    // --- Step 6: Build final JSON tree ---
    let mut final_tree = Vec::new();
    for group in groups {
        if let Some(group_id) = group.id.clone() {
            let props: Vec<&Property> = properties.iter().filter(|p| p.group_id == group_id).collect();
            let mut prop_nodes = Vec::new();
            for prop in props {
                if let Some(prop_id) = prop.id.clone() {
                    let zones_tree = property_zone_trees.remove(&prop_id).unwrap_or_else(Vec::new);
                    let prop_node = json!({
                        "_id": prop_id.to_hex(),
                        "name": prop.name,
                        "type": "property",
                        "children": zones_tree
                    });
                    prop_nodes.push(prop_node);
                }
            }
            let group_node = json!({
                "_id": group_id.to_hex(),
                "name": group.name,
                "type": "group",
                "children": prop_nodes
            });
            final_tree.push(group_node);
        }
    }

    HttpResponse::Ok().json(final_tree)
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(tree_endpoint);
}
