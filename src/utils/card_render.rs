use std::{collections::HashMap, sync::Arc, time::Instant};
use image::{DynamicImage, GenericImage, GenericImageView, ImageBuffer, ImageReader, Pixel, Rgba};
use log::warn;
use palette::{Srgb, Oklab, IntoColor};

use crate::{config::{CDN_CHARACTER_IMAGES_PATH, RENDER_TIMEOUT}, models::CardRenderRequestData};

use rayon::prelude::*;

pub fn render_card(data: &CardRenderRequestData, frames: &Arc<HashMap<String, DynamicImage>>, start_time: &Instant) -> Result<DynamicImage, String> {
    let image_path = if data.variant == 0 {
        format!("{}/{}.png", CDN_CHARACTER_IMAGES_PATH, data.id)
    } else {
        format!("{}/{}/{}{}.png", CDN_CHARACTER_IMAGES_PATH, data.id, if data.variant < 10 {'u'} else {'x'}, data.variant)
    };

    let character_image = match ImageReader::open(&image_path) {
        Ok(img) => match img.decode() {
            Ok(img) => img,
            Err(e) => {
                warn!("Failed render due to likely damaged character image on path: {}. Received error: {}", image_path, e);
                return Err(format!("failed request - failed to decode main image asset."))
            },
        }
        Err(e) => {
            warn!("Failed render due to missing character image on path: {}. Received error: {}", image_path, e);
            return Err(format!("failed request - requested card could not be rendered due to missing main image asset."));
        },
    };

    if start_time.elapsed().as_secs_f32() >= RENDER_TIMEOUT {
        return Err(format!("gateway timeout - asset render took more than {} seconds", RENDER_TIMEOUT));
    }
    let frame = data.frame_type.to_string();

    let (mask, decoration) = if data.kindled {
        (frames.get(&format!("{}-kindled-color", frame)), frames.get(&format!("{}-kindled-static", frame)))
    } else {
        (frames.get(&format!("{}-color", frame)), frames.get(&format!("{}-static", frame)))
    };

    if mask.is_none() {
        return Err(format!("failed request - \"{}\" frame type is invalid (doesn't exist)", frame));
    }

    if start_time.elapsed().as_secs_f32() >= RENDER_TIMEOUT {
        return Err(format!("gateway timeout - asset render took more than {} seconds", RENDER_TIMEOUT));
    }

    let mask = recolor_mask(mask.unwrap(), data.dye);
    let mut result = ImageBuffer::new(mask.width(), mask.height());

    if let Err(_) = result.copy_from(&character_image, 0, 0) {
        return Err(format!("server error - failed to copy character image onto final canvas"));
    };

    if start_time.elapsed().as_secs_f32() >= RENDER_TIMEOUT {
        return Err(format!("gateway timeout - asset render took more than {} seconds", RENDER_TIMEOUT));
    }


    if let Some(decoration) = decoration {
        result.par_enumerate_pixels_mut().for_each(|(x, y, p)| {
            let decoration_pixel = decoration.get_pixel(x, y);
            if decoration_pixel[3] == 0 {
                return
            }
            p.blend(&decoration_pixel);
        })
    }

    if start_time.elapsed().as_secs_f32() >= RENDER_TIMEOUT {
        return Err(format!("gateway timeout - asset render took more than {} seconds", RENDER_TIMEOUT));
    }

    result.par_enumerate_pixels_mut().for_each(|(x, y, p)| {
        let mask_pixel = mask.get_pixel(x, y);
        if mask_pixel[3] == 0 {
            return
        }
        p.blend(&mask_pixel);
    });

    if start_time.elapsed().as_secs_f32() >= RENDER_TIMEOUT {
        return Err(format!("gateway timeout - asset render took more than {} seconds", RENDER_TIMEOUT));
    }

    Ok(result.into())
}

fn recolor_mask(mask: &DynamicImage, dye: u32) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    // Dye int -> Oklab
    let overlay_rgb = Srgb::new(
        ((dye >> 16) & 0xFF) as f32 / 255.0,
        ((dye >> 8) & 0xFF) as f32 / 255.0,
        (dye & 0xFF) as f32 / 255.0,
    );

    let dye_lab: Oklab = overlay_rgb.into_linear().into_color();

    // Precomputed constants
    let light_blend_strength = 0.5;  // blend 50% toward dye lightness
    let blend_strength = 0.90; // blend 90% towards dye hue & saturation
    let inv_blend_strength = 1.0 - blend_strength;
    let chroma_boost_factor = 0.5;
    let max_chroma_squared = 1.0;

    let mut recolored = mask.to_rgba8();

    recolored.par_enumerate_pixels_mut().for_each(|(_, _, pixel)| {
        let (r, g, b, a) = (pixel[0], pixel[1], pixel[2], pixel[3]);

        if a == 0 {
            //*pixel = Rgba([0, 0, 0, 0]);
            return;
        }

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

        let final_rgb: Srgb<f32> = Srgb::from_linear(final_lab.into_color());
        let final_rgb = final_rgb.into_format::<u8>();

        *pixel = Rgba([final_rgb.red, final_rgb.green, final_rgb.blue, a]);
    });

    recolored
}