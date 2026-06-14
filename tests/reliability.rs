//! Reliability / robustness integration suite for manim-nano.
//!
//! Goal: prove the natural-language interpreter and core engine NEVER panic and
//! ALWAYS produce at least one frame, across messy and adversarial input — "as
//! reliable as Manim." Also includes a handful of property-style geometry checks
//! and an end-to-end export smoke test.

use manim_nano::nl::interpret;
use manim_nano::prelude::*;

/// The diverse, "well-formed" corpus. Every prompt here must interpret without
/// panicking and yield at least one frame.
const CORPUS: &[&str] = &[
    // --- simple single-shape creation ---
    "draw a circle",
    "draw a square",
    "draw a rectangle",
    "draw a triangle",
    "draw a pentagon",
    "draw a hexagon",
    "draw a line",
    "draw an arrow",
    "draw a dot",
    "show a circle",
    "create a square",
    "add a triangle",
    "display a hexagon",
    "place a dot",
    "put a circle on the screen",
    // --- colored shapes ---
    "draw a red circle",
    "draw a blue square",
    "draw a green triangle",
    "draw a yellow hexagon",
    "draw a purple pentagon",
    "draw a teal circle and an orange square",
    // --- multi-shape ---
    "draw a red circle and a blue square",
    "draw a circle, a square, and a triangle",
    "draw a green circle and a green square and a green triangle",
    // --- sequential (then) ---
    "draw a square then move it right",
    "draw a circle then move it left",
    "draw a circle, then make it green",
    "draw a square. then spin it 180 degrees",
    "draw a triangle then grow it then shrink it",
    // --- parallel (while / as / and) ---
    "move it left while a triangle fades in",
    "draw a circle while a square fades in",
    "draw a red circle then move it right while a blue square fades in",
    // --- recolor ---
    "make it green",
    "turn it red",
    "color the circle blue",
    "recolor it to yellow",
    "paint the square orange",
    // --- rotate / scale ---
    "spin it 180 degrees",
    "rotate the circle 90 degrees",
    "rotate it counterclockwise 45 degrees",
    "grow the circle",
    "shrink the square",
    "enlarge it",
    "scale up the triangle",
    "scale down the hexagon",
    "make the circle bigger",
    // --- text / write ---
    "write \"HELLO\"",
    "write 'world'",
    "show the text titled \"manim nano\"",
    "draw a label",
    "write a title",
    // --- waits ---
    "wait 2",
    "draw a circle. wait 1",
    "pause for 3 seconds",
    // --- movement variety ---
    "move it up",
    "move it down",
    "slide the circle right by 3",
    "shift the square to the center",
    "move the triangle upward",
    // --- fade ---
    "draw a circle then fade it out",
    "a triangle fades in",
    "make the square disappear",
    "hide the circle",
    // --- longer compound descriptions ---
    "draw a red circle, then move it right while a blue square fades in, then spin it 180 degrees",
    "create a green hexagon then grow it then turn it purple then move it left",
    "write \"INTRO\" then wait 1 then draw a circle",
];

/// Adversarial / edge-case inputs. These must NOT panic and must still yield
/// frames (the interpreter falls back to rendering the request as text).
fn adversarial_inputs() -> Vec<String> {
    vec![
        String::new(),                                             // empty string
        "   ".to_string(),                                         // whitespace only
        "\t\n  \n".to_string(),                                    // assorted whitespace
        "a".repeat(20_000),                                        // very long string
        "draw a circle ".repeat(2_000),                            // very long but meaningful-ish
        "draw a 🔴 circle ✨".to_string(),                         // unicode / emoji
        "🎉🎉🎉".to_string(),                                      // emoji only
        "!!!???...".to_string(),                                   // punctuation only
        ",,,;;;...".to_string(),                                   // delimiter-only
        "12345".to_string(),                                       // numbers only
        "move it right by 999999999".to_string(),                  // huge number
        "move it right by 99999999999999999999999999".to_string(), // overflow-ish
        "rotate it 1000000000 degrees".to_string(),                // huge rotation
        "move it right by -5".to_string(),                         // negative number
        "rotate it -270 degrees".to_string(),                      // negative rotation
        "wait -2".to_string(),                                     // negative wait
        "scale it by 0".to_string(),                               // zero scale-ish
        "a and b while c then d and e".to_string(),                // deeply nested connectors
        "draw and draw and draw while move and move".to_string(),
        "DRAW A CIRCLE".to_string(),     // all caps
        "DrAw A rEd CiRcLe".to_string(), // mixed casing
        "DRAW A RED CIRCLE THEN MOVE IT RIGHT".to_string(),
        "flibbertigibbet".to_string(), // pure nonsense
        "the quick brown fox jumps over the lazy dog".to_string(), // no commands
        "circle".to_string(),          // bare shape noun
        "red".to_string(),             // bare color
        "then then then".to_string(),  // connector spam
        "while while while".to_string(),
        "and and and".to_string(),
        ". ; \n . ; \n".to_string(),           // sentence-splitter spam
        "\u{0}\u{1}\u{2}".to_string(),         // control chars (incl. the parallel sentinel)
        "draw\u{1}a\u{1}circle".to_string(),   // embedded parallel sentinel
        "move it right by 1e308".to_string(),  // f64 near-max
        "move it right by inf".to_string(),    // not a number, falls back
        "move it right by NaN".to_string(),    // not parsed as number
        "draw a circle 你好 世界".to_string(), // CJK text
        "draw a \"unterminated quote".to_string(), // unbalanced quote
        "write \"\"".to_string(),              // empty quoted text
        "write \"   \"".to_string(),           // whitespace quoted text
    ]
}

