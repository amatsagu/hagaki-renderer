use axum::{body::Body, extract::Path, response::Response, Extension};
use base64::{engine::general_purpose::STANDARD_NO_PAD as Engine, Engine as _};
use image::DynamicImage;
use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};
use std::collections::HashMap;
use std::io::{BufWriter, Cursor};
use std::sync::Arc;
use std::time::Instant;
use log::{warn, error};

use crate::config::{CDN_RENDERS_PATH, RENDER_TIMEOUT};
use crate::models::FanRenderRequestData;
use crate::utils::render_fan;

#[axum_macros::debug_handler]
pub async fn handle_card_fan_request(Path(hash): Path<String>, Extension(frames): Extension<Arc<HashMap<String, DynamicImage>>>) -> Response<Body> {
    let start = Instant::now();

    let bytes = match Engine.decode(&hash) {
        Ok(bytes) => bytes,
        Err(_) => return Response::builder().status(400).body(Body::from("bad request - provided fan hash is invalid")).unwrap(),
    };
    let decoded: FanRenderRequestData = match serde_json::from_slice(&bytes) {
        Ok(decoded) => decoded,
        Err(_) => return Response::builder().status(400).body(Body::from("bad request - provided fan hash is a valid json but does not follow API structure")).unwrap(),
    };

    if start.elapsed().as_secs_f32() >= RENDER_TIMEOUT {
        return Response::builder().status(500).body(Body::from(format!("gateway timeout - asset render took more than {} seconds", RENDER_TIMEOUT))).unwrap();
    }
    
    if let Some(save_name) = &decoded.save_name {
        match tokio::fs::File::open(format!("{}/{}", CDN_RENDERS_PATH, save_name)).await {
            Ok(mut file) => {
                let mut buff = Vec::new();
                file.read_to_end(&mut buff).await.unwrap();
                return Response::builder()
                    .header("X-Source", "loaded from disk cache")
                    .header("X-Processing-Time", (start.elapsed().as_nanos() as f64 / 1_000_000.0).to_string() + "ms")
                    .header("Content-Type", "image/png")
                    .body(Body::from(buff)).unwrap();
            }
            Err(_) => (),
        }
    }

    let image = match render_fan(decoded.cards, &frames, &start) {
        Ok(image) => image,
        Err(e) => return Response::builder().status(500).body(Body::from(e)).unwrap(),
    };

    if start.elapsed().as_secs_f32() >= RENDER_TIMEOUT {
        return Response::builder().status(500).body(Body::from(format!("gateway timeout - asset render took more than {} seconds", RENDER_TIMEOUT))).unwrap();
    }

    let mut buffer = BufWriter::new(Cursor::new(Vec::new()));
    match image.write_to(&mut buffer, image::ImageFormat::Png) {
        Ok(_) =>(),
        Err(e) => {
            error!("Properly rendered a fan but failed to write it into final buffer. Received error: {} (hash request = {})", e, hash);
            return Response::builder().status(500).body(Body::from("server error - cannot write fan image to buffer")).unwrap();
        }
    };

    if start.elapsed().as_secs_f32() >= RENDER_TIMEOUT {
        return Response::builder().status(500).body(Body::from(format!("gateway timeout - asset render took more than {} seconds", RENDER_TIMEOUT))).unwrap();
    }

    let inner = buffer.into_inner().unwrap().into_inner();

    if let Some(save_name) = &decoded.save_name {
        let location = format!("{}/{}", CDN_RENDERS_PATH, save_name);
        match tokio::fs::File::create(location).await {
            Ok(mut file) => {
                file.write_all(&inner).await.unwrap();
            }
            Err(e) => {
                warn!("Failed to save freshly rendered fan asset into disk at path: {}/{}. Received error: {} (request hash = {})", CDN_RENDERS_PATH, save_name, e, hash);
            },
        }
    }
    
    Response::builder()
        .header("X-Source", "rendered on request")
        .header("X-Processing-Time", (start.elapsed().as_nanos() as f64 / 1_000_000.0).to_string() + "ms")
        .header("Content-Type", "image/png")
        .body(Body::from(inner))
        .unwrap()
}