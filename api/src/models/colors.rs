use prisma::{Hsv, Rgb};

use crate::{
    config::ColorFormat,
    functions::{ToHsv, ToRgb},
};

pub type RgbColor = Rgb<u8>;
pub type HsvColor = Hsv<f64>;

pub trait DesktopDyePayload {
    fn to_desktop_dye_payload(&self) -> String;
}

pub struct HomeAssistantHsbColor {
    pub hue: f64,
    pub saturation: f64,
    pub brightness: f64,
}

pub trait DisplayForColorFormat {
    fn display_for_color_format(&self, color_format: &ColorFormat) -> String;
}

impl HomeAssistantHsbColor {
    pub fn new(hue: f64, saturation: f64, brightness: f64) -> Self {
        Self {
            hue,
            saturation,
            brightness,
        }
    }
}

impl DesktopDyePayload for HomeAssistantHsbColor {
    fn to_desktop_dye_payload(&self) -> String {
        format!(
            "{:.3},{:.3},{:.3}",
            self.hue, self.saturation, self.brightness
        )
    }
}

pub struct HomeAssistantRgbColor {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl HomeAssistantRgbColor {
    pub fn new(red: u8, green: u8, blue: u8) -> Self {
        Self { red, green, blue }
    }
}

impl DesktopDyePayload for HomeAssistantRgbColor {
    fn to_desktop_dye_payload(&self) -> String {
        format!("{},{},{}", self.red, self.green, self.blue)
    }
}

pub struct HomeAssistantRgbbColor {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub brightness: f64,
}

impl HomeAssistantRgbbColor {
    pub fn new(red: u8, green: u8, blue: u8, brightness: f64) -> Self {
        Self {
            red,
            green,
            blue,
            brightness,
        }
    }
}

impl DesktopDyePayload for HomeAssistantRgbbColor {
    fn to_desktop_dye_payload(&self) -> String {
        format!(
            "{},{},{},{:.3}",
            self.red, self.green, self.blue, self.brightness
        )
    }
}

impl<T> DisplayForColorFormat for T
where
    T: ToRgb,
{
    fn display_for_color_format(&self, color_format: &ColorFormat) -> String {
        let rgb = self.to_rgb();
        let hsv = rgb.to_hsv();

        match color_format {
            ColorFormat::Rgb => {
                format!("RGB({}, {}, {})", rgb.red(), rgb.green(), rgb.blue())
            }
            ColorFormat::Rgbb => {
                format!(
                    "RGBB({},{},{},{:.3})",
                    rgb.red(),
                    rgb.green(),
                    rgb.blue(),
                    HomeAssistantHsbColor::from(rgb).brightness
                )
            }
            ColorFormat::Hsb => {
                format!(
                    "H: {:.3}°, S: {:.3}, B: {:.3}",
                    hsv.hue().0,
                    hsv.saturation(),
                    hsv.value(),
                )
            }
        }
    }
}
