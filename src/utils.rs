use std::{collections::HashMap, path::Path, sync::Arc};

use crate::{config::{CDN_PATH, DEFAULT_DYE_COLOR}, models::RenderRequestData};
use image::{imageops::overlay, load_from_memory, DynamicImage, GenericImageView, ImageBuffer, ImageReader, Pixel};

pub fn load_frames() -> HashMap<String, DynamicImage> {
    let mut frames = HashMap::new();
    let path = CDN_PATH.to_owned() + "/private/frame";
    let path = Path::new(&path);
    // println!("{}", path.canonicalize().unwrap().to_str().unwrap());
    for entry in std::fs::read_dir(path).unwrap() {

        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            for file in std::fs::read_dir(path).unwrap() {
                let path = file.unwrap().path();
                if path.is_file() && path.to_str().unwrap().ends_with(".png") {
                    // println!("{}", path.to_str().unwrap());
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

pub fn render_card(data: RenderRequestData, frames: Arc<HashMap<String, DynamicImage>>) -> Result<DynamicImage, String> {
    let image_path = if let Some(true) = data.custom_image {
        format!("{}/public/custom-character/{}.png", CDN_PATH, data.id)
    } else {
        format!("{}/public/character/{}.png", CDN_PATH, data.id)
    };
    // let character_image_buffer = tokio::fs::read(image_path).await.unwrap();
    // let character_image = load_from_memory(&character_image_buffer).unwrap();

    let character_image = match ImageReader::open(&image_path) {
        Ok(img) => match img.decode() {
            Ok(img) => img,
            Err(_) => return Err(format!("Failed to decode {}. Check if file is a valid image", image_path)),
        }
        Err(_) => return Err(format!("Card with id {} not found.", data.id)),
    };

    let (mask, decoration) = if let Some(frame) = data.frame {
        if let Some(true) = data.glow {
            (frames.get(&format!("{}-glow-mask", frame.to_string())), frames.get(&format!("{}-glow-static", frame.to_string())))
        } else {
            (frames.get(&format!("{}-mask", frame.to_string())), frames.get(&format!("{}-static", frame.to_string())))
        }
    } else {
        (None, None)
    };

    if mask.is_none() {
        return Ok(character_image)
    }

    let mask = mask.unwrap();

    let mut result = ImageBuffer::new(mask.width(), mask.height());
    let x = (mask.width() - character_image.width()) / 2 + data.offset_x.unwrap_or(0) as u32;
    let y = (mask.height() - character_image.height()) / 2 + data.offset_y.unwrap_or(0) as u32;

    overlay(&mut result, &character_image, x as i64, y as i64);

    let dye = data.dye.unwrap_or_else(|| DEFAULT_DYE_COLOR);
    let frame_color = image::Rgb::from([dye >> 16 & 0xFF, dye >> 8 & 0xFF, dye & 0xFF]);

    for (x, y, pixel) in mask.pixels().filter(|(_, _, p)| p[3] != 0) {
        let mask = pixel[0] as f32 / 255.0;
        result.get_pixel_mut(x, y).blend(&image::Rgba::from([
            (frame_color.0[0] as f32 * mask) as u8, 
            (frame_color.0[1] as f32 * mask) as u8, 
            (frame_color.0[2] as f32 * mask) as u8, 
            pixel[3]
        ]));
    }

    if let Some(decoration) = decoration {
        for (x, y, pixel) in decoration.pixels().filter(|(_, _, p)| p[3] != 0) {
            result.get_pixel_mut(x, y).blend(&pixel);
        }
    }

    Ok(result.into())
}