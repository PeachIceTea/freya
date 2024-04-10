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
