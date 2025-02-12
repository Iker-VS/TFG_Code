use mongodb::{Client, Database};
use once_cell::sync::OnceCell;

static DATABASE: OnceCell<Database> = OnceCell::new();

/// Inicializa la base de datos y la almacena en un singleton.
/// Si ya se creó, se devuelve la instancia existente.
pub async fn init_db() -> mongodb::error::Result<Database> {
    if let Some(db) = DATABASE.get() {
        return Ok(db.clone());
    }
    // En este ejemplo se usa la URI por defecto; modifícala según tus necesidades.
    let client = Client::with_uri_str("mongodb://localhost:27017").await?;
    // Se usa el nombre "mydatabase", pero puedes cambiarlo.
    let db = client.database("TFG");
    DATABASE
        .set(db.clone())
        .expect("Error al establecer la base de datos");
    Ok(db)
}
