use std::{path::Path, str::FromStr};

use axum::{
    body::Body,
    http::{header, HeaderMap, HeaderValue, Response, StatusCode},
    response::IntoResponse,
};
use once_cell::sync::Lazy;
use regex::Regex;
use tokio::{fs::File, io::AsyncSeekExt};
use tokio_util::io::ReaderStream;

const RANGE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^bytes=(\d+)-(\d+)?$").unwrap());

pub struct RangeHeader {
    pub start: u64,
    pub end: Option<u64>,
}

// parse range header
impl FromStr for RangeHeader {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let captures = RANGE_REGEX
            .captures(s)
            .ok_or_else(|| anyhow::anyhow!("invalid range header"))?;

        let start = captures
            .get(1)
            .ok_or_else(|| anyhow::anyhow!("invalid range header"))?
            .as_str()
            .parse::<u64>()?;

        let end = captures
            .get(2)
            .map(|m| m.as_str().parse::<u64>().ok())
            .flatten();

        Ok(Self { start, end })
    }
}

pub async fn send_file(path: &str, header: Option<&HeaderMap>) -> impl IntoResponse {
    // check if file exists
    let file_path = Path::new(path);
    if !file_path.exists() {
        return (StatusCode::NOT_FOUND).into_response();
    }

    // open file
    let mut file = match File::open(path).await {
        Ok(file) => file,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
    };

    // get range header
    let range = header.and_then(|h| {
        h.get("range")
            .map(|r| r.to_str().unwrap().parse::<RangeHeader>().ok())
            .flatten()
    });

    // seek to start of range if range header is present
    if let Some(range) = &range {
        if let Err(_) = file.seek(std::io::SeekFrom::Start(range.start)).await {
            return (StatusCode::INTERNAL_SERVER_ERROR).into_response();
        }
    }

    // get file size
    let file_size = match file.metadata().await {
        Ok(metadata) => metadata.len(),
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
    };
    // create stream body
    let body = Body::from_stream(ReaderStream::new(file));

    // create response
    let res =
        Response::builder().header(header::CONTENT_TYPE, HeaderValue::from_static("audio/mpeg"));

    let res = match range {
        Some(range) => {
            let end = range.end.unwrap_or(file_size - 1);
            let content_range = format!("bytes {}-{}/{}", range.start, end, file_size);
            res.header(header::CONTENT_RANGE, content_range)
                .header(header::CONTENT_LENGTH, end - range.start + 1)
                .status(StatusCode::PARTIAL_CONTENT)
        }
        None => res
            .header(header::CONTENT_LENGTH, file_size)
            .status(StatusCode::OK),
    };

    res.body(body).unwrap().into_response()
}
