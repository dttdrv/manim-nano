//! Coordinate systems — the heart of what Manim is loved for: **graphing
//! functions**. An [`Axes`] maps data coordinates to world space, draws axes
//! and (optionally) a grid, and turns a function `y = f(x)` into a curve
//! mobject you can `Create`/`Write` on screen.

use crate::color::{self, Color};
use crate::geometry::{Vec2, F};
use crate::mobject::{Mobject, SubPath};

/// An inclusive numeric range with a tick `step`.
#[derive(Clone, Copy, Debug)]
pub struct Range {
    pub min: F,
    pub max: F,
    pub step: F,
}

impl Range {
    pub fn new(min: F, max: F, step: F) -> Self {
        Range { min, max, step }
    }
    fn span(&self) -> F {
        let s = self.max - self.min;
        if s.abs() < 1e-12 {
            1.0
        } else {
            s
        }
    }
}

/// A 2D coordinate system: data ranges mapped onto a world-space rectangle.
#[derive(Clone, Copy, Debug)]
pub struct Axes {
    pub x: Range,
    pub y: Range,
    /// World-space width/height the axes occupy.
    pub x_length: F,
    pub y_length: F,
    /// World position of the center of the axes rectangle.
    pub center: Vec2,
    pub color: Color,
}

impl Axes {
    pub fn new(x: Range, y: Range) -> Self {
        Axes {
            x,
            y,
            x_length: 11.0,
            y_length: 6.0,
            center: Vec2::ZERO,
            color: color::GREY,
        }
    }

    /// A sensible default: x in [-5, 5], y in [-3, 3].
    pub fn default_axes() -> Self {
        Axes::new(Range::new(-5.0, 5.0, 1.0), Range::new(-3.0, 3.0, 1.0))
    }

    pub fn lengths(mut self, x_length: F, y_length: F) -> Self {
        self.x_length = x_length;
        self.y_length = y_length;
        self
    }
    pub fn centered_at(mut self, c: Vec2) -> Self {
        self.center = c;
        self
    }
    pub fn colored(mut self, c: Color) -> Self {
        self.color = c;
        self
    }

    /// Map data coordinates `(dx, dy)` to a world-space point.
    pub fn coords_to_point(&self, dx: F, dy: F) -> Vec2 {
        let tx = (dx - self.x.min) / self.x.span();
        let ty = (dy - self.y.min) / self.y.span();
        Vec2::new(
            self.center.x - self.x_length / 2.0 + tx * self.x_length,
            self.center.y - self.y_length / 2.0 + ty * self.y_length,
        )
    }

    fn x_axis_data_y(&self) -> F {
        0.0_f64.clamp(self.y.min, self.y.max)
    }
    fn y_axis_data_x(&self) -> F {
        0.0_f64.clamp(self.x.min, self.x.max)
    }

    /// The axes themselves: two lines with tick marks.
    pub fn axes_mobject(&self) -> Mobject {
        let mut paths = Vec::new();
        let tick = (self.x_length.min(self.y_length)) * 0.018 + 0.06;

        // x-axis along data y = clamp(0).
        let ay = self.x_axis_data_y();
        paths.push(SubPath::open(vec![
            self.coords_to_point(self.x.min, ay),
            self.coords_to_point(self.x.max, ay),
        ]));
        // y-axis along data x = clamp(0).
        let ax = self.y_axis_data_x();
        paths.push(SubPath::open(vec![
            self.coords_to_point(ax, self.y.min),
            self.coords_to_point(ax, self.y.max),
        ]));

        // Ticks.
        for value in steps(self.x) {
            let p = self.coords_to_point(value, ay);
            paths.push(SubPath::open(vec![
                Vec2::new(p.x, p.y - tick),
                Vec2::new(p.x, p.y + tick),
            ]));
        }
        for value in steps(self.y) {
            let p = self.coords_to_point(ax, value);
            paths.push(SubPath::open(vec![
                Vec2::new(p.x - tick, p.y),
                Vec2::new(p.x + tick, p.y),
            ]));
        }

        let mut m = Mobject::from_subpaths(paths);
        m.set_stroke(self.color, 0.03);
        m
    }

