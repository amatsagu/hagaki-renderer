use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CardRenderRequestData {
    pub id: u32,
    pub variant: u8,
    pub frame_type: u32,
    pub offset_x: Option<i32>,
    pub offset_y: Option<i32>,
    pub save_name: Option<String> // Jeśli podane to znaczy zapisz plik na dysku, dokładnie pod podaną nazwą.png
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FanRenderRequestData {
    pub cards: Vec<CardRenderRequestData>,
    pub save_name: Option<String>
}