//! Transform showcase: a circle morphs into a square, then a triangle, then
//! rotates, recolors, and fades out.
//!
//! Run with: cargo run --example transform
use manim_nano::prelude::*;

fn main() {
    let mut scene = Scene::new(854, 480).fps(30);

    // Create a circle.
    let shape = scene.add(Mobject::circle(1.4).colored(YELLOW));
    scene.play_one(create(shape), 1.0);

    // Morph it into a square, then into a triangle.
    scene.play_one(transform(shape, Mobject::square(2.6).colored(GREEN)), 1.0);
    scene.play_one(transform(shape, Mobject::triangle(1.6).colored(BLUE)), 1.0);

    // Rotate a quarter turn (angles are in radians).
    scene.play_one(rotate(shape, std::f64::consts::FRAC_PI_2), 1.0);

    // Recolor, hold, then fade out (which also removes the object).
    scene.play_one(set_color(shape, PINK), 0.8);
    scene.wait(0.5);
    scene.play_one(fade_out(shape), 1.0);
    scene.wait(0.4);

    scene.export("transform").expect("export transform");
    println!("wrote transform.gif ({} frames)", scene.frame_count());
}
