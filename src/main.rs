mod handlers;
mod config;
mod utils;
mod models;

use std::sync::Arc;

use axum::{body::Body, http::Response, routing::{delete, get}, serve, Extension, Router};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {

    let frames = Arc::new(utils::load_frames());

    let router = Router::new()
        .nest(
            "/render",
            Router::new()
                .route("/card/{hash}", get(handlers::render_card))
                .route("/fan/{hash}", get(handlers::render_fan))
                .route("/{file_name}", delete(handlers::render_remove)),
        )
        .fallback(|| async { Response::builder().status(418).body(Body::empty()).unwrap() })
        .layer(Extension(frames));

    let listener = TcpListener::bind(config::ADDRESS).await.unwrap();

    serve(listener, router).await
}

