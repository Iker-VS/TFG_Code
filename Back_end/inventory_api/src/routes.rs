use actix_web::web;

use crate::entities::{ancestors, group, image, item, property, search, tree, user, user_group, zone};

pub fn configure_private_routes(cfg: &mut web::ServiceConfig) {
    ancestors::configure_routes(cfg);
    group::configure_routes(cfg);
    //image::configure_routes(cfg);
    item::configure_routes(cfg);
    property::configure_routes(cfg);
    search::configure_routes(cfg);
    tree::configure_routes(cfg);
    user_group::configure_routes(cfg);
    user::configure_private_routes(cfg);
    zone::configure_routes(cfg);

}
pub fn configure_public_routes(cfg: &mut web::ServiceConfig){
    user::configure_public_routes(cfg);
    image::configure_routes(cfg); 
}