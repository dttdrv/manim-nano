//! RGBA colors (components in `0.0..=1.0`) plus the Manim-style named palette.

use crate::geometry::{lerp, F};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color {
    pub r: F,
    pub g: F,
    pub b: F,
    pub a: F,
}

impl Color {
    pub const fn rgba(r: F, g: F, b: F, a: F) -> Self {
        Color { r, g, b, a }
    }

    pub const fn rgb(r: F, g: F, b: F) -> Self {
        Color::rgba(r, g, b, 1.0)
    }

    /// Parse a `#rrggbb` or `#rgb` hex string.
    pub fn hex(s: &str) -> Option<Color> {
        let s = s.trim().trim_start_matches('#');
        let parse = |sub: &str| u8::from_str_radix(sub, 16).ok().map(|n| n as F / 255.0);
        match s.len() {
            6 => Some(Color::rgb(
                parse(&s[0..2])?,
                parse(&s[2..4])?,
                parse(&s[4..6])?,
            )),
            3 => {
                let dup = |c: char| {
                    let h = format!("{c}{c}");
                    parse(&h)
                };
                let mut ch = s.chars();
                Some(Color::rgb(
                    dup(ch.next()?)?,
                    dup(ch.next()?)?,
                    dup(ch.next()?)?,
                ))
            }
            _ => None,
        }
    }

    pub fn with_opacity(mut self, a: F) -> Color {
        self.a = a;
        self
    }

    pub fn lerp(self, other: Color, t: F) -> Color {
        Color::rgba(
            lerp(self.r, other.r, t),
            lerp(self.g, other.g, t),
            lerp(self.b, other.b, t),
            lerp(self.a, other.a, t),
        )
    }

    /// Convert to a tiny-skia color, multiplying alpha by `alpha_mult`.
    pub fn to_skia(self, alpha_mult: F) -> tiny_skia::Color {
        let f = |c: F| (c.clamp(0.0, 1.0) * 255.0).round() as u8;
        tiny_skia::Color::from_rgba8(f(self.r), f(self.g), f(self.b), f(self.a * alpha_mult))
    }

    /// Look up a color by common English name (used by the natural-language frontend).
    pub fn from_name(name: &str) -> Option<Color> {
        let n = name.trim().to_ascii_lowercase();
        let c = match n.as_str() {
            "white" => WHITE,
            "black" => BLACK,
            "grey" | "gray" => GREY,
            "red" => RED,
            "green" => GREEN,
            "blue" => BLUE,
            "yellow" => YELLOW,
            "orange" => ORANGE,
            "purple" | "violet" => PURPLE,
            "pink" => PINK,
            "teal" | "cyan" => TEAL,
            "magenta" => MAGENTA,
            "maroon" => MAROON,
            "gold" => GOLD,
            "brown" => BROWN,
            _ => {
                if n.starts_with('#') {
                    return Color::hex(&n);
                }
                return None;
            }
        };
        Some(c)
    }
}

// Palette mirroring Manim's default colors closely enough to feel familiar.
pub const WHITE: Color = Color::rgb(1.0, 1.0, 1.0);
pub const BLACK: Color = Color::rgb(0.0, 0.0, 0.0);
pub const GREY: Color = Color::rgb(0.535, 0.535, 0.535);
pub const RED: Color = Color::rgb(0.988, 0.384, 0.333);
pub const GREEN: Color = Color::rgb(0.514, 0.792, 0.353);
pub const BLUE: Color = Color::rgb(0.345, 0.769, 0.871);
pub const YELLOW: Color = Color::rgb(1.0, 1.0, 0.0);
pub const ORANGE: Color = Color::rgb(1.0, 0.529, 0.184);
pub const PURPLE: Color = Color::rgb(0.616, 0.31, 0.659);
pub const PINK: Color = Color::rgb(0.819, 0.262, 0.623);
pub const TEAL: Color = Color::rgb(0.356, 0.808, 0.706);
pub const MAGENTA: Color = Color::rgb(0.78, 0.18, 0.6);
pub const MAROON: Color = Color::rgb(0.756, 0.341, 0.447);
pub const GOLD: Color = Color::rgb(0.953, 0.776, 0.353);
pub const BROWN: Color = Color::rgb(0.541, 0.357, 0.247);

/// The default dark background, like Manim's.
pub const DARK_BG: Color = Color::rgb(0.043, 0.043, 0.055);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_parsing() {
        assert_eq!(Color::hex("#ffffff"), Some(WHITE));
        assert_eq!(Color::hex("#000"), Some(BLACK));
        assert_eq!(Color::hex("nonsense"), None);
    }

    #[test]
    fn names() {
        assert_eq!(Color::from_name("Red"), Some(RED));
        assert!(Color::from_name("chartreuse").is_none());
    }
}
