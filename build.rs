use tokio::process::Command;

#[tokio::main]
async fn main() {
    // Read .env file.
    dotenvy::dotenv().ok();

    // Get profile from environment.
    let profile = std::env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());

    // Allow cfg(profile = "dev") to be used in the main.rs file.
    println!("cargo:rustc-cfg=profile=\"{}\"", profile);

    // Migrate the database.
    let migration = migrate_database();

    // Build the frontend.
    let frontend = build_frontend(&profile);

    // Wait for the database to be migrated and the frontend to be built.
    tokio::join!(migration, frontend);
}

// Create a database file if none exists.
// sqlx uses the database file to infer types and check queries at compile time.
// To make it easier we create a database for sqlx to run against at build startup.
async fn migrate_database() {
    let database_path = std::env::var("DATABASE_PATH").unwrap_or_else(|_| "freya.db".to_string());

    // Check if database file exists.
    if !std::path::Path::new(&database_path).exists() {
        // Touch the database file.
        std::fs::File::create(&database_path).expect("Should create database file");
    }

    // Create the database pool.
    let pool: sqlx::Pool<sqlx::Sqlite> = sqlx::Pool::connect(&database_path)
        .await
        .expect("Should connect to database");

    match sqlx::migrate!().run(&pool).await {
        Ok(_) => {}
        Err(err) => panic!("Could not migrate database: {}", err),
    };
}

async fn build_frontend(profile: &str) {
    // Only build the frontend in release mode.
    if profile != "release" {
        return;
    }

    // Install npm dependencies.
    Command::new("npm")
        .arg("install")
        .current_dir("web")
        .status()
        .await
        .expect("Should be able to run npm install");

    // Build the frontend.
    let status = Command::new("npm")
        .arg("run")
        .arg("build")
        .current_dir("web")
        .status()
        .await
        .expect("Should be able to run npm build");

    // Ensure the build was successful.
    if !status.success() {
        panic!("npm build failed");
    }
}
