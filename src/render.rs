//! Rasterizes mobjects to pixels with tiny-skia (a pure-Rust, WASM-friendly
//! anti-aliased renderer).

use crate::geometry::{Vec2, F};
use crate::mobject::{Mobject, SubPath};
use tiny_skia::{FillRule, LineCap, LineJoin, Paint, PathBuilder, Pixmap, Stroke, Transform};

/// Maps world coordinates (origin-centered, y up) onto a pixel buffer.
#[derive(Clone, Copy, Debug)]
pub struct Camera {
    pub width: u32,
    pub height: u32,
    pub center: Vec2,
    /// World-space height of the visible frame.
    pub frame_height: F,
}

impl Camera {
    pub fn new(width: u32, height: u32, frame_height: F) -> Self {
        Camera {
            width,
            height,
            center: Vec2::ZERO,
            frame_height,
        }
    }

    pub fn scale(&self) -> F {
        self.height as F / self.frame_height
    }

    /// Half extent of the visible frame in world units: `(half_w, half_h)`.
    pub fn half_frame(&self) -> (F, F) {
        let hh = self.frame_height / 2.0;
        let hw = hh * self.width as F / self.height as F;
        (hw, hh)
    }

    pub fn world_to_px(&self, p: Vec2) -> (f32, f32) {
        let s = self.scale();
        let x = (p.x - self.center.x) * s + self.width as F / 2.0;
        let y = self.height as F / 2.0 - (p.y - self.center.y) * s;
        (x as f32, y as f32)
    }
}

fn build_path(subpaths: &[SubPath], cam: &Camera, force_closed: bool) -> Option<tiny_skia::Path> {
    let mut pb = PathBuilder::new();
    let mut any = false;
    for sp in subpaths {
        if sp.points.len() < 2 {
            continue;
        }
        let (x0, y0) = cam.world_to_px(sp.points[0]);
        pb.move_to(x0, y0);
        for p in &sp.points[1..] {
            let (x, y) = cam.world_to_px(*p);
            pb.line_to(x, y);
        }
        if sp.closed && force_closed {
            pb.close();
        }
        any = true;
    }
    if any {
        pb.finish()
    } else {
        None
    }
}

fn build_polylines(polys: &[Vec<Vec2>], cam: &Camera) -> Option<tiny_skia::Path> {
    let mut pb = PathBuilder::new();
    let mut any = false;
    for poly in polys {
        if poly.len() < 2 {
            continue;
        }
        let (x0, y0) = cam.world_to_px(poly[0]);
        pb.move_to(x0, y0);
        for p in &poly[1..] {
            let (x, y) = cam.world_to_px(*p);
            pb.line_to(x, y);
        }
        any = true;
    }
    if any {
        pb.finish()
    } else {
        None
    }
}

/// Return the first `frac` of the total path length as open world-space polylines.
fn truncate(paths: &[SubPath], frac: F) -> Vec<Vec<Vec2>> {
    let frac = frac.clamp(0.0, 1.0);
    let total: F = paths.iter().map(|p| p.length()).sum();
    if total < 1e-12 || frac <= 0.0 {
        return Vec::new();
    }
    let mut budget = total * frac;
    let mut out = Vec::new();
    for sp in paths {
        if budget <= 0.0 {
            break;
        }
        // Iterate edges of this subpath (closing edge included if closed).
        let mut pts: Vec<Vec2> = sp.points.clone();
        if sp.closed && pts.len() >= 2 {
            pts.push(sp.points[0]);
        }
        let mut current = vec![pts[0]];
        for w in pts.windows(2) {
            let seg = w[1] - w[0];
            let len = seg.length();
            if len <= budget {
                current.push(w[1]);
                budget -= len;
            } else {
                let t = budget / len;
                current.push(w[0] + seg * t);
                budget = 0.0;
                break;
            }
        }
        if current.len() >= 2 {
            out.push(current);
        }
        if budget <= 0.0 {
            break;
        }
    }
    out
}

/// Draw a single mobject onto the pixmap.
pub fn render_mobject(pixmap: &mut Pixmap, cam: &Camera, mob: &Mobject) {
    if mob.opacity <= 0.0 {
        return;
    }

    // Fill (fades in with `reveal`).
    if let Some(fc) = mob.style.fill {
        if let Some(path) = build_path(&mob.paths, cam, true) {
            let mut paint = Paint {
                anti_alias: true,
                ..Default::default()
            };
            paint.set_color(fc.to_skia(mob.opacity * mob.reveal));
            pixmap.fill_path(
                &path,
                &paint,
                FillRule::Winding,
                Transform::identity(),
                None,
            );
        }
    }

    // Stroke (draws on with `reveal`).
    if let Some(sc) = mob.style.stroke {
        if mob.style.stroke_width <= 0.0 {
            return;
        }
        let width_px = ((mob.style.stroke_width * cam.scale()) as f32).max(0.6);
        let stroke = Stroke {
            width: width_px,
            line_cap: LineCap::Round,
            line_join: LineJoin::Round,
            ..Default::default()
        };
        let mut paint = Paint {
            anti_alias: true,
            ..Default::default()
        };
        paint.set_color(sc.to_skia(mob.opacity));

        let path = if mob.reveal >= 0.999 {
            build_path(&mob.paths, cam, true)
        } else {
            let polys = truncate(&mob.paths, mob.reveal);
            build_polylines(&polys, cam)
        };
        if let Some(path) = path {
            pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color;

    #[test]
    fn renders_onto_pixmap() {
        let cam = Camera::new(80, 60, 8.0);
        let mut pm = Pixmap::new(80, 60).unwrap();
        pm.fill(color::DARK_BG.to_skia(1.0));
        let mut c = Mobject::circle(2.0);
        c.set_fill(color::RED);
        render_mobject(&mut pm, &cam, &c);
        // Some pixel should now be reddish near the center.
        let px = pm.pixel(40, 30).unwrap();
        assert!(px.red() > 100);
    }

    #[test]
    fn world_to_px_center() {
        let cam = Camera::new(100, 100, 10.0);
        let (x, y) = cam.world_to_px(Vec2::ZERO);
        assert!((x - 50.0).abs() < 1e-3 && (y - 50.0).abs() < 1e-3);
    }
}
