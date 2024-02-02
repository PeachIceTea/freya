use std::{path::PathBuf, time::Duration};

use anyhow::Result;
use once_cell::sync::Lazy;

// Base path for data storage.
// Read DATA_PATH from environment variable.
// If it is not set, use directory named "data" in the current directory.
pub static DATA_PATH: Lazy<PathBuf> = Lazy::new(|| {
    std::env::var("DATA_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("data"))
        .canonicalize()
        .expect("DATA_PATH should be a valid path")
});

// Static paths for subdirectories.
// The structure of the data directory is as follows:
// data
// ├── audio
// ├── covers
// ├── tmp
pub static AUDIO_PATH: Lazy<PathBuf> = Lazy::new(|| DATA_PATH.join("audio"));
pub static COVERS_PATH: Lazy<PathBuf> = Lazy::new(|| DATA_PATH.join("covers"));
pub static TMP_PATH: Lazy<PathBuf> = Lazy::new(|| DATA_PATH.join("tmp"));

// Create data directory with some known subdirectories.
pub fn create_data_directory() -> Result<()> {
    tracing::info!(
        "Creating data directory if it does not exist: {}",
        DATA_PATH.display()
    );

    // Create data directory.
    std::fs::create_dir_all(&*DATA_PATH)?;

    // Create subdirectories.
    let subdirectories = ["audio", "covers", "tmp"];
    for subdirectory in &subdirectories {
        std::fs::create_dir_all(DATA_PATH.join(subdirectory))?;
    }

    Ok(())
}

pub fn spawn_tmp_cleaning_task() {
    tokio::spawn(async move {
        loop {
            // Get 1 hour interval.
            let mut interval = tokio::time::interval(Duration::from_secs(60 * 60));

            loop {
                // Wait for tick.
                // First tick happens immediately.
                interval.tick().await;

                // Clean tmp directory.
                if let Err(e) = clean_tmp_directory() {
                    tracing::error!("Failed to clean tmp directory: {}", e);
                }
            }
        }
    });
}

// Delete all files older than 8 hours in the tmp directory.
fn clean_tmp_directory() -> Result<()> {
    tracing::info!("Cleaning tmp directory");

    let count = std::fs::read_dir(DATA_PATH.join("tmp"))?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let metadata = entry.metadata().ok()?;
            let modified = metadata.modified().ok()?;
            let duration = modified.elapsed().ok()?;
            if duration > Duration::from_secs(60 * 60 * 8) {
                Some(entry)
            } else {
                None
            }
        })
        .fold(0, |acc, entry| {
            if let Err(e) = std::fs::remove_file(entry.path()) {
                tracing::error!("Failed to clean tmp file: {}", e);
                acc
            } else {
                acc + 1
            }
        });

    tracing::info!("Cleaned {} tmp files", count);

    Ok(())
}
