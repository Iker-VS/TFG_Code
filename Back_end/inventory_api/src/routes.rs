use actix_web::web;

use crate::entities::user;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    user::configure_routes(cfg);
}
