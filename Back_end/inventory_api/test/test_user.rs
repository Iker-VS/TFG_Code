// tests/test_user.rs
use actix_web::{test, App, http::StatusCode, web};
use mongodb::{bson::doc, Client, Database};
use mongodb::bson::oid::ObjectId;
use std::env;
use mi_proyecto::entities::user::{configure_routes, User}; // Ajusta el path según tu proyecto

/// Configura y devuelve una base de datos de prueba.
/// Esta función limpia la colección "users" antes de cada test.
async fn setup_test_db() -> Database {
    // Puedes configurar la URI desde una variable de entorno o hardcodearla para pruebas.
    let client_uri = env::var("MONGO_URI").unwrap_or_else(|_| "mongodb://localhost:27017".to_string());
    let client = Client::with_uri_str(&client_uri).await.unwrap();
    let db = client.database("test_db");

    // Limpiar la colección "users" (y otras si es necesario) para empezar con una DB vacía.
    db.collection("users").delete_many(doc! {}, None).await.unwrap();
    db
}

#[actix_web::test]
async fn test_get_users_empty() {
    let db = setup_test_db().await;
    let app = test::init_service(
        App::new().app_data(web::Data::new(db.clone())).configure(configure_routes)
    )
    .await;

    let req = test::TestRequest::get().uri("/users").to_request();
    // Al estar la DB vacía, esperamos una lista vacía.
    let resp: Vec<User> = test::call_and_read_body_json(&app, req).await;
    assert!(resp.is_empty(), "La lista de usuarios debe estar vacía");
}

#[actix_web::test]
async fn test_create_user() {
    let db = setup_test_db().await;
    let app = test::init_service(
        App::new().app_data(web::Data::new(db.clone())).configure(configure_routes)
    )
    .await;

    // Creamos un usuario nuevo.
    let new_user = User::new(
        "test@example.com".to_string(),
        "hashed_password".to_string(),
        "Test User".to_string(),
        Some(false)
    );

    let req = test::TestRequest::post()
        .uri("/users")
        .set_json(&new_user)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK, "El usuario debería crearse exitosamente");

    // Verificamos que el usuario se haya insertado en la base de datos.
    let inserted = db.collection::<User>("users")
        .find_one(doc! {"mail": "test@example.com"}, None)
        .await
        .unwrap();
    assert!(inserted.is_some(), "El usuario debe existir en la base de datos");
}

#[actix_web::test]
async fn test_get_user_by_id() {
    let db = setup_test_db().await;
    
    // Insertamos un usuario de prueba directamente en la base de datos.
    let user = User::new(
        "gettest@example.com".to_string(),
        "hash".to_string(),
        "Get Test".to_string(),
        Some(false)
    );
    let insert_result = db.collection("users")
        .insert_one(&user, None)
        .await
        .unwrap();
    let obj_id = insert_result.inserted_id.as_object_id().unwrap().to_hex();

    let app = test::init_service(
        App::new().app_data(web::Data::new(db.clone())).configure(configure_routes)
    )
    .await;

    let req = test::TestRequest::get().uri(&format!("/users/{}", obj_id)).to_request();
    let resp: User = test::call_and_read_body_json(&app, req).await;
    assert_eq!(resp.mail, "gettest@example.com", "El correo del usuario debe coincidir");
}

#[actix_web::test]
async fn test_update_user() {
    let db = setup_test_db().await;
    
    // Insertamos un usuario para actualizar.
    let user = User::new(
        "updatetest@example.com".to_string(),
        "old_hash".to_string(),
        "Old Name".to_string(),
        Some(false)
    );
    let insert_result = db.collection("users")
        .insert_one(&user, None)
        .await
        .unwrap();
    let user_id = insert_result.inserted_id.as_object_id().unwrap().to_hex();

    let app = test::init_service(
        App::new().app_data(web::Data::new(db.clone())).configure(configure_routes)
    )
    .await;

    // Preparamos los datos actualizados.
    let updated_user = User::new(
        "updatetest@example.com".to_string(),
        "new_hash".to_string(),
        "New Name".to_string(),
        Some(true)
    );
    let req = test::TestRequest::put()
        .uri(&format!("/users/{}", user_id))
        .set_json(&updated_user)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK, "El usuario debería actualizarse correctamente");

    // Verificamos en la base de datos que los cambios se hayan aplicado.
    let updated = db.collection::<User>("users")
        .find_one(doc! {"_id": mongodb::bson::oid::ObjectId::parse_str(&user_id).unwrap()}, None)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(updated.name, "New Name");
    assert_eq!(updated.password_hash, "new_hash");
    assert_eq!(updated.admin, Some(true));
}

#[actix_web::test]
async fn test_delete_user() {
    let db = setup_test_db().await;
    
    // Insertamos un usuario para eliminar.
    let user = User::new(
        "deletetest@example.com".to_string(),
        "delete_hash".to_string(),
        "Delete User".to_string(),
        Some(false)
    );
    let insert_result = db.collection("users")
        .insert_one(&user, None)
        .await
        .unwrap();
    let user_id = insert_result.inserted_id.as_object_id().unwrap().to_hex();

    let app = test::init_service(
        App::new().app_data(web::Data::new(db.clone())).configure(configure_routes)
    )
    .await;

    let req = test::TestRequest::delete().uri(&format!("/users/{}", user_id)).to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK, "El usuario debería eliminarse");

    // Verificamos que el usuario se haya eliminado de la base de datos.
    let count = db.collection("users")
        .count_documents(doc! {"_id": mongodb::bson::oid::ObjectId::parse_str(&user_id).unwrap()}, None)
        .await
        .unwrap();
    assert_eq!(count, 0, "El usuario debe haber sido eliminado");
}
