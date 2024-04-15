use anyhow::{Context, Result};
use serde::Serialize;
use time::OffsetDateTime;

use super::Database;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: i64,

    pub name: String,

    #[serde(skip)]
    pub password: Option<String>,

    pub admin: bool,

    #[serde(with = "time::serde::iso8601")]
    pub created: OffsetDateTime,
    #[serde(with = "time::serde::iso8601")]
    pub modified: OffsetDateTime,
}

impl Database {
    // Get all users.
    pub async fn get_all_users(&self) -> Result<Vec<User>> {
        sqlx::query_as!(
            User,
            r#"
                SELECT 
                    id,
                    name,
                    NULL as "password: String",
                    admin,
                    created,
                    modified
                FROM users
            "#
        )
        .fetch_all(&self.pool)
        .await
        .context("Unable to get all users")
    }

    // Get user by id.
    pub async fn get_user(&self, user_id: i64) -> Result<User> {
        sqlx::query_as!(
            User,
            r#"
                SELECT 
                    id,
                    name,
                    NULL as "password: String",
                    admin,
                    created,
                    modified
                FROM users
                WHERE id = $1 
            "#,
            user_id,
        )
        .fetch_one(&self.pool)
        .await
        .context("Unable to get user")
    }

    // Get user by id and include password.
    pub async fn get_user_with_password(&self, username: &str) -> Result<User> {
        sqlx::query_as!(
            User,
            r#"
                SELECT 
                    id,
                    name,
                    password,
                    admin,
                    created,
                    modified
                FROM users
                WHERE name = $1 
            "#,
            username,
        )
        .fetch_one(&self.pool)
        .await
        .context("Unable to get user with password")
    }

    // Create user.
    pub async fn create_user(&self, username: &str, password: &str, admin: bool) -> Result<()> {
        sqlx::query!(
            r#"
                INSERT INTO users (name, password, admin)
                VALUES ($1, $2, $3)
            "#,
            username,
            password,
            admin
        )
        .execute(&self.pool)
        .await
        .context("Unable to create new user")
        .map(|_| ())
    }

    // Update user.
    pub async fn update_user(
        &self,
        user_id: i64,
        username: Option<String>,
        password: Option<String>,
        admin: Option<bool>,
    ) -> Result<()> {
        sqlx::query_as!(
            User,
            r#"
                UPDATE users
                SET
                    name = COALESCE(?, name),
                    password = COALESCE(?, password),
                    admin = COALESCE(?, admin)
                WHERE id = ?
            "#,
            username,
            password,
            admin,
            user_id
        )
        .execute(&self.pool)
        .await
        .context("Unable to update user")
        .map(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use sqlx::{Pool, Sqlite};

    use super::*;

    #[sqlx::test]
    async fn test_create_user(pool: Pool<Sqlite>) {
        let db = Database::new_test(pool);

        let username = "test_user";
        let password = "password123";
        let admin = false;

        db.create_user(username, password, admin)
            .await
            .expect("Should be able to create user");

        let user = db
            .get_user_with_password("test_user")
            .await
            .expect("Should be able to get user from database");

        assert_eq!(username, user.name);
        assert_eq!(user.password, Some(password.to_string()));
        assert_eq!(admin, user.admin);
    }

    #[sqlx::test(fixtures("user"))]
    async fn test_get_all_users(pool: Pool<Sqlite>) {
        let db = Database::new_test(pool);

        let users = db
            .get_all_users()
            .await
            .expect("Should be able to get all users");

        // Right now the migrations include a default user "admin".
        assert_eq!(users.len(), 2);
        assert_eq!(users[0].name, "admin");
        assert_eq!(users[1].name, "user");
    }

    #[sqlx::test(fixtures("user"))]
    async fn test_get_user(pool: Pool<Sqlite>) {
        let db = Database::new_test(pool);

        let user = db.get_user(2).await.expect("Should be able to get user");

        assert_eq!(user.id, 2);
        assert_eq!(user.name, "user");
        assert!(!user.admin);
    }

    #[sqlx::test(fixtures("user"))]
    async fn test_update_user(pool: Pool<Sqlite>) {
        let db = Database::new_test(pool);

        let username = "user";

        let updated_username = "updated_user";
        let updated_password = "password123";

        let user = db
            .get_user_with_password(username)
            .await
            .expect("Should be able to get user");

        db.update_user(
            user.id,
            Some(updated_username.to_string()),
            Some(updated_password.to_string()),
            Some(true),
        )
        .await
        .expect("Should be able to update user");

        let updated = db
            .get_user_with_password(updated_username)
            .await
            .expect("Should be able to get user");

        assert_eq!(updated.name, updated_username);
        assert_eq!(updated.password, Some(updated_password.to_string()));
        assert!(updated.admin);
    }

    #[sqlx::test(fixtures("user"))]
    async fn test_update_user_name(pool: Pool<Sqlite>) {
        let db = Database::new_test(pool);

        let username = "user";
        let updated_username = "updated_user";

        let user = db
            .get_user_with_password(username)
            .await
            .expect("Should be able to get user");

        db.update_user(user.id, Some(updated_username.to_string()), None, None)
            .await
            .expect("Should be able to only update username");
        let updated = db
            .get_user_with_password(updated_username)
            .await
            .expect("Should be able to get user");

        assert_eq!(updated.name, updated_username);
        assert_eq!(updated.password, user.password);
        assert_eq!(updated.admin, user.admin);
    }

    #[sqlx::test(fixtures("user"))]
    async fn test_update_user_password(pool: Pool<Sqlite>) {
        let db = Database::new_test(pool);

        let username = "user";
        let updated_password = "password123";

        let user = db
            .get_user_with_password(username)
            .await
            .expect("Should be able to get user");

        db.update_user(user.id, None, Some(updated_password.to_string()), None)
            .await
            .expect("Should be able to only update password");
        let updated = db
            .get_user_with_password(username)
            .await
            .expect("Should be able to get user");

        assert_eq!(updated.name, user.name);
        assert_eq!(updated.password, Some(updated_password.to_string()));
        assert_eq!(updated.admin, user.admin);
    }

    #[sqlx::test(fixtures("user"))]
    async fn test_update_user_admin(pool: Pool<Sqlite>) {
        let db = Database::new_test(pool);

        let username = "user";

        let user = db
            .get_user_with_password(username)
            .await
            .expect("Should be able to get user");

        db.update_user(user.id, None, None, Some(true))
            .await
            .expect("Should be able to only update admin");
        let updated = db
            .get_user_with_password(username)
            .await
            .expect("Should be able to get user");

        assert_eq!(updated.name, user.name);
        assert_eq!(updated.password, user.password);
        assert!(updated.admin);
    }
}
