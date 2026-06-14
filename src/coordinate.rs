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
        safe_clamp(0.0, self.y.min, self.y.max)
    }
    fn y_axis_data_x(&self) -> F {
        safe_clamp(0.0, self.x.min, self.x.max)
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

impl Axes {
    /// The world-space point on the graph of `f` at data `x`.
    pub fn point_at_x<G: Fn(F) -> F>(&self, f: G, x: F) -> Vec2 {
        self.coords_to_point(x, f(x))
    }

    /// A short tangent line to `f` at data `x` (slope via finite differences).
    pub fn tangent_line<G: Fn(F) -> F>(&self, f: G, x: F, half_len: F) -> Mobject {
        let h = 1e-4;
        let slope = (f(x + h) - f(x - h)) / (2.0 * h);
        let y = f(x);
        let mut m = if slope.is_finite() && y.is_finite() {
            let a = self.coords_to_point(x - half_len, y - slope * half_len);
            let b = self.coords_to_point(x + half_len, y + slope * half_len);
            Mobject::from_subpaths(vec![SubPath::open(vec![a, b])])
        } else {
            // Undefined/asymptotic slope: emit no geometry rather than NaN points.
            Mobject::from_subpaths(Vec::new())
        };
        m.set_stroke(color::RED, 0.04);
        m
    }

    /// Riemann rectangles approximating the area under `f` on `[x0, x1]`
    /// (midpoint rule). The classic calculus visualization.
    pub fn riemann_rectangles<G: Fn(F) -> F>(
        &self,
        f: G,
        x0: F,
        x1: F,
        n: usize,
        fill: Color,
    ) -> Mobject {
        let n = n.max(1);
        let baseline = safe_clamp(0.0, self.y.min, self.y.max);
        let dx = (x1 - x0) / n as F;
        let mut paths = Vec::new();
        for i in 0..n {
            let xl = x0 + i as F * dx;
            let xr = xl + dx;
            let xm = xl + dx * 0.5;
            let h = f(xm);
            if !h.is_finite() {
                continue;
            }
            let top = safe_clamp(h, self.y.min, self.y.max);
            paths.push(SubPath::closed(vec![
                self.coords_to_point(xl, baseline),
                self.coords_to_point(xr, baseline),
                self.coords_to_point(xr, top),
                self.coords_to_point(xl, top),
            ]));
        }
        if paths.is_empty() {
            paths.push(SubPath::closed(vec![self.center, self.center, self.center]));
        }
        let mut m = Mobject::from_subpaths(paths);
        m.set_fill(fill.with_opacity(0.45));
        m.set_stroke(fill, 0.02);
        m
    }

    /// The filled region under `f` between `x0` and `x1` (signed, relative to
    /// the axis baseline).
    pub fn area_under<G: Fn(F) -> F>(&self, f: G, x0: F, x1: F, fill: Color) -> Mobject {
        let baseline = safe_clamp(0.0, self.y.min, self.y.max);
        let samples = 240usize;
        let mut pts = Vec::new();
        for i in 0..=samples {
            let x = x0 + (x1 - x0) * i as F / samples as F;
            let y = f(x);
            if y.is_finite() {
                pts.push(self.coords_to_point(x, safe_clamp(y, self.y.min, self.y.max)));
            }
        }
        pts.push(self.coords_to_point(x1, baseline));
        pts.push(self.coords_to_point(x0, baseline));
        if pts.len() < 3 {
            pts = vec![self.center, self.center, self.center];
        }
        let mut m = Mobject::from_subpaths(vec![SubPath::closed(pts)]);
        m.set_fill(fill.with_opacity(0.4));
        m.set_stroke(fill, 0.02);
        m
    }

    /// Numeric tick labels along both axes (the origin label is omitted to keep
    /// it uncluttered). Returns one merged, axis-colored mobject.
    pub fn number_labels(&self) -> Mobject {
        let ay = self.x_axis_data_y();
        let ax = self.y_axis_data_x();
        let size = self.x_length.min(self.y_length) * 0.04 + 0.18;
        let mut labels = Vec::new();
        for value in steps(self.x) {
            if value.abs() < 1e-9 {
                continue;
            }
            let p = self.coords_to_point(value, ay);
            let mut t = Mobject::text(&fmt_num(value), size);
            t.set_fill(self.color);
            t.move_to(Vec2::new(p.x, p.y - size * 1.2));
            labels.push(t);
        }
        for value in steps(self.y) {
            if value.abs() < 1e-9 {
                continue;
            }
            let p = self.coords_to_point(ax, value);
            let mut t = Mobject::text(&fmt_num(value), size);
            t.set_fill(self.color);
            t.move_to(Vec2::new(p.x - size * 1.6, p.y));
            labels.push(t);
        }
        if labels.is_empty() {
            return Mobject::from_subpaths(Vec::new());
        }
        Mobject::merged(labels)
    }
}

/// `f64::clamp` panics if `min > max`; this tolerates inverted bounds.
fn safe_clamp(v: F, a: F, b: F) -> F {
    if a <= b {
        v.clamp(a, b)
    } else {
        v.clamp(b, a)
    }
}

/// Format a tick value compactly (integers without a decimal point).
fn fmt_num(v: F) -> String {
    if (v - v.round()).abs() < 1e-6 {
        format!("{}", v.round() as i64)
    } else {
        format!("{v:.1}")
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

    #[test]
    fn riemann_and_area_build() {
        let pi = std::f64::consts::PI;
        let ax = Axes::new(Range::new(0.0, pi, 1.0), Range::new(0.0, 1.2, 0.5));
        let rects = ax.riemann_rectangles(|x| x.sin(), 0.0, pi, 10, color::GREEN);
        assert_eq!(rects.paths.len(), 10);
        let area = ax.area_under(|x| x.sin(), 0.0, pi, color::BLUE);
        assert!(area.style.fill.is_some());
    }

    #[test]
    fn number_labels_and_tangent_build() {
        let ax = Axes::default_axes();
        let labels = ax.number_labels();
        assert!(!labels.paths.is_empty()); // ticks at -5..5, -3..3 minus origin
        let tan = ax.tangent_line(|x| x * x, 1.0, 1.0);
        assert!(tan.total_length() > 0.0);
    }

    #[test]
    fn inverted_range_does_not_panic() {
        // Defensive: a user could pass min > max; we must not panic.
        let ax = Axes::new(Range::new(5.0, -5.0, 1.0), Range::new(3.0, -3.0, 1.0));
        let _ = ax.axes_mobject();
        let _ = ax.riemann_rectangles(|x| x, -5.0, 5.0, 5, color::RED);
        let _ = ax.area_under(|x| x, -5.0, 5.0, color::RED);
    }
}
