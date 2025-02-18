#[cfg(test)]
mod tests {
    use super::*;
    use mongodb::{Client, Database};
    use crate::db::get_test_db;
    use crate::entities::user::{User, NewUser};

    #[tokio::test]
    async fn test_create_user() {
        let db = get_test_db().await;
        let users_collection = db.collection::<User>("users");
        
        // Asegurar que la base de datos está limpia
        users_collection.delete_many(doc! {}, None).await.unwrap();
        
        let new_user = NewUser {
            username: "testuser".to_string(),
            email: "testuser@example.com".to_string(),
            password_hash: "hashedpassword".to_string(),
        };
        
        let user = User::create(&db, new_user).await.unwrap();
        assert_eq!(user.username, "testuser");
        assert_eq!(user.email, "testuser@example.com");
    }

    #[tokio::test]
    async fn test_get_user_by_id() {
        let db = get_test_db().await;
        let users_collection = db.collection::<User>("users");
        
        let new_user = NewUser {
            username: "testuser2".to_string(),
            email: "testuser2@example.com".to_string(),
            password_hash: "hashedpassword".to_string(),
        };
        
        let user = User::create(&db, new_user).await.unwrap();
        let fetched_user = User::get_by_id(&db, user.id).await.unwrap();
        
        assert_eq!(fetched_user.id, user.id);
        assert_eq!(fetched_user.username, "testuser2");
    }

    #[tokio::test]
    async fn test_delete_user() {
        let db = get_test_db().await;
        let users_collection = db.collection::<User>("users");
        
        let new_user = NewUser {
            username: "todelete".to_string(),
            email: "todelete@example.com".to_string(),
            password_hash: "hashedpassword".to_string(),
        };
        
        let user = User::create(&db, new_user).await.unwrap();
        User::delete(&db, user.id).await.unwrap();
        
        let result = User::get_by_id(&db, user.id).await;
        assert!(result.is_err());
    }
}
