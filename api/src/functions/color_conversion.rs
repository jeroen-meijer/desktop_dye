use angular_units::Deg;
use prisma::{FromColor, Rgb};

use crate::models::colors::{
    HomeAssistantHsbColor, HomeAssistantRgbColor, HomeAssistantRgbbColor, HsvColor, RgbColor,
};

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

pub trait ToHexValue {
    fn to_hex_value(&self) -> String;
}

pub trait ToRgbVec {
    fn to_rgb_vec(&self) -> [u8; 3];
}

impl ToRgb for RgbColor {
    fn to_rgb(&self) -> RgbColor {
        *self
    }
}

impl ToRgb for Rgb<f64> {
    fn to_rgb(&self) -> RgbColor {
        RgbColor::new(
            f64_to_u8(self.red()),
            f64_to_u8(self.green()),
            f64_to_u8(self.blue()),
        )
    }
}

impl ToRgb for HomeAssistantRgbbColor {
    fn to_rgb(&self) -> RgbColor {
        RgbColor::new(self.red, self.green, self.blue)
    }
}

impl<T> ToHsv for T
where
    T: ToRgb,
{
    fn to_hsv(&self) -> HsvColor {
        let rgb = self.to_rgb();
        HsvColor::from_color(&Rgb::new(
            u8_to_f64(rgb.red()),
            u8_to_f64(rgb.green()),
            u8_to_f64(rgb.blue()),
        ))
    }
}

impl ToRgb for HsvColor {
    fn to_rgb(&self) -> RgbColor {
        Rgb::from_color(self).to_rgb()
    }
}

impl ToHsv for HomeAssistantHsbColor {
    fn to_hsv(&self) -> HsvColor {
        HsvColor::new(
            Deg(self.hue),
            self.saturation / 100.0,
            self.brightness / 100.0,
        )
    }
}

impl From<RgbColor> for HomeAssistantRgbColor {
    fn from(rgb: RgbColor) -> Self {
        Self::new(rgb.red(), rgb.green(), rgb.blue())
    }
}

impl From<HsvColor> for HomeAssistantRgbColor {
    fn from(hsv: HsvColor) -> Self {
        hsv.to_rgb().into()
    }
}

impl From<RgbColor> for HomeAssistantRgbbColor {
    fn from(rgb: RgbColor) -> Self {
        let hsv = rgb.to_hsv();
        Self::new(rgb.red(), rgb.green(), rgb.blue(), hsv.value() * 100.0)
    }
}

impl From<HsvColor> for HomeAssistantRgbbColor {
    fn from(hsv: HsvColor) -> Self {
        hsv.to_rgb().into()
    }
}

impl From<RgbColor> for HomeAssistantHsbColor {
    fn from(rgb: RgbColor) -> Self {
        rgb.to_hsv().into()
    }
}

impl From<HsvColor> for HomeAssistantHsbColor {
    fn from(hsv: HsvColor) -> Self {
        Self::new(
            round_float(3, hsv.hue().0),
            round_float(3, hsv.saturation() * 100.0),
            hsv.value() * 100.0,
        )
    }
}

impl<T> ToHexValue for T
where
    T: ToRgb,
{
    fn to_hex_value(&self) -> String {
        let rgb = self.to_rgb();

        format!("{:02x}{:02x}{:02x}", rgb.red(), rgb.green(), rgb.blue())
    }
}
