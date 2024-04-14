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
        let path = std::env::temp_dir().join(format!("freya-{}", random));

        std::fs::create_dir(&path).expect("Should be able to create temporary directory");
        assert_ne!(path, *TMP_PATH);

        // Delete created directory.
        fs::remove_dir_all(path).unwrap();
    }
}
