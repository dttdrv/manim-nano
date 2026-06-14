//! # manim-nano
//!
//! A tiny, fast, **runs-everywhere** animation engine inspired by
//! 3Blue1Brown's [Manim](https://github.com/3b1b/manim).
//!
//! - **Fast & portable:** written in Rust, rendered on the CPU with
//!   [`tiny-skia`] (pure Rust, no system dependencies). The same code compiles
//!   to a native binary on every OS *and* to WebAssembly for the browser.
//! - **Familiar API:** a `Scene` you build up with mobjects and `play(...)`
//!   animations, mirroring Manim's model (`Create`, `Write`, `FadeIn`,
//!   `Transform`, the `.animate`-style helpers, `UP`/`LEFT`/`RED` constants).
//! - **Easy to use:** describe an animation in plain English and the
//!   [`nl`] frontend compiles it into a scene — completely offline, no API,
//!   no key, no network. Free, forever.
//!
//! ```no_run
//! use manim_nano::prelude::*;
//!
//! let mut scene = Scene::preview();
//! let c = scene.add(Mobject::circle(1.5).colored(BLUE));
//! scene.play_one(create(c), 1.0);
//! scene.play_one(shift(c, RIGHT * 2.0), 1.0);
//! scene.export("hello").unwrap(); // writes hello.gif + hello.html
//! ```

pub mod animation;
pub mod color;
pub mod coordinate;
pub mod font;
pub mod geometry;
pub mod mobject;
pub mod nl;
pub mod render;
pub mod scene;

/// Browser (WebAssembly) bindings — compiled only with the `wasm` feature.
#[cfg(feature = "wasm")]
pub mod wasm;

/// Everything you need in one glob import.
pub mod prelude {
    pub use crate::animation::{
        create, fade_in, fade_in_from, fade_out, fade_out_to, grow, move_to, rotate, scale,
        set_color, shift, transform, uncreate, write, AnimSpec, Id,
    };
    pub use crate::color::{
        Color, BLACK, BLUE, BROWN, DARK_BG, GOLD, GREEN, GREY, MAGENTA, MAROON, ORANGE, PINK,
        PURPLE, RED, TEAL, WHITE, YELLOW,
    };
    pub use crate::coordinate::{Axes, Range};
    pub use crate::geometry::{v, Rate, Vec2, DL, DOWN, DR, F, LEFT, ORIGIN, RIGHT, UL, UP, UR};
    pub use crate::mobject::Mobject;
    pub use crate::render::Camera;
    pub use crate::scene::Scene;
}
