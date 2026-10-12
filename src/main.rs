//! The game for desktop (and later phones and tablets).
//!
//! Keys: arrows or WASD fly, Space fires, Enter starts a game,
//! Tab switches between one and two players, 1 toggles straight
//! missiles, P pauses, M mutes. On a touch screen: drag on the left
//! half to steer and speed up or slow down, touch the right half to fire,
//! and use the buttons along the top to start and pick players.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use delta_run::game::{Game, Input};
use delta_run::palette::Colour;
use delta_run::screen::{self, Canvas, RIVER_TOP, STATUS_BOTTOM};
#[cfg(feature = "sound")]
use delta_run::synth::Synth;
use delta_run::synth::Voice;
use macroquad::prelude::*;

const LINES: f32 = (STATUS_BOTTOM - RIVER_TOP + 1) as f32;
const WIDTH: f32 = screen::WIDTH as f32;
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

/// On-screen controls for touch screens.
#[derive(Default)]
struct Touch {
    used: bool,
    stick_origin: Option<(u64, Vec2)>,
}

const BUTTON_H: f32 = 56.0;

impl Touch {
    /// Adds the touch-screen state to the keyboard input.
    fn read(&mut self, input: &mut Input) {
        let (w, _) = (screen_width(), screen_height());
        let all = touches();
        if !all.is_empty() {
            self.used = true;
        }
        let mut stick_alive = false;
        for t in &all {
            let p = t.position;
            if p.y < BUTTON_H {
                // START and PLAYERS buttons along the top
                if p.x < w * 0.25 {
                    input.reset = true;
                } else if p.x > w * 0.75 {
                    input.select = true;
                }
                continue;
            }
            if p.x < w / 2.0 {
                let origin = match self.stick_origin {
                    Some((id, o)) if id == t.id => o,
                    _ => {
                        self.stick_origin = Some((t.id, p));
                        p
                    }
                };
                if !matches!(t.phase, TouchPhase::Ended | TouchPhase::Cancelled) {
                    stick_alive = true;
                }
                let d = p - origin;
                let dead = 18.0;
                input.left |= d.x < -dead;
                input.right |= d.x > dead;
                input.up |= d.y < -dead;
                input.down |= d.y > dead;
            } else if !matches!(t.phase, TouchPhase::Ended | TouchPhase::Cancelled) {
                input.fire = true;
            }
        }
        if !stick_alive {
            self.stick_origin = None;
        }
    }

    fn draw(&self) {
        if !self.used {
            return;
        }
        let w = screen_width();
        let c = Color::new(1.0, 1.0, 1.0, 0.25);
        draw_rectangle_lines(4.0, 4.0, w * 0.25 - 8.0, BUTTON_H - 8.0, 2.0, c);
        draw_text("START", 16.0, BUTTON_H * 0.65, 28.0, c);
        draw_rectangle_lines(w * 0.75 + 4.0, 4.0, w * 0.25 - 8.0, BUTTON_H - 8.0, 2.0, c);
        draw_text("PLAYERS", w * 0.75 + 16.0, BUTTON_H * 0.65, 28.0, c);
        if let Some((_, o)) = self.stick_origin {
            draw_circle_lines(o.x, o.y, 40.0, 2.0, c);
        }
    }
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
    let mut touch = Touch::default();

    let mut straight_missiles = false;
    let mut paused = false;
    let mut behind = 0.0f32;

    loop {
        if is_key_pressed(KeyCode::P) {
            paused = !paused;
        }
        if is_key_pressed(KeyCode::M) {
            let m = !sound.muted.load(Ordering::Relaxed);
            sound.muted.store(m, Ordering::Relaxed);
        }
        if is_key_pressed(KeyCode::Key1) {
            straight_missiles = !straight_missiles;
        }

        let mut input = keyboard();
        touch.read(&mut input);
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
        }
        if let Ok(mut r) = sound.regs.lock() {
            *r = game.sound.out;
            if paused {
                *r = Default::default();
            }
        }

        // Fit the picture in the window, keeping its shape and centring it.
        // The camera works in picture units, so only the window's shape
        // matters (on Retina screens points and pixels differ).
        clear_background(BLACK);
        let (sw, sh) = (screen_width(), screen_height());
        let pic_w = WIDTH * PIXEL_ASPECT;
        let (view_w, view_h) = if sw / sh > pic_w / LINES { (LINES * sw / sh, LINES) } else { (pic_w, pic_w * sh / sw) };
        set_camera(&Camera2D {
            target: vec2(WIDTH * PIXEL_ASPECT / 2.0, LINES / 2.0),
            zoom: vec2(2.0 / view_w, 2.0 / view_h),
            ..Default::default()
        });
        screen::draw(&shown, &mut Window);
        set_default_camera();
        touch.draw();
        if paused {
            let t = "PAUSED";
            let size = measure_text(t, None, 48, 1.0);
            draw_text(t, (sw - size.width) / 2.0, sh / 2.0, 48.0, WHITE);
        }
        next_frame().await
    }
}
