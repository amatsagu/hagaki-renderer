mod config;
mod handlers;
mod models;
mod utils;

use axum::{body::Body, http::Response, routing::get, serve, Extension, Router};
use log::{error, info};
use pretty_env_logger::init as init_logger;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::signal;

use crate::handlers::render::{
    handle_card_album_request, handle_card_fan_request, handle_card_request,
};

#[tokio::main]
async fn main() {
    init_logger();
    info!("Starting service...");

    let frames = Arc::new(utils::load_frames());

    let router = Router::new()
        .nest(
            "/render",
            Router::new()
                .route("/card/{hash}", get(handle_card_request))
                .route("/fan/{hash}", get(handle_card_fan_request))
                .route("/album/{hash}", get(handle_card_album_request)), //.route("/{file_name}", delete(handlers::render_remove))
        )
        .fallback(|| async { Response::builder().status(418).body(Body::empty()).unwrap() })
        .layer(Extension(frames));

    match TcpListener::bind(config::ADDRESS).await {
        Ok(listener) => {
            info!(
                "Starting http server at {}. Server is ready.",
                config::ADDRESS
            );
            if let Err(e) = serve(listener, router).with_graceful_shutdown(shutdown_signal()).await {
                error!("Server encountered an error: {}", e);
                std::process::exit(1);
            }
        }
        Err(err) => {
            error!("Failed to bind to {} tcp socket: {}", config::ADDRESS, err);
            std::process::exit(1);
        }
    }

    info!("Successfully closed http server. Bye!")
}

// https://github.com/tokio-rs/axum/blob/main/examples/graceful-shutdown/src/main.rs
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
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("Received signal to shutdown. Starting graceful shutdown!");
}
