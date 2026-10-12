//! Screen layout for phones, tablets and computers, and what a finger on
//! the steering pad means. No window code here, so it can be tested;
//! `main.rs` reads the touches and draws. docs/mobile.md explains the
//! choices and where they come from.
//!
//! The picture is cut in two: the river, which gets as much of the screen
//! as possible, and the status bar (score, fuel, planes), which becomes a
//! small panel. On a phone or tablet the controls are drawn over the
//! river's outer edges, which are always land.
//!
//! Sizes are fractions of the screen, so they work whatever unit the
//! screen is measured in (points on a Mac, pixels on a phone).

use crate::game::Input;
use crate::screen::{RIVER_BOTTOM, RIVER_TOP, STATUS_BOTTOM, WIDTH};

/// The whole picture: 160 double-width pixels by 199 lines.
pub const PICTURE_W: f32 = WIDTH as f32 * 2.0;
pub const PICTURE_H: f32 = (STATUS_BOTTOM - RIVER_TOP + 1) as f32;
/// The river part of the picture, in picture units.
pub const RIVER_H: f32 = (RIVER_BOTTOM - RIVER_TOP + 1) as f32;
/// The outer 16 pixels each side are always bank, so the river can be cut
/// down to the middle 128 pixels without hiding any water.
pub const WATER_W: f32 = PICTURE_W - 2.0 * 32.0;
/// The part of the status bar with the planes left, score and fuel gauge.
pub const HUD: Rect = Rect { x: 32.0, y: RIVER_H + 1.0, w: 192.0, h: PICTURE_H - RIVER_H - 1.0 };

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

    pub fn overlaps(&self, o: &Rect) -> bool {
        self.x < o.x + o.w && o.x < self.x + self.w && self.y < o.y + o.h && o.y < self.y + self.h
    }
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

    pub fn bounds(&self) -> Rect {
        Rect { x: self.x - self.r, y: self.y - self.r, w: 2.0 * self.r, h: 2.0 * self.r }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Button {
    /// Always there: top corner.
    Pause,
    Sound,
    /// Only on the menu (game over or paused), over the middle of the river.
    Start,
    Players,
    Resume,
}

/// Where everything goes on a screen `w` x `h`.
#[derive(Clone, Debug, PartialEq)]
pub struct Layout {
    /// Which part of the picture to show (picture units), and where.
    pub river_from: Rect,
    pub river: Rect,
    /// The status panel; `hud.w` is 0 when the status bar stays part of
    /// the picture (computers).
    pub hud: Rect,
    /// Portrait: the panel under the river that holds the status and the
    /// controls, like the lower half of a handheld console. Zero-sized
    /// otherwise.
    pub deck: Rect,
    /// Margins the system keeps for itself: notch, rounded corners, the
    /// home bar (estimated; the window library does not report them).
    pub safe: Rect,
    /// The steering pad (left thumb) and the fire button (right thumb).
    pub pad: Circle,
    pub fire: Circle,
    pub pause: Rect,
    pub sound: Rect,
    /// The two menu buttons, over the middle of the river.
    pub menu: [Rect; 2],
    /// A touch starting left of this steers; right of it fires.
    pub split_x: f32,
}

impl Layout {
    /// Without touch controls the whole picture fills as much of the
    /// window as it can. With them, the river is made as big as fits and
    /// the controls are laid over it (landscape) or below it (portrait).
    pub fn new(w: f32, h: f32, touch: bool) -> Layout {
        let off = Rect { x: -10.0, y: -10.0, w: 0.0, h: 0.0 };
        let none = Circle { x: -10.0, y: -10.0, r: 0.0 };
        if !touch {
            let s = (w / PICTURE_W).min(h / PICTURE_H);
            let (pw, ph) = (PICTURE_W * s, PICTURE_H * s);
            return Layout {
                river_from: Rect { x: 0.0, y: 0.0, w: PICTURE_W, h: PICTURE_H },
                river: Rect { x: (w - pw) / 2.0, y: (h - ph) / 2.0, w: pw, h: ph },
                hud: off,
                deck: off,
                safe: Rect { x: 0.0, y: 0.0, w, h },
                pad: none,
                fire: none,
                pause: off,
                sound: off,
                menu: [off, off],
                split_x: w / 2.0,
            };
        }

        let short = w.min(h);
        let ratio = w.max(h) / short;
        // Phones are long and thin, tablets nearer square; phones with a
        // notch or island are longer still (19.5:9 and up).
        let phone = ratio > 1.7;
        let notch = ratio > 1.9;
        let portrait = h > w;
        let safe = if !notch {
            let m = short * 0.02;
            Rect { x: m, y: m, w: w - 2.0 * m, h: h - 2.0 * m }
        } else if portrait {
            // status bar / island at the top, home bar at the bottom
            Rect { x: 0.0, y: h * 0.07, w, h: h * 0.89 }
        } else {
            // the notch can be on either side; home bar at the bottom
            Rect { x: w * 0.056, y: 0.0, w: w * 0.888, h: h * 0.945 }
        };

        // Thumb controls about 3 cm across on any device.
        let r = short
            * if !phone {
                0.105
            } else if portrait {
                0.21
            } else {
                0.19
            };
        let gap = r * 0.25;
        let icon = r * 0.62;

        // Scale the river (picture units to screen) so its water fits the
        // safe width, then as tall as the room allows.
        let hud_room = |s: f32| HUD.h * s * 0.5;
        let (s, hud_overlay) = if portrait {
            // below the river: the status panel, then the thumbs
            let below = gap + icon + gap + 2.0 * r + gap;
            let s = (safe.w / WATER_W).min((safe.h - below) / RIVER_H);
            (s, false)
        } else {
            let s = (safe.h / RIVER_H).min(safe.w / WATER_W);
            // Room left under the river for the panel (tablets)? Otherwise
            // it goes over the river's top corner.
            let tall = (safe.h / (RIVER_H + HUD.h * 0.5)).min(safe.w / WATER_W);
            if safe.h - RIVER_H * tall >= hud_room(tall) && tall >= s * 0.97 { (tall, false) } else { (s, true) }
        };
        // Show the banks too where the screen is wide enough.
        let shown_w = PICTURE_W.min(w / s);
        let river_from = Rect { x: (PICTURE_W - shown_w) / 2.0, y: 0.0, w: shown_w, h: RIVER_H };
        let (rw, rh) = (shown_w * s, RIVER_H * s);
        let top = if portrait { safe.y } else { safe.y + if hud_overlay { (safe.h - rh) / 2.0 } else { 0.0 } };
        let river = Rect { x: (w - rw) / 2.0, y: top, w: rw, h: rh };

        let hud_s = if portrait { (s * 0.8).min((w - 2.0 * icon - 4.0 * gap) / HUD.w) } else { s * 0.5 };
        let (hw, hh) = (HUD.w * hud_s, HUD.h * hud_s);
        let water_left = river.x + (river_from.x.max(32.0) - river_from.x) * s;
        let hud = if portrait {
            // centred under the river, between the sound and pause buttons
            Rect { x: (w - hw) / 2.0, y: river.y + rh + gap + (icon - hh).max(0.0) / 2.0, w: hw, h: hh }
        } else if hud_overlay {
            Rect { x: water_left.max(safe.x + gap * 0.5), y: river.y + gap * 0.5, w: hw, h: hh }
        } else {
            Rect { x: (w - hw) / 2.0, y: river.y + rh + gap * 0.25, w: hw, h: hh }
        };

        let corner_y = if portrait { hud.y + (hh - icon) / 2.0 } else { safe.y + gap * 0.5 };
        let right = safe.x + safe.w - gap * 0.5;
        let left = safe.x + gap * 0.5;
        let (pause, sound) = if portrait {
            (Rect { x: right - icon, y: corner_y, w: icon, h: icon }, Rect { x: left, y: corner_y, w: icon, h: icon })
        } else {
            (
                Rect { x: right - icon, y: corner_y, w: icon, h: icon },
                Rect { x: right - 2.0 * icon - gap * 0.5, y: corner_y, w: icon, h: icon },
            )
        };

        // Thumbs low in the corners, where they rest.
        let fire_r = r * 0.9;
        let thumbs_y = safe.y + safe.h - gap - r;
        let pad = Circle { x: safe.x + gap + r, y: thumbs_y, r };
        let fire = Circle { x: safe.x + safe.w - gap - r, y: thumbs_y, r: fire_r };

        // Menu: two wide buttons in the middle of the river.
        let mw = (river.w * 0.42).min(r * 3.2);
        let mh = (river.h * 0.16).min(r * 0.8);
        let my = river.y + river.h * 0.42;
        let menu = [
            Rect { x: w / 2.0 - mw / 2.0, y: my - mh - gap * 0.3, w: mw, h: mh },
            Rect { x: w / 2.0 - mw / 2.0, y: my + gap * 0.3, w: mw, h: mh },
        ];

        let deck = if portrait { Rect { x: 0.0, y: river.y + rh, w, h: h - river.y - rh } } else { off };

        Layout { river_from, river, hud, deck, safe, pad, fire, pause, sound, menu, split_x: w / 2.0 }
    }

    /// The button at `x`, `y`, if any; the menu buttons only count while
    /// the menu is up.
    pub fn button_at(&self, x: f32, y: f32, menu: [Button; 2], menu_up: bool) -> Option<Button> {
        if self.pause.contains(x, y) {
            return Some(Button::Pause);
        }
        if self.sound.contains(x, y) {
            return Some(Button::Sound);
        }
        if menu_up {
            for (b, r) in menu.into_iter().zip(self.menu) {
                if r.contains(x, y) {
                    return Some(b);
                }
            }
        }
        None
    }

    /// Screen x of picture x `px` (picture units) on the river.
    pub fn river_x(&self, px: f32) -> f32 {
        self.river.x + (px - self.river_from.x) * self.river.w / self.river_from.w
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

    fn inside(outer: &Rect, r: &Rect) -> bool {
        r.x >= outer.x - 0.01 && r.y >= outer.y - 0.01 && r.x + r.w <= outer.x + outer.w + 0.01 && r.y + r.h <= outer.y + outer.h + 0.01
    }

    /// iPhone, Android phones, iPad, Android tablet, older 16:9 phone.
    const SCREENS: [(f32, f32); 6] = [(390.0, 844.0), (1080.0, 2400.0), (768.0, 1024.0), (1668.0, 2388.0), (800.0, 1280.0), (375.0, 667.0)];

    #[test]
    fn layouts_fit_on_screen() {
        for (w, h) in SCREENS {
            for (w, h) in [(w, h), (h, w)] {
                let l = Layout::new(w, h, true);
                let what = format!("{w}x{h}");
                let screen = Rect { x: 0.0, y: 0.0, w, h };
                // the water (the middle of the river picture) is inside the safe area
                let water_l = l.river_x(32.0);
                let water_r = l.river_x(PICTURE_W - 32.0);
                assert!(water_l >= l.safe.x - 0.5 && water_r <= l.safe.x + l.safe.w + 0.5, "{what}: water {water_l}..{water_r}");
                assert!(l.river.y >= l.safe.y - 0.5 && l.river.y + l.river.h <= l.safe.y + l.safe.h + 0.5, "{what}: river {:?}", l.river);
                assert!(inside(&screen, &l.river), "{what}: river on screen");
                for (name, r) in
                    [("hud", l.hud), ("pause", l.pause), ("sound", l.sound), ("pad", l.pad.bounds()), ("fire", l.fire.bounds())]
                {
                    assert!(inside(&l.safe, &r), "{what}: {name} {r:?} outside {:?}", l.safe);
                }
                assert!(!l.pad.bounds().overlaps(&l.fire.bounds()), "{what}: pad and fire apart");
                assert!(l.pad.x < l.split_x && l.fire.x > l.split_x, "{what}: pad left, fire right");
                for r in [l.pause, l.sound] {
                    assert!(!r.overlaps(&l.hud) && !r.overlaps(&l.pad.bounds()) && !r.overlaps(&l.fire.bounds()), "{what}: {r:?}");
                }
                assert!(!l.pause.overlaps(&l.sound), "{what}");
                for m in l.menu {
                    assert!(inside(&l.river, &m), "{what}: menu on the river");
                }
                // the river keeps its shape
                let s = l.river.w / l.river_from.w;
                assert!((l.river.h / l.river_from.h - s).abs() < 0.01, "{what}: shape");
                if h > w {
                    // portrait: the river fills the width and nothing covers it
                    assert!(water_r - water_l >= l.safe.w * 0.99, "{what}: river fills the width");
                    for r in [l.hud, l.pause, l.sound, l.pad.bounds(), l.fire.bounds()] {
                        assert!(!r.overlaps(&l.river), "{what}: {r:?} covers the river");
                    }
                } else {
                    // landscape: the river is as tall as the safe area, or its water fills the width
                    let tall = l.river.h >= l.safe.h * 0.85;
                    let wide = water_r - water_l >= l.safe.w * 0.99;
                    assert!(tall || wide, "{what}: river {:?} too small", l.river);
                    // the thumbs cover the bank more than the water
                    assert!(l.pad.x - l.pad.r * 0.3 < water_l + (water_r - water_l) * 0.15, "{what}: pad too far in");
                }
            }
        }
    }

    #[test]
    fn desktop_fills_window() {
        let l = Layout::new(960.0, 640.0, false);
        assert_eq!(l.river.w, 960.0);
        assert_eq!(l.river_from.h, PICTURE_H);
        assert_eq!(l.hud.w, 0.0);
    }
}
