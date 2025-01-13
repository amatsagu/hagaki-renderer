use std::{collections::HashMap, path::Path, sync::Arc, time::Instant};

use crate::{config::{CDN_PATH, DEFAULT_DYE_COLOR, FAN_Y_OFFSETS, RENDER_TIMEOUT}, models::RenderRequestData};
use image::{imageops::overlay, load_from_memory, DynamicImage, GenericImage, GenericImageView, ImageBuffer, ImageReader, Pixel, Rgba};
use imageproc::geometric_transformations::{rotate_about_center, Interpolation};

use rayon::prelude::*;

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

pub fn render_card(data: RenderRequestData, frames: &Arc<HashMap<String, DynamicImage>>, start_time: &Instant) -> Result<DynamicImage, String> {
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

    if start_time.elapsed().as_secs_f32() >= RENDER_TIMEOUT {
        return Err(format!("Render took more than {} seconds", RENDER_TIMEOUT));
    }

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

    let dye = data.dye.unwrap_or_else(|| DEFAULT_DYE_COLOR);
    let frame_color = image::Rgb::from([dye >> 16 & 0xFF, dye >> 8 & 0xFF, dye & 0xFF]);

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
    x_offset: u32,
    index: usize
}

pub fn render_fan(data: Vec<RenderRequestData>, frames: &Arc<HashMap<String, DynamicImage>>, start_time: &Instant) -> Result<DynamicImage, String> {
    let mut images = Vec::new();
    let image_count = data.len();
    let mut x_offset = 0;

    for (i, card) in data.iter().enumerate() {
        let image = match render_card(card.clone(), &frames, start_time) {
            Ok(image) => image,
            Err(e) => return Err(e),
        };

        if start_time.elapsed().as_secs_f32() >= RENDER_TIMEOUT {
            return Err(format!("Render took more than {} seconds", RENDER_TIMEOUT));
        }

        let image = rotate_image(image, 5.0 * (i as isize - image_count as isize / 2) as f32);
        let offset = (image.width() as f32 * 0.6).ceil() as u32;
        images.push(IndexedImage {
            image,
            x_offset,
            index: i
        });
        x_offset += offset;
    }

    if start_time.elapsed().as_secs_f32() >= RENDER_TIMEOUT {
        return Err(format!("Render took more than {} seconds", RENDER_TIMEOUT));
    }
    
    let middle_h = images[images.len() / 2].image.height();
    let x = images.iter().enumerate().map(|(i, img)| {(img.image.width() as f32 * if i == 0 {1.0} else {0.6}) as u32}).sum();
    let y = images.iter().map(|img| img.image.height()).max().unwrap() + (FAN_Y_OFFSETS[image_count / 2] * middle_h as f32) as u32;

    let mut result = ImageBuffer::new(x, y);

    let mut rearranged = Vec::new();
    for i in 0..images.len() / 2 {
        rearranged.push(images.get(i).unwrap());
        rearranged.push(images.get(images.len() - i - 1).unwrap());
    }

    if start_time.elapsed().as_secs_f32() >= RENDER_TIMEOUT {
        return Err(format!("Render took more than {} seconds", RENDER_TIMEOUT));
    }

    rearranged.push(images.get(images.len() / 2).unwrap());
    for image in rearranged {
        let y_offset = FAN_Y_OFFSETS[(image.index as isize - image_count as isize / 2).abs() as usize] * middle_h as f32;
        overlay(&mut result, &image.image, image.x_offset as i64, y_offset as i64);

        if start_time.elapsed().as_secs_f32() >= RENDER_TIMEOUT {
            return Err(format!("Render took more than {} seconds", RENDER_TIMEOUT));
        }
    }

    if start_time.elapsed().as_secs_f32() >= RENDER_TIMEOUT {
        return Err(format!("Render took more than {} seconds", RENDER_TIMEOUT));
    }

    Ok(result.into())
}