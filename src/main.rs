mod api;
mod models;
mod state;
mod util;

use tracing_subscriber::prelude::*;

#[tokio::main]
async fn main() {
    // Read .env file.
    dotenvy::dotenv().ok();

    // Initialize tracing.
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                "freya,tower_http=debug,axum::rejection=trace"
                    .parse()
                    .unwrap()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Build application.
    let state = state::FreyaState::new().await;
    let app = api::build_router(state).await;

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
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}
