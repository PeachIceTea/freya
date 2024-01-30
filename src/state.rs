use sqlx::Sqlite;

type DB = sqlx::Pool<Sqlite>;

#[derive(Clone)]
pub struct FreyaState {
    pub db: DB,
}

impl FreyaState {
    pub async fn new() -> Self {
        // Get the database URL from the environment.
        // Default to "freya.db" if DATABASE_PATH is not set.
        let database_path =
            std::env::var("DATABASE_PATH").unwrap_or_else(|_| "freya.db".to_string());

        // Check if database file exists.
        if !std::path::Path::new(&database_path).exists() {
            // Touch the database file.
            std::fs::File::create(&database_path).expect("Should create database file");
        }

        // Create the database pool.
        let database_pool = sqlx::Pool::connect(&database_path)
            .await
            .expect("Should connect to database");

        // Migrate the database if NO_MIGRATE is not set.
        if std::env::var("NO_MIGRATE").is_err() {
            migrate(&database_pool).await;
        };
        Self { db: database_pool }
    }
}

#[derive(rust_embed::RustEmbed)]
#[folder = "migrations"]
struct Migrations;

async fn migrate(database: &DB) {
    // Check if migrations table exists.
    if sqlx::query("SELECT name FROM sqlite_master WHERE type='table' AND name='migrations'")
        .fetch_one(database)
        .await
        .is_err()
    {
        // Create migrations table.
        sqlx::query(
            "CREATE TABLE migrations (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            )",
        )
        .execute(database)
        .await
        .expect("Should create migrations table");
    }

    // Get the list of migrations.
    let mut migrations: Vec<String> = Migrations::iter()
        .map(|migration| migration.to_string())
        .collect();
    migrations.sort();

    // Get the list of migrations that have already been applied.
    let applied_migrations: Vec<String> = sqlx::query_as("SELECT name FROM migrations")
        .fetch_all(database)
        .await
        .expect("Should fetch applied migrations")
        .into_iter()
        .map(|migration: (String,)| migration.0)
        .collect();

    // Apply migrations that have not been applied.
    for migration in migrations {
        if !applied_migrations.contains(&migration) {
            // Read the migration file.
            let migration_file = Migrations::get(&migration)
                .expect("Should get migration file")
                .data
                .to_vec();

            // Run the migration.
            sqlx::query(
                &String::from_utf8(migration_file)
                    .expect("Should convert migration file to string"),
            )
            .execute(database)
            .await
            .expect("Should run migration");

            // Insert the migration into the migrations table.
            sqlx::query("INSERT INTO migrations (name) VALUES (?)")
                .bind(&migration)
                .execute(database)
                .await
                .expect("Should insert migration into migrations table");
        }
    }
}
