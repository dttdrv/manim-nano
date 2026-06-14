//! Mobjects: drawable objects made of vector subpaths plus a style.
//!
//! Like Manim's `VMobject`, geometry is stored as absolute points, so
//! transforms bake into the points and animations can interpolate them.

use crate::color::{self, Color};
use crate::font;
use crate::geometry::{Vec2, F};
use std::f64::consts::TAU;

/// One continuous subpath (a polyline). `closed` joins the last point to the first.
#[derive(Clone, Debug)]
pub struct SubPath {
    pub points: Vec<Vec2>,
    pub closed: bool,
}

impl SubPath {
    pub fn open(points: Vec<Vec2>) -> Self {
        SubPath {
            points,
            closed: false,
        }
    }
    pub fn closed(points: Vec<Vec2>) -> Self {
        SubPath {
            points,
            closed: true,
        }
    }

    /// Total arc length of the polyline (including the closing edge if closed).
    pub fn length(&self) -> F {
        if self.points.len() < 2 {
            return 0.0;
        }
        let mut total = 0.0;
        for w in self.points.windows(2) {
            total += (w[1] - w[0]).length();
        }
        if self.closed {
            total += (self.points[0] - *self.points.last().unwrap()).length();
        }
        total
    }
}

/// Stroke + fill styling.
#[derive(Clone, Copy, Debug)]
pub struct Style {
    pub fill: Option<Color>,
    pub stroke: Option<Color>,
    /// Stroke width in **world** units.
    pub stroke_width: F,
}

impl Default for Style {
    fn default() -> Self {
        Style {
            fill: None,
            stroke: Some(color::WHITE),
            stroke_width: 0.04,
        }
    }
}

/// A drawable object.
#[derive(Clone, Debug)]
pub struct Mobject {
    pub paths: Vec<SubPath>,
    pub style: Style,
    /// Fraction of the object that is "drawn on" (used by Create/Write). 0..=1.
    pub reveal: F,
    /// Overall opacity multiplier. 0..=1.
    pub opacity: F,
}

impl Mobject {
    fn from_paths(paths: Vec<SubPath>) -> Self {
        Mobject {
            paths,
            style: Style::default(),
            reveal: 1.0,
            opacity: 1.0,
        }
    }

    /// Build a mobject directly from raw subpaths (used by coordinate systems
    /// and anyone constructing custom geometry).
    pub fn from_subpaths(paths: Vec<SubPath>) -> Self {
        Self::from_paths(paths)
    }

    /// Merge several mobjects into one by concatenating their subpaths. The
    /// merged object takes the **first** mobject's style, so this is a
    /// lightweight "group" for objects that share styling (axis labels, ticks,
    /// a set of same-colored marks). Empty input yields an empty mobject.
    pub fn merged(mobs: Vec<Mobject>) -> Self {
        let style = mobs.first().map(|m| m.style).unwrap_or_default();
        let mut paths = Vec::new();
        for m in mobs {
            paths.extend(m.paths);
        }
        let mut out = Self::from_paths(paths);
        out.style = style;
        out
    }

    // ---- Constructors ---------------------------------------------------

    /// A circle of the given radius, centered at the origin (outline only).
    pub fn circle(radius: F) -> Self {
        Self::ellipse(radius, radius)
    }

    pub fn ellipse(rx: F, ry: F) -> Self {
        let n = 96;
        let pts = (0..n)
            .map(|i| {
                let a = i as F / n as F * TAU;
                Vec2::new(rx * a.cos(), ry * a.sin())
            })
            .collect();
        let mut m = Self::from_paths(vec![SubPath::closed(pts)]);
        m.style.stroke = Some(color::BLUE);
        m
    }

    /// A square with the given side length, centered at the origin.
    pub fn square(side: F) -> Self {
        Self::rectangle(side, side)
    }

    pub fn rectangle(w: F, h: F) -> Self {
        let (hw, hh) = (w / 2.0, h / 2.0);
        let pts = vec![
            Vec2::new(-hw, -hh),
            Vec2::new(hw, -hh),
            Vec2::new(hw, hh),
            Vec2::new(-hw, hh),
        ];
        Self::from_paths(vec![SubPath::closed(pts)])
    }

