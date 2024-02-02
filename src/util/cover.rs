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

            // Check if string starts with "file://", "extracted-file://" or "http://", "https://".
            if string.starts_with("file://") || string.starts_with("extracted-file://") {
                read_image(string)
            } else if string.starts_with("http://") || string.starts_with("https://") {
                download_image(string).await
            } else {
                bail!(
                    "Invalid cover image string.\nExpected a path or URL, instead got: {}",
                    string
                )
            }
        }
    }
}

async fn download_image(url: &str) -> Result<Vec<u8>> {
    // Send a GET request to the URL.
    let response = reqwest::get(url)
        .await
        .with_context(|| format!("Failed to send GET request to URL: {}", url))?;

    // Check if the response is successful.
    if !response.status().is_success() {
        bail!(
            "Failed to get image from URL: {}\nStatus code: {}",
            url,
            response.status()
        )
    }

    // Check if the response contains an image.
    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .context("Failed to get content type from response")?
        .to_str()
        .context("Failed to convert content type to string")?
        .to_string();
    if content_type.starts_with("image/") {
        bail!(
            "Response does not contain an image: {}\nContent type: {}",
            url,
            content_type
        )
    }

    // Read the response body.
    response
        .bytes()
        .await
        .with_context(|| format!("Failed to read response body from URL: {}", url))
        .map(|bytes| bytes.to_vec())
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
