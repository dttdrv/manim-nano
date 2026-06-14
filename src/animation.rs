//! Animations.
//!
//! [`AnimSpec`] is the ergonomic description you hand to `Scene::play`
//! (e.g. `create(id)`, `shift(id, LEFT)`). The scene resolves each spec into a
//! concrete [`Anim`] by snapshotting the target's current state, then samples
//! it across the run time.

use crate::color::Color;
use crate::geometry::{Vec2, F};
use crate::mobject::{Mobject, SubPath};

pub type Id = usize;

/// A high-level, declarative animation request.
#[derive(Clone, Debug)]
pub enum AnimSpec {
    /// Draw the object's outline on, fading in its fill.
    Create(Id),
    /// Reverse of `Create`; removes the object when finished.
    Uncreate(Id),
    /// Like `Create` (the natural choice for text).
    Write(Id),
    /// Fade in, optionally drifting in from `Vec2` away.
    FadeIn(Id, Vec2),
    /// Fade out (and remove), optionally drifting `Vec2` away.
    FadeOut(Id, Vec2),
    /// Scale up from a point.
    GrowFromCenter(Id),
    /// Translate by a delta.
    Shift(Id, Vec2),
    /// Translate so the center lands on a point.
    MoveTo(Id, Vec2),
    /// Multiply the current size by a factor.
    Scale(Id, F),
    /// Rotate by an angle (radians) about the center.
    Rotate(Id, F),
    /// Recolor.
    SetColor(Id, Color),
    /// Morph the object's geometry & style into `Mobject`.
    Transform(Id, Box<Mobject>),
    /// Hold the current frame.
    Wait,
}

impl AnimSpec {
    pub fn target(&self) -> Option<Id> {
        match self {
            AnimSpec::Create(i)
            | AnimSpec::Uncreate(i)
            | AnimSpec::Write(i)
            | AnimSpec::FadeIn(i, _)
            | AnimSpec::FadeOut(i, _)
            | AnimSpec::GrowFromCenter(i)
            | AnimSpec::Shift(i, _)
            | AnimSpec::MoveTo(i, _)
            | AnimSpec::Scale(i, _)
            | AnimSpec::Rotate(i, _)
            | AnimSpec::SetColor(i, _)
            | AnimSpec::Transform(i, _) => Some(*i),
            AnimSpec::Wait => None,
        }
    }
}

/// A resolved animation that owns the snapshots it needs to render.
#[derive(Clone, Debug)]
pub enum Anim {
    Morph {
        id: Id,
        start: Mobject,
        end: Mobject,
    },
    Rotate {
        id: Id,
        start: Mobject,
        pivot: Vec2,
        angle: F,
    },
    Scale {
        id: Id,
        start: Mobject,
        about: Vec2,
        factor: F,
    },
    Create {
        id: Id,
        start: Mobject,
    },
    Uncreate {
        id: Id,
        start: Mobject,
    },
    FadeIn {
        id: Id,
        start: Mobject,
        shift: Vec2,
    },
    FadeOut {
        id: Id,
        start: Mobject,
        shift: Vec2,
    },
    Grow {
        id: Id,
        start: Mobject,
    },
    Wait,
}

/// What to do with the target once the animation completes.
pub enum Outcome {
    Keep(Id, Mobject),
    Remove(Id),
    Nothing,
}

