use serde::Serialize;
use tokio::io;

// Audio file extensions (.mp3, .flac, .wav, .ogg, .m4a, .m4b, .opus)
pub const AUDIO_EXTENSIONS: [&str; 7] = ["mp3", "flac", "wav", "ogg", "m4a", "m4b", "opus"];

// image file extensions (.jpg, .jpeg, .png, .webp)
pub const IMAGE_EXTENSIONS: [&str; 4] = ["jpg", "jpeg", "png", "webp"];

// Categories of files we care about.
#[derive(PartialEq, Serialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum FileCategory {
    Directory,
    Audio,
    Image,
    File,
}

// Filesystem entry.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    name: String,
    path: String,
    category: FileCategory,
}

pub async fn get_file_system_list(path: &str) -> Result<Vec<Entry>, io::Error> {
    use FileCategory::*;

    let mut entries = Vec::new();

    // Read directory entries.
    let mut reader = tokio::fs::read_dir(path).await?;
    while let Some(entry) = reader.next_entry().await? {
        // Get entry name.
        let name = match entry.file_name().into_string() {
            Ok(name) => name,
            Err(_) => continue,
        };

        // Skip hidden entries.
        if name.starts_with('.') {
            continue;
        }

        // Get full path.
        let path = match entry.path().into_os_string().into_string() {
            Ok(path) => path,
            Err(_) => continue,
        };

        // Get entry category.
        let category = {
            let path = entry.path();
            if path.is_dir() {
                Directory
            } else if path.is_file() {
                let extension = path
                    .extension()
                    .unwrap_or_default()
                    .to_str()
                    .unwrap_or_default();
                if AUDIO_EXTENSIONS.contains(&extension) {
                    Audio
                } else if IMAGE_EXTENSIONS.contains(&extension) {
                    Image
                } else {
                    File
                }
            } else {
                File
            }
        };

        entries.push(Entry {
            name,
            category,
            path,
        });
    }

    // Sort entries, directories first then alphabetically
    entries.sort_by(|a, b| {
        if a.category == Directory && b.category != Directory {
            std::cmp::Ordering::Less
        } else if a.category != Directory && b.category == Directory {
            std::cmp::Ordering::Greater
        } else {
            a.name.cmp(&b.name)
        }
    });

    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use tokio::fs;

    async fn create_test_files(temp_dir_path: &str) {
        // Create an audio file in the temporary directory
        let file_path = Path::new(temp_dir_path).join("3.mp3");
        fs::write(&file_path, b"").await.unwrap();

        // Create an image file in the temporary directory
        let image_file_path = Path::new(temp_dir_path).join("2.png");
        fs::write(&image_file_path, b"").await.unwrap();

        // Create a directory in the temporary directory
        let dir_path = Path::new(temp_dir_path).join("test_dir");
        fs::create_dir(&dir_path).await.unwrap();

        // Create a miscellaneous file in the temporary directory
        let misc_file_path = Path::new(temp_dir_path).join("1.txt");
        fs::write(&misc_file_path, b"").await.unwrap();

        // Create a hidden file in the temporary directory
        let hidden_file_path = Path::new(temp_dir_path).join(".hidden");
        fs::write(&hidden_file_path, b"").await.unwrap();
    }

    #[tokio::test]
    async fn test_entry_count() {
        // Test case: Verify that get_file_system_list returns the correct number of entries
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_dir_path = temp_dir.path().to_str().unwrap();
        create_test_files(temp_dir_path).await;

        let result = get_file_system_list(temp_dir_path).await.unwrap();

        assert_eq!(result.len(), 4);
    }

    #[tokio::test]
    async fn test_alphabetical_order() {
        // Test case: Verify that get_file_system_list returns entries sorted alphabetically
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_dir_path = temp_dir.path().to_str().unwrap();
        create_test_files(temp_dir_path).await;

        let result = get_file_system_list(temp_dir_path).await.unwrap();

        assert_eq!(result[0].name, "test_dir");
        assert_eq!(result[1].name, "1.txt");
        assert_eq!(result[2].name, "2.png");
        assert_eq!(result[3].name, "3.mp3");
    }

    #[tokio::test]
    async fn test_entry_categories() {
        // Test case: Verify that get_file_system_list returns entries with correct categories
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_dir_path = temp_dir.path().to_str().unwrap();
        create_test_files(temp_dir_path).await;

        let result = get_file_system_list(temp_dir_path).await.unwrap();

        assert_eq!(result[0].category, FileCategory::Directory);
        assert_eq!(result[1].category, FileCategory::File);
        assert_eq!(result[2].category, FileCategory::Image);
        assert_eq!(result[3].category, FileCategory::Audio);
    }

    #[tokio::test]
    async fn test_entry_paths() {
        // Test case: Verify that get_file_system_list returns entries with correct paths
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_dir_path = temp_dir.path().to_str().unwrap();
        create_test_files(temp_dir_path).await;

        let result = get_file_system_list(temp_dir_path).await.unwrap();

        assert_eq!(result[0].path, format!("{}/test_dir", temp_dir_path));
        assert_eq!(result[1].path, format!("{}/1.txt", temp_dir_path));
        assert_eq!(result[2].path, format!("{}/2.png", temp_dir_path));
        assert_eq!(result[3].path, format!("{}/3.mp3", temp_dir_path));
    }

    #[tokio::test]
    async fn test_hidden_files_excluded() {
        // Test case: Verify that get_file_system_list excludes hidden files
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_dir_path = temp_dir.path().to_str().unwrap();
        create_test_files(temp_dir_path).await;

        let result = get_file_system_list(temp_dir_path).await.unwrap();

        assert!(!result.iter().any(|entry| entry.name.starts_with('.')));
    }

    #[tokio::test]
    async fn test_invalid_unicode_filename() {
        // Test case: Verify that get_file_system_list handles files with invalid Unicode names
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_dir_path = temp_dir.path().to_str().unwrap();

        // Create a file with an invalid Unicode name
        let invalid_file_name = b"invalid_\xfe_filename.txt";
        let invalid_file_path = unsafe {
            Path::new(temp_dir_path).join(std::ffi::OsStr::from_encoded_bytes_unchecked(
                invalid_file_name,
            ))
        };
        fs::write(&invalid_file_path, b"").await.unwrap();

        let result = get_file_system_list(temp_dir_path).await.unwrap();

        // Verify that the file with the invalid Unicode name is not included in the result
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_invalid_unicode_directory_name() {
        // Test case: Verify that get_file_system_list handles files in directories with invalid Unicode names
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_dir_path = temp_dir.path().to_str().unwrap();

        // Create a directory with an invalid Unicode name
        let invalid_dir_name = b"invalid_\xfe_directory";
        let invalid_dir_path = unsafe {
            Path::new(temp_dir_path).join(std::ffi::OsStr::from_encoded_bytes_unchecked(
                invalid_dir_name,
            ))
        };
        fs::create_dir(&invalid_dir_path).await.unwrap();

        // Create a file with a valid Unicode name inside the directory with an invalid Unicode name
        let valid_file_name = "valid_file.txt";
        let valid_file_path = invalid_dir_path.join(valid_file_name);
        fs::write(&valid_file_path, b"").await.unwrap();

        let result = get_file_system_list(temp_dir_path).await.unwrap();

        // Verify that the file inside the directory with the invalid Unicode name is not included in the result
        assert!(result.is_empty());
    }
}
