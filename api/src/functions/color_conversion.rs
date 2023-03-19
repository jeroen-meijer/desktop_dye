use angular_units::Deg;
use prisma::{FromColor, Hsv, Rgb};

use crate::models::colors::{HomeAssistantHsbColor, HsvColor, RgbColor};

const MAX_RGB_VALUE: f64 = std::u8::MAX as f64;

pub fn round_float(decimals: u8, value: f64) -> f64 {
    let rounding_carrier = 10_u64.pow(decimals as u32) as f64;
    (value * rounding_carrier).round() / rounding_carrier
}

pub fn u8_to_f64(component: u8) -> f64 {
    match component {
        0 => 0.0,
        std::u8::MAX => 1.0,
        _ => component as f64 / MAX_RGB_VALUE,
    }
}

pub fn f64_to_u8(component: f64) -> u8 {
    if component <= 0.0 {
        0
    } else if component >= 1.0 {
        std::u8::MAX
    } else {
        (component * MAX_RGB_VALUE) as u8
    }
}

pub trait ToRgb {
    fn to_rgb(&self) -> RgbColor;
}

pub trait ToHsv {
    fn to_hsv(&self) -> HsvColor;
}

pub trait ToHex {
    fn to_hex(&self) -> String;
}

pub trait ToRgbVec {
    fn to_rgb_vec(&self) -> [u8; 3];
}

impl ToRgb for Rgb<f64> {
    fn to_rgb(&self) -> RgbColor {
        Rgb::new(
            f64_to_u8(self.red()),
            f64_to_u8(self.green()),
            f64_to_u8(self.blue()),
        )
    }
}

impl ToHsv for RgbColor {
    fn to_hsv(&self) -> HsvColor {
        Hsv::from_color(&Rgb::new(
            u8_to_f64(self.red()),
            u8_to_f64(self.green()),
            u8_to_f64(self.blue()),
        ))
    }
}

impl ToHsv for Rgb<f64> {
    fn to_hsv(&self) -> HsvColor {
        self.to_rgb().to_hsv()
    }
}

impl ToRgb for HsvColor {
    fn to_rgb(&self) -> RgbColor {
        Rgb::from_color(self).to_rgb()
    }
}

impl ToHex for RgbColor {
    fn to_hex(&self) -> String {
        format!("{:02x}{:02x}{:02x}", self.red(), self.green(), self.blue())
    }
}

impl ToRgbVec for RgbColor {
    fn to_rgb_vec(&self) -> [u8; 3] {
        [self.red(), self.green(), self.blue()]
    }
}

impl ToHex for Rgb<f64> {
    fn to_hex(&self) -> String {
        self.to_rgb().to_hex()
    }
}

impl ToRgbVec for Rgb<f64> {
    fn to_rgb_vec(&self) -> [u8; 3] {
        self.to_rgb().to_rgb_vec()
    }
}

impl ToHex for HsvColor {
    fn to_hex(&self) -> String {
        self.to_rgb().to_hex()
    }
}

impl From<RgbColor> for HomeAssistantHsbColor {
    fn from(rgb: RgbColor) -> Self {
        HomeAssistantHsbColor::from(rgb.to_hsv())
    }
}

impl From<HsvColor> for HomeAssistantHsbColor {
    fn from(hsv: HsvColor) -> Self {
        HomeAssistantHsbColor::new(
            round_float(3, hsv.hue().0),
            round_float(3, hsv.saturation() * 100.0),
            f64_to_u8(hsv.value()),
        )
    }
}

impl ToRgb for HomeAssistantHsbColor {
    fn to_rgb(&self) -> RgbColor {
        self.to_hsv().to_rgb()
    }
}

impl ToHsv for HomeAssistantHsbColor {
    fn to_hsv(&self) -> HsvColor {
        Hsv::new(
            Deg(self.hue),
            self.saturation / 100.0,
            u8_to_f64(self.brightness),
        )
    }
}
