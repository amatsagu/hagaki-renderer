use std::{collections::HashMap, time::Instant};
use image::{DynamicImage, ImageReader, RgbaImage, Pixel};
use log::warn;
use rayon::prelude::*;

use crate::{
    config::{CDN_CHARACTER_IMAGES_PATH, RENDER_TIMEOUT}, 
    models::CardRenderRequestData
};

/// Renders a card by overlaying a frame on a character image.
/// Optimization: Uses parallelized buffer processing and correct alpha blending.
pub fn render_card(data: &CardRenderRequestData, frames: &HashMap<u32, DynamicImage>, start_time: &Instant) -> Result<DynamicImage, String> {
    let frame = match frames.get(&data.frame_type) {
        Some(f) => f,
        None => return Err("failed request - invalid frame type provided".to_string()),
    };

    let image_path = if data.variant == 0 {
        format!("{}/{}.png", CDN_CHARACTER_IMAGES_PATH, data.id)
    } else {
        format!("{}/{}/{}{}.png", CDN_CHARACTER_IMAGES_PATH, data.id, if data.variant < 10 {'u'} else {'x'}, data.variant)
    };

    let character_image = match ImageReader::open(&image_path) {
        Ok(img) => match img.decode() {
            Ok(img) => img,
            Err(e) => {
                warn!("Damaged image at {}: {}", image_path, e);
                return Err("failed request - failed to decode main image asset.".to_string())
            },
        }
        Err(e) => {
            warn!("Missing image at {}: {}", image_path, e);
            return Err("failed request - missing main image asset.".to_string());
        },
    };

    if start_time.elapsed().as_secs_f32() >= RENDER_TIMEOUT {
        return Err("gateway timeout - loading took too long".to_string());
    }

    // Convert to RGBA8. into_rgba8() may avoid allocation if already in that format.
    let mut canvas = character_image.into_rgba8();
    let frame_rgba = frame.as_rgba8().ok_or("server error - frame is not RGBA8")?;

    // Apply the frame overlay using the optimized blender
    apply_layer_optimized(&mut canvas, frame_rgba);

    if start_time.elapsed().as_secs_f32() >= RENDER_TIMEOUT {
        return Err("gateway timeout - render calculation took too long".to_string());
    }

    Ok(DynamicImage::ImageRgba8(canvas))
}

/// Optimized layer blending using Rayon for parallelism.
/// Uses the image crate's Pixel::blend for correct alpha handling and smoothness.
fn apply_layer_optimized(canvas: &mut RgbaImage, layer: &RgbaImage) {
    canvas.as_flat_samples_mut().samples.par_chunks_exact_mut(4)
        .zip(layer.as_flat_samples().samples.par_chunks_exact(4))
        .for_each(|(c, l)| {
            if l[3] == 0 { return; }
            if l[3] == 255 {
                c.copy_from_slice(l);
            } else {
                // Use image crate's own blending logic for correctness and smoothness
                let mut dst = image::Rgba([c[0], c[1], c[2], c[3]]);
                dst.blend(&image::Rgba([l[0], l[1], l[2], l[3]]));
                c[0] = dst[0];
                c[1] = dst[1];
                c[2] = dst[2];
                c[3] = dst[3];
            }
        });
}
