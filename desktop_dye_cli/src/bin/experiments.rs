use prisma::Rgb;

use colored::*;
use lab::Lab;
use screenshots::{Image, Screen};
use spinners::{Spinner, Spinners};
use std::{error::Error, time::Instant};

const NUM_COLORS: u8 = 5;

fn main() {
    println!("DesktopDye v{}", env!("CARGO_PKG_VERSION"));

    let start = Instant::now();
    let screens = Screen::all().unwrap();

    println!("Found {} screens in {:?}", &screens.len(), start.elapsed());
    for screen in &screens {
        let info = &screen.display_info;
        println!(
            "- {}: {}x{} ({} rotation)",
            info.id, info.width, info.height, info.rotation
        );
    }

    println!("Using primary screen");

    let target_screen: &Screen = screens
        .iter()
        .find(|screen| screen.display_info.is_primary)
        .expect("Failed to find primary screen");

    println!(
        "Using screen {} ({}x{})",
        target_screen.display_info.id,
        target_screen.display_info.width,
        target_screen.display_info.height
    );

    let mut s = create_spinner("Capturing screen");
    let (image, rgb_pixels) = capture_pixels(target_screen).unwrap();
    s.success();

    let mut s = create_spinner("Saving screenshot");
    let path = "target/screenshot.png";
    std::fs::write(path, image.buffer()).unwrap();
    s.success();
    println!("Saved screenshot to {}", path);
    println!();
    println!("Calculating color palettes ({} colors)", NUM_COLORS);

    struct ColorPalette<E>
    where
        E: std::error::Error,
    {
        name: String,
        colors: Result<Vec<(Rgb<u8>, Option<f32>)>, E>,
    }

    let mut palettes = vec![];

    let mut s = create_spinner("* color_thief");
    palettes.push(ColorPalette {
        name: "color_thief".to_string(),
        colors: {
            let colors_res = color_thief::get_palette(
                &rgb_pixels
                    .iter()
                    .flat_map(|rgb| vec![rgb.red(), rgb.green(), rgb.blue()])
                    .collect::<Vec<_>>(),
                color_thief::ColorFormat::Rgb,
                1,
                NUM_COLORS,
            );

            colors_res.map(|colors| {
                colors
                    .into_iter()
                    .map(|color| (Rgb::new(color.r, color.g, color.b), None))
                    .collect()
            })
        },
    });
    s.success();

    let mut s = create_spinner("* pigmnts");
    palettes.push(ColorPalette {
        name: "pigmnts".to_string(),
        colors: {
            // pigmnts::color::LAB::from_rgb(r, g, b)
            let lab_values = rgb_pixels
                .iter()
                .map(|rgb| pigmnts::color::LAB::from_rgb(rgb.red(), rgb.green(), rgb.blue()))
                .collect::<Vec<_>>();

            let colors_res = pigmnts::pigments_pixels(
                &lab_values,
                NUM_COLORS,
                pigmnts::weights::resolve_mood(&pigmnts::weights::Mood::Dominant),
                None,
            );

            Ok(colors_res
                .into_iter()
                .map(|(color, dominance)| {
                    // LAB has no to_rgb function. We need to do it manually
                    let [r, g, b] = Lab::to_rgb(&Lab {
                        l: color.l,
                        a: color.a,
                        b: color.b,
                    });
                    let rgb = Rgb::new(r, g, b);

                    (rgb, Some(dominance))
                })
                .collect::<Vec<_>>())
        },
    });
    s.success();

    /* API reference for kmeans_color

    pub fn get_kmeans<C: Calculate + Clone>(
        k: usize,                           // Number of clusters: the number of colors to return
        max_iter: usize,                // Maximum number of iterations: the algorithm will stop after this number of iterations even if it has not converged
        converge: f32,                 // Convergence threshold: the algorithm will stop when the difference between the previous and the current iteration is less than this threshold
        verbose: bool,                // Print debug info
        buf: &[C],                   // Buffer of colors
        seed: u64,                  // Seed for random number generator
    )
     */

    for palette in palettes {
        print!("{}: ", palette.name);
        let colors_res = palette.colors;

        let Ok(mut colors) = colors_res else {
            println!("failed to get palette: {:?}", colors_res.unwrap_err());
            println!();
            continue;
        };

        colors.sort_by(|(_, confidence_a), (_, confidence_b)| {
            confidence_b.partial_cmp(confidence_a).unwrap()
        });

        let colors_concat = colors
            .iter()
            .map(|(c, _)| c.to_hex().to_lowercase())
            .collect::<Vec<_>>()
            .join("-");

        println!("https://coolors.co/{}", colors_concat);

        for color in colors.into_iter() {
            let (color, dominance) = color;
            let (r, g, b) = (color.red(), color.green(), color.blue());
            let line = "  - ".to_string()
                + format!("#{} ({}, {}, {})", color.to_hex(), r, g, b)
                    .white()
                    .on_truecolor(r, g, b)
                    .to_string()
                    .as_str();
            print!("{}", line);

            if let Some(dominance) = dominance {
                print!(" ({:.2}%)", dominance * 100.0);
            }

            println!();
        }

        println!();
    }
}

// Any caller of this function must use the result
#[must_use = "creating a spinner without a way to stop it is useless"]
fn create_spinner<'a>(prompt: &'a str) -> Progress {
    Progress::new(prompt)
}

pub struct Progress {
    prompt: String,
    start_time: Instant,
    spinner: Spinner,
}

impl Progress {
    pub fn new(prompt: &str) -> Self {
        Self {
            prompt: prompt.into(),
            start_time: Instant::now(),
            spinner: Spinner::new(Spinners::Dots9, prompt.to_string() + "..."),
        }
    }

    pub fn success(&mut self) {
        self.spinner.stop_with_message(format!(
            "✅ {} (took {:2}ms)",
            self.prompt,
            self.start_time.elapsed().as_micros() / 1000
        ));
    }

    pub fn fail(&mut self) {
        self.spinner.stop_with_message(format!(
            "❌ {} (took {:2}ms)",
            self.prompt,
            self.start_time.elapsed().as_micros() / 1000
        ));
    }
}

fn capture_pixels(screen: &Screen) -> Result<(Image, Vec<Rgb<u8>>), Box<dyn Error>> {
    let image = screen.capture()?;
    let raw_png_buffer = image.buffer();

    let decoder = png::Decoder::new(&raw_png_buffer[..]);
    let mut reader = decoder.read_info()?;
    let mut pixel_buffer = vec![0; reader.output_buffer_size()];
    // Read the next frame. An APNG might contain multiple frames.
    let info = reader.next_frame(&mut pixel_buffer)?;

    if info.bit_depth != png::BitDepth::Eight {
        return Err("Only 8-bit PNGs are supported".into());
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
        _ => return Err("Only RGB and RGBA PNGs are supported".into()),
    }

    Ok((
        image,
        rgb_bytes
            .chunks_exact(4)
            .map(|chunk| Rgb::new(chunk[0], chunk[1], chunk[2]))
            .collect::<Vec<_>>(),
    ))
}

trait ToHex {
    fn to_hex(&self) -> String;
}

impl ToHex for Rgb<u8> {
    fn to_hex(&self) -> String {
        format!("{:02X}{:02X}{:02X}", self.red(), self.green(), self.blue())
    }
}
