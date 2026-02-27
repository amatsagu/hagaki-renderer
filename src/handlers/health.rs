use axum::{body::Body, response::Response};
use chrono::Utc;
use crate::START_TIME;

pub async fn handle_health_request() -> Response<Body> {
    let uptime = Utc::now() - *START_TIME;
    let uptime_str = format!("{} seconds", uptime.num_seconds());

    Response::builder()
        .status(200)
        .header("X-App-Uptime", uptime_str)
        .header("X-App-Start-Time", START_TIME.to_rfc3339())
        .body(Body::from("OK"))
        .unwrap()
}
