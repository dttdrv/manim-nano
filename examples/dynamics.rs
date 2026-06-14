//! Dynamic animation via updaters: a dot rides along a sine curve while a
//! trail traces its path — the iconic Manim "ValueTracker + updater" effect.
//! Run: cargo run --example dynamics
use manim_nano::prelude::*;

fn main() {
    let mut scene = Scene::new(854, 480).fps(30);
    let ax = Axes::new(Range::new(-6.5, 6.5, 1.0), Range::new(-2.2, 2.2, 1.0)).lengths(11.0, 5.6);

    let grid = scene.add(ax.grid_mobject());
    let axes = scene.add(ax.axes_mobject());
    let curve = scene.add(ax.plot(|x| x.sin()).colored(BLUE));
    scene.play_all(&[create(grid), create(axes), create(curve)], 1.2);

    // A dot and a trail, both driven by the same parameter alpha in [0, 1].
    let dot = scene.add(Mobject::dot(ax.point_at_x(|x| x.sin(), -6.0), 0.12).colored(YELLOW));
    let trail = scene.add(Mobject::line(ORIGIN, ORIGIN).colored(RED));

    let x0 = -6.0;
    let x1 = 6.0;
    scene.play_updaters(
        4.0,
        Rate::Linear,
        vec![
            (
                dot,
                Box::new(move |a| {
                    let x = x0 + (x1 - x0) * a;
                    Mobject::dot(ax.point_at_x(|x| x.sin(), x), 0.12).colored(YELLOW)
                }),
            ),
            (
                trail,
                Box::new(move |a| {
                    let x = x0 + (x1 - x0) * a;
                    let n = 120;
                    let pts: Vec<Vec2> = (0..=n)
                        .map(|i| {
                            let xi = x0 + (x - x0) * i as f64 / n as f64;
                            ax.point_at_x(|x| x.sin(), xi)
                        })
                        .collect();
                    let mut m =
                        Mobject::from_subpaths(vec![manim_nano::mobject::SubPath::open(pts)]);
                    m.set_stroke(RED, 0.06);
                    m
                }),
            ),
        ],
    );
    scene.wait(0.6);

    scene.export("dynamics").expect("export");
    println!("dynamics.gif — {} frames", scene.frame_count());
}