#[test]
fn corpus_never_panics_and_always_has_frames() {
    for &prompt in CORPUS {
        let scene = interpret(prompt);
        assert!(
            scene.frame_count() > 0,
            "corpus prompt produced zero frames: {prompt:?}"
        );
    }
}

#[test]
fn corpus_size_is_substantial() {
    assert!(
        CORPUS.len() >= 40,
        "corpus should have 40+ prompts, has {}",
        CORPUS.len()
    );
}

#[test]
fn adversarial_inputs_never_panic_and_always_have_frames() {
    for input in adversarial_inputs() {
        let preview: String = input.chars().take(40).collect();
        let scene = interpret(&input);
        assert!(
            scene.frame_count() > 0,
            "adversarial input produced zero frames (len={}): {preview:?}",
            input.len()
        );
    }
}

// ---- Property-style geometry tests --------------------------------------

const EPS: f64 = 1e-6;

fn approx(a: Vec2, b: Vec2, eps: f64) -> bool {
    (a - b).length() <= eps
}

#[test]
fn rotate_then_unrotate_is_identity() {
    // Rotating by +θ then −θ about the SAME fixed pivot is a true inverse and
    // must restore the original center and bbox. (Note: `rotate()` pivots about
    // the bbox-center, which itself shifts as a non-symmetric shape turns, so
    // rotate(+θ) followed by rotate(−θ) is intentionally *not* tested here.)
    for shape in [
        Mobject::circle(1.3),
        Mobject::square(2.0),
        Mobject::triangle(1.4),
        Mobject::regular_polygon(7, 1.1),
    ] {
        let theta = 0.937_f64; // arbitrary angle
        let pivot = v(0.5, -0.25); // fixed, off-center pivot
        let center_before = shape.center();
        let (min_b, max_b) = shape.bbox();

        let mut m = shape.clone();
        m.rotate_about(theta, pivot);
        m.rotate_about(-theta, pivot);

        assert!(
            approx(m.center(), center_before, EPS),
            "center drifted after rotate_about(+θ)+rotate_about(-θ): {:?} vs {:?}",
            m.center(),
            center_before
        );
        let (min_a, max_a) = m.bbox();
        assert!(
            approx(min_a, min_b, 1e-5) && approx(max_a, max_b, 1e-5),
            "bbox drifted after rotate round-trip"
        );
    }
}

#[test]
fn rotate_full_turn_restores_shape() {
    // A full 2π turn about the center restores everything regardless of pivot
    // drift, since +2π maps every point back to itself.
    use std::f64::consts::TAU;
    let mut m = Mobject::square(2.0);
    m.shift(v(1.0, 1.0));
    let (min_b, max_b) = m.bbox();
    let center_before = m.center();
    m.rotate(TAU);
    let (min_a, max_a) = m.bbox();
    assert!(
        approx(m.center(), center_before, 1e-5)
            && approx(min_a, min_b, 1e-5)
            && approx(max_a, max_b, 1e-5),
        "full turn did not restore the shape"
    );
}

#[test]
fn shift_then_unshift_is_identity() {
    let mut m = Mobject::square(2.0);
    let center_before = m.center();
    let delta = v(3.7, -2.1);
    m.shift(delta);
    m.shift(-delta);
    assert!(
        approx(m.center(), center_before, EPS),
        "shift round-trip did not restore center"
    );
}

#[test]
fn scale_about_center_keeps_center_fixed() {
    for factor in [0.25_f64, 0.5, 1.0, 2.0, 5.0] {
        let mut m = Mobject::rectangle(3.0, 2.0);
        m.shift(v(1.0, 1.0)); // off-origin center
        let center_before = m.center();
        m.scale(factor); // scale() pivots about the current center
        assert!(
            approx(m.center(), center_before, 1e-5),
            "scaling about center moved the center (factor={factor})"
        );
    }
}

#[test]
fn scale_up_then_down_is_identity() {
    let mut m = Mobject::triangle(1.4);
    let (min_b, max_b) = m.bbox();
    m.scale(3.0);
    m.scale(1.0 / 3.0);
    let (min_a, max_a) = m.bbox();
    assert!(
        approx(min_a, min_b, 1e-5) && approx(max_a, max_b, 1e-5),
        "scale up/down round-trip changed bbox"
    );
}

// ---- End-to-end export smoke test ---------------------------------------

#[test]
fn export_gif_smoke_test() {
    let mut scene = interpret("draw a red circle then move it right while a blue square fades in");
    assert!(scene.frame_count() > 0, "interpreted scene had no frames");

    let mut path = std::env::temp_dir();
    path.push(format!("manim_nano_reliability_{}.gif", std::process::id()));

    scene.save_gif(&path).expect("save_gif should succeed");

    let meta = std::fs::metadata(&path).expect("gif file should exist");
    assert!(meta.len() > 0, "exported gif is empty");

    // Optional sanity check: GIF magic bytes.
    let bytes = std::fs::read(&path).expect("read gif back");
    assert!(
        bytes.starts_with(b"GIF8"),
        "exported file is not a GIF (missing magic bytes)"
    );

    let _ = std::fs::remove_file(&path);
}
