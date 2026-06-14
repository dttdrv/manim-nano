//! The `Scene` ties everything together: it holds mobjects, plays animations,
//! and exports the result as an animated GIF, PNG frames, or a standalone
//! HTML player — the formats that play *everywhere* without extra tooling.

use crate::animation::{Anim, AnimSpec, Id, Outcome};
use crate::color::{self, Color};
use crate::geometry::{Rate, Vec2, F};
use crate::mobject::Mobject;
use crate::render::{render_mobject, Camera};
use std::collections::HashMap;
use std::io;
use std::path::Path;
use tiny_skia::Pixmap;

pub struct Scene {
    pub cam: Camera,
    pub bg: Color,
    pub fps: u32,
    items: Vec<Option<Mobject>>,
    frames: Vec<Pixmap>,
}

impl Scene {
    /// A new scene at the given pixel resolution (16:9 world frame, 8 units tall).
    pub fn new(width: u32, height: u32) -> Self {
        Scene {
            cam: Camera::new(width, height, 8.0),
            bg: color::DARK_BG,
            fps: 30,
            items: Vec::new(),
            frames: Vec::new(),
        }
    }

    /// A 480x270 scene — small and fast, good for previews and CI.
    pub fn preview() -> Self {
        Scene::new(480, 270)
    }

    // ---- Builder-style configuration ------------------------------------

    pub fn background(mut self, c: Color) -> Self {
        self.bg = c;
        self
    }
    pub fn fps(mut self, fps: u32) -> Self {
        self.fps = fps.max(1);
        self
    }
    pub fn frame_height(mut self, h: F) -> Self {
        self.cam.frame_height = h;
        self
    }
    pub fn camera_center(mut self, c: Vec2) -> Self {
        self.cam.center = c;
        self
    }

    /// `(half_width, half_height)` of the visible frame in world units.
    pub fn half_frame(&self) -> (F, F) {
        self.cam.half_frame()
    }

    // ---- Mobject management ---------------------------------------------

    /// Add a mobject; it becomes visible immediately. Returns its handle.
    pub fn add(&mut self, mob: Mobject) -> Id {
        self.items.push(Some(mob));
        self.items.len() - 1
    }

    pub fn get(&self, id: Id) -> Option<&Mobject> {
        self.items.get(id).and_then(|o| o.as_ref())
    }
    pub fn get_mut(&mut self, id: Id) -> Option<&mut Mobject> {
        self.items.get_mut(id).and_then(|o| o.as_mut())
    }

    // ---- Playback -------------------------------------------------------

    /// Play one or more animations in parallel over `run_time` seconds.
    pub fn play(&mut self, specs: &[AnimSpec], run_time: F, rate: Rate) {
        let resolved: Vec<Anim> = specs
            .iter()
            .map(|s| {
                let cur = s.target().and_then(|i| self.get(i));
                Anim::resolve(s, cur)
            })
            .collect();

        let n = ((run_time * self.fps as F).round() as usize).max(1);
        for f in 1..=n {
            let alpha = rate.apply(f as F / n as F);
            let mut overrides: HashMap<Id, Mobject> = HashMap::new();
            for anim in &resolved {
                if let (Some(id), Some(m)) = (anim.target(), anim.apply(alpha)) {
                    overrides.insert(id, m);
                }
            }
            let pm = self.render_frame(&overrides);
            self.frames.push(pm);
        }

        for anim in &resolved {
            match anim.outcome() {
                Outcome::Keep(id, m) => {
                    if let Some(slot) = self.items.get_mut(id) {
                        *slot = Some(m);
                    }
                }
                Outcome::Remove(id) => {
                    if let Some(slot) = self.items.get_mut(id) {
                        *slot = None;
                    }
                }
                Outcome::Nothing => {}
            }
        }
    }

    /// Convenience: play a single animation with the default (smooth) rate.
    pub fn play_one(&mut self, spec: AnimSpec, run_time: F) {
        self.play(&[spec], run_time, Rate::default());
    }

    /// Play several animations together with the default rate.
    pub fn play_all(&mut self, specs: &[AnimSpec], run_time: F) {
        self.play(specs, run_time, Rate::default());
    }

    /// Hold the current frame for `seconds`.
    pub fn wait(&mut self, seconds: F) {
        let n = ((seconds * self.fps as F).round() as usize).max(1);
        let empty = HashMap::new();
        for _ in 0..n {
            let pm = self.render_frame(&empty);
            self.frames.push(pm);
        }
    }

    fn render_frame(&self, overrides: &HashMap<Id, Mobject>) -> Pixmap {
        let mut pm = Pixmap::new(self.cam.width, self.cam.height).expect("valid pixmap dimensions");
        pm.fill(self.bg.to_skia(1.0));
        for (i, item) in self.items.iter().enumerate() {
            if let Some(m) = overrides.get(&i) {
                render_mobject(&mut pm, &self.cam, m);
            } else if let Some(m) = item {
                render_mobject(&mut pm, &self.cam, m);
            }
        }
        pm
    }

