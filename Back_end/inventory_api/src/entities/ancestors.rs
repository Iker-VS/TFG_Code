use actix_web::{get, web, HttpResponse, Responder};
use mongodb::bson::{doc, oid::ObjectId};
use mongodb::Database;
use serde_json::json;

use crate::entities::group::Group;
use crate::entities::property::Property;
use crate::entities::zone::Zone;
use crate::entities::item::Item;

#[get("/ancestors/{id}")]
pub async fn get_ancestors_handler(db: web::Data<Database>, path: web::Path<String>) -> impl Responder {
    let id_str = path.into_inner();

    // Data holders para la cadena de ancestros
    let mut group: Option<Group> = None;
    let mut property: Option<Property> = None;
    let mut zones: Vec<Zone> = Vec::new();

    let obj_id = match ObjectId::parse_str(&id_str) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("ID inválido"),
    };

    let db_ref = db.get_ref();
    let items_coll = db_ref.collection::<Item>("items");
    let zones_coll = db_ref.collection::<Zone>("zones");
    let props_coll = db_ref.collection::<Property>("properties");
    let groups_coll = db_ref.collection::<Group>("groups");

    // Intentamos determinar el tipo de entidad asociada al id recibido.
    // No se incluye la entidad en la respuesta, solo los ancestros.

    // 1. Si el id corresponde a un item:
    if let Ok(Some(found_item)) = items_coll.find_one(doc! {"_id": obj_id.clone()}).await {
        // A partir del item, se usa su zone_id para iniciar la cadena ascendente.
        let mut current_zone_id = found_item.zone_id;
        while let Ok(Some(zone)) = zones_coll.find_one(doc! {"_id": current_zone_id.clone()}).await {
            zones.push(zone.clone());
            if let Some(parent) = zone.parent_zone_id {
                current_zone_id = parent;
            } else {
                break;
            }
        }
        zones.reverse();
        if let Some(last_zone) = zones.last() {
            if let Ok(Some(prop)) = props_coll.find_one(doc! {"_id": last_zone.property_id.clone()}).await {
                property = Some(prop);
            }
        }
    }
    // 2. Si el id corresponde a una zone:
    else if let Ok(Some(found_zone)) = zones_coll.find_one(doc! {"_id": obj_id.clone()}).await {
        // Se omite la zona en sí y se inicia la búsqueda a partir de su parent_zone_id.
        if let Some(parent) = found_zone.parent_zone_id {
            let mut current_zone_id = parent;
            while let Ok(Some(zone)) = zones_coll.find_one(doc! {"_id": current_zone_id.clone()}).await {
                zones.push(zone.clone());
                if let Some(next_parent) = zone.parent_zone_id {
                    current_zone_id = next_parent;
                } else {
                    break;
                }
            }
            zones.reverse();
            if let Some(last_zone) = zones.last() {
                if let Ok(Some(prop)) = props_coll.find_one(doc! {"_id": last_zone.property_id.clone()}).await {
                    property = Some(prop);
                }
            }
        }
    }
    // 3. Si el id corresponde a una property:
    else if let Ok(Some(found_prop)) = props_coll.find_one(doc! {"_id": obj_id.clone()}).await {
        // No se incluye la property en la respuesta; se obtiene el grupo a partir de ella.
        if let Ok(Some(gr)) = groups_coll.find_one(doc! {"_id": found_prop.group_id.clone()}).await {
            group = Some(gr);
        }
    }
    // 4. Si el id corresponde a un group:
    else if let Ok(Some(_found_group)) = groups_coll.find_one(doc! {"_id": obj_id.clone()}).await {
        // Si es un group, no hay ancestros.
    }
    else {
        return HttpResponse::NotFound().body("No se encontró objeto con el ID suministrado");
    }

    // Si aún no se ha obtenido una property y hay zonas en cadena, intentar obtenerla
    if property.is_none() && !zones.is_empty() {
        if let Some(last_zone) = zones.last() {
            if let Ok(Some(prop)) = props_coll.find_one(doc! {"_id": last_zone.property_id.clone()}).await {
                property = Some(prop);
            }
        }
    }

    // Si se obtuvo property y no se halló el grupo, buscarlo
    if property.is_some() && group.is_none() {
        if let Some(ref prop) = property {
            if let Ok(Some(gr)) = groups_coll.find_one(doc! {"_id": prop.group_id.clone()}).await {
                group = Some(gr);
            }
        }
    }

    let response = json!({
        "group": group,
        "property": property,
        "zones": zones
    });
    HttpResponse::Ok().json(response)
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_ancestors_handler);
}
