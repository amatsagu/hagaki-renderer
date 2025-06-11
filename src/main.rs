mod handlers;
mod config;
mod models;
mod utils;

use std::sync::Arc;

use axum::{body::Body, http::Response, routing::{delete, get}, serve, Extension, Router};
use tokio::net::TcpListener;
use crate::handlers::render::{handle_card_album_request, handle_card_request, handle_card_fan_request};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {

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

    let listener = TcpListener::bind(config::ADDRESS).await.unwrap();

    serve(listener, router).await
}

