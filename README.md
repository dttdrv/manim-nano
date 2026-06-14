# manim-nano

A tiny, fast, **runs-everywhere** animation engine inspired by 3Blue1Brown's [Manim](https://github.com/3b1b/manim) — with a plain-English frontend.

📚 See [MANIM.md](MANIM.md) for what Manim is, what it does best, and how manim-nano honors it.

## Why manim-nano?

- **Fast** — written in Rust and rendered entirely on the CPU with [`tiny-skia`](https://crates.io/crates/tiny-skia), a pure-Rust rasterizer. No system dependencies, no GPU required.
- **Everywhere** — pure Rust, so the same code compiles to a native binary on every OS **and** to WebAssembly for the browser. Output is a looping GIF plus a standalone HTML player that plays in any viewer.
- **Easy** — describe an animation in plain English and the natural-language frontend compiles it into a scene. Or use the Manim-like Rust API directly.
- **Free** — completely **offline**. No API, no key, no network, no telemetry. Free, forever.

## Quick start

Render the built-in showcase:

```bash
cargo run -- demo
```

Compile a plain-English description into an animation:

```bash
cargo run -- say "Draw a red circle, then move it right while a blue square fades in"
```

Each command writes two files next to each other: `<name>.gif` (a looping animation) and `<name>.html` (a standalone player page you can open in any browser). `demo` writes `demo.gif` + `demo.html`; `say` writes `animation.gif` + `animation.html` by default. Pass an output name as the final argument to override it, e.g. `cargo run -- demo myclip`.

## Library usage

Everything you need is in the prelude. Build a `Scene`, add `Mobject`s, and `play(...)` animations — just like Manim.

```rust
use manim_nano::prelude::*;

fn main() {
    // A small, fast preview scene (480x270). Use Scene::new(w, h) for any size.
    let mut scene = Scene::preview();

    // Add a title and write it on.
    let mut title = Mobject::text("HELLO MANIM", 0.7).colored(BLUE);
    title.move_to(UP * 2.6);
    let title = scene.add(title);
    scene.play_one(write(title), 1.2);

    // Create a circle, then morph it into a square.
    let c = scene.add(Mobject::circle(1.4).colored(YELLOW));
    scene.play_one(create(c), 1.0);
    scene.play_one(transform(c, Mobject::square(2.6).colored(GREEN)), 1.0);

    // Play several animations in parallel over the same duration.
    scene.play_all(
        &[shift(c, RIGHT * 2.0), rotate(c, std::f64::consts::FRAC_PI_4)],
        1.0,
    );

    // Scale, recolor, hold, then fade out (fade_out also removes the object).
    scene.play_one(scale(c, 1.5), 0.8);
    scene.play_one(set_color(c, PINK), 0.8);
    scene.wait(0.5);
    scene.play_one(fade_out(c), 1.0);

    // Write hello.gif + hello.html.
    scene.export("hello").expect("export");
}
```

A few API notes:

- `Scene::new(width, height)` for an explicit resolution; `Scene::preview()` for a fast 480x270 scene. Builder methods `.fps(n)`, `.background(color)` chain off either.
- `scene.add(mob)` returns an `Id` handle you pass to animation constructors.
- Animation constructors: `create`, `uncreate`, `write`, `fade_in`, `fade_in_from`, `fade_out`, `fade_out_to`, `grow`, `shift`, `move_to`, `scale`, `rotate`, `set_color`, `transform`.
- `play_one(spec, run_time)` plays one animation; `play_all(&[..], run_time)` plays several in parallel; `play(&[..], run_time, rate)` lets you pick an easing `Rate` (`Linear`, `Smooth`, `RushInto`, `RushFrom`, `ThereAndBack`).
- Angles for `rotate` are in **radians** (e.g. `std::f64::consts::FRAC_PI_2` for a quarter turn).
- `Mobject` has chainable by-value style helpers `.colored(c)`, `.filled(c)`, `.at(p)`, and mutating helpers `.move_to`, `.shift`, `.rotate`, `.scale`, `.set_color`, `.set_fill`, `.set_stroke`.

See `examples/hello_circle.rs`, `examples/transform.rs`, and `examples/natural_language.rs`. Run any of them with `cargo run --example <name>`.

## Coming from Manim?

| Manim (Python)                  | manim-nano (Rust)                                          |
| ------------------------------- | --------------------------------------------------------- |
| `class S(Scene): ...`           | `let mut scene = Scene::new(854, 480);`                   |
| `c = Circle()`                  | `let c = scene.add(Mobject::circle(1.0));`                |
| `Square()`                      | `Mobject::square(2.0)`                                     |
| `Triangle()`                    | `Mobject::triangle(1.4)`                                   |
| `RegularPolygon(n=5)`           | `Mobject::regular_polygon(5, 1.4)`                         |
| `Text("hi")`                    | `Mobject::text("HI", 0.8)`                                 |
| `c.set_color(RED)` (builder)    | `Mobject::circle(1.0).colored(RED)`                       |
| `self.add(c)`                   | `let id = scene.add(c);`                                   |
| `self.play(Create(c))`          | `scene.play_one(create(id), 1.0);`                        |
| `self.play(Uncreate(c))`        | `scene.play_one(uncreate(id), 1.0);`                      |
| `self.play(Write(t))`           | `scene.play_one(write(id), 1.0);`                         |
| `self.play(FadeIn(c))`          | `scene.play_one(fade_in(id), 1.0);`                       |
| `self.play(FadeOut(c))`         | `scene.play_one(fade_out(id), 1.0);`                      |
| `self.play(GrowFromCenter(c))`  | `scene.play_one(grow(id), 1.0);`                          |
| `self.play(Transform(c, Square()))` | `scene.play_one(transform(id, Mobject::square(2.0)), 1.0);` |
| `self.play(c.animate.shift(LEFT))`  | `scene.play_one(shift(id, LEFT), 1.0);`               |
| `self.play(c.animate.move_to(ORIGIN))` | `scene.play_one(move_to(id, ORIGIN), 1.0);`        |
| `self.play(c.animate.scale(2))` | `scene.play_one(scale(id, 2.0), 1.0);`                   |
| `self.play(Rotate(c, PI/2))`    | `scene.play_one(rotate(id, PI / 2.0), 1.0);`             |
| `self.play(c.animate.set_color(BLUE))` | `scene.play_one(set_color(id, BLUE), 1.0);`        |
| `self.play(A, B)` (parallel)    | `scene.play_all(&[a, b], 1.0);`                          |
| `self.wait()`                   | `scene.wait(1.0);`                                        |
| `UP`, `DOWN`, `LEFT`, `RIGHT`   | `UP`, `DOWN`, `LEFT`, `RIGHT` (also `UL`, `UR`, `DL`, `DR`, `ORIGIN`) |
| `RED`, `BLUE`, `YELLOW`, ...    | `RED`, `BLUE`, `YELLOW`, ... (same names)                |

## Natural-language guide

The natural-language frontend is a deterministic, offline parser. Call it from the CLI (`cargo run -- say "..."`) or from code via `manim_nano::nl::interpret("...")`. It understands:

**Shapes:** circle, square, rectangle (`rect`), triangle, pentagon, hexagon, line, arrow, dot (`point`), text (`label`, `word`, `title`).

**Colors:** red, green, blue, yellow, orange, purple (`violet`), pink, teal (`cyan`), white, black, grey (`gray`), gold, brown, magenta, maroon — or any `#rrggbb` / `#rgb` hex code.

**Verbs:**

- **Draw / create / show / add / make / place / put / display** a shape.
- **Write / text** some words (use quotes for exact text, e.g. `write "hello"`).
- **Fade in** an object.
- **Fade out / disappear / vanish / remove / hide** an object.
- **Move / shift / slide** left / right / up / down — optionally **by N** units (e.g. `move it right by 3`), or `to the center`.
- **Grow / enlarge / expand** and **shrink / contract** (`scale up` / `scale down`).
- **Rotate / spin / turn** — optionally **N degrees**, optionally `counterclockwise`.
- **Make / turn / color it `<color>`** to recolor.
- **Plot / graph** a function — `sine` (`wave`), `cosine`, `tangent`, `parabola` (`quadratic`), `cubic`, or `exponential` — drawn on labeled axes with a faint grid (the 3Blue1Brown-style graph). Add a color word to recolor the curve, or add **`area`** / **`riemann`** to overlay the area under the curve or Riemann rectangles (e.g. `graph a parabola with riemann rectangles`). For richer calculus and dynamic (dot-tracing) scenes, see `examples/calculus.rs` and `examples/dynamics.rs`.
- **Apply a matrix / shear / linear transformation to the grid / plane** — animates a coordinate grid and basis vectors (î green, ĵ red) under a 2×2 matrix. Recognizes `shear`, `rotation` (with degrees), `reflection` (`flip`), `scale`/`stretch`, and `squish` (the "Essence of Linear Algebra" effect).
- **Wait N** (or `pause N`) to hold for N seconds.

**Connectors:**

- **then** (and sentence breaks like `.` `;`) run clauses **sequentially**.
- **and / while / as** run clauses **in parallel** within the same sentence.

Example sentences:

```text
Draw a red circle, then move it right while a blue square fades in.
```

```text
Show a green hexagon, then rotate it 120 degrees and make it gold.
```

```text
Create a pentagon, grow it, then shrink it, then fade it out.
```

```text
Write "MANIM NANO", then move it up, then wait 1.
```

```text
Draw a #ff8800 triangle, spin it counterclockwise, then make it teal.
```

```text
Plot a sine wave.
```

When the parser recognizes nothing, it falls back to showing your request as on-screen text — so you always get a result.

## Output formats

Every scene can be exported in formats that play **everywhere** without extra tooling:

- **Looping GIF** — `scene.save_gif("out.gif")`. Plays in any browser, chat app, or image viewer.
- **Standalone HTML player** — `scene.save_html("out.html", "out.gif")`. A tiny self-contained page that displays the animation.
- **PNG frames** — `scene.save_frames("frames_dir/")` writes one numbered PNG per frame; `scene.save_last_png("final.png")` writes just the last frame.
- **`scene.export("name")`** — convenience that writes both `name.gif` and a sibling `name.html` player in one call.

## Architecture

The engine is a small, linear pipeline:

```text
geometry  →  color  →  font  →  mobject  →  render (tiny-skia)  →  animation  →  scene  →  nl
```

- **geometry** — `Vec2` math, world-direction constants (`UP`/`DOWN`/`LEFT`/`RIGHT`/...), and easing `Rate` functions.
- **color** — RGBA `Color`, the Manim-style named palette, and hex parsing.
- **font** — an embedded 8x8 bitmap font, so text needs no system fonts.
- **mobject** — drawable objects (`Mobject`) built from vector subpaths plus a stroke/fill style; constructors and transforms.
- **render** — rasterizes a `Mobject` onto a `tiny-skia` pixmap through a `Camera`.
- **animation** — declarative `AnimSpec`s (`create`, `shift`, `transform`, ...) resolved into concrete `Anim`s and sampled across the run time, interpolating geometry and style.
- **scene** — holds mobjects, plays animations, accumulates frames, and exports them.
- **nl** — the natural-language frontend that compiles English into a `Scene`.

## Limitations

- The natural-language parser is a **heuristic**, keyword-driven interpreter, not a full grammar — phrase it simply and it will do the right thing; unusual wording may be ignored or fall back to text.
- Text uses a built-in **8x8 bitmap font** that renders best in **uppercase**; it is intentionally tiny rather than typographically perfect.
- The engine is **2D only** (y points up, like Manim's frame). There is no 3D, no LaTeX/math typesetting, and no audio.

## License

MIT — see [LICENSE](LICENSE). Copyright (c) 2026 manim-nano contributors.
