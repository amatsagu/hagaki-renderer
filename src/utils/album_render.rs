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

    let aspect_bias = 1.35;
    let mut cols = ((aspect_bias * image_count as f32).sqrt()).ceil() as u32;
    cols = cols.min(image_count as u32);
    let rows = ((image_count as f32) / (cols as f32)).ceil() as u32;
    
    let x = cols * max_width + (cols + 1) * ALBUM_CARD_PADDING;
    let y = rows * max_height + (rows + 1) * ALBUM_CARD_PADDING;
    
    let mut result = ImageBuffer::new(x, y);

    if start_time.elapsed().as_secs_f32() >= RENDER_TIMEOUT {
        return Err(format!("Render took more than {} seconds", RENDER_TIMEOUT));
    }

    for (i, image) in images.iter().enumerate() {
        let idx = i as u32;
        let col = idx % cols;
        let row = idx / cols;

        let x = (ALBUM_CARD_PADDING + col * (max_width + ALBUM_CARD_PADDING)) as i64;
        let y = (ALBUM_CARD_PADDING + row * (max_height + ALBUM_CARD_PADDING)) as i64;

        overlay(&mut result, image, x, y);

        if start_time.elapsed().as_secs_f32() >= RENDER_TIMEOUT {
            return Err(format!("Render took more than {} seconds", RENDER_TIMEOUT));
        }
    }

    Ok(result.into())
}