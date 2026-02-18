use std::{collections::HashMap};
use image::{image_dimensions, ImageReader, DynamicImage};
use log::info;

use crate::config::{CDN_FRAMES_PATH};

pub fn load_frames() -> HashMap<u32, DynamicImage> {
    let mut frames = HashMap::new();
    let entries = match std::fs::read_dir(CDN_FRAMES_PATH) {
        Ok(entries) => entries,
        Err(e) => {
            log::error!("Failed to read frames directory {}: {}", CDN_FRAMES_PATH, e);
            return frames;
        }
    };

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("png") {
            if let Ok((w, h)) = image_dimensions(&path) {
                if w == 550 && h == 800 {
                    let stem = path.file_stem().unwrap().to_str().unwrap();
                    if let Ok(id) = stem.parse::<u32>() {
                        if let Ok(reader) = ImageReader::open(&path) {
                            if let Ok(img) = reader.decode() {
                                frames.insert(id, img);
                            }
                        }
                    }
                }
            }
        }
    }
    info!("Loaded {} frames from {}", frames.len(), CDN_FRAMES_PATH);
    frames
}