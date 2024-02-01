use sqlx::Sqlite;

#[derive(Clone)]
pub struct FreyaState {
    pub db: sqlx::Pool<Sqlite>,
}

impl FreyaState {
    pub async fn new() -> Self {
        // Get the database URL from the environment.
        // Default to "freya.db" if DATABASE_PATH is not set.
        let database_path =
            std::env::var("DATABASE_PATH").unwrap_or_else(|_| "freya.db".to_string());

        // Create the database pool.
        let database_pool = migrate::open_database(&database_path).await;

        // Migrate the database if NO_MIGRATE is not set.
        if std::env::var("NO_MIGRATE").is_err() {
            migrate::migrate(&database_pool).await;
        };
        Self { db: database_pool }
    }
}
