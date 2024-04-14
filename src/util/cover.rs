use anyhow::{bail, Context, Result};
use axum::body::Bytes;
use axum_typed_multipart::FieldData;

use super::storage::TMP_PATH;

pub static RANDOM_FILE_NAME_LENGTH: usize = 12;

pub async fn get_cover_bytes(data: FieldData<Bytes>) -> Result<Vec<u8>> {
    // Check if data is an image or string.
    match &data.metadata.content_type {
        Some(_) => {
            // Check if data contains an image.
            let content_type = data
                .metadata
                .content_type
                .as_ref()
                .context("Failed to get content type from data")?;
            if !content_type.starts_with("image/") {
                bail!(
                    "Data does not contain an image.\nContent type: {}",
                    content_type
                )
            }

            Ok(data.contents.to_vec())
        }
        None => {
            // Try turning data into a string.
            let string =
                std::str::from_utf8(&data.contents).context("Failed to convert data to string")?;
            read_image(string)
        }
    }
}

fn read_image(path: &str) -> Result<Vec<u8>> {
    // Extract scheme from path.
    let scheme = path
        .split("://")
        .next()
        .context("Failed to extract scheme from path")?;

    // Get absolute path for file.
    let path = match scheme {
        "file" => path[7..].to_string(),
        "extracted-file" => TMP_PATH.join(&path[17..]).to_string_lossy().to_string(),
        _ => bail!("Invalid scheme in path: {}", scheme),
    };

    // Read the file.
    std::fs::read(&path).with_context(|| format!("Failed to read image file: {}", path))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Bytes;
    use axum_typed_multipart::FieldData;
    use std::fs;
    use tempfile::NamedTempFile;

    const TEST_IMAGE_PATH: &str = "placeholder-cover.jpg";

    #[tokio::test]
    async fn test_get_cover_bytes_with_image() {
        // Test case: FieldData contains a valid image
        let image_data = fs::read(TEST_IMAGE_PATH).unwrap();
        let field_data = FieldData {
            contents: Bytes::from(image_data.clone()),
            metadata: axum_typed_multipart::FieldMetadata {
                content_type: Some("image/jpeg".to_string()),
                ..Default::default()
            },
        };

        let result = get_cover_bytes(field_data).await.unwrap();
        assert_eq!(result, image_data);
    }

    #[tokio::test]
    async fn test_get_cover_bytes_with_non_image() {
        // Test case: FieldData contains non-image data
        let field_data = FieldData {
            contents: Bytes::from("not an image"),
            metadata: axum_typed_multipart::FieldMetadata {
                content_type: Some("text/plain".to_string()),
                ..Default::default()
            },
        };

        let result = get_cover_bytes(field_data).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_cover_bytes_with_file_path() {
        // Test case: FieldData contains a file path starting with "file://"
        let temp_file = NamedTempFile::new().unwrap();
        let file_path = temp_file.path().to_str().unwrap();
        let image_data = fs::read(TEST_IMAGE_PATH).unwrap();
        fs::write(file_path, &image_data).unwrap();

        let field_data = FieldData {
            contents: Bytes::from(format!("file://{}", file_path)),
            metadata: axum_typed_multipart::FieldMetadata::default(),
        };

        let result = get_cover_bytes(field_data).await.unwrap();
        assert_eq!(result, image_data);
    }

    #[tokio::test]
    async fn test_get_cover_bytes_with_extracted_file_path() {
        // Test case: FieldData contains a file path starting with "extracted-file://"
        let temp_file = NamedTempFile::new_in(&*TMP_PATH).unwrap();
        let file_path = temp_file.path().to_str().unwrap();
        let image_data = fs::read(TEST_IMAGE_PATH).unwrap();
        fs::write(file_path, &image_data).unwrap();

        let field_data = FieldData {
            contents: Bytes::from(format!("extracted-file://{}", file_path)),
            metadata: axum_typed_multipart::FieldMetadata::default(),
        };

        let result = get_cover_bytes(field_data).await.unwrap();
        assert_eq!(result, image_data);
    }

    #[tokio::test]
    async fn test_get_cover_bytes_with_invalid_path() {
        // Test case: FieldData contains an invalid path
        let field_data = FieldData {
            contents: Bytes::from("invalid://path"),
            metadata: axum_typed_multipart::FieldMetadata::default(),
        };

        let result = get_cover_bytes(field_data).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_cover_bytes_with_non_existing_file() {
        // Test case: FieldData contains a path to a non-existing file
        let non_existing_file = "path/to/non_existing_file.jpg";
        let field_data = FieldData {
            contents: Bytes::from(format!("file://{}", non_existing_file)),
            metadata: axum_typed_multipart::FieldMetadata::default(),
        };

        let result = get_cover_bytes(field_data).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Failed to read image file"));
    }
}
