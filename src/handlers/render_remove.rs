use axum::{
    extract::Path,
    http::{HeaderMap, Response},
};
use std::fs::remove_file;
use crate::config::{AUTH_TOKEN, CDN_RENDERS_PATH};

#[axum_macros::debug_handler]
pub async fn render_remove(headers: HeaderMap, Path(file_name): Path<String>) -> Response<String> {
    match headers.get("Authorization") {
        Some(value) if value == AUTH_TOKEN => {
            let location = format!("{}/{}", CDN_RENDERS_PATH, file_name);
            
            match remove_file(&location) {
                Ok(_) => Response::builder()
                    .body("Success".to_string())
                    .unwrap(),
                Err(_) => Response::builder()
                    .status(404)
                    .body("File not found".to_string())
                    .unwrap(),
            }
        }
        _ => Response::builder()
            .status(401)
            .body("Unauthorized".to_string())
            .unwrap(),
    }
}