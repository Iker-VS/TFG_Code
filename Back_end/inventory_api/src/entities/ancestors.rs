use actix_web::{get, web, HttpResponse, Responder};
use mongodb::bson::{doc, oid::ObjectId};
use mongodb::Database;
use serde_json::json;

use crate::entities::group::Group;
use crate::entities::item::Item;
use crate::entities::property::Property;
use crate::entities::zone::Zone;

#[get("/ancestors/{id}")]
pub async fn get_ancestors_handler(
    db: web::Data<Database>,
    path: web::Path<String>,
) -> impl Responder {
    let id_str = path.into_inner();
    let obj_id = match ObjectId::parse_str(&id_str) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("ID inválido"),
    };

    let db_ref = db.get_ref();
    let items_coll = db_ref.collection::<Item>("items");
    let zones_coll = db_ref.collection::<Zone>("zones");
    let props_coll = db_ref.collection::<Property>("properties");
    let groups_coll = db_ref.collection::<Group>("groups");

    let mut found = false;
    let mut current_zone_id: Option<ObjectId> = None;
    let mut zones: Vec<Zone> = Vec::new();
    let mut property: Option<Property> = None;
    let mut group: Option<Group> = None;

    // Paso 1: Buscar si es un item
    if let Ok(Some(item)) = items_coll.find_one(doc! {"_id": obj_id.clone()}).await {
        found = true;
        current_zone_id = Some(item.zone_id);
    }

    // Paso 2: Si no es item, buscar si es zona
    if !found {
        if let Ok(Some(zone)) = zones_coll.find_one(doc! {"_id": obj_id.clone()}).await {
            current_zone_id = zone.id;
        }
    }

    // Paso 3: Recorrer zonas hasta llegar a la propiedad
    if let Some(mut zone_id) = current_zone_id {
        let mut first_time = true;
        loop {
            if let Ok(Some(zone)) = zones_coll.find_one(doc! {"_id": zone_id.clone()}).await {
                // Saltar el primer elemento si first_time es true
                if first_time {
                    first_time = false;
                } else {
                    // Insertar al principio para que la última zona esté arriba
                    zones.insert(0, zone.clone());
                }
                // Si parent_zone_id == property_id, parar
                if let Some(parent_zone_id) = zone.parent_zone_id {
                    if parent_zone_id == zone.property_id {
                        // break sin añadir de nuevo la zona
                        break;
                    } else {
                        zone_id = parent_zone_id;
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        // La última zona (la que cumple parent_zone_id == property_id) se usa solo para obtener la propiedad
        if let Ok(Some(last_zone)) = zones_coll.find_one(doc! {"_id": zone_id.clone()}).await {
            if let Ok(Some(prop)) = props_coll
                .find_one(doc! {"_id": last_zone.property_id.clone()})
                .await
            {
                property = Some(prop.clone());
            }
        }
    }

    // Paso 4: Buscar propiedad si no se encontró antes
    if property.is_none() {
        if let Ok(Some(prop)) = props_coll.find_one(doc! {"_id": obj_id.clone()}).await {
            property = Some(prop.clone());
        }
    }

    // Paso 5: Buscar grupo si se tiene propiedad
    if let Some(ref prop) = property {
        if let Ok(Some(gr)) = groups_coll
            .find_one(doc! {"_id": prop.group_id.clone()})
            .await
        {
            group = Some(gr);
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
