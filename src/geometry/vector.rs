use std::ops::{Mul, Sub};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Vector2f {
    pub x: f32,
    pub y: f32,
}

impl Vector2f {
    pub fn from_angle(angle: f32) -> Self {
        Self {
            x: angle.cos(),
            y: angle.sin(),
        }
    }

    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn angle(&self) -> f32 {
        self.y.atan2(self.x)
    }
}

impl Default for Vector2f {
    fn default() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
}

impl Sub for Vector2f {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl Mul<f32> for Vector2f {
    type Output = Self;

    fn mul(self, scalar: f32) -> Self {
        Self {
            x: self.x * scalar,
            y: self.y * scalar,
        }
    }
}