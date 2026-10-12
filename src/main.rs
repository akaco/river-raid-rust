//! The game for computers, phones and tablets.
//!
//! Keys: arrows or WASD fly, Space fires, Enter starts a game,
//! Tab switches between one and two players, 1 toggles straight
//! missiles, P pauses, M mutes, T shows the touch controls (the mouse
//! then works as a finger). On a touch screen: the pad on the left steers
//! (left and right) and changes speed (up and down), the button on the
//! right fires, and small buttons start a game, pick players, pause and
//! mute. After a game ends, FIRE starts the next one.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use std::collections::HashMap;

use delta_run::game::{Game, Input, Phase};
use delta_run::palette::Colour;
use delta_run::screen::{self, Canvas, RIVER_TOP};
#[cfg(feature = "sound")]
use delta_run::synth::Synth;
use delta_run::synth::Voice;
use delta_run::touch::{Button, Layout, PICTURE_H, PICTURE_W, pad_direction};
use macroquad::prelude::*;

/// Pixels are drawn twice as wide as they are tall.
const PIXEL_ASPECT: f32 = 2.0;
const FRAME_TIME: f32 = 1.0 / 60.0;
#[cfg(feature = "sound")]
const SAMPLE_RATE: usize = 44_100;

fn window_conf() -> Conf {
    Conf { window_title: delta_run::GAME_NAME.to_owned(), window_width: 960, window_height: 640, high_dpi: true, ..Default::default() }
}

struct Sound {
    regs: Arc<Mutex<[Voice; 2]>>,
    muted: Arc<AtomicBool>,
    // Keeps the output stream open; sound stops when this is dropped.
    #[cfg(feature = "sound")]
    _device: Option<tinyaudio::OutputDevice>,
}

impl Sound {
    #[cfg(not(feature = "sound"))]
    fn start() -> Sound {
        Sound { regs: Default::default(), muted: Default::default() }
    }

    #[cfg(feature = "sound")]
    fn start() -> Sound {
        let regs = Arc::new(Mutex::new([Voice::default(); 2]));
        let muted = Arc::new(AtomicBool::new(false));
        let params = tinyaudio::OutputDeviceParameters { sample_rate: SAMPLE_RATE, channels_count: 1, channel_sample_count: 1024 };
        let (r, m) = (regs.clone(), muted.clone());
        let mut synth = Synth::new(SAMPLE_RATE as u32);
        let device = tinyaudio::run_output_device(params, move |data: &mut [f32]| {
            if let Ok(regs) = r.lock() {
                synth.set(&regs);
            }
            let volume = if m.load(Ordering::Relaxed) { 0.0 } else { 0.5 };
            for s in data.iter_mut() {
                *s = synth.sample() * volume;
            }
        });
        if let Err(e) = &device {
            eprintln!("no sound: {e}");
        }
        Sound { regs, muted, _device: device.ok() }
    }
}

/// What a finger on the screen is doing, decided when it lands.
#[derive(Clone, Copy)]
enum Finger {
    /// Steering, measured from this point.
    Pad(Vec2),
    Fire,
    /// Pressed one of the small buttons (or started a game); ignored after.
    Done,
}

/// On-screen controls for phones and tablets; see `delta_run::touch`.
#[derive(Default)]
struct Controls {
    /// Shown once the screen has been touched (or T pressed on a computer).
    shown: bool,
    fingers: HashMap<u64, Finger>,
    /// Taps that have not reached the game yet: a tap can be shorter than
    /// a game frame.
    taps: Input,
    /// Lights up the fire button and the pad direction.
    lit: Input,
}

/// Mouse clicks act as a finger when the controls are shown on a computer.
const MOUSE_FINGER: u64 = u64::MAX;

/// A touch that stays inside this many pad radii of the pad steers from
/// the pad's centre; one further away steers from where it landed.
const PAD_REACH: f32 = 1.6;