impl Anim {
    /// Build a concrete animation from a spec and the target's current state.
    pub fn resolve(spec: &AnimSpec, current: Option<&Mobject>) -> Anim {
        let cur = || current.cloned().unwrap_or_else(|| Mobject::circle(0.0));
        match spec {
            AnimSpec::Create(id) => Anim::Create {
                id: *id,
                start: cur(),
            },
            AnimSpec::Write(id) => Anim::Create {
                id: *id,
                start: cur(),
            },
            AnimSpec::Uncreate(id) => Anim::Uncreate {
                id: *id,
                start: cur(),
            },
            AnimSpec::FadeIn(id, sh) => Anim::FadeIn {
                id: *id,
                start: cur(),
                shift: *sh,
            },
            AnimSpec::FadeOut(id, sh) => Anim::FadeOut {
                id: *id,
                start: cur(),
                shift: *sh,
            },
            AnimSpec::GrowFromCenter(id) => Anim::Grow {
                id: *id,
                start: cur(),
            },
            AnimSpec::Shift(id, d) => {
                let start = cur();
                let mut end = start.clone();
                end.shift(*d);
                Anim::Morph {
                    id: *id,
                    start,
                    end,
                }
            }
            AnimSpec::MoveTo(id, p) => {
                let start = cur();
                let mut end = start.clone();
                end.move_to(*p);
                Anim::Morph {
                    id: *id,
                    start,
                    end,
                }
            }
            AnimSpec::SetColor(id, c) => {
                let start = cur();
                let mut end = start.clone();
                end.set_color(*c);
                Anim::Morph {
                    id: *id,
                    start,
                    end,
                }
            }
            AnimSpec::Transform(id, target) => {
                let start = cur();
                Anim::Morph {
                    id: *id,
                    start,
                    end: (**target).clone(),
                }
            }
            AnimSpec::Scale(id, f) => {
                let start = cur();
                let about = start.center();
                Anim::Scale {
                    id: *id,
                    start,
                    about,
                    factor: *f,
                }
            }
            AnimSpec::Rotate(id, a) => {
                let start = cur();
                let pivot = start.center();
                Anim::Rotate {
                    id: *id,
                    start,
                    pivot,
                    angle: *a,
                }
            }
            AnimSpec::Wait => Anim::Wait,
        }
    }

    pub fn target(&self) -> Option<Id> {
        match self {
            Anim::Morph { id, .. }
            | Anim::Rotate { id, .. }
            | Anim::Scale { id, .. }
            | Anim::Create { id, .. }
            | Anim::Uncreate { id, .. }
            | Anim::FadeIn { id, .. }
            | Anim::FadeOut { id, .. }
            | Anim::Grow { id, .. } => Some(*id),
            Anim::Wait => None,
        }
    }

    /// The rendered state of the target at eased fraction `alpha in [0, 1]`.
    pub fn apply(&self, alpha: F) -> Option<Mobject> {
        match self {
            Anim::Morph { start, end, .. } => Some(lerp_mobject(start, end, alpha)),
            Anim::Rotate {
                start,
                pivot,
                angle,
                ..
            } => {
                let mut m = start.clone();
                m.rotate_about(angle * alpha, *pivot);
                Some(m)
            }
            Anim::Scale {
                start,
                about,
                factor,
                ..
            } => {
                let f = 1.0 + (factor - 1.0) * alpha;
                let mut m = start.clone();
                m.scale_about(f, *about);
                Some(m)
            }
            Anim::Create { start, .. } => {
                let mut m = start.clone();
                m.reveal = alpha;
                Some(m)
            }
            Anim::Uncreate { start, .. } => {
                let mut m = start.clone();
                m.reveal = 1.0 - alpha;
                Some(m)
            }
            Anim::FadeIn { start, shift, .. } => {
                let mut m = start.clone();
                m.shift(*shift * (alpha - 1.0));
                m.opacity = alpha;
                Some(m)
            }
            Anim::FadeOut { start, shift, .. } => {
                let mut m = start.clone();
                m.shift(*shift * alpha);
                m.opacity = 1.0 - alpha;
                Some(m)
            }
            Anim::Grow { start, .. } => {
                let mut m = start.clone();
                let about = m.center();
                m.scale_about(alpha.max(1e-3), about);
                Some(m)
            }
            Anim::Wait => None,
        }
    }

