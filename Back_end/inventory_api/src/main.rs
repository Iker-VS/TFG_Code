use actix_web::{web, App, HttpServer};
use actix_cors::Cors; // <-- Asegúrate de agregar esta importación
use middleware::auth::AuthMiddleware;

mod db;
mod entities;
mod middleware;
mod routes;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Inicializa la conexión a la base de datos.
    // La función init_db se asegura de que solo se cree una única instancia (singleton).
    let database = db::init_db()
        .await
        .expect("Error al inicializar la base de datos");

    // Configura el servidor HTTP y define las rutas.
    HttpServer::new(move || {
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
    .bind(("172.30.188.140", 8000))?
    .run()
    .await
}
