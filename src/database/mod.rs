pub mod book;
pub mod chapter;
pub mod file;
pub mod library;
pub mod session;
pub mod user;

use sqlx::Sqlite;

#[derive(Clone)]
pub struct Database {
    pub pool: sqlx::Pool<Sqlite>,
}

impl Database {
    pub async fn new(path: &str) -> Self {
        // Check if database file exists.
        if !std::path::Path::new(path).exists() {
            // Touch the database file.
            std::fs::File::create(path).expect("Should create database file");
        }

        // Create the database pool.
        let pool = sqlx::Pool::connect(&path)
            .await
            .expect("Should connect to database");

        Self { pool }
    }

    pub async fn migrate(&self) {
        match sqlx::migrate!().run(&self.pool).await {
            Ok(_) => {}
            Err(err) => panic!("Could not migrate database: {}", err),
        };
    }
}
