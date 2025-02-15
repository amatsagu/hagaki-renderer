use std::{collections::HashMap, sync::Arc, time::Instant};

use crate::{config::{CDN_CARD_IMAGES_PATH, CDN_CHARACTER_IMAGES_PATH, CDN_FRAMES_PATH, FAN_CARD_ANGLE, FAN_CIRCLE_CENTER_DISTANCE, RENDER_TIMEOUT}, models::CardRenderRequestData};
use image::{imageops::overlay, load_from_memory, DynamicImage, GenericImage, GenericImageView, ImageBuffer, ImageReader, Pixel, Rgba};
use imageproc::geometric_transformations::{rotate_about_center, Interpolation};

use rayon::prelude::*;

pub fn load_frames() -> HashMap<String, DynamicImage> {
    let mut frames = HashMap::new();
    // let path = CDN_FRAMES_PATH;
    // let path = Path::new(&path);
    // println!("{}", path.canonicalize().unwrap().to_str().unwrap());
    for entry in std::fs::read_dir(CDN_FRAMES_PATH).unwrap() {

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

pub fn render_card(data: &CardRenderRequestData, frames: &Arc<HashMap<String, DynamicImage>>, start_time: &Instant) -> Result<DynamicImage, String> {

    // if data.target_card {
    //     return ImageReader::open(format!("{}/{}.png", CDN_CARD_IMAGES_PATH, data.id))
    //         .map_err(|_| format!("Custom card with id {} not found.", data.id))
    //         .map(|img| img.decode().unwrap());
    // }

    let image_path = format!("{}/{}.png", if data.target_card { CDN_CARD_IMAGES_PATH } else { CDN_CHARACTER_IMAGES_PATH }, data.id);

    // let character_image_buffer = tokio::fs::read(image_path).await.unwrap();
    // let character_image = load_from_memory(&character_image_buffer).unwrap();

    let character_image = match ImageReader::open(&image_path) {
        Ok(img) => match img.decode() {
            Ok(img) => img,
            Err(_) => return Err(format!("Failed to decode {}. Check if file is a valid image", image_path)),
        }
        Err(_) => return Err(format!("Card with id {} not found.", data.id)),
    };

    if start_time.elapsed().as_secs_f32() >= RENDER_TIMEOUT {
        return Err(format!("Render took more than {} seconds", RENDER_TIMEOUT));
    }
    let frame = data.frame_type.to_string();

    let (mask, decoration) = if data.glow {
        (frames.get(&format!("{}-glow-mask", frame)), frames.get(&format!("{}-glow-static", frame)))
    } else {
        (frames.get(&format!("{}-mask", frame)), frames.get(&format!("{}-static", frame)))
    };

    if mask.is_none() {
        return Err(format!("Frame {} not found", frame));
    }

    if start_time.elapsed().as_secs_f32() >= RENDER_TIMEOUT {
        return Err(format!("Render took more than {} seconds", RENDER_TIMEOUT));
    }

    let mask = mask.unwrap();

    let mut result = ImageBuffer::new(mask.width(), mask.height());
    let x = (mask.width() - character_image.width()) / 2 + data.offset_x.unwrap_or(0) as u32;
    let y = (mask.height() - character_image.height()) / 2 + data.offset_y.unwrap_or(0) as u32;

    if let Err(_) = result.copy_from(&character_image, x, y) {
        return Err("Failed to copy character image to result image".to_string());
    };

    if start_time.elapsed().as_secs_f32() >= RENDER_TIMEOUT {
        return Err(format!("Render took more than {} seconds", RENDER_TIMEOUT));
    }
    // overlay(&mut result, &character_image, x as i64, y as i64);

    let frame_color = image::Rgb::from([data.dye >> 16 & 0xFF, data.dye >> 8 & 0xFF, data.dye & 0xFF]);

    result.par_enumerate_pixels_mut().for_each(|(x, y, p)| {
        let mask_pixel = mask.get_pixel(x, y);
        if mask_pixel[3] == 0 {
            return
        }
        let mask = mask_pixel[0] as f32 / 255.0;
        p.blend(&image::Rgba::from([
            (frame_color.0[0] as f32 * mask) as u8, 
            (frame_color.0[1] as f32 * mask) as u8, 
            (frame_color.0[2] as f32 * mask) as u8, 
            mask_pixel[3]
        ]));
    });

    if start_time.elapsed().as_secs_f32() >= RENDER_TIMEOUT {
        return Err(format!("Render took more than {} seconds", RENDER_TIMEOUT));
    }

    // for (x, y, pixel) in mask.pixels().filter(|(_, _, p)| p[3] != 0) {
    //     let mask = pixel[0] as f32 / 255.0;
    //     result.get_pixel_mut(x, y).blend(&image::Rgba::from([
    //         (frame_color.0[0] as f32 * mask) as u8, 
    //         (frame_color.0[1] as f32 * mask) as u8, 
    //         (frame_color.0[2] as f32 * mask) as u8, 
    //         pixel[3]
    //     ]));
    // }

    if let Some(decoration) = decoration {
        // for (x, y, pixel) in decoration.pixels().filter(|(_, _, p)| p[3] != 0) {
        //     result.get_pixel_mut(x, y).blend(&pixel);
        // }

        result.par_enumerate_pixels_mut().for_each(|(x, y, p)| {
            let decoration_pixel = decoration.get_pixel(x, y);
            if decoration_pixel[3] == 0 {
                return
            }
            p.blend(&decoration_pixel);
        })
    }

    if start_time.elapsed().as_secs_f32() >= RENDER_TIMEOUT {
        return Err(format!("Render took more than {} seconds", RENDER_TIMEOUT));
    }

    Ok(result.into())
}

fn rotate_image(image: DynamicImage, angle: f32) -> DynamicImage {
    let (width, height) = image.dimensions();
    let rad = angle.to_radians();
    let new_width = (width as f32 * rad.cos().abs() + height as f32 * rad.sin().abs()).ceil() as u32;
    let new_height = (width as f32 * rad.sin().abs() + height as f32 * rad.cos().abs()).ceil() as u32;
    // println!("{}x{} -> {}x{}", width, height, new_width, new_height);
    let mut result = ImageBuffer::new(new_width, new_height);
    let diff_x = (new_width - width) / 2;
    let diff_y = (new_height - height) / 2;
    
    result.par_enumerate_pixels_mut()
          .filter(|(x, y, _)| *x >= diff_x && *x < diff_x + width && *y < diff_y + height && *y >= diff_y)
          .for_each(|(x, y, p)| {
        *p = image.get_pixel((x - diff_x) as u32, (y - diff_y) as u32);
    });
    // result.copy_from(&image, diff_x, diff_y).unwrap();

    rotate_about_center(&result, rad, Interpolation::Bicubic, Rgba::from([0, 0, 0, 0])).into()
    
}

struct IndexedImage {
    image: DynamicImage,
    center_offset: Position
}

#[derive(Debug, Default, Clone, Copy)]
struct Position {
    x: f32,
    y: f32,
    angle: f32
}

pub fn render_fan(data: Vec<CardRenderRequestData>, frames: &Arc<HashMap<String, DynamicImage>>, start_time: &Instant) -> Result<DynamicImage, String> {
    let mut images = Vec::new();
    let image_count = data.len();

    let mut positions = vec![Position::default(); image_count];

    for (i, card) in positions.iter_mut().enumerate() {
        let position_index = i as f32 - image_count as f32 / 2.0 + 0.5;
        let angle = (FAN_CARD_ANGLE * position_index).to_radians();
        card.x = FAN_CIRCLE_CENTER_DISTANCE * angle.sin();
        card.y = (FAN_CIRCLE_CENTER_DISTANCE * angle.cos() - FAN_CIRCLE_CENTER_DISTANCE).abs();
        card.angle = angle;
        // println!("{}: {}\t{}x{}", i, angle.to_degrees().round() as i32, card.x, card.y);
    }

    for (i, card) in data.iter().enumerate() {
        let mut image = match render_card(&card.clone(), &frames, start_time) {
            Ok(image) => image,
            Err(e) => return Err(e),
        };

        if start_time.elapsed().as_secs_f32() >= RENDER_TIMEOUT {
            return Err(format!("Render took more than {} seconds", RENDER_TIMEOUT));
        }

        let mut rotation_angle = positions[i].angle.to_degrees().round();

        if rotation_angle > 45.0 && rotation_angle < 135.0 {
            rotation_angle = rotation_angle - 90.0;
            image = image.rotate90();
        }

        if rotation_angle < -45.0 && rotation_angle > -135.0 {
            rotation_angle = rotation_angle + 90.0;
            image = image.rotate270();
        }

        let image = rotate_image(image, rotation_angle);
        images.push(IndexedImage {
            image,
            center_offset: positions[i]
        });
    }

    if start_time.elapsed().as_secs_f32() >= RENDER_TIMEOUT {
        return Err(format!("Render took more than {} seconds", RENDER_TIMEOUT));
    }
    
    let center_height = images[image_count / 2].image.height() / 2;
    let x = {
        let mut x = images.iter().map(|image| image.center_offset.x as u32).max().unwrap();
        x *= 2;
        x += images.iter().max_by(|a, b| a.center_offset.x.partial_cmp(&b.center_offset.x).unwrap()).unwrap().image.width() / 2;
        x += images.iter().min_by(|a, b| a.center_offset.x.partial_cmp(&b.center_offset.x).unwrap()).unwrap().image.width() / 2;
        x
    };

    let y = {
        let mut y = center_height;
        y += images[0].image.height().max(images[image_count - 1].image.height()) / 2;
        y += images[0].center_offset.y.abs().ceil() as u32;
        y
    };

    let mut result = ImageBuffer::new(x, y);

    let mut rearranged = Vec::new();
    for i in 0..images.len() / 2 {
        rearranged.push(images.get(images.len() - i - 1).unwrap());
        rearranged.push(images.get(i).unwrap());
    }

    if start_time.elapsed().as_secs_f32() >= RENDER_TIMEOUT {
        return Err(format!("Render took more than {} seconds", RENDER_TIMEOUT));
    }

    if images.len() % 2 == 1 {
        rearranged.push(images.get(images.len() / 2).unwrap());
    }
    for image in rearranged {
        let x = (image.center_offset.x.ceil() as i64 + result.width() as i64 / 2) - image.image.width() as i64 / 2;
        let y = (image.center_offset.y.ceil() as i64 + center_height as i64) - image.image.height() as i64 / 2;

        // println!("{}: ({}, {})", image.index, x, y);

        overlay(&mut result, &image.image, x, y);

        // image.image.save(format!("fan-{}.png", image.index)).unwrap();


        if start_time.elapsed().as_secs_f32() >= RENDER_TIMEOUT {
            return Err(format!("Render took more than {} seconds", RENDER_TIMEOUT));
        }
    }

    if start_time.elapsed().as_secs_f32() >= RENDER_TIMEOUT {
        return Err(format!("Render took more than {} seconds", RENDER_TIMEOUT));
    }

    Ok(result.into())
}