use serde::{Deserialize, Serialize};

use crate::config::FrameType;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RenderRequestData {
    pub id: u32,
    pub dye: Option<u32>,
    pub frame: Option<FrameType>,
    pub custom_image: Option<bool>,
    pub glow: Option<bool>,
    pub offset_x: Option<i32>,
    pub offset_y: Option<i32>,
    pub save: Option<bool>
}