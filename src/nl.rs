//! Natural-language frontend: turn an English description into an animated
//! [`Scene`]. This is the deterministic, offline interpreter — no network, no
//! API key. (An optional Claude-powered path lives behind the `ai` feature.)
//!
//! Example: `"Draw a red circle, then move it right while a blue square fades in."`

use crate::animation::*;
use crate::color::{Color, BLUE, WHITE};
use crate::geometry::{Rate, Vec2, DOWN, F, LEFT, ORIGIN, RIGHT, UP};
use crate::mobject::Mobject;
use crate::scene::Scene;
use std::f64::consts::PI;

struct ObjRef {
    shape: String,
    color: Option<Color>,
    id: Id,
}

#[derive(Default)]
struct Ctx {
    objs: Vec<ObjRef>,
    last: Option<Id>,
    last_center: Vec2,
    created: usize,
}

/// Interpret an English description and return a ready-to-export scene.
pub fn interpret(text: &str) -> Scene {
    interpret_into(text, Scene::new(854, 480).fps(30))
}

/// Interpret into a pre-configured scene (resolution, background, fps...).
pub fn interpret_into(text: &str, mut scene: Scene) -> Scene {
    let mut ctx = Ctx::default();
    for sentence in split_sentences(text) {
        let parts = split_parallel(&sentence);
        let mut group: Vec<AnimSpec> = Vec::new();
        let mut run_time: F = 1.0;
        let mut wait_only: Option<F> = None;

        for part in &parts {
            let (specs, rt, is_wait) = parse_clause(part, &mut scene, &mut ctx);
            if is_wait {
                wait_only = Some(rt);
                continue;
            }
            run_time = run_time.max(rt);
            group.extend(specs);
        }

        if let Some(t) = wait_only {
            if group.is_empty() {
                scene.wait(t);
                continue;
            }
        }
        if !group.is_empty() {
            scene.play(&group, run_time, Rate::Smooth);
        }
    }

    if scene.frame_count() == 0 {
        // Nothing recognized — at least show the request as text.
        let id = scene.add(Mobject::text(&to_caption(text), 0.7).colored(WHITE));
        scene.play_one(write(id), 1.5);
        scene.wait(0.5);
    }
    scene
}

// ---- Sentence / clause splitting ----------------------------------------

