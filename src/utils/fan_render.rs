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
    let image_count = data.len();

    // Precompute positions (it's cheap, no need to parallelize)
    let positions: Vec<Position> = (0..image_count)
        .map(|i| {
            let position_index = i as f32 - image_count as f32 / 2.0 + 0.5;
            let angle = (FAN_CARD_ANGLE * position_index).to_radians();
            Position {
                x: FAN_CIRCLE_CENTER_DISTANCE * angle.sin(),
                y: (FAN_CIRCLE_CENTER_DISTANCE * angle.cos() - FAN_CIRCLE_CENTER_DISTANCE).abs(),
                angle,
            }
        })
        .collect();

    // Parallel render + rotation
    let images_results: Vec<_> = data
        .into_par_iter()
        .zip(positions.par_iter())
        .map(|(card, pos)| {
            let image = render_card(&card, frames, start_time)?;

            let mut rotation_angle = pos.angle.to_degrees().round();
            let mut image = image;

            if rotation_angle > 45.0 && rotation_angle < 135.0 {
                rotation_angle -= 90.0;
                image = image.rotate90();
            }
            if rotation_angle < -45.0 && rotation_angle > -135.0 {
                rotation_angle += 90.0;
                image = image.rotate270();
            }

            let image = rotate_image(image, rotation_angle);
            Ok(IndexedImage {
                image,
                center_offset: *pos,
            })
        })
        .collect();

    let mut images = Vec::with_capacity(image_count);
    for result in images_results {
        match result {
            Ok(img) => images.push(img),
            Err(e) => return Err(e),
        }
    }

    if start_time.elapsed().as_secs_f32() >= RENDER_TIMEOUT {
        return Err(format!("gateway timeout - asset render took more than {} seconds", RENDER_TIMEOUT));
    }

    // Calculate output image size
    let center_height = images[image_count / 2].image.height() / 2;
    let x = {
        let mut x = images.iter().map(|img| img.center_offset.x as u32).max().unwrap();
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

    // Rearrange rendering order for overlapping stacking
    let mut rearranged = Vec::new();
    for i in 0..images.len() / 2 {
        rearranged.push(images.get(images.len() - i - 1).unwrap());
        rearranged.push(images.get(i).unwrap());
    }
    if images.len() % 2 == 1 {
        rearranged.push(images.get(images.len() / 2).unwrap());
    }

    for image in rearranged {
        let draw_x = (image.center_offset.x.ceil() as i64 + result.width() as i64 / 2) - image.image.width() as i64 / 2;
        let draw_y = (image.center_offset.y.ceil() as i64 + center_height as i64) - image.image.height() as i64 / 2;

        overlay(&mut result, &image.image, draw_x, draw_y);

        if start_time.elapsed().as_secs_f32() >= RENDER_TIMEOUT {
            return Err(format!("gateway timeout - asset render took more than {} seconds", RENDER_TIMEOUT));
        }
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