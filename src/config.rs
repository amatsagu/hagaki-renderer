#![allow(dead_code)]
use std::{collections::HashMap, sync::LazyLock};

use serde_repr::{Deserialize_repr, Serialize_repr};

pub const ADDRESS: &str = "127.0.0.1:8899";
pub const AUTH_TOKEN: &str = "a1fe0d2d2469bb472016d667be975b51";

pub const CDN_FRAMES_PATH: &str = "../asset/private/frame";
pub const CDN_CHARACTER_IMAGES_PATH: &str = "../asset/private/character";
pub const CDN_CARD_IMAGES_PATH: &str = "../asset/private/custom-character-card";
pub const CDN_RENDERS_PATH: &str = "../asset/public/render";

pub const RENDER_TIMEOUT: f32 = 10.0; // in seconds

pub const FAN_CARD_ANGLE: f32 = 5.0;
pub const FAN_CIRCLE_CENTER_DISTANCE: f32 = 2000.0;

pub const ALBUM_CARD_PADDING: u32 = 10;

pub const FRAME_TABLE: LazyLock<HashMap<FrameType, FrameDetails>> = LazyLock::new(|| {
    HashMap::from([
        (FrameType::DefaultFrame, FrameDetails {
            name: "default",
            static_model: true,
            color_model: true,
            extendable: false, // ???
            width: 550,
            height: 800
        })
    ])
});

#[repr(u8)]
#[derive(Eq, PartialEq, Hash, Serialize_repr, Deserialize_repr, Debug, Clone)]
pub enum FrameType {
    DefaultFrame = 0
}

impl ToString for FrameType {
    fn to_string(&self) -> String {
        match self {
            FrameType::DefaultFrame => "default".to_string(),
            // FrameType::BetaFrame => "beta".to_string(),
            // FrameType::EdoHiganFrame => "edo-higan".to_string()
        }
    }
}

pub struct FrameDetails {
    pub name: &'static str,
    pub static_model: bool,
    pub color_model: bool,
    pub extendable: bool,
    pub width: u32,
    pub height: u32
}