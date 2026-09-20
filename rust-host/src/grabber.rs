//! Framebuffer readback + image encoding. Rust mirror of `host/gl/FrameGrabber.java`.
//!
//! `glReadPixels` comes bottom-up, so rows are flipped; alpha is forced opaque so PNGs don't
//! leak transparency (Java parity). JPEG via `jpeg-encoder`, PNG via `png`.

use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use anyhow::{Context, Result};

use easy_imgui_renderer::glow::{self, HasContext as _};

/// RGBA8 readback of the bound framebuffer (already flipped top-down, alpha opaque).
pub struct Frame {
    pub data: Vec<u8>,
    pub width: i32,
    pub height: i32,
}

pub fn grab(gl: &glow::Context, width: i32, height: i32) -> Frame {
    let mut data = vec![0u8; (width.max(0) as usize) * (height.max(0) as usize) * 4];
    if data.is_empty() {
        return Frame {
            data,
            width,
            height,
        };
    }
    unsafe {
        gl.pixel_store_i32(glow::PACK_ALIGNMENT, 1);
        gl.read_pixels(
            0,
            0,
            width,
            height,
            glow::RGBA,
            glow::UNSIGNED_BYTE,
            glow::PixelPackData::Slice(Some(&mut data)),
        );
    }
    flip_and_opaque(&mut data, width as usize, height as usize);
    Frame {
        data,
        width,
        height,
    }
}

/// GL reads bottom-up; flip rows in place and force alpha to 0xFF (Java `flipAndOpaque`).
fn flip_and_opaque(data: &mut [u8], width: usize, height: usize) {
    let row_len = width * 4;
    for y in 0..height / 2 {
        let top = y * row_len;
        let bottom = (height - 1 - y) * row_len;
        for x in 0..row_len {
            data.swap(top + x, bottom + x);
        }
    }
    for px in data.chunks_exact_mut(4) {
        px[3] = 0xFF;
    }
}

pub fn jpeg(frame: &Frame, quality: u8) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    let encoder = jpeg_encoder::Encoder::new(&mut out, quality);
    encoder.encode(
        &frame.data,
        frame.width as u16,
        frame.height as u16,
        jpeg_encoder::ColorType::Rgba,
    )?;
    Ok(out)
}

pub fn png(frame: &Frame, path: &Path) -> Result<()> {
    let file = File::create(path).with_context(|| format!("create {}", path.display()))?;
    let mut encoder = png::Encoder::new(BufWriter::new(file), frame.width as u32, frame.height as u32);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header()?;
    writer.write_image_data(&frame.data)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flip_and_opaque_works() {
        // 2x2: first row (in GL memory = bottom) [1,1,1,1] [2,2,2,0], second row [3,3,3,3] [4,4,4,4]
        let mut data = vec![1, 1, 1, 1, 2, 2, 2, 0, 3, 3, 3, 3, 4, 4, 4, 4];
        flip_and_opaque(&mut data, 2, 2);
        // After flip: top row is what was second; all alpha forced opaque
        assert_eq!(&data[..8], &[3, 3, 3, 255, 4, 4, 4, 255]);
        assert_eq!(&data[8..], &[1, 1, 1, 255, 2, 2, 2, 255]);
    }

    #[test]
    fn jpeg_encodes() {
        let frame = Frame {
            data: vec![128; 40 * 10 * 4],
            width: 40,
            height: 10,
        };
        let jpeg = jpeg(&frame, 80).unwrap();
        assert!(jpeg.len() > 100); // real encoded data
        assert_eq!(&jpeg[..2], &[0xFF, 0xD8]); // JPEG SOI marker
    }
}
