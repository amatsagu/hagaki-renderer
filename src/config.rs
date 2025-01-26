#![allow(dead_code)]
use std::{collections::HashMap, sync::LazyLock};

use serde_repr::{Deserialize_repr, Serialize_repr};

pub const ADDRESS: &str = "127.0.0.1:8899";

pub const CDN_PATH: &str = "../cdn";
pub const CDN_FRAMES_PATH: &str = "../cdn/private/frame";
pub const CDN_CHARACTER_IMAGES_PATH: &str = "../cdn/public/character";
pub const CDN_CARD_IMAGES_PATH: &str = "../cdn/private/custom-card"; // W większości pusty, zawiera tylko custom obrazki
pub const CDN_RENDERS_PATH: &str = "../cdn/public/render"; // I tutaj mogą być różne rzeczy, nie tylko render kart ale w przyszłości i inne rzeczy, dlatego custom name w payload

pub const CHARACTER_IMAGE_X: u32 = 245;
pub const CHARACTER_IMAGE_Y: u32 = 370;
pub const CARD_MAX_X: u32 = 303;
pub const CARD_MAX_Y: u32 = 428;
pub const DEFAULT_DYE_COLOR: u32 = 8289918;
pub const RENDER_TIMEOUT: f32 = 60.0; // in seconds

pub const FAN_CARD_ANGLE: f32 = 5.0;
pub const FAN_CIRCLE_CENTER_DISTANCE: f32 = 2000.0;

pub const FRAME_TABLE: LazyLock<HashMap<FrameType, FrameDetails>> = LazyLock::new(|| {
    HashMap::from([(FrameType::DefaultFrame, FrameDetails {
        name: "default",
        static_model: false,
        mask_model: true,
        width: 245,
        height: 370
    }),
    (FrameType::BetaFrame, FrameDetails {
        name: "beta",
        static_model: false,
        mask_model: true,
        width: 251,
        height: 376
    }),
    (FrameType::EdoHiganFrame, FrameDetails {
        name: "edo-higan",
        static_model: true,
        mask_model: true,
        width: 303,
        height: 428
    })])
});

#[repr(u8)]
#[derive(Eq, PartialEq, Hash, Serialize_repr, Deserialize_repr, Debug, Clone)]
pub enum FrameType {
    DefaultFrame = 0,
    BetaFrame,
    EdoHiganFrame
}

impl ToString for FrameType {
    fn to_string(&self) -> String {
        match self {
            FrameType::DefaultFrame => "default".to_string(),
            FrameType::BetaFrame => "beta".to_string(),
            FrameType::EdoHiganFrame => "edo-higan".to_string()
        }
    }
}

pub struct FrameDetails {
    pub name: &'static str,
    pub static_model: bool,
    pub mask_model: bool,
    pub width: u32,
    pub height: u32
}