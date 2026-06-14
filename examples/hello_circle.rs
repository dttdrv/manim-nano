//! Minimal library-usage example.
//! Run with: cargo run --example hello_circle
use manim_nano::prelude::*;

fn main() {
    let mut scene = Scene::new(854, 480).fps(30);

    let mut title = Mobject::text("HELLO MANIM", 0.7).colored(BLUE);
    title.move_to(UP * 2.6);
    let title = scene.add(title);
    scene.play_one(write(title), 1.2);

    let c = scene.add(Mobject::circle(1.4).colored(YELLOW));
    scene.play_one(create(c), 1.0);
    scene.play_one(transform(c, Mobject::square(2.6).colored(GREEN)), 1.0);
    scene.play_all(
        &[
            shift(c, RIGHT * 2.0),
            rotate(c, std::f64::consts::FRAC_PI_4),
        ],
        1.0,
    );

    scene.export("hello_circle").expect("export");
    println!("wrote hello_circle.gif ({} frames)", scene.frame_count());
}
