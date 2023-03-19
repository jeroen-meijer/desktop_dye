use crate::config::ColorSelectionMode;
use crate::config::DesktopDyeConfig;
use crate::functions::*;
use crate::models::colors::HsvColor;
use angular_units::Deg;
use anyhow::*;
use colored::Colorize;
use prisma::Hsv;
use screenshots::Screen;

const BRIGHTNESS_THRESHOLD: f64 = 0.80;

pub async fn get_colors_from_screen(
    config: &DesktopDyeConfig,
    screen: &Screen,
) -> Result<Vec<HsvColor>> {
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
            let mut hsv = color.to_hsv();

            hsv.set_saturation((hsv.saturation() + 0.2).min(1.0));
            hsv.set_value((hsv.value() + 0.2).min(1.0));

            hsv
        })
        .collect::<Vec<_>>();

    let most_dominant_color = dominant_colors[0];
    println!(
        "Dominant color: {}",
        format!("#{}", most_dominant_color.to_hex_value())
            .bold()
            .white()
            .on_truecolor(
                most_dominant_color.to_rgb().red(),
                most_dominant_color.to_rgb().green(),
                most_dominant_color.to_rgb().blue()
            )
            .to_string()
    );

    Ok(apply_color_correction(
        dominant_colors,
        &config.mode,
        &config.hue_shift,
        &config.brightness_factor,
    ))
}

fn apply_color_correction(
    colors: Vec<HsvColor>,
    mode: &ColorSelectionMode,
    hue_shift: &f64,
    brightness_factor: &f64,
) -> Vec<HsvColor> {
    let mode_adjusted_colors = match mode {
        crate::config::ColorSelectionMode::Default => colors,
        crate::config::ColorSelectionMode::Brightness => {
            let bright_colors = colors
                .iter()
                .filter(|color| color.value() > BRIGHTNESS_THRESHOLD)
                .cloned()
                .collect::<Vec<_>>();

            let primary_color = bright_colors
                .first()
                .map(|color| color.clone())
                .unwrap_or_else(|| {
                    let colors = &mut colors.clone();
                    colors.sort_by(|a, b| b.value().partial_cmp(&a.value()).unwrap());

                    colors[0]
                })
                .clone();

            let mut final_colors = vec![primary_color];
            final_colors.extend(
                colors
                    .iter()
                    .filter(|color| *color != &primary_color)
                    .collect::<Vec<_>>(),
            );

            final_colors
        }
        crate::config::ColorSelectionMode::HueShift => {
            let primary_hsv = colors[0];

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

                final_colors.push(hsv);
            }

            final_colors
        }
    };

    if brightness_factor == &1.0 {
        return mode_adjusted_colors;
    }

    mode_adjusted_colors
        .into_iter()
        .map(|color| {
            let mut hsv = color;
            hsv.set_value((hsv.value() * brightness_factor).min(1.0));
            hsv
        })
        .collect()
}
