use actix_web::{web, App, HttpServer};

mod db;
mod routes;

mod entities;

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
            .configure(routes::configure_routes)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