    /// Final disposition of the target after the animation ends.
    pub fn outcome(&self) -> Outcome {
        match self {
            Anim::Morph { id, end, .. } => Outcome::Keep(*id, end.clone()),
            Anim::Rotate {
                id,
                start,
                pivot,
                angle,
            } => {
                let mut m = start.clone();
                m.rotate_about(*angle, *pivot);
                Outcome::Keep(*id, m)
            }
            Anim::Scale {
                id,
                start,
                about,
                factor,
            } => {
                let mut m = start.clone();
                m.scale_about(*factor, *about);
                Outcome::Keep(*id, m)
            }
            Anim::Create { id, start } => {
                let mut m = start.clone();
                m.reveal = 1.0;
                Outcome::Keep(*id, m)
            }
            Anim::Uncreate { id, .. } => Outcome::Remove(*id),
            Anim::FadeIn { id, start, .. } => {
                let mut m = start.clone();
                m.opacity = 1.0;
                Outcome::Keep(*id, m)
            }
            Anim::FadeOut { id, .. } => Outcome::Remove(*id),
            Anim::Grow { id, start } => Outcome::Keep(*id, start.clone()),
            Anim::Wait => Outcome::Nothing,
        }
    }
}

/// Interpolate two mobjects (geometry + style). They are aligned first so they
/// share a structure, allowing morphs between different shapes.
pub fn lerp_mobject(a: &Mobject, b: &Mobject, t: F) -> Mobject {
    let (aa, bb) = align(a, b);
    let mut out = aa.clone();
    for (sp_out, sp_b) in out.paths.iter_mut().zip(bb.paths.iter()) {
        for (p, q) in sp_out.points.iter_mut().zip(sp_b.points.iter()) {
            *p = p.lerp(*q, t);
        }
    }
    out.style.fill = lerp_opt_color(a.style.fill, b.style.fill, t);
    out.style.stroke = lerp_opt_color(a.style.stroke, b.style.stroke, t);
    out.style.stroke_width =
        a.style.stroke_width + (b.style.stroke_width - a.style.stroke_width) * t;
    out.opacity = a.opacity + (b.opacity - a.opacity) * t;
    out.reveal = a.reveal + (b.reveal - a.reveal) * t;
    out
}

fn lerp_opt_color(a: Option<Color>, b: Option<Color>, t: F) -> Option<Color> {
    match (a, b) {
        (Some(x), Some(y)) => Some(x.lerp(y, t)),
        (Some(x), None) => Some(x.with_opacity(x.a * (1.0 - t))),
        (None, Some(y)) => Some(y.with_opacity(y.a * t)),
        (None, None) => None,
    }
}

/// Produce two mobjects with matching subpath counts and point counts.
fn align(a: &Mobject, b: &Mobject) -> (Mobject, Mobject) {
    let n = a.paths.len().max(b.paths.len()).max(1);
    let a_center = a.center();
    let b_center = b.center();
    let mut a_out = a.clone();
    let mut b_out = b.clone();

    while a_out.paths.len() < n {
        a_out.paths.push(SubPath::closed(vec![a_center]));
    }
    while b_out.paths.len() < n {
        b_out.paths.push(SubPath::closed(vec![b_center]));
    }

    for i in 0..n {
        let m = a_out.paths[i]
            .points
            .len()
            .max(b_out.paths[i].points.len())
            .max(2);
        let closed = a_out.paths[i].closed || b_out.paths[i].closed;
        let ar = resample(&a_out.paths[i], m);
        let br = resample(&b_out.paths[i], m);
        a_out.paths[i] = SubPath { points: ar, closed };
        b_out.paths[i] = SubPath { points: br, closed };
    }
    (a_out, b_out)
}

