//! Natural-language frontend example: compile a plain-English description into
//! an animated scene, entirely offline (no API, no key, no network).
//!
//! Run with: cargo run --example natural_language
use manim_nano::nl;

fn main() {
    let description = "Draw a red circle, then move it right while a blue square \
                       fades in. Then make the circle green and rotate it 90 \
                       degrees. Then fade out the square.";

    let mut scene = nl::interpret(description);

    scene
        .export("natural_language")
        .expect("export natural_language");
    println!(
        "wrote natural_language.gif ({} frames)",
        scene.frame_count()
    );
}
