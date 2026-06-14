//! `manim` — the command-line interface for manim-nano.
//!
//! Everything here runs offline and for free. No API, no key, no network.
//!
//!   manim demo [out]            render the built-in showcase
//!   manim say "<description>" [out]   compile English into an animation
//!   manim --help

use manim_nano::nl;
use manim_nano::prelude::*;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let cmd = args.first().map(|s| s.as_str()).unwrap_or("help");

    match cmd {
        "demo" => {
            let base = args.get(1).map(String::as_str).unwrap_or("demo");
            let mut scene = build_demo();
            finish(&mut scene, base)
        }
        "say" => {
            let Some(text) = args.get(1) else {
                eprintln!("usage: manim say \"<description>\" [out]");
                return ExitCode::FAILURE;
            };
            let base = args.get(2).map(String::as_str).unwrap_or("animation");
            let mut scene = nl::interpret(text);
            finish(&mut scene, base)
        }
        "version" | "--version" | "-V" => {
            println!("manim-nano {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        _ => {
            print_help();
            ExitCode::SUCCESS
        }
    }
}

fn finish(scene: &mut Scene, base: &str) -> ExitCode {
    match scene.export(base) {
        Ok(()) => {
            println!(
                "rendered {} frames -> {base}.gif (+ {base}.html player)",
                scene.frame_count()
            );
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("error writing output: {e}");
            ExitCode::FAILURE
        }
    }
}

/// A showcase that exercises most of the engine in one short clip.
fn build_demo() -> Scene {
    let mut s = Scene::new(854, 480).fps(30).background(DARK_BG);
    let (_, hh) = s.half_frame();

    // Title.
    let mut title = Mobject::text("MANIM NANO", 0.7).colored(BLUE);
    title.move_to(UP * (hh - 1.0));
    let title = s.add(title);
    s.play_one(write(title), 1.2);

    // A circle is created, then morphs into a square.
    let circle = s.add(Mobject::circle(1.3).colored(YELLOW).at(LEFT * 3.0));
    s.play_one(create(circle), 1.0);
    s.play_one(
        transform(circle, Mobject::square(2.4).colored(GREEN).at(LEFT * 3.0)),
        1.0,
    );

    // A second shape grows and spins.
    let tri = s.add(Mobject::triangle(1.3).colored(RED).at(RIGHT * 3.0));
    s.play_all(&[grow(tri), rotate(tri, std::f64::consts::TAU)], 1.4);

    // Move them together and recolor.
    s.play_all(&[shift(circle, RIGHT * 3.0), shift(tri, LEFT * 3.0)], 1.2);
    s.play_all(&[set_color(circle, PINK), set_color(tri, TEAL)], 0.8);

    s.wait(0.5);
    s.play_all(&[fade_out_to(circle, DOWN), fade_out_to(tri, UP)], 1.0);
    s.wait(0.4);
    s
}

fn print_help() {
    println!(
        "manim-nano {v} — a tiny, fast, runs-everywhere animation engine.\n\
         \n\
         USAGE:\n\
         \x20 manim demo [out]                 render the built-in showcase\n\
         \x20 manim say \"<description>\" [out]   compile plain English into an animation\n\
         \x20 manim --version\n\
         \n\
         OUTPUT:\n\
         \x20 <out>.gif   looping animation (plays in any browser/viewer)\n\
         \x20 <out>.html  standalone player page\n\
         \n\
         EXAMPLES:\n\
         \x20 manim demo\n\
         \x20 manim say \"Draw a red circle, then move it right while a blue square fades in\"\n\
         \n\
         Everything runs offline and for free — no API, no key, no network.",
        v = env!("CARGO_PKG_VERSION")
    );
}
