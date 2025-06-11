use std::{collections::HashMap, sync::Arc, time::Instant};
use image::{imageops::overlay, DynamicImage, GenericImageView, ImageBuffer, Rgba};
use imageproc::geometric_transformations::{rotate_about_center, Interpolation};

use crate::{config::{FAN_CARD_ANGLE, FAN_CIRCLE_CENTER_DISTANCE, RENDER_TIMEOUT}};
use crate::models::CardRenderRequestData;
use crate::utils::render_card;

use rayon::prelude::*;

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

fn rotate_image(image: DynamicImage, angle: f32) -> DynamicImage {
    let (width, height) = image.dimensions();
    let rad = angle.to_radians();
    let new_width = (width as f32 * rad.cos().abs() + height as f32 * rad.sin().abs()).ceil() as u32;
    let new_height = (width as f32 * rad.sin().abs() + height as f32 * rad.cos().abs()).ceil() as u32;
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