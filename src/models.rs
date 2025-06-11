use serde::{Deserialize, Serialize};

use crate::config::FrameType;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CardRenderRequestData {
    pub id: u32,
    pub variant: u8,
    pub dye: u32,
    pub kindled: bool,
    pub frame_type: FrameType,
    pub offset_x: Option<i32>,
    pub offset_y: Option<i32>,
    pub save_name: Option<String> // Jeśli podane to znaczy zapisz plik na dysku, dokładnie pod podaną nazwą.png
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FanRenderRequestData {
    pub cards: Vec<CardRenderRequestData>,
    pub save_name: Option<String>
}