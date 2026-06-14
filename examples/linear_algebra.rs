//! Essence of Linear Algebra: a coordinate grid and basis vectors morph under
//! a 2x2 matrix. Run: cargo run --example linear_algebra
use manim_nano::prelude::*;

type Mat = [[f64; 2]; 2];
const IDENTITY: Mat = [[1.0, 0.0], [0.0, 1.0]];

fn lerp_mat(a: Mat, b: Mat, t: f64) -> Mat {
    let l = |x: f64, y: f64| x + (y - x) * t;
    [
        [l(a[0][0], b[0][0]), l(a[0][1], b[0][1])],
        [l(a[1][0], b[1][0]), l(a[1][1], b[1][1])],
    ]
}

fn main() {
    let mut scene = Scene::new(854, 480).fps(30);
    let ax = Axes::new(Range::new(-7.0, 7.0, 1.0), Range::new(-4.0, 4.0, 1.0)).lengths(14.0, 8.0);

    let grid0 = ax.grid_mobject();
    let mut i0 = Mobject::arrow(ORIGIN, v(1.0, 0.0));
    i0.set_color(GREEN);
    let mut j0 = Mobject::arrow(ORIGIN, v(0.0, 1.0));
    j0.set_color(RED);

    let grid = scene.add(grid0.clone());
    let ihat = scene.add(i0.clone());
    let jhat = scene.add(j0.clone());
    scene.play_all(&[create(grid), create(ihat), create(jhat)], 1.0);

    // The matrix [[1, 1], [0, 1]] — a horizontal shear.
    let m: Mat = [[1.0, 1.0], [0.0, 1.0]];
    let about = ORIGIN;
    scene.play_updaters(
        2.2,
        Rate::Smooth,
        vec![
            (
                grid,
                Box::new(move |a| {
                    let mut g = grid0.clone();
                    g.apply_matrix(lerp_mat(IDENTITY, m, a), about);
                    g
                }),
            ),
            (
                ihat,
                Box::new(move |a| {
                    let mut v = i0.clone();
                    v.apply_matrix(lerp_mat(IDENTITY, m, a), about);
                    v
                }),
            ),
            (
                jhat,
                Box::new(move |a| {
                    let mut v = j0.clone();
                    v.apply_matrix(lerp_mat(IDENTITY, m, a), about);
                    v
                }),
            ),
        ],
    );
    scene.wait(0.6);

    scene.export("linear_algebra").expect("export");
    println!("linear_algebra.gif — {} frames", scene.frame_count());
}