fn split_sentences(text: &str) -> Vec<String> {
    let lowered = text.to_lowercase();
    lowered
        .split(['.', ';', '\n'])
        .flat_map(|s| s.split(" then "))
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

fn split_parallel(sentence: &str) -> Vec<String> {
    let mut s = sentence.to_string();
    for sep in [" while ", " as ", " and "] {
        s = s.replace(sep, "\u{1}");
    }
    s.split('\u{1}')
        .map(|p| p.trim().to_string())
        .filter(|p| !p.is_empty())
        .collect()
}

// ---- Clause parsing -----------------------------------------------------

/// Returns `(specs, run_time, is_wait)`.
fn parse_clause(clause: &str, scene: &mut Scene, ctx: &mut Ctx) -> (Vec<AnimSpec>, F, bool) {
    let words: Vec<&str> = clause.split_whitespace().collect();
    if words.is_empty() {
        return (vec![], 1.0, false);
    }

    // wait / pause
    if has(&words, &["wait", "pause"]) {
        let secs = find_number(&words).unwrap_or(1.0);
        return (vec![], secs, true);
    }

    // disappear / fade out / remove
    if has(&words, &["disappear", "vanish", "remove", "hide"]) || phrase(clause, "fade out") {
        if let Some(id) = resolve_target(&words, ctx) {
            return (vec![fade_out(id)], 0.8, false);
        }
    }

    // recolor: "make/turn/color it <color>"
    if has(&words, &["make", "turn", "color", "recolor", "paint"]) {
        if let (Some(id), Some(c)) = (resolve_target(&words, ctx), find_color(&words)) {
            // Avoid treating "turn 90 degrees" as recolor when no color present.
            return (vec![set_color(id, c)], 0.8, false);
        }
    }

    // movement
    if has(&words, &["move", "shift", "slide"]) {
        if let Some(id) = resolve_target(&words, ctx) {
            if phrase(clause, "to the center") || phrase(clause, "to center") {
                return (vec![move_to(id, ORIGIN)], 1.0, false);
            }
            if let Some(dir) = find_direction(&words) {
                let dist = find_number(&words).unwrap_or(2.5);
                ctx.touch(id, ctx.last_center + dir * dist);
                return (vec![shift(id, dir * dist)], 1.0, false);
            }
        }
    }

    // grow / shrink
    if has(&words, &["grow", "enlarge", "bigger", "expand"]) || phrase(clause, "scale up") {
        if let Some(id) = resolve_target(&words, ctx) {
            return (vec![scale(id, 1.7)], 0.9, false);
        }
    }
    if has(&words, &["shrink", "smaller", "contract"]) || phrase(clause, "scale down") {
        if let Some(id) = resolve_target(&words, ctx) {
            return (vec![scale(id, 0.55)], 0.9, false);
        }
    }

    // rotate / spin
    if has(&words, &["rotate", "spin", "turn"]) && find_color(&words).is_none() {
        if let Some(id) = resolve_target(&words, ctx) {
            let deg = find_number(&words).unwrap_or(90.0);
            let dir = if has(&words, &["counterclockwise", "ccw", "left"]) {
                1.0
            } else {
                -1.0
            };
            return (vec![rotate(id, dir * deg * PI / 180.0)], 1.1, false);
        }
    }

    // creation: draw / show / create / add / write a <shape/text>
    if has(
        &words,
        &[
            "draw", "show", "create", "add", "make", "place", "put", "write", "display",
        ],
    ) {
        if let Some((mob, shape, color)) = build_object(clause, &words, ctx, scene) {
            let is_text = shape == "text";
            let id = place_and_add(scene, ctx, mob, shape.clone(), color);
            let spec = if is_text { write(id) } else { create(id) };
            return (vec![spec], 1.2, false);
        }
    }

    // Bare fade-in
    if phrase(clause, "fade in") || phrase(clause, "fades in") {
        if let Some((mob, shape, color)) = build_object(clause, &words, ctx, scene) {
            let id = place_and_add(scene, ctx, mob, shape, color);
            return (vec![fade_in_from(id, DOWN * 0.6)], 1.0, false);
        }
        if let Some(id) = resolve_target(&words, ctx) {
            return (vec![fade_in_from(id, DOWN * 0.6)], 1.0, false);
        }
    }

    // Fallback: if a shape is mentioned at all, draw it.
    if find_shape(&words).is_some() {
        if let Some((mob, shape, color)) = build_object(clause, &words, ctx, scene) {
            let id = place_and_add(scene, ctx, mob, shape, color);
            return (vec![create(id)], 1.2, false);
        }
    }

    (vec![], 1.0, false)
}

// ---- Object construction & placement ------------------------------------

fn build_object(
    clause: &str,
    words: &[&str],
    _ctx: &Ctx,
    _scene: &Scene,
) -> Option<(Mobject, String, Option<Color>)> {
    let color = find_color(words);
    let shape = find_shape(words);

    let (mob, shape_name) = match shape {
        Some("circle") => (Mobject::circle(1.0), "circle"),
        Some("square") => (Mobject::square(2.0), "square"),
        Some("rectangle") | Some("rect") => (Mobject::rectangle(3.0, 2.0), "rectangle"),
        Some("triangle") => (Mobject::triangle(1.4), "triangle"),
        Some("pentagon") => (Mobject::regular_polygon(5, 1.4), "pentagon"),
        Some("hexagon") => (Mobject::regular_polygon(6, 1.4), "hexagon"),
        Some("line") => (Mobject::line(LEFT * 2.5, RIGHT * 2.5), "line"),
        Some("arrow") => (Mobject::arrow(LEFT * 2.5, RIGHT * 2.5), "arrow"),
        Some("dot") | Some("point") => (Mobject::dot(ORIGIN, 0.14), "dot"),
        Some("text") | Some("label") | Some("word") | Some("title") => {
            let content = extract_text(clause);
            (Mobject::text(&content, 0.8), "text")
        }
        _ => {
            // No explicit shape: if it's a "write"/"text" request, make text.
            if has(words, &["write", "text", "title", "label"]) {
                let content = extract_text(clause);
                (Mobject::text(&content, 0.8), "text")
            } else {
                return None;
            }
        }
    };

    let mut mob = mob;
    if let Some(c) = color {
        mob.set_color(c);
    } else if shape_name == "circle" {
        mob.set_color(BLUE);
    }
    Some((mob, shape_name.to_string(), color))
}

fn place_and_add(
    scene: &mut Scene,
    ctx: &mut Ctx,
    mut mob: Mobject,
    shape: String,
    color: Option<Color>,
) -> Id {
    // Lay successive creations out in a row so they don't overlap.
    let pos = if ctx.created == 0 {
        ORIGIN
    } else {
        ctx.last_center + RIGHT * 3.0
    };
    mob.move_to(pos);
    let id = scene.add(mob);
    ctx.created += 1;
    ctx.last = Some(id);
    ctx.last_center = pos;
    ctx.objs.push(ObjRef { shape, color, id });
    id
}

impl Ctx {
    fn touch(&mut self, id: Id, center: Vec2) {
        self.last = Some(id);
        self.last_center = center;
    }
}

// ---- Token helpers ------------------------------------------------------

fn has(words: &[&str], keys: &[&str]) -> bool {
    words.iter().any(|w| keys.contains(&strip(w)))
}

fn phrase(clause: &str, p: &str) -> bool {
    clause.contains(p)
}

fn strip(w: &str) -> &str {
    w.trim_matches(|c: char| !c.is_alphanumeric())
}

fn find_color(words: &[&str]) -> Option<Color> {
    words.iter().find_map(|w| Color::from_name(strip(w)))
}

fn find_shape<'a>(words: &[&'a str]) -> Option<&'a str> {
    const SHAPES: &[&str] = &[
        "circle",
        "square",
        "rectangle",
        "rect",
        "triangle",
        "pentagon",
        "hexagon",
        "line",
        "arrow",
        "dot",
        "point",
        "text",
        "label",
        "word",
        "title",
    ];
    words.iter().map(|w| strip(w)).find(|w| SHAPES.contains(w))
}

