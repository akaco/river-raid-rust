//! On-screen controls for phones and tablets: where the picture, the pad,
//! the fire button and the small buttons go on a screen of any shape, and
//! what a finger on the pad means. No window code here, so it can be
//! tested; `main.rs` reads the touches and draws.
//!
//! Sizes are fractions of the screen, so they work whatever unit the
//! screen is measured in (points on a Mac, pixels on a phone).

use crate::game::Input;
use crate::screen::{RIVER_TOP, STATUS_BOTTOM, WIDTH};

/// The picture's shape: 160 double-width pixels by 199 lines.
pub const PICTURE_W: f32 = WIDTH as f32 * 2.0;
pub const PICTURE_H: f32 = (STATUS_BOTTOM - RIVER_TOP + 1) as f32;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub fn contains(&self, x: f32, y: f32) -> bool {
        x >= self.x && x < self.x + self.w && y >= self.y && y < self.y + self.h
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Button {
    Start,
    Players,
    Pause,
    Sound,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Circle {
    pub x: f32,
    pub y: f32,
    pub r: f32,
}

impl Circle {
    pub fn contains(&self, x: f32, y: f32, scale: f32) -> bool {
        (x - self.x).powi(2) + (y - self.y).powi(2) <= (self.r * scale).powi(2)
    }
}

/// Where everything goes on a screen `w` x `h`.
#[derive(Clone, Debug, PartialEq)]
pub struct Layout {
    /// The game picture.
    pub picture: Rect,
    /// The steering pad (left thumb).
    pub pad: Circle,
    /// The fire button (right thumb).
    pub fire: Circle,
    pub buttons: [(Button, Rect); 4],
    /// A touch starting left of this steers; right of it fires.
    pub split_x: f32,
}

impl Layout {
    /// Without touch controls the picture fills as much of the screen as
    /// it can. With them, it makes room: in portrait the picture goes at
    /// the top and the controls below it; in landscape the controls go
    /// either side of it.
    pub fn new(w: f32, h: f32, touch: bool) -> Layout {
        let fit =
            |w: f32, h: f32| if w / h > PICTURE_W / PICTURE_H { (h * PICTURE_W / PICTURE_H, h) } else { (w, w * PICTURE_H / PICTURE_W) };
        let short = w.min(h);
        // Phones are long and thin, tablets nearer square. Phones get
        // bigger controls for the size of the screen, so that on both
        // the pad is about 3 cm across.
        let phone = w.max(h) / short > 1.7;
        // (a little smaller on a phone held sideways, where room is short)
        let size = match (phone, h > w) {
            (false, _) => 0.11,
            (true, true) => 0.2,
            (true, false) => 0.18,
        };
        let r = short * size;
        let gap = r * 0.3;
        let fire_r = r * 0.85;
        let button_h = r * 0.5;

        if !touch {
            let (pw, ph) = fit(w, h);
            let picture = Rect { x: (w - pw) / 2.0, y: (h - ph) / 2.0, w: pw, h: ph };
            let off = Rect { x: -1.0, y: -1.0, w: 0.0, h: 0.0 };
            let none = Circle { x: -1.0, y: -1.0, r: 0.0 };
            return Layout {
                picture,
                pad: none,
                fire: none,
                buttons: [(Button::Start, off), (Button::Players, off), (Button::Pause, off), (Button::Sound, off)],
                split_x: w / 2.0,
            };
        }

        if h > w {
            // Portrait: picture at the top, below the notch; a row of
            // small buttons; then the pad and fire button at thumb height.
            let top = h * if phone { 0.06 } else { 0.03 };
            let bottom = h * if phone { 0.05 } else { 0.03 };
            let controls = gap + button_h + gap + 2.0 * r + bottom;
            let (pw, ph) = fit(w, (h - top - controls).max(0.0));
            let picture = Rect { x: (w - pw) / 2.0, y: top, w: pw, h: ph };
            let button_w = (r * 1.4).min((w - 5.0 * gap) / 4.0);
            let row_gap = (w - 4.0 * button_w) / 5.0;
            let row_y = top + ph + gap;
            let at = |i: f32| Rect { x: row_gap + i * (button_w + row_gap), y: row_y, w: button_w, h: button_h };
            let thumbs_y = h - bottom - r;
            let side = w * 0.05;
            Layout {
                picture,
                pad: Circle { x: side + r, y: thumbs_y, r },
                fire: Circle { x: w - side - r, y: thumbs_y, r: fire_r },
                buttons: [(Button::Start, at(0.0)), (Button::Players, at(1.0)), (Button::Pause, at(2.0)), (Button::Sound, at(3.0))],
                split_x: w / 2.0,
            }
        } else {
            // Landscape: controls either side of the picture. The picture
            // shrinks a little if that makes room for them, but never
            // below 72% of full size; past that the controls sit over its
            // lower corners.
            let notch = if phone { w * 0.05 } else { 0.0 };
            let (full_w, _) = fit(w, h);
            let room = w - 2.0 * (notch + 2.0 * r + 2.0 * gap);
            let pw = full_w.min(room.max(full_w * 0.72));
            let ph = pw * PICTURE_H / PICTURE_W;
            let picture = Rect { x: (w - pw) / 2.0, y: (h - ph) / 2.0, w: pw, h: ph };
            let margin = picture.x;
            // centre the pad in the margin if it fits, else keep it in from the edge
            let pad_x = ((notch + margin) / 2.0).max(notch + gap + r);
            let thumbs_y = h - h * 0.08 - r;
            let button_w = (r * 1.4).min((margin - notch - 2.0 * gap).max(r));
            let column = |x: f32, i: f32| Rect { x: x - button_w / 2.0, y: gap + i * (button_h + gap), w: button_w, h: button_h };
            Layout {
                picture,
                pad: Circle { x: pad_x, y: thumbs_y, r },
                fire: Circle { x: w - pad_x, y: thumbs_y, r: fire_r },
                buttons: [
                    (Button::Start, column(pad_x, 0.0)),
                    (Button::Players, column(pad_x, 1.0)),
                    (Button::Pause, column(w - pad_x, 0.0)),
                    (Button::Sound, column(w - pad_x, 1.0)),
                ],
                split_x: w / 2.0,
            }
        }
    }

    pub fn button_at(&self, x: f32, y: f32) -> Option<Button> {
        self.buttons.iter().find(|(_, r)| r.contains(x, y)).map(|&(b, _)| b)
    }
}

/// Sets the stick in `input` for a finger `dx`, `dy` from the centre of a
/// pad of radius `r`. Eight directions; steering wins near the diagonals,
/// so a sideways push does not change speed by accident.
pub fn pad_direction(input: &mut Input, dx: f32, dy: f32, r: f32) {
    let (ax, ay) = (dx.abs(), dy.abs());
    if ax.hypot(ay) < r * 0.2 {
        return;
    }
    if ax > ay * 0.4 {
        input.left |= dx < 0.0;
        input.right |= dx > 0.0;
    }
    if ay > ax * 0.7 {
        input.up |= dy < 0.0;
        input.down |= dy > 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dir(dx: f32, dy: f32) -> (bool, bool, bool, bool) {
        let mut i = Input::default();
        pad_direction(&mut i, dx, dy, 100.0);
        (i.left, i.right, i.up, i.down)
    }

    #[test]
    fn pad_directions() {
        assert_eq!(dir(5.0, 5.0), (false, false, false, false), "dead centre");
        assert_eq!(dir(-60.0, 0.0), (true, false, false, false));
        assert_eq!(dir(60.0, -20.0), (false, true, false, false), "mostly sideways steers only");
        assert_eq!(dir(0.0, -60.0), (false, false, true, false));
        assert_eq!(dir(10.0, 60.0), (false, false, false, true));
        assert_eq!(dir(-50.0, -50.0), (true, false, true, false), "diagonal");
    }

    fn inside(outer: (f32, f32), r: Rect) -> bool {
        r.x >= -0.01 && r.y >= -0.01 && r.x + r.w <= outer.0 + 0.01 && r.y + r.h <= outer.1 + 0.01
    }

    fn overlaps(a: Rect, b: Rect) -> bool {
        a.x < b.x + b.w && b.x < a.x + a.w && a.y < b.y + b.h && b.y < a.y + a.h
    }

    fn circle_box(c: Circle) -> Rect {
        Rect { x: c.x - c.r, y: c.y - c.r, w: 2.0 * c.r, h: 2.0 * c.r }
    }

    #[test]
    fn layouts_fit_on_screen() {
        // phones and tablets, both ways round
        for (w, h) in [(390.0, 844.0), (1080.0, 2400.0), (768.0, 1024.0), (1668.0, 2388.0), (800.0, 1280.0)] {
            for (w, h) in [(w, h), (h, w)] {
                let l = Layout::new(w, h, true);
                let s = (w, h);
                let what = format!("{w}x{h}");
                assert!(inside(s, l.picture), "{what}: picture {:?}", l.picture);
                assert!(inside(s, circle_box(l.pad)) && inside(s, circle_box(l.fire)), "{what}: thumbs");
                assert!(!overlaps(circle_box(l.pad), circle_box(l.fire)), "{what}: pad and fire apart");
                assert!(l.pad.x < l.split_x && l.fire.x > l.split_x, "{what}: pad left, fire right");
                for (b, r) in l.buttons {
                    assert!(inside(s, r), "{what}: {b:?} {r:?}");
                    assert!(!overlaps(r, circle_box(l.pad)) && !overlaps(r, circle_box(l.fire)), "{what}: {b:?} clear of thumbs");
                }
                // the picture keeps its shape and stays big
                assert!((l.picture.w / l.picture.h - PICTURE_W / PICTURE_H).abs() < 0.01, "{what}: shape");
                assert!(l.picture.w >= 0.7 * w.min(h * PICTURE_W / PICTURE_H), "{what}: picture too small");
                if h > w {
                    // portrait: nothing covers the picture
                    assert!(!overlaps(l.picture, circle_box(l.pad)) && !overlaps(l.picture, circle_box(l.fire)), "{what}");
                    assert!(l.buttons.iter().all(|&(_, r)| !overlaps(l.picture, r)), "{what}");
                }
            }
        }
    }

    #[test]
    fn desktop_fills_window() {
        let l = Layout::new(960.0, 640.0, false);
        assert_eq!(l.picture.h, 960.0 * PICTURE_H / PICTURE_W);
        assert_eq!(l.picture.w, 960.0);
    }
}
