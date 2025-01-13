use axum::{body::Body, extract::Path, response::Response, Extension};
use base64::{engine::general_purpose::STANDARD_NO_PAD as Engine, Engine as _};
use image::DynamicImage;
use std::collections::HashMap;
use std::io::{BufWriter, Cursor};
use std::sync::Arc;
use std::time::Instant;

use crate::models::RenderRequestData;
use crate::utils;

#[axum_macros::debug_handler]
pub async fn render_fan(
    Path(hash): Path<String>, 
    Extension(frames): Extension<Arc<HashMap<String, DynamicImage>>>
) -> Response<Body> {
    let start = Instant::now();

    let bytes = match Engine.decode(hash) {
        Ok(bytes) => bytes,
        Err(_) => return Response::builder().status(400).body(Body::from("Invalid card hash")).unwrap(),
    };
    let decoded: Vec<RenderRequestData> = match serde_json::from_slice(&bytes) {
        Ok(decoded) => decoded,
        Err(_) => return Response::builder().status(400).body(Body::from("Hash contains invalid data")).unwrap(),
    };

    let image = match utils::render_fan(decoded, &frames) {
        Ok(image) => image,
        Err(e) => return Response::builder().status(500).body(Body::from(e)).unwrap(),
    };

    let mut buffer = BufWriter::new(Cursor::new(Vec::new()));
    match image.write_to(&mut buffer, image::ImageFormat::Png) {
        Ok(_) =>(),
        Err(_) => return Response::builder().status(500).body(Body::from("Cannot write image to buffer")).unwrap(),
    };
    
    Response::builder()
        .header("X-Processing-Time", (start.elapsed().as_nanos() as f64 / 1_000_000.0).to_string())
        .header("Content-Type", "image/png")
        .body(Body::from(buffer.into_inner().unwrap().into_inner()))
        .unwrap()
}