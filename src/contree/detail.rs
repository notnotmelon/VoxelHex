use crate::{contree::types::Albedo};
use num_traits::Zero;
use std::ops::{Add, Div};

//####################################################################################
//     █████████   █████       ███████████  ██████████ ██████████      ███████
//   ███░░░░░███ ░░███       ░░███░░░░░███░░███░░░░░█░░███░░░░███   ███░░░░░███
//  ░███    ░███  ░███        ░███    ░███ ░███  █ ░  ░███   ░░███ ███     ░░███
//  ░███████████  ░███        ░██████████  ░██████    ░███    ░███░███      ░███
//  ░███░░░░░███  ░███        ░███░░░░░███ ░███░░█    ░███    ░███░███      ░███
//  ░███    ░███  ░███      █ ░███    ░███ ░███ ░   █ ░███    ███ ░░███     ███
//  █████   █████ ███████████ ███████████  ██████████ ██████████   ░░░███████░
// ░░░░░   ░░░░░ ░░░░░░░░░░░ ░░░░░░░░░░░  ░░░░░░░░░░ ░░░░░░░░░░      ░░░░░░░
//####################################################################################

impl Albedo {
    pub fn with_red(mut self, r: u8) -> Self {
        self.r = r;
        self
    }

    pub fn with_green(mut self, g: u8) -> Self {
        self.g = g;
        self
    }

    pub fn with_blue(mut self, b: u8) -> Self {
        self.b = b;
        self
    }

    pub fn with_alpha(mut self, a: u8) -> Self {
        self.a = a;
        self
    }

    pub fn is_transparent(&self) -> bool {
        self.a == 0
    }

    pub fn distance_from(&self, other: &Albedo) -> f32 {
        let distance_r = self.r as f32 - other.r as f32;
        let distance_g = self.g as f32 - other.g as f32;
        let distance_b = self.b as f32 - other.b as f32;
        let distance_a = self.a as f32 - other.a as f32;
        (distance_r.powf(2.) + distance_g.powf(2.) + distance_b.powf(2.) + distance_a.powf(2.))
            .sqrt()
    }
}

impl From<u32> for Albedo {
    fn from(value: u32) -> Self {
        let a = (value & 0x000000FF) as u8;
        let b = ((value & 0x0000FF00) >> 8) as u8;
        let g = ((value & 0x00FF0000) >> 16) as u8;
        let r = ((value & 0xFF000000) >> 24) as u8;

        Albedo::default()
            .with_red(r)
            .with_green(g)
            .with_blue(b)
            .with_alpha(a)
    }
}

impl Add for Albedo {
    type Output = Albedo;
    fn add(self, other: Albedo) -> Albedo {
        Albedo {
            r: self.r + other.r,
            g: self.g + other.g,
            b: self.b + other.b,
            a: self.a + other.a,
        }
    }
}

impl Div<f32> for Albedo {
    type Output = Albedo;
    fn div(self, divisor: f32) -> Albedo {
        Albedo {
            r: (self.r as f32 / divisor).round() as u8,
            g: (self.g as f32 / divisor).round() as u8,
            b: (self.b as f32 / divisor).round() as u8,
            a: (self.a as f32 / divisor).round() as u8,
        }
    }
}

impl Zero for Albedo {
    fn zero() -> Self {
        Self {
            r: 0,
            g: 0,
            b: 0,
            a: 0,
        }
    }
    fn is_zero(&self) -> bool {
        self.r == 0 && self.g == 0 && self.b == 0 && self.a == 0
    }
}