/// Resample a subpath to exactly `m` points, equally spaced by arc length.
fn resample(sp: &SubPath, m: usize) -> Vec<Vec2> {
    let mut pts = sp.points.clone();
    if pts.is_empty() {
        return vec![Vec2::ZERO; m];
    }
    if pts.len() == 1 {
        return vec![pts[0]; m];
    }
    if sp.closed {
        pts.push(pts[0]);
    }
    // Cumulative lengths.
    let mut cum = vec![0.0];
    for w in pts.windows(2) {
        cum.push(cum.last().unwrap() + (w[1] - w[0]).length());
    }
    let total = *cum.last().unwrap();
    if total < 1e-12 {
        return vec![pts[0]; m];
    }
    let mut out = Vec::with_capacity(m);
    for i in 0..m {
        let target = total * i as F / (m.saturating_sub(1).max(1)) as F;
        // Find the segment containing `target`.
        let mut seg = 0;
        while seg + 1 < cum.len() && cum[seg + 1] < target {
            seg += 1;
        }
        let seg_len = cum[seg + 1] - cum[seg];
        let local = if seg_len > 1e-12 {
            (target - cum[seg]) / seg_len
        } else {
            0.0
        };
        out.push(pts[seg].lerp(pts[seg + 1], local));
    }
    out
}

// ---- Ergonomic constructors (re-exported in the prelude) ----------------

pub fn create(id: Id) -> AnimSpec {
    AnimSpec::Create(id)
}
pub fn uncreate(id: Id) -> AnimSpec {
    AnimSpec::Uncreate(id)
}
pub fn write(id: Id) -> AnimSpec {
    AnimSpec::Write(id)
}
pub fn fade_in(id: Id) -> AnimSpec {
    AnimSpec::FadeIn(id, Vec2::ZERO)
}
pub fn fade_in_from(id: Id, dir: Vec2) -> AnimSpec {
    AnimSpec::FadeIn(id, dir)
}
pub fn fade_out(id: Id) -> AnimSpec {
    AnimSpec::FadeOut(id, Vec2::ZERO)
}
pub fn fade_out_to(id: Id, dir: Vec2) -> AnimSpec {
    AnimSpec::FadeOut(id, dir)
}
pub fn grow(id: Id) -> AnimSpec {
    AnimSpec::GrowFromCenter(id)
}
pub fn shift(id: Id, delta: Vec2) -> AnimSpec {
    AnimSpec::Shift(id, delta)
}
pub fn move_to(id: Id, dest: Vec2) -> AnimSpec {
    AnimSpec::MoveTo(id, dest)
}
pub fn scale(id: Id, factor: F) -> AnimSpec {
    AnimSpec::Scale(id, factor)
}
pub fn rotate(id: Id, angle: F) -> AnimSpec {
    AnimSpec::Rotate(id, angle)
}
pub fn set_color(id: Id, c: Color) -> AnimSpec {
    AnimSpec::SetColor(id, c)
}
pub fn transform(id: Id, target: Mobject) -> AnimSpec {
    AnimSpec::Transform(id, Box::new(target))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color;
    use crate::geometry::{v, RIGHT};

    #[test]
    fn morph_midpoint() {
        let a = Mobject::square(2.0);
        let mut b = Mobject::square(2.0);
        b.shift(RIGHT * 4.0);
        let anim = Anim::Morph {
            id: 0,
            start: a,
            end: b,
        };
        let mid = anim.apply(0.5).unwrap();
        assert!((mid.center() - v(2.0, 0.0)).length() < 1e-6);
    }

    #[test]
    fn create_reveals() {
        let a = Mobject::circle(1.0);
        let anim = Anim::Create { id: 0, start: a };
        assert!((anim.apply(0.0).unwrap().reveal - 0.0).abs() < 1e-9);
        assert!((anim.apply(1.0).unwrap().reveal - 1.0).abs() < 1e-9);
    }

    #[test]
    fn align_different_shapes() {
        let a = Mobject::circle(1.0);
        let b = Mobject::text("HI", 1.0);
        let mid = lerp_mobject(&a, &b, 0.5);
        // No panic, and produces geometry.
        assert!(!mid.paths.is_empty());
        let _ = color::RED;
    }
}
