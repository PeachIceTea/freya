use axum::{
    http::{header, Uri},
    response::IntoResponse,
};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "web/dist"]
struct WebAssets;

pub async fn serve_frontend(uri: Uri) -> impl IntoResponse {
    // Get the path from the URI.
    let path = uri.path().trim_start_matches('/');

    // Get file from the web assets.
    let file = match WebAssets::get(path) {
        Some(file) => file,
        None => WebAssets::get("index.html").expect("Should be able to get index.html"),
    };

    // Serve the file.
    (
        [(header::CONTENT_TYPE, file.metadata.mimetype())],
        file.data,
    )
        .into_response()
}