    /// A faint full grid (the "NumberPlane" look).
    pub fn grid_mobject(&self) -> Mobject {
        let mut paths = Vec::new();
        for value in steps(self.x) {
            paths.push(SubPath::open(vec![
                self.coords_to_point(value, self.y.min),
                self.coords_to_point(value, self.y.max),
            ]));
        }
        for value in steps(self.y) {
            paths.push(SubPath::open(vec![
                self.coords_to_point(self.x.min, value),
                self.coords_to_point(self.x.max, value),
            ]));
        }
        let mut m = Mobject::from_subpaths(paths);
        m.set_stroke(self.color.with_opacity(0.35), 0.02);
        m
    }

    /// Graph `y = f(x)` across the x-range as a curve mobject (default yellow).
    /// The curve is split wherever it leaves the y-range or `f` is non-finite,
    /// so vertical asymptotes and out-of-frame regions render cleanly.
    pub fn plot<G: Fn(F) -> F>(&self, f: G) -> Mobject {
        let samples = 400usize;
        let mut paths: Vec<SubPath> = Vec::new();
        let mut current: Vec<Vec2> = Vec::new();
        let pad = self.y.span() * 0.02;
        for i in 0..=samples {
            let dx = self.x.min + self.x.span() * i as F / samples as F;
            let dy = f(dx);
            let in_view = dy.is_finite() && dy >= self.y.min - pad && dy <= self.y.max + pad;
            if in_view {
                current.push(self.coords_to_point(dx, dy));
            } else if current.len() >= 2 {
                paths.push(SubPath::open(std::mem::take(&mut current)));
            } else {
                current.clear();
            }
        }
        if current.len() >= 2 {
            paths.push(SubPath::open(current));
        }
        if paths.is_empty() {
            // Nothing was in view; emit a degenerate point so the object is valid.
            paths.push(SubPath::open(vec![self.center, self.center]));
        }
        let mut m = Mobject::from_subpaths(paths);
        m.set_stroke(color::YELLOW, 0.05);
        m
    }

    /// Graph a parametric curve `t -> (x(t), y(t))` over `t_range`.
    pub fn plot_parametric<G: Fn(F) -> (F, F)>(&self, f: G, t_range: Range) -> Mobject {
        let samples = 500usize;
        let mut pts = Vec::new();
        for i in 0..=samples {
            let t = t_range.min + t_range.span() * i as F / samples as F;
            let (dx, dy) = f(t);
            if dx.is_finite() && dy.is_finite() {
                pts.push(self.coords_to_point(dx, dy));
            }
        }
        if pts.len() < 2 {
            pts = vec![self.center, self.center];
        }
        let mut m = Mobject::from_subpaths(vec![SubPath::open(pts)]);
        m.set_stroke(color::TEAL, 0.05);
        m
    }
}

/// Iterate the tick values of a range (inclusive), guarding against bad steps.
fn steps(r: Range) -> Vec<F> {
    let mut out = Vec::new();
    if r.step <= 1e-9 {
        return out;
    }
    let mut v = r.min;
    let mut guard = 0;
    while v <= r.max + 1e-9 && guard < 100_000 {
        out.push(v);
        v += r.step;
        guard += 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn origin_maps_to_center() {
        let ax = Axes::new(Range::new(-1.0, 1.0, 1.0), Range::new(-1.0, 1.0, 1.0));
        let p = ax.coords_to_point(0.0, 0.0);
        assert!(p.x.abs() < 1e-9 && p.y.abs() < 1e-9);
    }

    #[test]
    fn corners_map_to_rectangle() {
        let ax = Axes::default_axes().lengths(10.0, 6.0);
        let tr = ax.coords_to_point(5.0, 3.0);
        assert!((tr.x - 5.0).abs() < 1e-9 && (tr.y - 3.0).abs() < 1e-9);
    }

    #[test]
    fn plot_sine_has_geometry() {
        let ax = Axes::new(Range::new(-6.0, 6.0, 1.0), Range::new(-2.0, 2.0, 1.0));
        let curve = ax.plot(|x| x.sin());
        assert!(!curve.paths.is_empty());
        assert!(curve.total_length() > 1.0);
    }

    #[test]
    fn plot_handles_asymptote_without_panicking() {
        let ax = Axes::new(Range::new(-1.5, 1.5, 0.5), Range::new(-4.0, 4.0, 1.0));
        let curve = ax.plot(|x| x.tan());
        // tan has an asymptote in range; should split into multiple pieces.
        assert!(!curve.paths.is_empty());
    }

    #[test]
    fn axes_and_grid_build() {
        let ax = Axes::default_axes();
        assert!(!ax.axes_mobject().paths.is_empty());
        assert!(!ax.grid_mobject().paths.is_empty());
    }
}
