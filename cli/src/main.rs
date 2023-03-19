mod progress;

use anyhow::*;
use colored::Colorize;
use desktop_dye_api::{
    config::DesktopDyeConfig,
    functions::{get_colors_from_screen, ToHex, ToRgb},
    models::colors::{HomeAssistantHsbColor, HsvColor},
};
use home_assistant_api::{HomeAssistantApi, HomeAssistantConfig};
use prisma::{Lerp, Rgb};
use progress::Progress;
use rand::Rng;
use screenshots::Screen;

const MAX_FAILURES: u8 = 3;

#[tokio::main]
async fn main() -> Result<()> {
    print_title();

    let config_path = DesktopDyeConfig::get_file_path()?.display().to_string();

    let mut p = Progress::new(format!("Checking config file (at {})", config_path).as_str());

    if !DesktopDyeConfig::exists()? {
        DesktopDyeConfig::create_default()?;
        p.success();
        println!(
            "No config file found. Created default config file at {}",
            config_path
        );
        return Err(anyhow!("DesktopDye cannot run without editing the config file. Please view and edit the config file and rerun."));
    }

    let config =
        DesktopDyeConfig::get().context(format!("Failed to get config at\n  {}", config_path))?;
    p.success();

    println!(
        "Color selection mode set to {}",
        config.mode.to_string().italic()
    );

    let mut p = Progress::new("Checking Home Assistant connection");
    let api = HomeAssistantApi::new(&HomeAssistantConfig::new(
        config.ha_endpoint.clone(),
        config.ha_token.clone(),
    ));
    if let Err(e) = api.get_status().await {
        p.fail();
        return Err(anyhow!(
            "Failed to connect to Home Assistant. Please check your config file at\n  {}\n\nError: {}",
            config_path,
            e
        ));
    }
    p.success();

    let mut p = Progress::new("Checking for screens");
    let screens = Screen::all().unwrap();
    if screens.is_empty() {
        p.fail();
        return Err(anyhow!(
            "No screens found. Please check your config file at\n  {}",
            config_path
        ));
    }
    p.success();
    println!("Found {} screens:", &screens.len());
    for screen in &screens {
        print!(
            "  - id: {} ({}x{})",
            screen.display_info.id, screen.display_info.width, screen.display_info.height
        );
        if screen.display_info.is_primary {
            print!(" (primary)");
        }
        println!();
    }

    let target_screen: &Screen;

    if let Some(screen_id) = config.screen_id {
        target_screen = screens
            .iter()
            .find(|screen| screen.display_info.id == screen_id)
            .ok_or_else(|| anyhow!("Failed to find screen with id {}", screen_id))?;
        println!("Using screen with id {} (from config)", screen_id);
    } else {
        target_screen = screens
            .iter()
            .find(|screen| screen.display_info.is_primary)
            .ok_or_else(|| anyhow!("Failed to find primary screen"))?;
        println!(
            "Using screen with id {} (the primary screen)",
            target_screen.display_info.id
        );
    }

    let mut failures = 0;
    let mut last_submission_time: std::time::Instant;
    let mut last_colors: Option<Vec<HsvColor>> = None;

    loop {
        last_submission_time = std::time::Instant::now();
        let res = capture_and_submit(&api, &config, target_screen, last_colors.as_ref()).await;
        if let Err(e) = res {
            failures += 1;
            if failures >= MAX_FAILURES {
                println!("Too many failures. Exiting...");
                return Err(e);
            }
            println!("Error: {}", e);
            println!("Retrying in 5 seconds...");
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        } else {
            last_colors = Some(res.unwrap());
            failures = 0;

            let seconds_remaining =
                config.capture_interval - last_submission_time.elapsed().as_secs_f64();
            if seconds_remaining > 0.0 {
                println!(
                    "Waiting {} second(s) for next capture ({:.2}s remaining)...",
                    config.capture_interval, seconds_remaining
                );
                tokio::time::sleep(std::time::Duration::from_secs_f64(seconds_remaining)).await;
            }
        }
    }
}

