//! Graphing — what Manim is loved for. Run: cargo run --example graph
use manim_nano::prelude::*;

fn main() {
    let mut scene = Scene::new(854, 480).fps(30);

    let ax = Axes::new(Range::new(-6.5, 6.5, 1.0), Range::new(-2.2, 2.2, 1.0)).lengths(11.0, 5.6);

    let grid = scene.add(ax.grid_mobject());
    let axes = scene.add(ax.axes_mobject());
    scene.play_all(&[create(grid), create(axes)], 1.2);

    let sine = scene.add(ax.plot(|x| x.sin()).colored(YELLOW));
    scene.play_one(create(sine), 2.0);

    let cosine = scene.add(ax.plot(|x| x.cos()).colored(TEAL));
    scene.play_one(create(cosine), 2.0);

    scene.wait(0.6);
    scene.export("graph").expect("export");
    println!("graph.gif — {} frames", scene.frame_count());
}
