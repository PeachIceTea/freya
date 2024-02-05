use serde::Serialize;
use tokio::io;

// Audio file extensions (.mp3, .flac, .wav, .ogg, .m4a, .m4b, .opus)
pub const AUDIO_EXTENSIONS: [&str; 7] = ["mp3", "flac", "wav", "ogg", "m4a", "m4b", "opus"];

// image file extensions (.jpg, .jpeg, .png, .webp)
pub const IMAGE_EXTENSIONS: [&str; 4] = ["jpg", "jpeg", "png", "webp"];

// Categories of files we care about.
#[derive(PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum FileCategory {
    Directory,
    Audio,
    Image,
    File,
}

// Filesystem entry.
#[derive(Serialize)]
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
