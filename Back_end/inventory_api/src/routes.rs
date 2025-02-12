use actix_web::web;
use tokio::signal::windows::ctrl_logoff;

use crate::entities::{group, item, log, property, user, user_group, zone};

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    group::configure_routes(cfg);
    item::configure_routes(cfg);
    log::configure_routes(cfg);
    property::configure_routes(cfg);
    user_group::configure_routes(cfg);
    user::configure_routes(cfg);
    zone::configure_routes(cfg);

}
