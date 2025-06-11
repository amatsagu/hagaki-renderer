use std::{collections::HashMap, sync::Arc, time::Instant};

use image::{imageops::overlay, DynamicImage, ImageBuffer};

use crate::config::{ALBUM_CARD_PADDING, RENDER_TIMEOUT};
use crate::models::CardRenderRequestData;
use crate::utils::render_card;

// use rayon::prelude::*;

pub fn render_album(data: Vec<CardRenderRequestData>, frames: &Arc<HashMap<String, DynamicImage>>, start_time: &Instant) -> Result<DynamicImage, String> {
    let mut images = Vec::new();
    let image_count = data.len();

    let mut max_width = 0;
    let mut max_height = 0;

    for card in data {
        let image = match render_card(&card.clone(), &frames, start_time) {
            Ok(image) => image,
            Err(e) => return Err(e),
        };

        if start_time.elapsed().as_secs_f32() >= RENDER_TIMEOUT {
            return Err(format!("Render took more than {} seconds", RENDER_TIMEOUT));
        }

        max_width = max_width.max(image.width());
        max_height = max_height.max(image.height());

        images.push(image);
    }

    if start_time.elapsed().as_secs_f32() >= RENDER_TIMEOUT {
        return Err(format!("Render took more than {} seconds", RENDER_TIMEOUT));
    }
    
    let row_items = (image_count as f64).sqrt().ceil() as usize;
    let column_items = if row_items * row_items == image_count {
        row_items
    } else {
        image_count / row_items + 1
    };

    let x = row_items as u32 * (max_width + ALBUM_CARD_PADDING);
    let y = column_items as u32 * (max_height + ALBUM_CARD_PADDING);
    
    let mut result = ImageBuffer::new(x, y);

    if start_time.elapsed().as_secs_f32() >= RENDER_TIMEOUT {
        return Err(format!("Render took more than {} seconds", RENDER_TIMEOUT));
    }

    for (i, image) in images.iter().enumerate() {
        let mut x = ((max_width + ALBUM_CARD_PADDING) as f32 * ((i % row_items) as f32 + 0.5)) as i64 - (image.width() / 2) as i64;
        let y = ((max_height + ALBUM_CARD_PADDING) as f32 * ((i / row_items) as f32 + 0.5)) as i64 - (image.height() / 2) as i64;

        if i / row_items == column_items - 1 {
            if row_items * column_items != image_count {
                x += ((max_width + ALBUM_CARD_PADDING) as f32 * (row_items * column_items - image_count) as f32 / 2.0) as i64;
            }
        }

        // println!("{}: ({}, {})", image.index, x, y);

        overlay(&mut result, image, x, y);

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