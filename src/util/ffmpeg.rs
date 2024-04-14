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
#[serde(rename_all = "camelCase")]
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
        .arg(path)
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
    if let Some(tags) = format.tags {
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
        .arg(path)
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
        .arg(path)
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
#[derive(Deserialize)]
struct FFProbeChaptersOutput {
    chapters: Option<Vec<FFProbeChapter>>,
}

#[derive(Deserialize)]
struct FFProbeChapter {
    id: i64,
    start_time: String,
    end_time: String,
    tags: Option<HashMap<String, String>>,
}

pub struct Chapters {
    pub name: String,
    pub start: f64,
    pub end: f64,
}

pub async fn ffprobe_chapters(path: &str) -> Result<Vec<Chapters>> {
    // ffprobe -i ${filePath} -v quiet -print_format json -show_chapters
    let output = tokio::process::Command::new("ffprobe")
        .arg("-i")
        .arg(path)
        .arg("-v")
        .arg("quiet")
        .arg("-print_format")
        .arg("json")
        .arg("-show_chapters")
        .output()
        .await?;

    if !output.status.success() {
        //TODO: Actually print error. Calling -v quiet removes any error messages that could be
        //shown here.
        bail!("ffprobe failed: {:?}", output.stderr);
    }

    let output_string =
        String::from_utf8(output.stdout).context("couldn't parse ffprobe output")?;
    let output: FFProbeChaptersOutput = serde_json::from_str(&output_string)?;
    let chapters = output
        .chapters
        .context("ffprobe output does not contain chapters")?;

    chapters
        .iter()
        .map(|ff_chapter| {
            let start_time = ff_chapter
                .start_time
                .parse::<f64>()
                .context("Failed to parse start time")?;
            let end_time = ff_chapter
                .end_time
                .parse::<f64>()
                .context("Failed to parse end time")?;
            let name = ff_chapter
                .tags
                .as_ref()
                .and_then(|tags| tags.get("title"))
                .unwrap_or(&format!("Chapter {}", ff_chapter.id))
                .to_string();
            Ok(Chapters {
                name,
                start: start_time,
                end: end_time,
            })
        })
        .collect::<Result<Vec<Chapters>>>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    // The output from ffmpeg isn't perfectly the defined length. We set a tolerance to accept
    // tiny differences.
    // As an example the 5 second file created in test_ffprobe_duration reports a 5.041633 length.
    const TOLERANCE: f64 = 0.1;

    #[test]
    fn test_is_ffmpeg_installed() {
        // Test case: Verify that is_ffmpeg_installed returns Ok if ffmpeg and ffprobe are installed
        let result = is_ffmpeg_installed();
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_ffprobe_book_details() {
        // Test case: Verify that ffprobe_book_details extracts the correct book details
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir
            .path()
            .join("test.mp3")
            .to_string_lossy()
            .to_string();

        // Create a sample audio file with metadata tags
        // ffmpeg -f lavfi -i sine=frequency=1000:duration=5 -metadata album="Test Album" -metadata artist="Test Artist" ${file_path}
        let _ = tokio::process::Command::new("ffmpeg")
            .arg("-f")
            .arg("lavfi")
            .arg("-i")
            .arg("sine=frequency=1000:duration=5")
            .arg("-metadata")
            .arg("album=Test Album")
            .arg("-metadata")
            .arg("artist=Test Artist")
            .arg(&file_path)
            .output()
            .await
            .unwrap();

        let result = ffprobe_book_details(&file_path).await.unwrap();
        assert_eq!(result.title, Some("Test Album".to_string()));
        assert_eq!(result.author, Some("Test Artist".to_string()));
    }

    #[tokio::test]
    async fn test_ffprobe_duration() {
        // Test case: Verify that ffprobe_duration extracts the correct audio duration
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir
            .path()
            .join("test.mp3")
            .to_string_lossy()
            .to_string();

        // Create a sample audio file with a known duration
        // ffmpeg -f lavfi -i sine=frequency=1000:duration=5 ${file_path}
        let _ = tokio::process::Command::new("ffmpeg")
            .arg("-f")
            .arg("lavfi")
            .arg("-i")
            .arg("sine=frequency=1000:duration=5")
            .arg(&file_path)
            .output()
            .await
            .unwrap();

        let result = ffprobe_duration(&file_path).await.unwrap();
        assert!((result - 5.0).abs() < TOLERANCE);
    }

    #[tokio::test]
    async fn test_ffprobe_chapters() {
        // Test case: Verify that ffprobe_chapters extracts the correct chapter information
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir
            .path()
            .join("test.m4b")
            .to_string_lossy()
            .to_string();
        let chapters_path = temp_dir
            .path()
            .join("chapters.txt")
            .to_string_lossy()
            .to_string();

        // Create chapters file.
        std::fs::write(
            &chapters_path,
            r#";FFMETADATA1
[CHAPTER]
TIMEBASE=1/1000
START=0
END=2500
title=Start

[CHAPTER]
TIMEBASE=1/1000
START=2500
END=5000
title=End
"#,
        )
        .expect("Should be able to create chapters.txt");

        // Create a sample audio file with chapters
        // ffmpeg -i ${chapters_path} -f lavfi -i sine=frequency=1000:duration=5 -acodec aac -map_metadata 1 ${file_path}
        let _ = tokio::process::Command::new("ffmpeg")
            .arg("-i")
            .arg(&chapters_path)
            .arg("-f")
            .arg("lavfi")
            .arg("-i")
            .arg("sine=frequency=1000:duration=5")
            .arg("-acodec")
            .arg("aac")
            .arg(&file_path)
            .output()
            .await
            .unwrap();

        let result = ffprobe_chapters(&file_path).await.unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].name, "Start".to_string());
        assert_eq!(result[0].start, 0.0);
        assert_eq!(result[0].end, 2.5);
        assert_eq!(result[1].name, "End".to_string());
        assert_eq!(result[1].start, 2.5);
        assert_eq!(result[1].end, 5.0);
    }
}
