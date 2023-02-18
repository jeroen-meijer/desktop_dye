use chrono::{Datelike, Timelike};
use colored::Colorize;
use spinners::{Spinner, Spinners};
use std::time::Instant;

pub struct Progress {
    prompt: String,
    start_time: Instant,
    spinner: Spinner,
}

impl Progress {
    fn get_current_date_time_str() -> String {
        let now = chrono::Local::now();
        format!(
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
            now.year(),
            now.month(),
            now.day(),
            now.hour(),
            now.minute(),
            now.second()
        )
    }

    pub fn new(prompt: &str) -> Self {
        let now = Instant::now();

        let initial_prompt = format!("[{}] {}...", Self::get_current_date_time_str(), prompt);
        Self {
            prompt: prompt.into(),
            start_time: now,
            spinner: Spinner::new(Spinners::Dots, initial_prompt),
        }
    }

    pub fn success(&mut self) {
        let prompt = format!("[{}] {}", Self::get_current_date_time_str(), self.prompt);

        self.spinner.stop_with_message(format!(
            "{} {} (took {:.2}ms)",
            "✔".green(),
            prompt,
            self.start_time.elapsed().as_micros() / 1000
        ));
    }

    pub fn fail(&mut self) {
        let prompt = format!("[{}] {}", Self::get_current_date_time_str(), self.prompt).bold();

        self.spinner.stop_with_message(format!(
            "{} {} (took {:.2}ms)",
            "✘".red(),
            prompt,
            self.start_time.elapsed().as_micros() / 1000
        ));
    }
}