    /// A regular `n`-gon inscribed in a circle of the given radius.
    pub fn regular_polygon(n: usize, radius: F) -> Self {
        let n = n.max(3);
        let pts = (0..n)
            .map(|i| {
                let a = i as F / n as F * TAU + std::f64::consts::FRAC_PI_2;
                Vec2::new(radius * a.cos(), radius * a.sin())
            })
            .collect();
        Self::from_paths(vec![SubPath::closed(pts)])
    }

    pub fn triangle(radius: F) -> Self {
        Self::regular_polygon(3, radius)
    }

    pub fn line(a: Vec2, b: Vec2) -> Self {
        Self::from_paths(vec![SubPath::open(vec![a, b])])
    }

    /// A small filled dot (defaults to white fill).
    pub fn dot(center: Vec2, radius: F) -> Self {
        let mut m = Self::circle(radius);
        for p in &mut m.paths[0].points {
            *p = *p + center;
        }
        m.style.fill = Some(color::WHITE);
        m.style.stroke = None;
        m
    }

    /// An arrow from `a` to `b` (a line plus a filled arrowhead).
    pub fn arrow(a: Vec2, b: Vec2) -> Self {
        let dir = (b - a).normalized();
        let perp = Vec2::new(-dir.y, dir.x);
        let head = 0.25;
        let tip = b;
        let base = b - dir * head;
        let left = base + perp * (head * 0.5);
        let right = base - perp * (head * 0.5);
        let shaft = SubPath::open(vec![a, base]);
        let arrowhead = SubPath::closed(vec![tip, left, right]);
        let mut m = Self::from_paths(vec![shaft, arrowhead]);
        m.style.fill = Some(color::WHITE);
        m
    }

    pub fn polygon(points: Vec<Vec2>) -> Self {
        Self::from_paths(vec![SubPath::closed(points)])
    }

    /// Text rendered with the built-in bitmap font (fill only).
    pub fn text(content: &str, height: F) -> Self {
        let polys = font::text_polygons(content, height);
        let paths = polys.into_iter().map(SubPath::closed).collect();
        let mut m = Self::from_paths(paths);
        m.style.fill = Some(color::WHITE);
        m.style.stroke = None;
        m
    }

    // ---- Style (mutating, chainable) ------------------------------------

    pub fn set_fill(&mut self, c: Color) -> &mut Self {
        self.style.fill = Some(c);
        self
    }
    pub fn set_stroke(&mut self, c: Color, width: F) -> &mut Self {
        self.style.stroke = Some(c);
        self.style.stroke_width = width;
        self
    }
    /// Set both stroke and (if present) fill to `c`, like Manim's `set_color`.
    pub fn set_color(&mut self, c: Color) -> &mut Self {
        if self.style.stroke.is_some() {
            self.style.stroke = Some(c);
        }
        if self.style.fill.is_some() {
            self.style.fill = Some(c);
        }
        // If a text/dot object only has fill, recolor it.
        if self.style.stroke.is_none() && self.style.fill.is_some() {
            self.style.fill = Some(c);
        }
        self
    }
    pub fn set_opacity(&mut self, a: F) -> &mut Self {
        self.opacity = a;
        self
    }

    // ---- Style (by value, for one-liners) -------------------------------

    pub fn colored(mut self, c: Color) -> Self {
        self.set_color(c);
        self
    }
    pub fn filled(mut self, c: Color) -> Self {
        self.set_fill(c);
        self
    }
    pub fn at(mut self, p: Vec2) -> Self {
        self.move_to(p);
        self
    }

    // ---- Geometry / transforms ------------------------------------------

    fn for_each_point(&mut self, mut f: impl FnMut(Vec2) -> Vec2) {
        for sp in &mut self.paths {
            for p in &mut sp.points {
                *p = f(*p);
            }
        }
    }

    /// Axis-aligned bounding box as `(min, max)`.
    pub fn bbox(&self) -> (Vec2, Vec2) {
        let mut min = Vec2::new(F::INFINITY, F::INFINITY);
        let mut max = Vec2::new(F::NEG_INFINITY, F::NEG_INFINITY);
        for sp in &self.paths {
            for p in &sp.points {
                min.x = min.x.min(p.x);
                min.y = min.y.min(p.y);
                max.x = max.x.max(p.x);
                max.y = max.y.max(p.y);
            }
        }
        if !min.x.is_finite() {
            (Vec2::ZERO, Vec2::ZERO)
        } else {
            (min, max)
        }
    }

