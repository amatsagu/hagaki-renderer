use std::{collections::HashMap};
use image::{DynamicImage, load_from_memory};

use crate::config::{CDN_FRAMES_PATH};

pub fn load_frames() -> HashMap<String, DynamicImage> {
    let mut frames = HashMap::new();
    for entry in std::fs::read_dir(CDN_FRAMES_PATH).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            for file in std::fs::read_dir(path).unwrap() {
                let path = file.unwrap().path();
                if path.is_file() && path.to_str().unwrap().ends_with(".png") {
                    let file_buffer = std::fs::read(&path).unwrap();
                    let img = load_from_memory(&file_buffer).unwrap();
                    let name = format!("{}-{}", entry.file_name().into_string().unwrap(), path.file_stem().unwrap().to_str().unwrap());
                    frames.insert(name, img);
                }
            }
        }
    }
    frames
}