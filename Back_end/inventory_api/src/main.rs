use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use middleware::auth::AuthMiddleware;

use crate::log::write_log;

mod db;
mod entities;
mod log;
mod middleware;
mod routes;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    write_log("[START] Iniciando el programa").ok();

    let database = db::init_db()
        .await
        .map(|db| {
            write_log("[START] Base de datos inicializada correctamente").ok();
            db
        })
        .expect("Error al inicializar la base de datos");

    let result = HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost:8081")
            .allow_any_header()
            .allow_any_method()
            .supports_credentials();

        App::new()
            .wrap(cors)
            .app_data(web::Data::new(database.clone()))
            .service(web::scope("/public").configure(routes::configure_public_routes))
            .service(
                web::scope("/private")
                    .wrap(AuthMiddleware)
                    .configure(routes::configure_private_routes),
            )
    })
    .bind(("192.168.1.142", 8000))?
    .run()
    .await;

    write_log("[STOP] Programa finalizado").ok();
    result
}
