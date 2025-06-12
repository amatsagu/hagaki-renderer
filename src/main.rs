mod handlers;
mod config;
mod models;
mod utils;

use std::sync::Arc;
use axum::{body::Body, http::Response, routing::{get}, serve, Extension, Router};
use tokio::net::TcpListener;
use log::{info, error};
use pretty_env_logger::init as init_logger;

use crate::handlers::render::{handle_card_album_request, handle_card_request, handle_card_fan_request};

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
                .route("/album/{hash}", get(handle_card_album_request))
                //.route("/{file_name}", delete(handlers::render_remove))
        )
        .fallback(|| async { Response::builder().status(418).body(Body::empty()).unwrap() })
        .layer(Extension(frames));

    match TcpListener::bind(config::ADDRESS).await {
        Ok(listener) => {
            info!("Starting http server at {}. Server is ready.", config::ADDRESS);
            if let Err(e) = serve(listener, router).await {
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

