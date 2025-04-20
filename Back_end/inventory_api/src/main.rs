use actix_web::{web, App, HttpServer};
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
        App::new()
            // Se inyecta la base de datos como estado compartido en la app.
            .app_data(web::Data::new(database.clone()))
            // Se configuran las rutas definidas en el módulo routes.
            .service(web::scope("/public").configure(routes::configure_public_routes))
            .service(
                web::scope("/private")
                    .wrap(AuthMiddleware)
                    .configure(routes::configure_private_routes),
            )
    })
    .bind(("192.168.1.141", 8000))?
    .run()
    .await
}
