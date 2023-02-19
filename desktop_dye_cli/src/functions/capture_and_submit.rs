use crate::config::ColorSelectionMode;
use crate::config::DesktopDyeConfig;
use crate::functions::*;
use crate::progress::*;
use angular_units::Deg;
use anyhow::*;
use colored::Colorize;
use home_assistant_api::HomeAssistantApi;
use prisma::Hsv;
use prisma::Rgb;
use screenshots::Screen;

const BRIGHTNESS_THRESHOLD: f64 = 0.80;

pub async fn capture_and_submit(
    api: &HomeAssistantApi,
    config: &DesktopDyeConfig,
    screen: &Screen,
    last_colors: Option<&Vec<Rgb<u8>>>,
) -> Result<Vec<Rgb<u8>>> {
    let mut p = Progress::new("Capturing screen");
    let pixels = capture_pixels(screen).context("Failed to capture screen");
    if let Err(e) = pixels {
        p.fail();
        return Err(e);
    }
    let pixels = pixels.unwrap();
    p.success();

    let mut p = Progress::new("Calculating dominant colors");
    let dominant_colors =
        calculate_dominant_colors(&pixels, super::DominantColorAlgorithm::ColorThief);
    if dominant_colors.is_empty() {
        p.fail();
        return Err(anyhow!(
            "Failed to calculate dominant colors, got 0 results"
        ));
    }
    p.success();

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

    let final_colors = apply_color_mode(dominant_colors, &config.mode, &config.hue_shift);

    println!("Final colors:");
    for color in &final_colors {
        println!(
            "  - {}",
            format!(
                "#{} ({}, {}, {})",
                color.to_hex(),
                color.red(),
                color.green(),
                color.blue(),
            )
            .bold()
            .white()
            .on_truecolor(color.red(), color.green(), color.blue())
            .to_string()
        );
    }

    if let Some(last_colors) = last_colors {
        if last_colors == &final_colors {
            println!("Colors haven't changed, skipping submission");
            return Ok(final_colors);
        }
    }

    let colors_payload = &final_colors
        .iter()
        .map(|color| format!("{},{},{}", color.red(), color.green(), color.blue()))
        .collect::<Vec<_>>()
        .join(" ");

    println!("Sending colors value: \"{}\"", colors_payload);

    let mut p = Progress::new("Submitting colors to Home Assistant");
    let api_res = api
        .set_state(
            config.ha_target_entity_id.to_owned(),
            colors_payload.clone(),
            None,
            true,
        )
        .await
        .context("Failed to submit colors to Home Assistant");

    // let base_data: DataMap = hashmap! {
    //   "rgb_color".into() => target_color.to_rgb_vec().iter().map(|v| serde_json::Value::Number((*v).into())).collect::<Vec<_>>().into(),
    //   "transition".into() => 1.into(),
    // };
    // for entity_id in target_entity_ids {
    //     let mut data = base_data.clone();
    //     data.insert("entity_id".into(), entity_id.into());

    //     let api_res = api
    //         .call_services("light".into(), "turn_on".into(), Some(data))
    //         .await
    //         .context(format!(
    //             "Failed to submit colors to Home Assistant for entity {}",
    //             entity_id
    //         ));
    // }

    if let Err(e) = api_res {
        p.fail();
        return Err(e);
    }

    p.success();

    Ok(final_colors)
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