impl Controls {
    /// The touches as game input, plus the small buttons pressed this frame.
    /// `game_over_for` is how long the game has been over, in frames.
    fn read(&mut self, layout: &Layout, game_over_for: u32) -> (Input, Vec<Button>) {
        let mut all = touches();
        let mouse = is_mouse_button_down(MouseButton::Left) || is_mouse_button_released(MouseButton::Left);
        if all.is_empty() && self.shown && mouse {
            let phase = if is_mouse_button_released(MouseButton::Left) { TouchPhase::Ended } else { TouchPhase::Moved };
            all.push(Touch { id: MOUSE_FINGER, phase, position: mouse_position().into() });
        }
        if !all.is_empty() {
            self.shown = true;
        }
        let mut input = Input::default();
        let mut pressed = vec![];
        for t in &all {
            let p = t.position;
            let finger = match self.fingers.get(&t.id) {
                Some(&f) => f,
                None => {
                    // a new finger: what it does depends on where it lands
                    let f = if let Some(b) = layout.button_at(p.x, p.y) {
                        match b {
                            Button::Start => self.taps.reset = true,
                            Button::Players => self.taps.select = true,
                            _ => {}
                        }
                        pressed.push(b);
                        Finger::Done
                    } else if p.x < layout.split_x {
                        let pad = layout.pad;
                        Finger::Pad(if pad.contains(p.x, p.y, PAD_REACH) { vec2(pad.x, pad.y) } else { p })
                    } else if game_over_for > 90 {
                        // after a game ends, the fire button starts the next one
                        self.taps.reset = true;
                        Finger::Done
                    } else {
                        self.taps.fire = true;
                        Finger::Fire
                    };
                    self.fingers.insert(t.id, f);
                    f
                }
            };
            match finger {
                Finger::Pad(centre) => {
                    let d = p - centre;
                    pad_direction(&mut input, d.x, d.y, layout.pad.r);
                }
                Finger::Fire => input.fire = true,
                Finger::Done => {}
            }
            if matches!(t.phase, TouchPhase::Ended | TouchPhase::Cancelled) {
                self.fingers.remove(&t.id);
            }
        }
        // forget fingers the system dropped without saying
        self.fingers.retain(|id, _| all.iter().any(|t| t.id == *id));
        self.lit = input;
        input.fire |= self.taps.fire;
        input.reset |= self.taps.reset;
        input.select |= self.taps.select;
        (input, pressed)
    }

    /// Called after the game has run at least one frame with the input.
    fn taps_used(&mut self) {
        self.taps = Input::default();
    }

    fn draw(&self, layout: &Layout, paused: bool, muted: bool, two_players: bool) {
        if !self.shown {
            return;
        }
        let dim = Color::new(1.0, 1.0, 1.0, 0.22);
        let bright = Color::new(1.0, 1.0, 1.0, 0.55);
        let lit = Color::new(1.0, 1.0, 1.0, 0.3);
        let pad = layout.pad;
        let line = (pad.r * 0.04).max(1.5);

        // the pad: a ring with a triangle for each direction
        draw_circle(pad.x, pad.y, pad.r, Color::new(1.0, 1.0, 1.0, 0.06));
        draw_circle_lines(pad.x, pad.y, pad.r, line, dim);
        let i = self.lit;
        for (dx, dy, on) in [(-1.0, 0.0, i.left), (1.0, 0.0, i.right), (0.0, -1.0, i.up), (0.0, 1.0, i.down)] {
            let tip = vec2(pad.x + dx * pad.r * 0.8, pad.y + dy * pad.r * 0.8);
            let back = vec2(pad.x + dx * pad.r * 0.45, pad.y + dy * pad.r * 0.45);
            let side = vec2(-dy, dx) * pad.r * 0.2;
            // steering arrows are bigger than the speed ones
            let (back, side) = if dx != 0.0 { (back - vec2(dx, 0.0) * pad.r * 0.1, side * 1.3) } else { (back, side) };
            draw_triangle(tip, back + side, back - side, if on { bright } else { dim });
        }
        let f = layout.fire;
        draw_circle(f.x, f.y, f.r, if i.fire { lit } else { Color::new(1.0, 1.0, 1.0, 0.06) });
        draw_circle_lines(f.x, f.y, f.r, line, if i.fire { bright } else { dim });
        label(f.x, f.y, f.r * 1.2, f.r * 0.4, "FIRE", if i.fire { WHITE } else { bright });

        for (b, r) in layout.buttons {
            let text = match b {
                Button::Start => "START",
                Button::Players => {
                    if two_players {
                        "2 PLAYER"
                    } else {
                        "1 PLAYER"
                    }
                }
                Button::Pause => {
                    if paused {
                        "GO ON"
                    } else {
                        "PAUSE"
                    }
                }
                Button::Sound => {
                    if muted {
                        "MUTED"
                    } else {
                        "SOUND"
                    }
                }
            };
            draw_rectangle_lines(r.x, r.y, r.w, r.h, line, dim);
            label(r.x + r.w / 2.0, r.y + r.h / 2.0, r.w * 0.85, r.h * 0.45, text, bright);
        }
    }
}

/// Text centred on `x`, `y`, as big as fits in `w` x `h`.
fn label(x: f32, y: f32, w: f32, h: f32, text: &str, colour: Color) {
    let width = measure_text(text, None, 100, 1.0).width / 100.0;
    let size = h.min(w / width).max(6.0) as u16;
    let m = measure_text(text, None, size, 1.0);
    draw_text(text, x - m.width / 2.0, y + m.offset_y / 2.0, size as f32, colour);
}