    fn ensure_frames(&mut self) {
        if self.frames.is_empty() {
            let empty = HashMap::new();
            let pm = self.render_frame(&empty);
            self.frames.push(pm);
        }
    }

    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }

    // ---- Exporters ------------------------------------------------------

    /// Encode the animation as a looping GIF into any [`io::Write`] sink.
    ///
    /// Shared by [`Scene::save_gif`] (file output) and the WASM entry point
    /// (in-memory `Vec<u8>` output), so both produce identical bytes.
    fn write_gif<W: io::Write>(&mut self, writer: W) -> io::Result<()> {
        self.ensure_frames();
        let (w, h) = (self.cam.width as u16, self.cam.height as u16);
        let mut encoder = gif::Encoder::new(writer, w, h, &[]).map_err(io::Error::other)?;
        encoder
            .set_repeat(gif::Repeat::Infinite)
            .map_err(io::Error::other)?;
        let delay = ((100.0 / self.fps as F).round() as u16).max(2);
        for pm in &self.frames {
            let mut rgba = pm.data().to_vec();
            let mut frame = gif::Frame::from_rgba_speed(w, h, &mut rgba, 10);
            frame.delay = delay;
            encoder.write_frame(&frame).map_err(io::Error::other)?;
        }
        Ok(())
    }

    /// Encode the animation as a looping GIF into an in-memory buffer.
    ///
    /// Handy where there is no filesystem — e.g. returning the bytes to the
    /// browser from WebAssembly.
    pub fn encode_gif(&mut self) -> io::Result<Vec<u8>> {
        let mut buf: Vec<u8> = Vec::new();
        self.write_gif(io::Cursor::new(&mut buf))?;
        Ok(buf)
    }

    /// Write the animation as a looping GIF (plays in every browser & viewer).
    pub fn save_gif(&mut self, path: impl AsRef<Path>) -> io::Result<()> {
        create_parent(path.as_ref())?;
        let file = std::fs::File::create(path)?;
        self.write_gif(io::BufWriter::new(file))
    }

    /// Write every frame as a numbered PNG into `dir`.
    pub fn save_frames(&mut self, dir: impl AsRef<Path>) -> io::Result<()> {
        self.ensure_frames();
        let dir = dir.as_ref();
        std::fs::create_dir_all(dir)?;
        for (i, pm) in self.frames.iter().enumerate() {
            let path = dir.join(format!("frame_{i:05}.png"));
            pm.save_png(&path).map_err(io::Error::other)?;
        }
        Ok(())
    }

    /// Write just the final frame as a PNG.
    pub fn save_last_png(&mut self, path: impl AsRef<Path>) -> io::Result<()> {
        self.ensure_frames();
        let pm = self.frames.last().unwrap();
        pm.save_png(path).map_err(io::Error::other)
    }

    /// Write a tiny self-contained HTML page that displays `media_filename`
    /// (typically the sibling GIF) — so the result plays in any browser.
    pub fn save_html(&self, path: impl AsRef<Path>, media_filename: &str) -> io::Result<()> {
        create_parent(path.as_ref())?;
        let html = format!(
            "<!doctype html>\n<html><head><meta charset=\"utf-8\">\n\
             <title>manim-nano</title>\n\
             <style>html,body{{margin:0;height:100%;display:grid;place-items:center;\
             background:#0b0b0e;font-family:system-ui,sans-serif;color:#ccc}}\
             img{{max-width:95vw;max-height:85vh;image-rendering:auto;\
             box-shadow:0 8px 40px rgba(0,0,0,.6);border-radius:6px}}\
             p{{opacity:.6;font-size:13px}}</style></head>\n\
             <body><div><img src=\"{media}\" alt=\"animation\">\
             <p>rendered with manim-nano</p></div></body></html>\n",
            media = media_filename
        );
        std::fs::write(path, html)
    }

    /// Convenience: write `<base>.gif` and a sibling `<base>.html` player.
    pub fn export(&mut self, base: &str) -> io::Result<()> {
        let gif_path = format!("{base}.gif");
        self.save_gif(&gif_path)?;
        let media = Path::new(&gif_path)
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| gif_path.clone());
        self.save_html(format!("{base}.html"), &media)?;
        Ok(())
    }
}

fn create_parent(path: &Path) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::animation::{create, shift};
    use crate::geometry::RIGHT;

    #[test]
    fn plays_and_counts_frames() {
        let mut s = Scene::new(64, 36).fps(10);
        let c = s.add(Mobject::circle(1.0));
        s.play_one(create(c), 1.0); // ~10 frames
        s.play_one(shift(c, RIGHT * 2.0), 1.0); // ~10 more
        s.wait(0.5); // ~5 more
        assert!(s.frame_count() >= 24);
    }

    #[test]
    fn removal_via_fade_out() {
        let mut s = Scene::new(32, 32).fps(4);
        let c = s.add(Mobject::circle(1.0));
        s.play_one(crate::animation::fade_out(c), 1.0);
        assert!(s.get(c).is_none());
    }

    #[test]
    fn exports_gif(/* smoke test for the encoder */) {
        let mut s = Scene::new(32, 32).fps(4);
        let c = s.add(Mobject::circle(1.0).colored(color::RED));
        s.play_one(create(c), 0.5);
        let dir = std::env::temp_dir().join("manim_nano_test");
        std::fs::create_dir_all(&dir).unwrap();
        let gif = dir.join("t.gif");
        s.save_gif(&gif).unwrap();
        assert!(std::fs::metadata(&gif).unwrap().len() > 0);
    }
}
