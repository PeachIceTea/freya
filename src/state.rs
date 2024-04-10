use crate::database::Database;

#[derive(Clone)]
pub struct FreyaState {
    pub database: Database,
}

impl FreyaState {
    pub async fn new() -> Self {
        // Get the database URL from the environment.
        // Default to "freya.db" if DATABASE_PATH is not set.
        let database_path =
            std::env::var("DATABASE_PATH").unwrap_or_else(|_| "freya.db".to_string());

        // Create instance of database.
        let database = Database::new(&database_path).await;

        // Migrate the database if NO_MIGRATE is not set.
        if std::env::var("NO_MIGRATE").is_err() {
            database.migrate().await;
        };
        Self { database }
    }
}
