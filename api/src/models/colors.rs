use prisma::{Hsv, Rgb};

pub type RgbColor = Rgb<u8>;
pub type HsvColor = Hsv<f64>;
pub struct HomeAssistantHsbColor {
    pub hue: f64,
    pub saturation: f64,
    pub brightness: u8,
}

impl HomeAssistantHsbColor {
    pub fn new(hue: f64, saturation: f64, brightness: u8) -> Self {
        Self {
            hue,
            saturation,
            brightness,
        }
    }

    pub fn to_desktop_dye_hsb_string(&self) -> String {
        format!("{:.3},{:.3},{}", self.hue, self.saturation, self.brightness)
    }
}
