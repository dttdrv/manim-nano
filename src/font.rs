//! A self-contained bitmap font (no system fonts, no font files) so that
//! `Text` works identically on every platform — native or WebAssembly.
//!
//! Each glyph is an 8x8 bitmap; every lit pixel becomes a small square polygon.
//! Text therefore becomes ordinary vector geometry that fills, strokes,
//! transforms, and "writes on" like any other mobject.

use crate::geometry::{Vec2, F};
use font8x8::UnicodeFonts;

/// Build the geometry for a string as a list of closed square polygons,
/// laid out left to right and centered on the origin.
///
/// `cap_height` is the world-space height of a glyph cell.
pub fn text_polygons(text: &str, cap_height: F) -> Vec<Vec<Vec2>> {
    let px = cap_height / 8.0; // size of one bitmap pixel in world units
    let advance = 8.0 * px; // monospace cell width
    let total_width = advance * text.chars().count().max(1) as F;
    let x0 = -total_width / 2.0;
    let y_top = cap_height / 2.0;

    let mut polys = Vec::new();
    for (i, ch) in text.chars().enumerate() {
        let glyph = match font8x8::BASIC_FONTS.get(ch) {
            Some(g) => g,
            None => continue, // unsupported glyph -> blank
        };
        let gx = x0 + i as F * advance;
        for (row, bits) in glyph.iter().enumerate() {
            for col in 0..8 {
                if (bits >> col) & 1 == 1 {
                    // bit 0 = leftmost column, row 0 = top row.
                    let left = gx + col as F * px;
                    let top = y_top - row as F * px;
                    polys.push(vec![
                        Vec2::new(left, top),
                        Vec2::new(left + px, top),
                        Vec2::new(left + px, top - px),
                        Vec2::new(left, top - px),
                    ]);
                }
            }
        }
    }
    polys
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_some_pixels() {
        let polys = text_polygons("AB", 1.0);
        assert!(!polys.is_empty());
        // every polygon is a square
        assert!(polys.iter().all(|p| p.len() == 4));
    }

    #[test]
    fn blank_for_unknown() {
        // A space has no lit pixels.
        assert!(text_polygons(" ", 1.0).is_empty());
    }
}
