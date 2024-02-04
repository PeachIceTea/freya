use std::{collections::HashMap, process::Command};

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

use super::{cover::RANDOM_FILE_NAME_LENGTH, random::random_string, storage::TMP_PATH};

pub fn is_ffmpeg_installed() -> Result<()> {
    let ffmpeg_output = Command::new("ffmpeg").arg("-version").output()?;

    let ffprobe_output = Command::new("ffprobe").arg("-version").output()?;

    match (
        ffmpeg_output.status.success(),
        ffprobe_output.status.success(),
    ) {
        (true, true) => Ok(()),
        (false, true) => bail!(
            "ffmpeg is not installed or not in PATH: {}",
            String::from_utf8_lossy(&ffmpeg_output.stderr)
        ),
        (true, false) => bail!(
            "ffprobe is not installed or not in PATH: {}",
            String::from_utf8_lossy(&ffprobe_output.stderr)
        ),
        (false, false) => bail!(
            "ffmpeg and ffprobe are not installed or not in PATH: {}",
            String::from_utf8_lossy(&ffmpeg_output.stderr)
        ),
    }
}

#[derive(Deserialize)]
pub struct FFProbeOutput {
    streams: Option<Vec<FFProbeStream>>,
    format: Option<FFProbeFormat>,
}

#[derive(Deserialize)]
pub struct FFProbeStream {
    codec_type: String,
    duration: String,
}

#[derive(Deserialize)]
pub struct FFProbeFormat {
    tags: Option<HashMap<String, String>>,
}

#[derive(Serialize)]
pub struct FileInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    author: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    cover: Option<String>,
}

pub async fn ffprobe_book_details(path: &str) -> Result<FileInfo> {
    // ffprobe -i ${filePath} -v quiet -print_format json -show_format
    let output = tokio::process::Command::new("ffprobe")
        .arg("-i")
        .arg(&path)
        .arg("-v")
        .arg("quiet")
        .arg("-print_format")
        .arg("json")
        .arg("-show_format")
        .output()
        .await?;

    if !output.status.success() {
        bail!("ffprobe failed: {:?}", output.stderr);
    }

    // Parse ffprobe output.
    let output_string =
        String::from_utf8(output.stdout).context("couldn't parse ffprobe output")?;

    let output: FFProbeOutput = serde_json::from_str(&output_string)?;
    let format = output
        .format
        .context("ffprobe output does not ciontain format")?;

    // Try to get title, author and cover from tags.
    let mut info = FileInfo {
        title: None,
        author: None,
        cover: None,
    };
    if !format.tags.is_none() {
        let tags = format.tags.unwrap();
        if tags.contains_key("album") {
            info.title = Some(tags.get("album").unwrap().to_string());
        } else if tags.contains_key("title") {
            info.title = Some(tags.get("title").unwrap().to_string());
        }

        if tags.contains_key("author") {
            info.author = Some(tags.get("author").unwrap().to_string());
        } else if tags.contains_key("artist") {
            info.author = Some(tags.get("artist").unwrap().to_string());
        }
    }

    // Extract cover image.
    let tmp_file_name = random_string(RANDOM_FILE_NAME_LENGTH);
    // ffmpeg -i ${filePath} -v quiet -an -vcodec copy data/tmp/${random}.jpg
    let output = tokio::process::Command::new("ffmpeg")
        .arg("-i")
        .arg(&path)
        .arg("-v")
        .arg("quiet")
        .arg("-an")
        .arg("-vcodec")
        .arg("copy")
        .arg(TMP_PATH.join(format!("{}.jpg", tmp_file_name)))
        .output()
        .await?;
    if output.status.success() {
        info.cover = Some(format!("extracted-file://{}.jpg", tmp_file_name));
    }

    Ok(info)
}

pub async fn ffprobe_duration(path: &str) -> Result<f64> {
    // ffprobe -i ${filePath} -v quiet -print_format json -show_streams
    let output = tokio::process::Command::new("ffprobe")
        .arg("-i")
        .arg(&path)
        .arg("-v")
        .arg("quiet")
        .arg("-print_format")
        .arg("json")
        .arg("-show_streams")
        .output()
        .await?;

    if !output.status.success() {
        bail!("ffprobe failed: {:?}", output.stderr);
    }

    let output_string =
        String::from_utf8(output.stdout).context("couldn't parse ffprobe output")?;
    let output: FFProbeOutput = serde_json::from_str(&output_string)?;
    let streams = output
        .streams
        .context("ffprobe output does not contain streams")?;

    streams
        .iter()
        .find(|stream| stream.codec_type == "audio")
        .map_or(Err(anyhow::Error::msg("No audio stream found")), |stream| {
            stream
                .duration
                .parse::<f64>()
                .context("Failed to parse duration")
        })
}
