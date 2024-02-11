mod api;
mod models;
#[cfg(profile = "release")]
mod serve;
mod state;
mod util;

use axum::Router;
use tokio::signal;
#[cfg(profile = "release")]
use tower_http::compression::CompressionLayer;
use tracing_subscriber::prelude::*;

use crate::util::storage::TMP_PATH;

#[tokio::main]
async fn main() {
    // Read .env file.
    dotenvy::dotenv().ok();

    // Initialize tracing.
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                "freya,migrate,tower_http=debug,axum::rejection=trace"
                    .parse()
                    .unwrap()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Check if ffmpeg and ffprobe are installed.
    util::ffmpeg::is_ffmpeg_installed().expect("Should be able to access ffmpeg and ffprobe");

    // Build application.
    let state: state::FreyaState = state::FreyaState::new().await;

    // Include the frontend in the release profile.
    #[cfg(profile = "release")]
    let app = Router::new()
        .nest("/api", api::build_router(state).await)
        .fallback(serve::serve_frontend)
        .layer(CompressionLayer::new());

    // Build only the API in the debug profile.
    #[cfg(profile = "debug")]
    let app = Router::new().nest("/api", api::build_router(state).await);

    // Get the port from the environment.
    // Default to 3000 if PORT is not set.
    let port: String = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse()
        .expect("PORT should be a number");

    // Start the server.
    tracing::info!("Starting server at http://localhost:{}", port);
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .expect("Should be able to bind to port");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {shutdown()},
        _ = terminate => {shutdown()},
    }
}

fn shutdown() {
    tracing::info!("Received shutdown signal. Shutting down...");

    // Remove temporary directory.
    // According to https://github.com/rust-lang/rust/issues/29497 this might fail on Windows.
    if let Err(err) = std::fs::remove_dir_all(&*TMP_PATH) {
        tracing::error!("Failed to remove temporary directory: {}", err);
    }
}