    /// Geometric center (bounding-box center).
    pub fn center(&self) -> Vec2 {
        let (min, max) = self.bbox();
        (min + max) * 0.5
    }

    pub fn shift(&mut self, delta: Vec2) -> &mut Self {
        self.for_each_point(|p| p + delta);
        self
    }

    pub fn move_to(&mut self, dest: Vec2) -> &mut Self {
        let delta = dest - self.center();
        self.shift(delta)
    }

    pub fn scale_about(&mut self, factor: F, about: Vec2) -> &mut Self {
        self.for_each_point(|p| about + (p - about) * factor);
        self
    }
    pub fn scale(&mut self, factor: F) -> &mut Self {
        let c = self.center();
        self.scale_about(factor, c)
    }

    pub fn rotate_about(&mut self, angle: F, about: Vec2) -> &mut Self {
        self.for_each_point(|p| about + (p - about).rotate(angle));
        self
    }
    pub fn rotate(&mut self, angle: F) -> &mut Self {
        let c = self.center();
        self.rotate_about(angle, c)
    }

    /// Move so this object's edge sits against the frame edge in `direction`.
    /// `frame` is `(half_width, half_height)` of the camera in world units.
    pub fn to_edge(&mut self, direction: Vec2, frame: (F, F), buff: F) -> &mut Self {
        let (min, max) = self.bbox();
        let c = self.center();
        let mut delta = Vec2::ZERO;
        if direction.x < 0.0 {
            delta.x = -frame.0 + buff - min.x;
        } else if direction.x > 0.0 {
            delta.x = frame.0 - buff - max.x;
        }
        if direction.y < 0.0 {
            delta.y = -frame.1 + buff - min.y;
        } else if direction.y > 0.0 {
            delta.y = frame.1 - buff - max.y;
        }
        let _ = c;
        self.shift(delta)
    }

    /// Place this object next to `other` in the given direction.
    pub fn next_to(&mut self, other: &Mobject, direction: Vec2, buff: F) -> &mut Self {
        let (omin, omax) = other.bbox();
        let (smin, smax) = self.bbox();
        let ocenter = (omin + omax) * 0.5;
        let scenter = (smin + smax) * 0.5;
        let mut dest = ocenter;
        if direction.x != 0.0 {
            let half = (smax.x - smin.x) / 2.0 + (omax.x - omin.x) / 2.0 + buff;
            dest.x = ocenter.x + direction.x.signum() * half;
        }
        if direction.y != 0.0 {
            let half = (smax.y - smin.y) / 2.0 + (omax.y - omin.y) / 2.0 + buff;
            dest.y = ocenter.y + direction.y.signum() * half;
        }
        let delta = dest - scenter;
        self.shift(delta)
    }

    /// Total path length across all subpaths (used for the "draw-on" effect).
    pub fn total_length(&self) -> F {
        self.paths.iter().map(|p| p.length()).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::{v, RIGHT};

    #[test]
    fn circle_centered_radius() {
        let c = Mobject::circle(2.0);
        let (min, max) = c.bbox();
        assert!((max.x - 2.0).abs() < 1e-6 && (min.x + 2.0).abs() < 1e-6);
        assert!(c.center().length() < 1e-6);
    }

    #[test]
    fn shift_moves_center() {
        let mut s = Mobject::square(1.0);
        s.shift(RIGHT * 3.0);
        assert!((s.center() - v(3.0, 0.0)).length() < 1e-9);
    }

    #[test]
    fn scale_grows_bbox() {
        let mut s = Mobject::square(1.0);
        let before = s.bbox();
        s.scale(2.0);
        let after = s.bbox();
        assert!((after.1.x - before.1.x * 2.0).abs() < 1e-9);
    }

    #[test]
    fn text_has_geometry() {
        let t = Mobject::text("HI", 1.0);
        assert!(!t.paths.is_empty());
        assert!(t.style.fill.is_some());
    }
}
