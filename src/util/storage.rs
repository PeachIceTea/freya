use std::path::PathBuf;
use std::sync::LazyLock;

use super::random::random_string;

/// The root directory for all user-accessible media files.
/// Controlled by the `DEFAULT_DIRECTORY` environment variable (default: `/`).
/// All file access from the API is bounded to this directory.
pub static FREYA_MEDIA_ROOT: LazyLock<PathBuf> =
    LazyLock::new(|| std::env::var("DEFAULT_DIRECTORY").map_or_else(|_| PathBuf::from("/"), PathBuf::from));

pub static TMP_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    // Create temporary directory.
    let random = random_string(12);
    let path = std::env::temp_dir().join(format!("freya-{random}"));
    std::fs::create_dir(&path).expect("Should be able to create temporary directory");
    path
});

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_tmp_path_creation() {
        // Ensure the temporary directory is created
        assert!(TMP_PATH.exists());
        assert!(TMP_PATH.is_dir());
    }

    #[test]
    fn test_tmp_path_name() {
        // Check if the temporary directory name starts with "freya-"
        let dir_name = TMP_PATH.file_name().unwrap().to_str().unwrap();
        assert!(dir_name.starts_with("freya-"));
    }

    #[test]
    fn test_tmp_path_uniqueness() {
        // Create another temporary directory and ensure it's different from TMP_PATH
        let random = random_string(12);
        let path = std::env::temp_dir().join(format!("freya-{random}"));

        std::fs::create_dir(&path).expect("Should be able to create temporary directory");
        assert_ne!(path, *TMP_PATH);

        // Delete created directory.
        fs::remove_dir_all(path).unwrap();
    }
}