fn find_number(words: &[&str]) -> Option<F> {
    words.iter().find_map(|w| strip(w).parse::<F>().ok())
}

fn find_direction(words: &[&str]) -> Option<Vec2> {
    for w in words {
        match strip(w) {
            "left" => return Some(LEFT),
            "right" => return Some(RIGHT),
            "up" | "upward" | "upwards" => return Some(UP),
            "down" | "downward" | "downwards" => return Some(DOWN),
            _ => {}
        }
    }
    None
}

fn resolve_target(words: &[&str], ctx: &Ctx) -> Option<Id> {
    let shape = find_shape(words);
    let color = find_color(words);
    if shape.is_some() || color.is_some() {
        // Most-recent object matching the mentioned shape/color.
        for o in ctx.objs.iter().rev() {
            let shape_ok =
                shape.is_none_or(|s| o.shape == s || (s == "rect" && o.shape == "rectangle"));
            let color_ok = color.is_none_or(|c| o.color == Some(c));
            if shape_ok && color_ok {
                return Some(o.id);
            }
        }
    }
    ctx.last
}

/// Pull a quoted string, or the words after a "write/text/say" keyword.
fn extract_text(clause: &str) -> String {
    if let Some(start) = clause.find('"') {
        if let Some(end) = clause[start + 1..].find('"') {
            return clause[start + 1..start + 1 + end].trim().to_uppercase();
        }
    }
    if let Some(start) = clause.find('\'') {
        if let Some(end) = clause[start + 1..].find('\'') {
            return clause[start + 1..start + 1 + end].trim().to_uppercase();
        }
    }
    for kw in ["write ", "text ", "say ", "saying ", "reading ", "titled "] {
        if let Some(i) = clause.find(kw) {
            let rest = clause[i + kw.len()..].trim();
            let rest = rest.trim_start_matches(['"', '\'']);
            let stop: &[_] = &['"', '\'', ',', ';', '.'];
            let rest = rest.split(stop).next().unwrap_or(rest);
            if !rest.is_empty() {
                return rest.trim().to_uppercase();
            }
        }
    }
    "TEXT".to_string()
}

fn to_caption(text: &str) -> String {
    let t: String = text.trim().chars().take(40).collect();
    t.to_uppercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn draws_a_circle() {
        let s = interpret("Draw a red circle");
        assert!(s.frame_count() > 0);
    }

    #[test]
    fn sequential_and_parallel() {
        let s = interpret("Draw a red circle, then move it right while a blue square fades in");
        // Should have produced multiple play segments worth of frames.
        assert!(s.frame_count() > 30);
    }

    #[test]
    fn unknown_falls_back_to_text() {
        let s = interpret("flibbertigibbet");
        assert!(s.frame_count() > 0);
    }

    #[test]
    fn wait_is_recognized() {
        let mut a = interpret("draw a square");
        let before = a.frame_count();
        a = interpret("draw a square. wait 1");
        assert!(a.frame_count() > before);
    }
}
