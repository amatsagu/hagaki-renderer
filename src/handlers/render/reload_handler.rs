use axum::{body::Body, response::Response, Extension, extract::Query};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use image::DynamicImage;
use log::info;

use crate::utils::load_frames;

pub async fn handle_reload_request(
    Query(params): Query<HashMap<String, String>>,
    Extension(frames): Extension<Arc<RwLock<HashMap<u32, DynamicImage>>>>
) -> Response<Body> {
    if params.get("force").map(|v| v == "true").unwrap_or(false) {
        info!("Force reloading frames...");
        let mut frames_guard = frames.write().await;
        *frames_guard = load_frames();
        drop(frames_guard);
        info!("Frames reloaded successfully.");
        
        Response::builder()
            .status(200)
            .body(Body::from("Frames reloaded successfully"))
            .unwrap()
    } else {
        Response::builder()
            .status(400)
            .body(Body::from("Missing force=true query parameter"))
            .unwrap()
    }
}
