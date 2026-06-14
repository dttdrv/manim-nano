//! 2D vector math, world directions, and animation rate (easing) functions.

use std::ops::{Add, Mul, Neg, Sub};

/// The scalar type used throughout the engine.
pub type F = f64;

/// A 2D point / vector in **world** space (y points up, like in Manim).
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Vec2 {
    pub x: F,
    pub y: F,
}

impl Vec2 {
    pub const ZERO: Vec2 = Vec2 { x: 0.0, y: 0.0 };

    pub const fn new(x: F, y: F) -> Self {
        Vec2 { x, y }
    }

    pub fn length(self) -> F {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn normalized(self) -> Vec2 {
        let l = self.length();
        if l < 1e-12 {
            Vec2::ZERO
        } else {
            self * (1.0 / l)
        }
    }

    /// Rotate counter-clockwise by `angle` radians about the origin.
    pub fn rotate(self, angle: F) -> Vec2 {
        let (s, c) = angle.sin_cos();
        Vec2::new(self.x * c - self.y * s, self.x * s + self.y * c)
    }

    pub fn lerp(self, other: Vec2, t: F) -> Vec2 {
        self + (other - self) * t
    }
}

impl Add for Vec2 {
    type Output = Vec2;
    fn add(self, o: Vec2) -> Vec2 {
        Vec2::new(self.x + o.x, self.y + o.y)
    }
}
impl Sub for Vec2 {
    type Output = Vec2;
    fn sub(self, o: Vec2) -> Vec2 {
        Vec2::new(self.x - o.x, self.y - o.y)
    }
}
impl Mul<F> for Vec2 {
    type Output = Vec2;
    fn mul(self, s: F) -> Vec2 {
        Vec2::new(self.x * s, self.y * s)
    }
}
impl Neg for Vec2 {
    type Output = Vec2;
    fn neg(self) -> Vec2 {
        Vec2::new(-self.x, -self.y)
    }
}

/// Construct a [`Vec2`] tersely: `v(1.0, 2.0)`.
pub fn v(x: F, y: F) -> Vec2 {
    Vec2::new(x, y)
}

// Manim-style world-direction constants (one world unit in each direction).
pub const ORIGIN: Vec2 = Vec2::new(0.0, 0.0);
pub const UP: Vec2 = Vec2::new(0.0, 1.0);
pub const DOWN: Vec2 = Vec2::new(0.0, -1.0);
pub const LEFT: Vec2 = Vec2::new(-1.0, 0.0);
pub const RIGHT: Vec2 = Vec2::new(1.0, 0.0);
pub const UL: Vec2 = Vec2::new(-1.0, 1.0);
pub const UR: Vec2 = Vec2::new(1.0, 1.0);
pub const DL: Vec2 = Vec2::new(-1.0, -1.0);
pub const DR: Vec2 = Vec2::new(1.0, -1.0);

/// Linear interpolation between two scalars.
pub fn lerp(a: F, b: F, t: F) -> F {
    a + (b - a) * t
}

/// Animation rate functions (a.k.a. easing). They map a time fraction
/// `t in [0, 1]` to an eased fraction, mirroring Manim's `rate_functions`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Rate {
    Linear,
    /// Smooth ease-in-out (Manim's default `smooth`).
    #[default]
    Smooth,
    /// Accelerate from rest, end at full speed.
    RushInto,
    /// Start at full speed, decelerate to rest.
    RushFrom,
    /// Go to the target and return (ping-pong).
    ThereAndBack,
}

impl Rate {
    pub fn apply(self, t: F) -> F {
        let t = t.clamp(0.0, 1.0);
        match self {
            Rate::Linear => t,
            Rate::Smooth => smooth(t),
            Rate::RushInto => 2.0 * smooth(t / 2.0),
            Rate::RushFrom => 2.0 * smooth(t / 2.0 + 0.5) - 1.0,
            Rate::ThereAndBack => {
                if t < 0.5 {
                    smooth(2.0 * t)
                } else {
                    smooth(2.0 * (1.0 - t))
                }
            }
        }
    }
}

/// Smootherstep: zero first and second derivatives at the endpoints.
fn smooth(t: F) -> F {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rotate_quarter_turn() {
        let r = RIGHT.rotate(std::f64::consts::FRAC_PI_2);
        assert!((r.x - 0.0).abs() < 1e-9);
        assert!((r.y - 1.0).abs() < 1e-9);
    }

    #[test]
    fn rate_endpoints() {
        for r in [Rate::Linear, Rate::Smooth, Rate::RushInto, Rate::RushFrom] {
            assert!((r.apply(0.0) - 0.0).abs() < 1e-9);
            assert!((r.apply(1.0) - 1.0).abs() < 1e-9);
        }
    }
}
