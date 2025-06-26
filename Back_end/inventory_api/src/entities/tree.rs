use actix_web::{get, web, HttpMessage, HttpRequest, HttpResponse, Responder};
use mongodb::bson::{doc, oid::ObjectId};
use mongodb::Database;
use serde_json::json;
use futures_util::stream::TryStreamExt;
use std::collections::HashMap;
use crate::log::write_log;

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
    let ug_cursor = match usergroup_coll.find(ug_filter).await {
        Ok(cursor) => cursor,
        Err(e) => {
            write_log(&format!("GET /tree - Error buscando userGroup: {}", e)).ok();
            return HttpResponse::InternalServerError().body(e.to_string());
        },
    };
    let usergroups: Vec<UserGroup> = ug_cursor.try_collect().await.unwrap_or_else(|_| Vec::new());
    let group_ids: Vec<ObjectId> = usergroups.iter().map(|ug| ug.group_id.clone()).collect();

    // --- Step 2: Concurrently query groups and properties ---
    let groups_coll = db_ref.collection::<Group>("groups");
    let prop_coll = db_ref.collection::<Property>("properties");
    let groups_future = groups_coll.find(doc! { "_id": { "$in": &group_ids } });
    let properties_future = prop_coll.find(doc! { "groupId": { "$in": &group_ids } });
    let groups_cursor = match groups_future.await {
        Ok(cursor) => cursor,
        Err(e) => {
            write_log(&format!("GET /tree - Error buscando grupos: {}", e)).ok();
            return HttpResponse::InternalServerError().body("Failed to fetch groups");
        },
    };
    let prop_cursor = match properties_future.await {
        Ok(cursor) => cursor,
        Err(e) => {
            write_log(&format!("GET /tree - Error buscando propiedades: {}", e)).ok();
            return HttpResponse::InternalServerError().body("Failed to fetch properties");
        },
    };
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
        let cursor = match zones_future.await {
            Ok(c) => c,
            Err(e) => {
                write_log(&format!("GET /tree - Error buscando zonas: {}", e)).ok();
                return HttpResponse::InternalServerError().body("Failed to fetch zones");
            },
        };
        cursor.try_collect().await.unwrap_or_else(|_| Vec::new())
    };
    let zone_ids: Vec<ObjectId> = zones.iter().filter_map(|z| z.id.clone()).collect();
    let items: Vec<Item> = {
        let cursor = match items_coll.find(doc! { "zoneId": { "$in": &zone_ids } }).await {
            Ok(c) => c,
            Err(e) => {
                write_log(&format!("GET /tree - Error buscando items: {}", e)).ok();
                return HttpResponse::InternalServerError().body("Failed to fetch items");
            },
        };
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
    let mut zones_by_property: HashMap<ObjectId, Vec<&Zone>> = HashMap::new();
    for zone in &zones {
        let prop_id = zone.property_id.clone();
        zones_by_property.entry(prop_id)
            .or_insert_with(Vec::new)
            .push(zone);
    }

    // --- Step 5: Build zone tree for each property --- 
    // Construye el árbol de zonas de manera recursiva, permitiendo anidamiento infinito
    fn build_zone_tree(
        parent_id: Option<&ObjectId>,
        zones_by_parent: &HashMap<Option<ObjectId>, Vec<&Zone>>,
        items_by_zone: &HashMap<ObjectId, Vec<serde_json::Value>>
    ) -> Vec<serde_json::Value> {
        let mut nodes = Vec::new();
        if let Some(zones) = zones_by_parent.get(&parent_id.cloned()) {
            for zone in zones {
                if let Some(zid) = &zone.id {
                    // Recursivamente busca subzonas
                    let children_zones = build_zone_tree(Some(zid), zones_by_parent, items_by_zone);
                    // Items para esta zona
                    let mut children = children_zones;
                    // Si no hay subzonas, añade los items como hijos
                    if children.is_empty() {
                        if let Some(items) = items_by_zone.get(zid) {
                            children = items.clone();
                        }
                    } else {
                        // Si hay subzonas, también puedes añadir los items aquí si lo deseas
                        if let Some(items) = items_by_zone.get(zid) {
                            children.extend(items.clone());
                        }
                    }
                    let node = json!({
                        "_id": zid.to_hex(),
                        "name": zone.name,
                        "type": "zone",
                        "children": children
                    });
                    nodes.push(node);
                }
            }
        }
        nodes
    }
    // Prepara el mapa de zonas por propertyId y por parent_zone_id (por propiedad)
    let mut zones_by_property: HashMap<ObjectId, Vec<&Zone>> = HashMap::new();
    for zone in zones.iter() {
        zones_by_property.entry(zone.property_id.clone()).or_insert_with(Vec::new).push(zone);
    }
    let mut property_zone_trees: HashMap<ObjectId, Vec<serde_json::Value>> = HashMap::new();
    for (prop_id, zones_vec) in zones_by_property.iter() {
        let mut zones_by_parent: HashMap<Option<ObjectId>, Vec<&Zone>> = HashMap::new();
        let mut all_zone_ids = std::collections::HashSet::new();
        for zone in zones_vec.iter() {
            zones_by_parent.entry(zone.parent_zone_id.clone()).or_insert_with(Vec::new).push(*zone);
            if let Some(zid) = &zone.id {
                all_zone_ids.insert(zid.clone());
            }
        }
        let mut roots = Vec::new();
        let mut root_count = 0;
        for zone in zones_vec.iter() {
            let is_root = match &zone.parent_zone_id {
                None => true,
                Some(pid) => !all_zone_ids.contains(pid),
            };
            if is_root {
                root_count += 1;
                if let Some(zid) = &zone.id {
                    let tree = build_zone_tree(Some(zid), &zones_by_parent, &items_by_zone);
                    let mut children = tree;
                    if children.is_empty() {
                        if let Some(items) = items_by_zone.get(zid) {
                            children = items.clone();
                        }
                    } else {
                        if let Some(items) = items_by_zone.get(zid) {
                            children.extend(items.clone());
                        }
                    }
                    let node = json!({
                        "_id": zid.to_hex(),
                        "name": zone.name,
                        "type": "zone",
                        "children": children
                    });
                    roots.push(node);
                }
            }
        }
        let prop_name = properties.iter().find(|p| p.id.as_ref() == Some(prop_id)).map(|p| p.name.clone()).unwrap_or_else(|| "<desconocido>".to_string());
        write_log(&format!("GET /tree - Propiedad '{}' ({}): {} zonas raíz detectadas", prop_name, prop_id.to_hex(), root_count)).ok();
        if roots.is_empty() {
            write_log(&format!("GET /tree - Propiedad '{}' ({}): sin zonas raíz, children vacío", prop_name, prop_id.to_hex())).ok();
        }
        property_zone_trees.insert(prop_id.clone(), roots);
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
    write_log(&format!("GET /tree - Árbol generado con {} grupos", final_tree.len())).ok();
    HttpResponse::Ok().json(final_tree)
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(tree_endpoint);
}
