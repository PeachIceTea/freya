use freya_migrate::migrate;
use tokio::process::Command;

#[tokio::main]
async fn main() {
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
async fn migrate_database() {
    // sqlx uses the database file to infer types and check queries at compile time.
    // Migrate the database.
    let pool = sqlx::Pool::connect("freya.db")
        .await
        .expect("Should connect to database");
    migrate(&pool).await
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
