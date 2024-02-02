#[tokio::main]
async fn main() {
    migrate_database().await;
}

// Create a database file if none exists.
async fn migrate_database() {
    // sqlx uses the database file to infer types and check queries at compile time.
    let database = freya_migrate::open_database("freya.db").await;

    // Migrate the database.
    freya_migrate::migrate(&database).await;
}
