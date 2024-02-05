use std::path::PathBuf;

use once_cell::sync::Lazy;

use super::random::random_string;

pub static TMP_PATH: Lazy<PathBuf> = Lazy::new(|| {
    // Create temporary directory.
    let random = random_string(12);
    let path = std::env::temp_dir().join(format!("freya-{}", random));
    std::fs::create_dir(&path).expect("Should be able to create temporary directory");
    path
});
