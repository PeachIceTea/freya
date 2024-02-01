#[tokio::main]
async fn main() {
    // Create a database file if none exists.
    // sqlx uses the database file to infer types and check queries at compile time.
    let database = migrate::open_database("freya.db").await;

    // Migrate the database.
    migrate::migrate(&database).await;
}
