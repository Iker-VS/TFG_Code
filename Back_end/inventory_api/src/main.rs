use actix_web::{web, App, HttpServer};
use actix_cors::Cors; // <-- Asegúrate de agregar esta importación
use middleware::auth::AuthMiddleware;

use crate::log::write_log;

mod db;
mod entities;
mod middleware;
mod routes;
mod log;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    write_log("[START] Iniciando el programa").ok();

    // Inicializa la conexión a la base de datos.
    // La función init_db se asegura de que solo se cree una única instancia (singleton).
    let database = db::init_db()
        .await
        .map(|db| {
            write_log("[START] Base de datos inicializada correctamente").ok();
            db
        })
        .expect("Error al inicializar la base de datos");

    // Configura el servidor HTTP y define las rutas.
    let result = HttpServer::new(move || {
        // Configura el CORS para permitir el origen http://localhost:8081
        let cors = Cors::default()
            .allowed_origin("http://localhost:8081")
            .allow_any_header()
            .allow_any_method()
            .supports_credentials();

        App::new()
            // Se aplica CORS a todas las rutas.
            .wrap(cors)
            // Se inyecta la base de datos como estado compartido en la app.
            .app_data(web::Data::new(database.clone()))
            // Rutas públicas.
            .service(web::scope("/public").configure(routes::configure_public_routes))
            // Rutas privadas protegidas con auth middleware.
            .service(
                web::scope("/private")
                    .wrap(AuthMiddleware)
                    .configure(routes::configure_private_routes),
            )
    })
    .bind(("127.0.0.1", 8000))? //127.0.0.1 or 172.30.188.140
    .run()
    .await;

    write_log("[STOP] Programa finalizado").ok();
    result
}
