use crate::config::ColorSelectionMode;
use crate::config::DesktopDyeConfig;
use crate::functions::*;
use angular_units::Deg;
use anyhow::*;
use colored::Colorize;
use prisma::Hsv;
use prisma::Rgb;
use screenshots::Screen;

const BRIGHTNESS_THRESHOLD: f64 = 0.80;

pub async fn get_colors_from_screen(
    config: &DesktopDyeConfig,
    screen: &Screen,
) -> Result<Vec<Rgb<u8>>> {
    let pixels = capture_pixels(screen).context("Failed to capture screen")?;
    let dominant_colors =
        calculate_dominant_colors(&pixels, &config.algorithm, &config.sample_size);
    if dominant_colors.is_empty() {
        return Err(anyhow!(
            "Failed to calculate dominant colors, got 0 results"
        ));
    }

    let dominant_colors = dominant_colors
        .into_iter()
        .map(|color| {
            let mut hsv = rgb_to_hsv(&color);

            hsv.set_saturation((hsv.saturation() + 0.2).min(1.0));
            hsv.set_value((hsv.value() + 0.2).min(1.0));

            hsv_to_rgb(&hsv)
        })
        .collect::<Vec<_>>();

    let most_dominant_color = dominant_colors[0];
    println!(
        "Dominant color: {}",
        format!("#{}", most_dominant_color.to_hex())
            .bold()
            .white()
            .on_truecolor(
                most_dominant_color.red(),
                most_dominant_color.green(),
                most_dominant_color.blue()
            )
            .to_string()
    );

    Ok(apply_color_mode(
        dominant_colors,
        &config.mode,
        &config.hue_shift,
    ))
}

fn apply_color_mode(
    colors: Vec<Rgb<u8>>,
    mode: &ColorSelectionMode,
    hue_shift: &f64,
) -> Vec<Rgb<u8>> {
    match mode {
        crate::config::ColorSelectionMode::Default => colors,
        crate::config::ColorSelectionMode::Brightness => {
            let primary_color = colors
                .iter()
                .map(rgb_to_hsv)
                .filter(|color| color.value() > BRIGHTNESS_THRESHOLD)
                .collect::<Vec<_>>()
                .first()
                .map(hsv_to_rgb)
                .unwrap_or_else(|| {
                    let colors = &mut colors.clone();
                    colors.sort_by(|a, b| {
                        rgb_to_hsv(b)
                            .value()
                            .partial_cmp(&rgb_to_hsv(a).value())
                            .unwrap()
                    });

                    colors[0]
                });

            let mut final_colors = vec![primary_color];
            final_colors.extend(
                colors
                    .iter()
                    .filter(|color| color != &&primary_color)
                    .cloned()
                    .collect::<Vec<_>>(),
            );

            final_colors
        }
        crate::config::ColorSelectionMode::HueShift => {
            let primary_hsv = rgb_to_hsv(&colors[0]);

            let colors_len = colors.len();
            if colors_len == 1 {
                return colors;
            }

            let lower_hue = primary_hsv.hue().0 - hue_shift;
            let hue_step = hue_shift * 2.0 / (colors_len - 1) as f64;

            let mut final_colors = vec![];

            for i in 0..colors_len {
                let hue = lower_hue + hue_step * i as f64;
                let hue = if hue < 0.0 {
                    hue + 360.0
                } else if hue > 360.0 {
                    hue - 360.0
                } else {
                    hue
                };

                let hsv = Hsv::new(Deg(hue), primary_hsv.saturation(), primary_hsv.value());

                final_colors.push(hsv_to_rgb(&hsv));
            }

            final_colors
        }
    }
}
