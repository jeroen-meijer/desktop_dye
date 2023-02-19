mod config;
mod functions;
mod progress;

use anyhow::*;
use colored::Colorize;
use config::DesktopDyeConfig;
use functions::*;
use home_assistant_api::{HomeAssistantApi, HomeAssistantConfig};
use prisma::Rgb;
use progress::Progress;
use screenshots::Screen;

const NUM_COLORS: u8 = 3;
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
    let mut last_colors: Option<Vec<Rgb<u8>>> = None;

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
            println!(
                "Waiting {} second(s) for next capture ({:.2}s remaining)...",
                config.capture_interval, seconds_remaining
            );
            tokio::time::sleep(std::time::Duration::from_secs_f64(seconds_remaining)).await;
        }
    }
}
