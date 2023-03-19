use anyhow::*;
use prisma::Rgb;
use screenshots::Screen;

use crate::models::colors::RgbColor;

pub fn capture_pixels(screen: &Screen) -> Result<Vec<RgbColor>> {
    let image = screen.capture()?;
    let raw_png_buffer = image.buffer();

    let decoder = png::Decoder::new(&raw_png_buffer[..]);
    let mut reader = decoder.read_info()?;
    let mut pixel_buffer = vec![0; reader.output_buffer_size()];
    // Read the next frame. An APNG might contain multiple frames.
    let info = reader.next_frame(&mut pixel_buffer)?;

    if info.bit_depth != png::BitDepth::Eight {
        return Err(anyhow!(
            "Only 8-bit PNGs are supported, but the bit depth of the screen capture is {:?}",
            info.bit_depth
        ));
    }

    let mut rgb_bytes = pixel_buffer[..info.buffer_size()].to_vec();
    match info.color_type {
        png::ColorType::Rgb => {}
        png::ColorType::Rgba => {
            rgb_bytes = rgb_bytes
                .chunks_exact(4)
                .flat_map(|chunk| &chunk[..3])
                .copied()
                .collect::<Vec<_>>();
        }
        _ => {
            return Err(anyhow!(
                "Only RGB and RGBA PNGs screen captures are supported, but the color type is {:?}",
                info.color_type
            ))
        }
    }

    Ok(rgb_bytes
        .chunks_exact(4)
        .map(|chunk| Rgb::new(chunk[0], chunk[1], chunk[2]))
        .collect::<Vec<_>>())
}
