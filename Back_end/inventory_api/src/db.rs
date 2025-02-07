// src/db.rs

use mongodb::{Client, Database};
use tokio::sync::OnceCell;

// Creamos un OnceCell para almacenar la instancia de Database de forma global.
static DB_INSTANCE: OnceCell<Database> = OnceCell::const_new();

/// Inicializa la conexión a la base de datos.
/// Esta función debe llamarse al iniciar la aplicación, por ejemplo, en el main.
pub async fn init_db(uri: &str, db_name: &str) -> &'static Database {
    DB_INSTANCE
        .get_or_init(async {
            let client = Client::with_uri_str(uri)
                .await
                .expect("Error al conectar a MongoDB");
            client.database(db_name)
        })
        .await
}

/// Devuelve la instancia de la base de datos ya inicializada.
/// Si no ha sido inicializada, se producirá un panic.
pub fn get_db() -> &'static Database {
    DB_INSTANCE
        .get()
        .expect("La base de datos no ha sido inicializada. Llama a init_db() primero.")
}
