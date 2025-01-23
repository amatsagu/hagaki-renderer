use serde::{Deserialize, Serialize};

use crate::config::FrameType;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CardRenderRequestData {
    pub id: u32,
    pub target_card: bool, // Jeśli jest false to <id> jest postaci, jeśli true to id/kod konkretnej karty
    pub dye: u32,
    pub glow: bool,
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