fn keyboard() -> Input {
    let down = |keys: &[KeyCode]| keys.iter().any(|&k| is_key_down(k));
    Input {
        up: down(&[KeyCode::Up, KeyCode::W]),
        down: down(&[KeyCode::Down, KeyCode::S]),
        left: down(&[KeyCode::Left, KeyCode::A]),
        right: down(&[KeyCode::Right, KeyCode::D]),
        fire: down(&[KeyCode::Space]),
        reset: down(&[KeyCode::Enter, KeyCode::KpEnter]),
        select: down(&[KeyCode::Tab]),
        ..Input::default()
    }
}

/// Draws the game's rectangles, in picture units (see the camera in `main`).
struct Window;

impl Canvas for Window {
    fn rect(&mut self, x: u8, y: u8, w: u8, h: u8, colour: Colour) {
        let rgb = colour;
        let c = Color::from_rgba((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8, 255);
        draw_rectangle(x as f32 * PIXEL_ASPECT, (y - RIVER_TOP) as f32, w as f32 * PIXEL_ASPECT, h as f32, c);
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut game = Game::new();
    // the game as the last frame's picture showed it
    let mut shown = game.clone();
    let sound = Sound::start();
    // DELTA_RUN_TOUCH=1 shows the touch controls from the start (for screenshots)
    let mut controls = Controls { shown: std::env::var_os("DELTA_RUN_TOUCH").is_some(), ..Default::default() };
    simulate_mouse_with_touch(false);

    let mut straight_missiles = false;
    let mut paused = false;
    let mut behind = 0.0f32;
    let mut game_over_for = 0u32;

    loop {
        let (sw, sh) = (screen_width(), screen_height());
        let layout = Layout::new(sw, sh, controls.shown);
        let (mut input, pressed) = controls.read(&layout, game_over_for);
        let key = |k| is_key_pressed(k);
        if key(KeyCode::P) || pressed.contains(&Button::Pause) {
            paused = !paused;
        }
        if key(KeyCode::M) || pressed.contains(&Button::Sound) {
            let m = !sound.muted.load(Ordering::Relaxed);
            sound.muted.store(m, Ordering::Relaxed);
        }
        if key(KeyCode::Key1) {
            straight_missiles = !straight_missiles;
        }
        if key(KeyCode::T) {
            controls.shown = !controls.shown;
        }
        if pressed.contains(&Button::Start) {
            paused = false;
        }
        let keys = keyboard();
        input.up |= keys.up;
        input.down |= keys.down;
        input.left |= keys.left;
        input.right |= keys.right;
        input.fire |= keys.fire;
        input.reset |= keys.reset;
        input.select |= keys.select;
        input.straight_missiles = straight_missiles;
        input.one_joystick = true;

        // Run the game at 60 frames a second, whatever the
        // display's refresh rate.
        if paused {
            behind = 0.0;
        } else {
            behind = (behind + get_frame_time()).min(FRAME_TIME * 4.0);
        }
        while behind >= FRAME_TIME {
            behind -= FRAME_TIME;
            game.start_frame(input);
            shown = game.clone();
            game.finish_frame();
            controls.taps_used();
            game_over_for = if game.phase == Phase::GameOver { game_over_for.saturating_add(1) } else { 0 };
        }
        if paused {
            controls.taps_used();
        }
        if let Ok(mut r) = sound.regs.lock() {
            *r = game.sound.out;
            if paused {
                *r = Default::default();
            }
        }

        // Draw the picture where the layout puts it, keeping its shape.
        // The camera works in picture units, so it does not matter whether
        // the screen is measured in points or pixels.
        clear_background(BLACK);
        let p = layout.picture;
        let zoom = vec2(2.0 * p.w / (sw * PICTURE_W), 2.0 * p.h / (sh * PICTURE_H));
        let centre = vec2(2.0 * (p.x + p.w / 2.0) / sw - 1.0, 2.0 * (p.y + p.h / 2.0) / sh - 1.0);
        set_camera(&Camera2D { target: vec2(PICTURE_W / 2.0, PICTURE_H / 2.0) - centre / zoom, zoom, ..Default::default() });
        screen::draw(&shown, &mut Window);
        set_default_camera();
        controls.draw(&layout, paused, sound.muted.load(Ordering::Relaxed), game.two_players);
        if paused {
            label(p.x + p.w / 2.0, p.y + p.h / 2.0, p.w * 0.5, p.h * 0.15, "PAUSED", WHITE);
        }
        next_frame().await
    }
}
