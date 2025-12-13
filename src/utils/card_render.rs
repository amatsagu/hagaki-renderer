use std::{collections::HashMap, sync::Arc, time::Instant};
use image::{DynamicImage, GenericImage, GenericImageView, ImageBuffer, ImageReader, Pixel, Rgba};
use log::warn;
use palette::{Srgb, Oklab, IntoColor};
use rayon::prelude::*;

use crate::{
    config::{CDN_CHARACTER_IMAGES_PATH, RENDER_TIMEOUT, FRAME_TABLE}, 
    models::CardRenderRequestData
};

pub fn render_card(data: &CardRenderRequestData, frames: &Arc<HashMap<String, DynamicImage>>, start_time: &Instant) -> Result<DynamicImage, String> {
    let frame_details = match FRAME_TABLE.get(&data.frame_type) {
        Some(details) => details,
        None => return Err("failed request - invalid frame type provided".to_string()),
    };

    // Silently switch back to base version if frame is not extendable.
    // It should technically error but older versions of Hagaki may rely on this behavior!
    let use_kindled = if data.kindled && !frame_details.extendable { false } else { data.kindled };
    
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

    let suffix = if use_kindled { "-kindled" } else { "" };
    let get_layer = |is_needed: bool, type_name: &str| -> Result<Option<&DynamicImage>, String> {
        if !is_needed { return Ok(None); }
        let key = format!("{}{}-{}", frame_details.name, suffix, type_name);
        match frames.get(&key) {
            Some(img) => Ok(Some(img)),
            None => Err(format!("server error - missing required asset: {}", key)),
        }
    };

    let color_layer_ref = get_layer(frame_details.color_model, "color")?;
    let static_layer_ref = get_layer(frame_details.static_model, "static")?;

    let width = frame_details.width;
    let height = frame_details.height;
    
    let mut result = ImageBuffer::new(width, height);
    if let Err(_) = result.copy_from(&character_image, 0, 0) {
        return Err("server error - failed to copy character".to_string());
    };

    if let Some(decoration) = static_layer_ref {
        apply_layer(&mut result, decoration, width, height);
    }

    if let Some(mask) = color_layer_ref {
        apply_dyed_layer(&mut result, mask, data.dye, width, height);
    }

    if start_time.elapsed().as_secs_f32() >= RENDER_TIMEOUT {
        return Err("gateway timeout - render calculation took too long".to_string());
    }

    Ok(result.into())
}

/// Applies a static layer (like a frame overlay)
/// optimization: converts DynamicImage to Buffer slice to avoid per-pixel dispatch
fn apply_layer(canvas: &mut ImageBuffer<Rgba<u8>, Vec<u8>>, layer: &DynamicImage, w: u32, h: u32) {
    let (lw, lh) = layer.dimensions();
    let fits = lw == w && lh == h;

    // Fast path: direct buffer access if the image is already Rgba8 (highly likely)
    if let Some(layer_buf) = layer.as_rgba8() {
        canvas.par_enumerate_pixels_mut().for_each(|(x, y, p)| {
            if !fits && (x >= lw || y >= lh) { return; }
            
            let pixel = layer_buf.get_pixel(x, y);
            if pixel[3] == 0 { return; } // Skip transparent early
            p.blend(pixel);
        });
    } else {
        // Slow path: Dynamic dispatch fallback
        canvas.par_enumerate_pixels_mut().for_each(|(x, y, p)| {
            if !fits && (x >= lw || y >= lh) { return; }
            let pixel = layer.get_pixel(x, y);
            if pixel[3] == 0 { return; }
            p.blend(&pixel);
        });
    }
}

/// Calculates Oklab dye physics and blends in a single pass
fn apply_dyed_layer(canvas: &mut ImageBuffer<Rgba<u8>, Vec<u8>>, mask: &DynamicImage, dye: u32, w: u32, h: u32) {
    let overlay_rgb = Srgb::new(
        ((dye >> 16) & 0xFF) as f32 / 255.0,
        ((dye >> 8) & 0xFF) as f32 / 255.0,
        (dye & 0xFF) as f32 / 255.0,
    );
    let dye_lab: Oklab = overlay_rgb.into_linear().into_color();

    let blend_strength = 0.90;
    let inv_blend_strength = 0.10;
    let light_blend_strength = 0.5;
    let chroma_boost_factor = 0.5;
    let max_chroma_squared = 1.0;

    let (mw, mh) = mask.dimensions();
    let fits = mw == w && mh == h;

    let mask_buf_opt = mask.as_rgba8();

    canvas.par_enumerate_pixels_mut().for_each(|(x, y, canvas_pixel)| {
        if !fits && (x >= mw || y >= mh) { return; }

        let (r, g, b, a) = if let Some(buf) = mask_buf_opt {
            let p = buf.get_pixel(x, y);
            (p[0], p[1], p[2], p[3])
        } else {
            let p = mask.get_pixel(x, y);
            (p[0], p[1], p[2], p[3])
        };

        if a == 0 { return; }

        let orig_rgb = Srgb::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0);
        let orig_lab: Oklab = orig_rgb.into_linear().into_color();

        let chroma_boost = 1.0 + chroma_boost_factor * (1.0 - orig_lab.l);

        let mut final_a = (orig_lab.a * inv_blend_strength + dye_lab.a * blend_strength) * chroma_boost;
        let mut final_b = (orig_lab.b * inv_blend_strength + dye_lab.b * blend_strength) * chroma_boost;

        let chroma_squared = final_a * final_a + final_b * final_b;

        if chroma_squared > max_chroma_squared {
            let scale = max_chroma_squared / chroma_squared.sqrt();
            final_a *= scale;
            final_b *= scale;
        }

        let final_lab = Oklab {
            l: orig_lab.l * (1.0 - light_blend_strength) + dye_lab.l * light_blend_strength,
            a: final_a,
            b: final_b,
        };

        let final_rgb_float: Srgb<f32> = Srgb::from_linear(final_lab.into_color());
        let final_u8 = final_rgb_float.into_format::<u8>();
        
        let dyed_pixel = Rgba([final_u8.red, final_u8.green, final_u8.blue, a]);
        canvas_pixel.blend(&dyed_pixel);
    });
}