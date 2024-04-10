#[derive(rust_embed::RustEmbed)]
#[folder = "sql"]
struct Migrations;

pub async fn migrate(db: &sqlx::Pool<sqlx::Sqlite>) {
    tracing::info!("Migrating database");

    // Check if migrations table exists.
    if sqlx::query("SELECT name FROM sqlite_master WHERE type='table' AND name='migrations'")
        .fetch_one(db)
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
        .execute(db)
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
        .fetch_all(db)
        .await
        .expect("Should fetch applied migrations")
        .into_iter()
        .map(|migration: (String,)| migration.0)
        .collect();

    // Apply migrations that have not been applied.
    for migration in migrations {
        tracing::info!("Checking migration: {}", &migration);
        if !applied_migrations.contains(&migration) {
            tracing::info!("Applying migration: {}", &migration);

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
            .execute(db)
            .await
            .expect("Should run migration");

            // Insert the migration into the migrations table.
            sqlx::query("INSERT INTO migrations (name) VALUES (?)")
                .bind(&migration)
                .execute(db)
                .await
                .expect("Should insert migration into migrations table");
        }
    }
}
