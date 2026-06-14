//! Calculus visualization — Riemann rectangles converging to the area under a
//! curve, then a tangent line. The kind of thing Manim is made for.
//! Run: cargo run --example calculus
use manim_nano::prelude::*;

fn main() {
    let mut scene = Scene::new(854, 480).fps(30);
    let ax = Axes::new(Range::new(0.0, 3.2, 1.0), Range::new(0.0, 1.2, 0.5)).lengths(11.0, 5.4);

    let grid = scene.add(ax.grid_mobject());
    let axes = scene.add(ax.axes_mobject());
    let labels = scene.add(ax.number_labels());
    scene.play_all(&[create(grid), create(axes), create(labels)], 1.2);

    let f = |x: f64| x.sin();
    let curve = scene.add(ax.plot(f).colored(YELLOW));
    scene.play_one(create(curve), 1.5);

    // Riemann rectangles under one hump of the sine curve.
    let rects = scene.add(ax.riemann_rectangles(f, 0.0, std::f64::consts::PI, 14, GREEN));
    scene.play_one(create(rects), 1.5);
    scene.indicate(rects, 0.8);

    // Replace them with the exact area.
    let area = scene.add(ax.area_under(f, 0.0, std::f64::consts::PI, BLUE));
    scene.play_all(&[fade_out(rects), fade_in(area)], 1.0);

    // A tangent line at x = 0.6.
    let tan = scene.add(ax.tangent_line(f, 0.6, 1.0));
    scene.play_one(create(tan), 1.0);
    scene.wait(0.6);

    scene.export("calculus").expect("export");
    println!("calculus.gif — {} frames", scene.frame_count());
}
