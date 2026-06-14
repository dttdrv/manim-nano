# Manim, and how manim-nano honors it

This document explains what Manim is, what it does best, and the features its users
love most — then maps that honestly onto what **manim-nano** supports today. The Manim
claims here were cross-checked against the real 3Blue1Brown source (the `manimlib`/ManimGL
repository), not memory alone.

---

## 1. What Manim is

**Manim** is the **Mathematical Animation Engine** — *"an engine for precise programmatic
animations, designed for creating explanatory math videos"* (3b1b's own README). It was
created by **Grant Sanderson** to produce the animations in his [3Blue1Brown](https://www.3blue1brown.com/)
YouTube videos (linear algebra, calculus, neural networks, the essence-of series).

There are **two** Manims, and it matters which one you mean:

- **`manimlib` / ManimGL** — Grant Sanderson's original, personal repository
  ([`3b1b/manim`](https://github.com/3b1b/manim)). It is OpenGL-rendered (hence "GL"),
  installed as the pip package **`manimgl`**, and optimized for an interactive,
  video-production workflow (it can even drop you into an IPython shell mid-scene via
  `self.embed()`).
- **Manim Community Edition (`manim`)** — a 2020 community fork
  ([`ManimCommunity/manim`](https://github.com/ManimCommunity/manim)) aimed at being more
  stable, better tested, documented, and friendlier to newcomers. Installed as **`manim`**.

The two share a mental model and most class names but have diverged in details
(for example, ManimGL's `ShowCreation` is `Create` in Community, and ManimGL's
`Tex`/`TexText` are `MathTex`/`Tex` in Community). manim-nano draws on the *shared*
ideas rather than one specific edition.

### The core mental model

Both editions revolve around the same loop. You subclass `Scene`, implement
`construct()`, and orchestrate **mobjects** ("mathematical objects") through **animations**:

```python
from manim import *   # or: from manimlib import *

class MyScene(Scene):
    def construct(self):
        circle = Circle()              # a mobject
        self.play(Create(circle))      # animate it onto the screen
        self.play(circle.animate.shift(RIGHT))
        self.wait()                    # hold the final frame
```

The three verbs that carry almost everything:

- **`self.add(mobject)`** — place a mobject in the scene with no animation.
- **`self.play(Animation(mobject), run_time=...)`** — run one or more animations, in
  parallel, over a duration. Pass `rate_func=...` to control the easing.
- **`self.wait(seconds)`** — hold the current frame.

A scene is rendered to video with `ffmpeg`. System requirements include FFmpeg, OpenGL,
and **LaTeX** (optional, but needed for any math typesetting).

---

## 2. What Manim does best

Manim's whole reason to exist is **animating mathematics** — not generic motion graphics.
Its original motivation, quoted verbatim in `example_scenes.py`, was *"to better illustrate
mathematical functions as transformations."* The library is at its best here:

- **Graphing functions.** `Axes` and `NumberPlane` lay down a coordinate system;
  `axes.get_graph(lambda x: ...)` (Community: `axes.plot(...)`) draws `y = f(x)` by sampling
  points and smoothly interpolating. `ParametricCurve` / `ParametricFunction` handle
  parametric and polar curves. `axes.c2p(x, y)` ("coords-to-point") maps math coordinates to
  screen points so you can pin dots and labels onto a graph.
- **Linear algebra and matrix transforms.** The signature 3b1b move: lay down a
  `NumberPlane` grid and apply a matrix to it — `grid.animate.apply_matrix([[1, 1], [0, 1]])` —
  so the audience *sees* a shear or rotation deform space itself. `ComplexPlane` extends this
  to maps like `z → z²` via `apply_complex_function`.
- **Calculus visualizations.** Areas under curves, Riemann rectangles, tangent lines
  (`TangentLine`), and secants approaching a derivative — all anchored to an `Axes` and
  driven by a moving parameter.
- **Geometric transformations.** `Transform` / `ReplacementTransform` morph one shape into
  another; `Rotate`, `ApplyMatrix`, and `ApplyPointwiseFunction` warp geometry continuously.
- **Transforming one equation into another.** `Tex`/`MathTex` typeset LaTeX, and
  `TransformMatchingTex` / `TransformMatchingStrings` line up matching sub-expressions so a
  formula visibly *rearranges* — e.g. `A² + B² = C²` flowing into `A = √((C+B)(C−B))`. This is
  one of Manim's most distinctive party tricks.
- **Vectors and vector fields.** `Vector`, `Arrow`, and vector-field mobjects show flows and
  directions over a plane.
- **"Show-don't-tell" visual proofs.** Because every object is a precise mathematical entity
  with exact coordinates, you can build rigorous, reproducible visual arguments rather than
  hand-wavy illustrations.
- **Dynamic, data-driven scenes.** A `ValueTracker` holds a number you animate; **updaters**
  (`mobject.add_updater(...)`, `always_redraw(...)`) recompute dependent mobjects every frame,
  so a dot rides a curve, a brace tracks a width, or a decimal counts up live.
- **A controllable camera.** ManimGL's `self.frame` (Community's `MovingCamera` / scene frame)
  can pan, zoom, and reorient to focus attention — `self.frame.animate.scale(0.75).to_corner(UL)`.

---

## 3. The animations and features users love most

These are the things people reach for again and again, and *why* they land:

- **The draw-on effect — `Write` and `ShowCreation` (Community: `Create`).** Lines and curves
  trace themselves onto the screen, and `Write` extends this to text/LaTeX, drawing the border
  then filling. It is the iconic Manim look: ideas appear to be *constructed* in front of you
  rather than just popping in. (`ShowCreation` literally subclasses a `ShowPartial` animation
  that reveals an increasing fraction of each stroke.)
- **`Transform` / `ReplacementTransform` / `TransformMatchingTex` morphing.** Continuously
  interpolating one mobject's points into another's is how Manim turns "step A becomes step B"
  into a single fluid motion. Matching variants align shared pieces so equations *rearrange*
  instead of cross-fading — the visual that makes algebra feel inevitable.
- **`FadeIn` / `FadeOut` (and `FadeTransform`).** The clean, low-key way to introduce and
  retire elements, optionally with a directional `shift`, so the screen never feels cluttered.
- **Smooth `rate_functions`.** Manim's default easing is `smooth`, a cubic with *zero first and
  second derivatives at both ends* (`(t³)(10s² + 5st + t²)` for `s = 1 − t`), giving that gentle,
  professional ease-in/ease-out. The family also includes `linear`, `rush_into`, `rush_from`,
  `there_and_back`, `running_start`, `overshoot`, and `wiggle`. This easing is a big part of why
  Manim videos feel *polished* rather than mechanical.
- **`ValueTracker` + updaters.** The engine of dynamic explainers: animate one number and let
  everything else follow. This is what makes a tangent line sweep along a curve or a label
  recompute itself live — abstract dependencies become visible motion.
- **LaTeX math typesetting.** First-class `Tex`/`MathTex` rendering means real mathematical
  notation — integrals, fractions, Greek letters, matrices (`IntegerMatrix`) — and you can index
  into sub-expressions by substring to color or animate just one term.
- **Color gradients and the signature palette.** Named colors come in tonal families
  (`BLUE_A`…`BLUE_E`, etc.), with median shortcuts like `BLUE = BLUE_C`. `set_color_by_gradient`
  paints across a range, and the whole thing sits on Manim's **signature dark background** — the
  instantly recognizable 3Blue1Brown aesthetic.
- **The camera.** Panning and zooming to direct the viewer's eye turns a static board into a
  guided tour.

The throughline: every one of these exists to **turn abstract math into visual intuition**, and
to do it in a way that is **polished and reproducible** — the same script renders the same clean
explainer video every time.

---

## 4. A short, idiomatic Manim example

Adapted from the `TexTransformExample` in 3b1b's `example_scenes.py` — rearranging the
Pythagorean theorem:

```python
class PythagoreanRearrange(Scene):
    def construct(self):
        t2c = {"A": BLUE, "B": TEAL, "C": GREEN}     # color each variable
        kw = dict(font_size=72, t2c=t2c)
        line1 = Tex("A^2 + B^2 = C^2", **kw)
        line2 = Tex("A^2 = C^2 - B^2", **kw)

        self.add(line1)
        self.play(
            TransformMatchingStrings(          # line up A^2, B^2, C^2 across the two lines
                line1.copy(), line2,
                key_map={"+": "-"},            # the '+' flows into the '-'
                path_arc=90 * DEG,             # parts swing into place along an arc
            ),
        )
        self.wait()
```

**Why it clicks:** the matching transform keeps `A²`, `B²`, and `C²` *the same objects*
across both lines, so the eye tracks each term as it slides to its new home. You don't just
*read* that the equation was rearranged — you *watch* it happen. That fusion of rigor and
motion is Manim in one screen.

---

## 5. manim-nano fidelity table

manim-nano is a tiny Rust reimplementation of Manim's *ideas*. Here is an honest mapping of
the beloved capabilities above onto what it supports today (verified against `src/lib.rs`'s
prelude and the README):

| Beloved Manim capability | manim-nano status | Note |
| --- | --- | --- |
| `Scene` + `play` + `wait` loop | **Supported** | `Scene::new(w,h)` / `Scene::preview()`, `play_one` / `play_all` / `play`, `wait`. |
| Shapes (Circle, Square, Polygon, Line, Arrow, Dot, Text) | **Supported** | `Mobject::circle/square/rectangle/triangle/regular_polygon/text`, etc. |
| Draw-on: `ShowCreation`/`Create`, `Write`, `Uncreate` | **Supported** | `create`, `write`, `uncreate` animation constructors. |
| `FadeIn` / `FadeOut` (incl. directional) | **Supported** | `fade_in`, `fade_in_from`, `fade_out`, `fade_out_to`. |
| `GrowFromCenter` | **Supported** | `grow`. |
| Move / shift / scale / rotate / recolor | **Supported** | `shift`, `move_to`, `scale`, `rotate` (radians), `set_color`. |
| `Transform` morphing | **Partial** | `transform` morphs via polyline interpolation between shapes; no point-matching `TransformMatchingTex`. |
| Smooth `rate_functions` / easing | **Supported** | `Rate`: `Linear`, `Smooth`, `RushInto`, `RushFrom`, `ThereAndBack`. |
| Signature dark background + named palette | **Supported** | `DARK_BG` plus Manim-style names (`RED`, `BLUE`, `YELLOW`, …) and hex. |
| `Axes` + graphing `f(x)` (`get_graph`/`plot`) | **Supported** | `Axes` with `coords_to_point`, `plot(function)`, `plot_parametric`, and numeric tick labels. |
| Calculus visuals (Riemann rectangles, area, tangent) | **Supported** | `Axes::riemann_rectangles`, `area_under`, `tangent_line`, `point_at_x`. |
| `NumberPlane`-style grid | **Supported** | `Axes::grid_mobject()` draws the faint coordinate grid. |
| Matrix / linear transforms of a grid | **Supported** | `Mobject::apply_matrix` + updaters morph a grid & basis vectors under a 2x2 matrix; `manim say "apply a shear to the grid"`. |
| LaTeX / `Tex` / `MathTex` typesetting | **Planned** | Text uses a built-in 8×8 bitmap font; no LaTeX yet. |
| `ValueTracker` + updaters (per-frame recompute) | **Supported** | `Scene::play_updaters` / `play_updater` drive per-frame closures (a dot tracing a curve, value-driven labels); plus `Scene::indicate`. |
| Moving / zooming camera | **Planned** | `Camera` renders a fixed frame; no animated camera moves yet. |
| 3D scenes / surfaces | **Planned** | Engine is 2D only (y points up, like Manim's frame). |
| Output you can play anywhere | **Supported (and a manim-nano strength)** | Looping GIF, PNG frames, and a standalone HTML player — no FFmpeg required. |
| Native + browser | **Supported (beyond Manim)** | Same Rust compiles to a native binary *and* to WebAssembly. |
| Natural-language frontend | **Supported (beyond Manim)** | Describe a scene in plain English; an offline parser compiles it. |

**Honest summary:** manim-nano already covers the day-to-day Manim core — the scene loop, the
shapes, the draw-on and fade animations, transform/grow, the easing curves, the dark palette,
and now `Axes` + `plot` graphing, calculus visuals (Riemann rectangles, area under a curve,
tangent lines), updater-driven dynamic animation (a dot tracing a curve), and linear
transformations of a grid (the "Essence of Linear Algebra" effect). It does **not** yet have
LaTeX/`MathTex`, 3D, or a moving camera — those are **Planned**. In exchange it adds three things classic Manim does not ship: zero-dependency
GIF/PNG/HTML output, a WebAssembly build, and a natural-language frontend.

---

## 6. Design philosophy

manim-nano keeps Manim's **spirit** while deliberately shedding its weight. It animates *math*,
it embraces the **draw-on aesthetic** of `Write`/`Create`, it uses the same **smooth easing**
that makes motion feel intentional, and it sits on the same **dark background** with familiar
color and direction constants — so a Manim user feels at home immediately.

Where it parts ways is in the costs. Classic Manim asks for Python, FFmpeg, OpenGL, and a full
LaTeX install; manim-nano is a single small Rust crate that rasterizes on the CPU with no system
dependencies, ships an embedded bitmap font instead of LaTeX, and writes formats — GIF, PNG, and
a self-contained HTML player — that open anywhere with no extra tooling. It is **tiny,
dependency-light, free, and fully offline** (no API, no key, no network, no telemetry), and the
*same* code runs **everywhere**: a native binary on any OS, or WebAssembly in the browser.

The goal isn't to replace Manim — it's to carry the part of Manim that turns abstract math into
visual intuition into places Manim can't easily go.
