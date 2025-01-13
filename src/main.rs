mod handlers;
mod config;
mod utils;
mod models;

use std::sync::Arc;

use axum::{body::Body, http::Response, routing::get, serve, Extension, Router};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {

    let frames = Arc::new(utils::load_frames());

    let router = Router::new()
        .route("/render/card/{hash}", get(handlers::render_card))
        .route("/render/fan/{hash}", get(handlers::render_fan))
        .fallback(|| async { Response::builder().status(418).body(Body::empty()).unwrap() })
        .layer(Extension(frames));

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();

    serve(listener, router).await
}

