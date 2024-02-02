use std::io;

use anyhow::{bail, Context, Result};
use axum::body::Bytes;
use axum_typed_multipart::FieldData;

use super::list_fs::IMAGE_EXTENSIONS;

pub struct Cover {
    path: String,
}

impl Cover {
    pub async fn new(data: &FieldData<Bytes>) -> Result<Self> {
        // Check if data is an image or string.
        match &data.metadata.content_type {
            Some(_) => Self::new_from_bytes(data),
            None => Self::new_from_string(data).await,
        }
    }

    async fn new_from_string(data: &FieldData<Bytes>) -> Result<Self> {
        // Try turning data into a string.
        let string =
            std::str::from_utf8(&data.contents).context("Failed to convert data to string")?;

        // Check if string starts with "file://" or "http://" or "https://".
        if string.starts_with("file://") {
            Self::new_from_path(&string[7..])
        } else if string.starts_with("http://") || string.starts_with("https://") {
            Self::new_image_from_url(string).await
        } else {
            bail!(
                "Invalid cover image string.\nExpected a path or URL, instead got: {}",
                string
            )
        }
    }

    fn new_from_path(path: &str) -> Result<Self> {
        // Check if path contains image file.
        let extension = std::path::Path::new(path)
            .extension()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default();
        if !IMAGE_EXTENSIONS.contains(&extension) {
            bail!("Expected path for an image file, instead got: {}", path)
        }

        // Check if the path exists.
        if !std::path::Path::new(path).exists() {
            bail!("Path for cover image does not exist: {}", path)
        }

        Ok(Self {
            path: path.to_string(),
        })
    }

    async fn new_image_from_url(url: &str) -> Result<Self> {
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
            .context("Failed to convert content type to string")?;
        if content_type.starts_with("image/") {
            bail!(
                "Response does not contain an image: {}\nContent type: {}",
                url,
                content_type
            )
        }

        // Read the response body.
        let bytes = response
            .bytes()
            .await
            .with_context(|| format!("Failed to read response body from URL: {}", url))?;
    }

    fn new_from_bytes(data: &FieldData<Bytes>) -> Result<Self> {
        todo!()
    }
}
