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

    #[tokio::test]
    async fn test_get_file_system_list() {
        //// Setup
        // Create a temporary directory for testing
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_dir_path = temp_dir.path().to_str().unwrap();

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

        //// Execution
        // Call the function with the path of the temporary directory
        let result = get_file_system_list(temp_dir_path).await.unwrap();

        //// Verification

        // Should only return 4 entries
        assert_eq!(result.len(), 4);

        // The first entry should be the directory
        assert_eq!(result[0].name, "test_dir");
        assert_eq!(result[0].category, FileCategory::Directory);
        assert_eq!(result[0].path, format!("{}/test_dir", temp_dir_path));

        // The second entry should be 1.txt based on the alphabetical order
        assert_eq!(result[1].name, "1.txt");
        assert_eq!(result[1].category, FileCategory::File);
        assert_eq!(result[1].path, misc_file_path.to_str().unwrap());

        // The third entry should be 2.png based on the alphabetical order
        assert_eq!(result[2].name, "2.png");
        assert_eq!(result[2].category, FileCategory::Image);
        assert_eq!(result[2].path, image_file_path.to_str().unwrap());

        // The fourth entry should be 3.mp3 based on the alphabetical order
        assert_eq!(result[3].name, "3.mp3");
        assert_eq!(result[3].category, FileCategory::Audio);
        assert_eq!(result[3].path, file_path.to_str().unwrap());
    }
}
