use prisma::{FromColor, Hsv, Rgb};

const MAX_RGB_VALUE: f64 = std::u8::MAX as f64;

pub fn rgb_to_hsv(rgb: &Rgb<u8>) -> Hsv<f64> {
    Hsv::from_color(&Rgb::new(
        rgb.red() as f64 / MAX_RGB_VALUE,
        rgb.green() as f64 / MAX_RGB_VALUE,
        rgb.blue() as f64 / MAX_RGB_VALUE,
    ))
}

pub fn hsv_to_rgb(hsv: &Hsv<f64>) -> Rgb<u8> {
    rgb_f64_to_rgb_u8(&Rgb::from_color(hsv))
}

pub fn rgb_f64_to_rgb_u8(rgb: &Rgb<f64>) -> Rgb<u8> {
    Rgb::new(
        (rgb.red() * MAX_RGB_VALUE) as u8,
        (rgb.green() * MAX_RGB_VALUE) as u8,
        (rgb.blue() * MAX_RGB_VALUE) as u8,
    )
}

pub trait ToHex {
    fn to_hex(&self) -> String;
}

pub trait ToRgbVec {
    fn to_rgb_vec(&self) -> [u8; 3];
}

impl ToHex for Rgb<u8> {
    fn to_hex(&self) -> String {
        format!("{:02x}{:02x}{:02x}", self.red(), self.green(), self.blue())
    }
}

impl ToRgbVec for Rgb<u8> {
    fn to_rgb_vec(&self) -> [u8; 3] {
        [self.red(), self.green(), self.blue()]
    }
}

impl ToHex for Rgb<f64> {
    fn to_hex(&self) -> String {
        rgb_f64_to_rgb_u8(self).to_hex()
    }
}

impl ToRgbVec for Rgb<f64> {
    fn to_rgb_vec(&self) -> [u8; 3] {
        rgb_f64_to_rgb_u8(self).to_rgb_vec()
    }
}

impl ToHex for Hsv<f64> {
    fn to_hex(&self) -> String {
        hsv_to_rgb(self).to_hex()
    }
}