async fn capture_and_submit(
    api: &HomeAssistantApi,
    config: &DesktopDyeConfig,
    screen: &Screen,
    last_colors: Option<&Vec<HsvColor>>,
) -> Result<Vec<HsvColor>> {
    let mut p = Progress::new("Getting colors from screen");
    let colors_res = get_colors_from_screen(config, screen).await;
    if let Err(e) = colors_res {
        p.fail();
        return Err(e);
    }
    p.success();
    let colors = colors_res.unwrap();

    if let Some(last_colors) = last_colors {
        if last_colors == &colors {
            println!("Colors haven't changed, skipping submission");
            return Ok(colors);
        }
    }

    for (i, color) in colors.iter().enumerate() {
        let rgb_color = color.to_rgb();
        print!("  {}. ", i + 1);

        let hex = format!("#{}", color.to_hex());
        let formatted = match config.color_format {
            desktop_dye_api::config::ColorFormat::Rgb => {
                format!(
                    "RGB({}, {}, {})",
                    rgb_color.red(),
                    rgb_color.green(),
                    rgb_color.blue()
                )
            }
            desktop_dye_api::config::ColorFormat::Rgbb => {
                format!(
                    "RGBB({},{},{},{})",
                    rgb_color.red(),
                    rgb_color.green(),
                    rgb_color.blue(),
                    HomeAssistantHsbColor::from(*color).brightness
                )
            }
            desktop_dye_api::config::ColorFormat::Hsb => {
                format!(
                    "H: {:.3}°, S: {:.3}, B: {:.3}",
                    color.hue().0,
                    color.saturation(),
                    color.value(),
                )
            }
        };

        let foreground_color = if color.value() > 0.5 {
            colored::Color::Black
        } else {
            colored::Color::White
        };

        println!(
            "{}",
            format!("{} ({})", hex, formatted)
                .bold()
                .color(foreground_color)
                .on_truecolor(rgb_color.red(), rgb_color.green(), rgb_color.blue())
                .to_string()
        );

        // .bold()
        // .white()
        // .on_truecolor(rgb_color.red(), rgb_color.green(), rgb_color.blue())
        // .to_string()
    }

    let colors_payload = colors
        .clone()
        .into_iter()
        .map(|color| match config.color_format {
            desktop_dye_api::config::ColorFormat::Rgb => {
                let rgb = color.to_rgb();
                format!("{},{},{}", rgb.red(), rgb.green(), rgb.blue())
            }
            desktop_dye_api::config::ColorFormat::Rgbb => {
                let rgb = color.to_rgb();
                format!(
                    "{},{},{},{}",
                    rgb.red(),
                    rgb.green(),
                    rgb.blue(),
                    HomeAssistantHsbColor::from(color).brightness
                )
            }
            desktop_dye_api::config::ColorFormat::Hsb => {
                HomeAssistantHsbColor::from(color).to_desktop_dye_hsb_string()
            }
        })
        .collect::<Vec<_>>()
        .join(" ");

    println!(
        "Sending colors value ({}): \"{}\"",
        match config.color_format {
            desktop_dye_api::config::ColorFormat::Rgb => "RGB",
            desktop_dye_api::config::ColorFormat::Rgbb => "RGBB",
            desktop_dye_api::config::ColorFormat::Hsb => "HSB",
        },
        colors_payload
    );

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

    if let Err(e) = api_res {
        p.fail();
        return Err(e);
    }

    p.success();

    Ok(colors)
}

fn print_title() {
    const PACKAGE_NAME: &str = "DesktopDye";
    let mut possible_colors = vec![
        Rgb::new(152, 31, 172),
        Rgb::new(255, 0, 106),
        Rgb::new(0, 140, 255),
        Rgb::new(255, 140, 0),
    ];
    let mut rng = rand::thread_rng();
    let start_color = possible_colors.remove(rng.gen_range(0..possible_colors.len()));
    let end_color = possible_colors.remove(rng.gen_range(0..possible_colors.len()));

    let chars = PACKAGE_NAME.chars().collect::<Vec<_>>();
    let char_count = chars.len();
    let colors = (0..char_count)
        .map(|i| start_color.lerp(&end_color, i as f64 / char_count as f64))
        .collect::<Vec<_>>();

    let colored_package_name = chars
        .into_iter()
        .zip(colors.into_iter())
        .map(|(c, color)| {
            c.to_string()
                .on_truecolor(color.red(), color.green(), color.blue())
                .to_string()
        })
        .collect::<String>()
        .bold()
        .truecolor(255, 255, 255);

    println!(
        "\n{} v{}\n",
        colored_package_name,
        env!("CARGO_PKG_VERSION")
    );
}
