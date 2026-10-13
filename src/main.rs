//! The game for computers, phones and tablets.
//!
//! Keys: arrows or WASD fly, Space fires, Enter starts a game,
//! Tab switches between one and two players, 1 toggles straight
//! missiles, P pauses, M mutes, T shows the touch controls (the mouse
//! then works as a finger). Game controllers work too (see `gamepad`).
//!
//! On a touch screen: the pad on the left steers (left and right) and
//! changes speed (up and down), the button on the right fires, pause and
//! sound are in the top corner, and a menu over the river starts a game.
//! After a game ends, FIRE starts the next one too.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use delta_run::game::{Game, Input, Phase};
use delta_run::palette::{self, Colour};
use delta_run::screen::{self, Canvas, RIVER_TOP};
#[cfg(feature = "sound")]
use delta_run::synth::Synth;
use delta_run::synth::Voice;
use delta_run::touch::{self, Button, Layout, PICTURE_H, PICTURE_W, pad_direction};
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
    /// Pressed a button (or started a game); ignored after.
    Done,
}

/// On-screen controls for phones and tablets; see `delta_run::touch`.
#[derive(Default)]
struct Controls {
    /// Shown once the screen has been touched (or T pressed on a computer);
    /// hidden again when a key or game controller is used.
    shown: bool,
    fingers: HashMap<u64, Finger>,
    /// Taps that have not reached the game yet: a tap can be shorter than
    /// a game frame.
    taps: Input,
    /// What the fingers are pressing, to light up the controls.
    lit: Input,
    /// Where the steering finger is, for the knob.
    knob: Option<(Vec2, Vec2)>,
}

/// Mouse clicks act as a finger when the controls are shown on a computer.
const MOUSE_FINGER: u64 = u64::MAX;

/// A touch that stays inside this many pad radii of the pad steers from
/// the pad's centre; one further away steers from where it landed.
const PAD_REACH: f32 = 1.6;

/// What the menu over the river offers, if it is up.
fn menu(game: &Game, paused: bool, game_over_for: u32) -> Option<[Button; 2]> {
    if paused {
        Some([Button::Resume, Button::Start])
    } else if game.phase == Phase::GameOver && game_over_for > 60 {
        Some([Button::Start, Button::Players])
    } else {
        None
    }
}

impl Controls {
    /// The touches as game input, plus the buttons pressed this frame.
    fn read(&mut self, layout: &Layout, menu: Option<[Button; 2]>) -> (Input, Vec<Button>) {
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
        self.knob = None;
        for t in &all {
            let p = t.position;
            let finger = match self.fingers.get(&t.id) {
                Some(&f) => f,
                None => {
                    // a new finger: what it does depends on where it lands
                    let menu_buttons = menu.unwrap_or([Button::Start, Button::Players]);
                    let f = if let Some(b) = layout.button_at(p.x, p.y, menu_buttons, menu.is_some()) {
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
                    } else if menu == Some([Button::Start, Button::Players]) {
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
                    self.knob = Some((centre, d));
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

    fn draw(&self, layout: &Layout, menu: Option<[Button; 2]>, state: &Shown) {
        let r = layout.pad.r;
        let line = (r * 0.035).max(1.5);
        let shade = Color::new(0.0, 0.0, 0.0, 0.28);
        let ink = Color::new(1.0, 1.0, 1.0, 0.45);
        let strong = Color::new(1.0, 1.0, 1.0, 0.85);

        if self.shown {
            // the pad: a dark disc with a ring, arrows, and a knob under the thumb
            let pad = layout.pad;
            draw_circle(pad.x, pad.y, pad.r, shade);
            draw_circle_lines(pad.x, pad.y, pad.r, line, ink);
            let i = self.lit;
            for (dx, dy, on) in [(-1.0, 0.0, i.left), (1.0, 0.0, i.right), (0.0, -1.0, i.up), (0.0, 1.0, i.down)] {
                let tip = vec2(pad.x + dx * pad.r * 0.86, pad.y + dy * pad.r * 0.86);
                let back = vec2(pad.x + dx * pad.r * 0.64, pad.y + dy * pad.r * 0.64);
                // steering arrows are bigger than the speed ones
                let side = vec2(-dy, dx) * pad.r * if dx != 0.0 { 0.2 } else { 0.14 };
                draw_triangle(tip, back + side, back - side, if on { strong } else { ink });
            }
            let (centre, d) = self.knob.unwrap_or((vec2(pad.x, pad.y), Vec2::ZERO));
            let d = d.clamp_length_max(pad.r * 0.45);
            let held = self.knob.is_some();
            draw_circle(centre.x + d.x, centre.y + d.y, pad.r * 0.36, Color::new(1.0, 1.0, 1.0, if held { 0.4 } else { 0.16 }));
            if held && centre != vec2(pad.x, pad.y) {
                // steering from where the finger landed
                draw_circle_lines(centre.x, centre.y, pad.r * 0.8, line, ink);
            }

            let f = layout.fire;
            draw_circle(f.x, f.y, f.r, if i.fire { Color::new(1.0, 0.35, 0.2, 0.45) } else { shade });
            draw_circle_lines(f.x, f.y, f.r, line * 1.5, if i.fire { strong } else { ink });
            label(f.x, f.y, f.r * 1.1, f.r * 0.36, "FIRE", if i.fire { WHITE } else { strong });

            icon_button(layout.pause, line, if state.paused { Icon::Play } else { Icon::Pause });
            icon_button(layout.sound, line, if state.muted { Icon::Muted } else { Icon::Sound });
        }

        if let Some(buttons) = menu {
            for (b, rect) in buttons.into_iter().zip(layout.menu) {
                let text = match b {
                    Button::Start => {
                        if state.paused {
                            "NEW GAME"
                        } else {
                            "START"
                        }
                    }
                    Button::Players => {
                        if state.two_players {
                            "2 PLAYERS"
                        } else {
                            "1 PLAYER"
                        }
                    }
                    Button::Resume => "RESUME",
                    Button::Pause | Button::Sound => "",
                };
                if self.shown {
                    draw_rectangle(rect.x, rect.y, rect.w, rect.h, Color::new(0.0, 0.0, 0.0, 0.6));
                    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, line * 1.5, strong);
                    label(rect.x + rect.w / 2.0, rect.y + rect.h / 2.0, rect.w * 0.8, rect.h * 0.5, text, WHITE);
                }
            }
        }
    }
}

/// What the controls show besides the fingers.
struct Shown {
    paused: bool,
    muted: bool,
    two_players: bool,
}

enum Icon {
    Pause,
    Play,
    Sound,
    Muted,
}

/// A round button with a little picture.
fn icon_button(r: touch::Rect, line: f32, icon: Icon) {
    let (cx, cy, u) = (r.x + r.w / 2.0, r.y + r.h / 2.0, r.w / 2.0);
    let ink = Color::new(1.0, 1.0, 1.0, 0.8);
    draw_circle(cx, cy, u, Color::new(0.0, 0.0, 0.0, 0.35));
    draw_circle_lines(cx, cy, u, line, Color::new(1.0, 1.0, 1.0, 0.45));
    match icon {
        Icon::Pause => {
            draw_rectangle(cx - u * 0.32, cy - u * 0.38, u * 0.22, u * 0.76, ink);
            draw_rectangle(cx + u * 0.1, cy - u * 0.38, u * 0.22, u * 0.76, ink);
        }
        Icon::Play => draw_triangle(vec2(cx - u * 0.25, cy - u * 0.4), vec2(cx - u * 0.25, cy + u * 0.4), vec2(cx + u * 0.42, cy), ink),
        Icon::Sound | Icon::Muted => {
            // a speaker: box and cone
            draw_rectangle(cx - u * 0.5, cy - u * 0.16, u * 0.22, u * 0.32, ink);
            draw_triangle(vec2(cx - u * 0.3, cy), vec2(cx + u * 0.05, cy - u * 0.42), vec2(cx + u * 0.05, cy + u * 0.42), ink);
            draw_rectangle(cx - u * 0.3, cy - u * 0.16, u * 0.1, u * 0.32, ink);
            if matches!(icon, Icon::Sound) {
                for k in [0.22, 0.4] {
                    draw_line(cx + u * 0.15 + u * k * 0.5, cy - u * k, cx + u * 0.15 + u * k, cy, line, ink);
                    draw_line(cx + u * 0.15 + u * k, cy, cx + u * 0.15 + u * k * 0.5, cy + u * k, line, ink);
                }
            } else {
                draw_line(cx + u * 0.18, cy - u * 0.22, cx + u * 0.55, cy + u * 0.22, line * 1.3, ink);
                draw_line(cx + u * 0.18, cy + u * 0.22, cx + u * 0.55, cy - u * 0.22, line * 1.3, ink);
            }
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
        // (Pause also comes from the START button of an Android controller)
        reset: down(&[KeyCode::Enter, KeyCode::KpEnter]),
        select: down(&[KeyCode::Tab]),
        ..Input::default()
    }
}

/// Draws the game's rectangles, in picture units, into the picture.
struct Window;

impl Canvas for Window {
    fn rect(&mut self, x: u8, y: u8, w: u8, h: u8, colour: Colour) {
        let rgb = colour;
        let c = Color::from_rgba((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8, 255);
        draw_rectangle(x as f32 * PIXEL_ASPECT, (y - RIVER_TOP) as f32, w as f32 * PIXEL_ASPECT, h as f32, c);
    }
}

/// Copies part of the picture (`from`, picture units) to the screen.
fn blit(picture: &Texture2D, from: touch::Rect, to: touch::Rect, colour: Color) {
    let params = DrawTextureParams {
        dest_size: Some(vec2(to.w, to.h)),
        source: Some(Rect::new(from.x, from.y, from.w, from.h)),
        ..Default::default()
    };
    draw_texture_ex(picture, to.x, to.y, colour, params);
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
    // the whole picture is drawn here first, then cut up onto the screen
    let picture = render_target(PICTURE_W as u32, PICTURE_H as u32);
    picture.texture.set_filter(FilterMode::Nearest);

    let mut straight_missiles = false;
    let mut paused = false;
    let mut behind = 0.0f32;
    let mut game_over_for = 0u32;
    let mut was_flying = false;

    loop {
        let (sw, sh) = (screen_width(), screen_height());
        let layout = Layout::new(sw, sh, controls.shown);
        let menu_now = menu(&game, paused, game_over_for);
        let (mut input, pressed) = controls.read(&layout, menu_now);
        let pad = gamepad::read();
        let key = |k| is_key_pressed(k);
        if get_last_key_pressed().is_some_and(|k| k != KeyCode::T) || pad.any {
            // a keyboard or controller is in use: the screen is all game
            controls.shown = false;
        }
        let start_pressed = key(KeyCode::Pause) || pad.start;
        if start_pressed && game.phase == Phase::GameOver && !paused {
            input.reset = true;
        } else if key(KeyCode::P) || start_pressed || pressed.contains(&Button::Pause) || pressed.contains(&Button::Resume) {
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
        for i in [keyboard(), pad.input] {
            input.up |= i.up;
            input.down |= i.down;
            input.left |= i.left;
            input.right |= i.right;
            input.fire |= i.fire;
            input.reset |= i.reset;
            input.select |= i.select;
        }
        input.straight_missiles = straight_missiles;
        input.one_joystick = true;

        // Run the game at 60 frames a second, whatever the
        // display's refresh rate.
        if paused {
            behind = 0.0;
            controls.taps_used();
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
        let flying = matches!(game.phase, Phase::Flying | Phase::Ready);
        if was_flying && matches!(game.phase, Phase::Crashed(_)) {
            gamepad::bump();
        }
        was_flying = flying;
        if let Ok(mut r) = sound.regs.lock() {
            *r = game.sound.out;
            if paused {
                *r = Default::default();
            }
        }

        // Draw the whole picture, then put the river and the status panel
        // where the layout says.
        set_camera(&Camera2D {
            zoom: vec2(2.0 / PICTURE_W, 2.0 / PICTURE_H),
            target: vec2(PICTURE_W / 2.0, PICTURE_H / 2.0),
            render_target: Some(picture.clone()),
            ..Default::default()
        });
        clear_background(BLACK);
        screen::draw(&shown, &mut Window);
        set_default_camera();

        clear_background(BLACK);
        let d = layout.deck;
        if d.w > 0.0 {
            let c = palette::STATUS;
            draw_rectangle(d.x, d.y, d.w, d.h, Color::from_rgba((c >> 16) as u8, (c >> 8) as u8, c as u8, 255));
        }
        let (from, to) = (layout.river_from, layout.river);
        if to.x > 0.0 && controls.shown {
            // Stretch the outermost column (always bank) to the screen's
            // edges, so the land runs on under the notch.
            let edge = |x: f32| touch::Rect { x, y: from.y, w: 1.0, h: from.h };
            blit(&picture.texture, edge(from.x), touch::Rect { x: 0.0, y: to.y, w: to.x, h: to.h }, WHITE);
            let right = to.x + to.w;
            blit(&picture.texture, edge(from.x + from.w - 1.0), touch::Rect { x: right, y: to.y, w: sw - right, h: to.h }, WHITE);
        }
        blit(&picture.texture, from, to, WHITE);
        if layout.hud.w > 0.0 {
            blit(&picture.texture, touch::HUD, layout.hud, Color::new(1.0, 1.0, 1.0, 0.9));
        }
        let menu_now = menu(&game, paused, game_over_for);
        let state = Shown { paused, muted: sound.muted.load(Ordering::Relaxed), two_players: game.two_players };
        controls.draw(&layout, menu_now, &state);
        if paused && !controls.shown {
            let p = layout.river;
            label(p.x + p.w / 2.0, p.y + p.h / 2.0, p.w * 0.5, p.h * 0.15, "PAUSED", WHITE);
        }
        next_frame().await
    }
}

/// Game controllers and vibration. Android sends controller buttons as
/// keys (see vendor/miniquad); on iPhone and iPad they come from Apple's
/// GameController framework, read here.
#[cfg(not(target_os = "ios"))]
mod gamepad {
    use delta_run::game::Input;

    #[derive(Default)]
    pub struct Pad {
        pub input: Input,
        pub start: bool,
        /// Anything pressed at all.
        pub any: bool,
    }

    pub fn read() -> Pad {
        Pad::default()
    }

    /// A short vibration, where the device can.
    pub fn bump() {}
}

#[cfg(target_os = "ios")]
mod gamepad {
    use delta_run::game::Input;
    use objc::runtime::{BOOL, NO, Object};
    use objc::{class, msg_send, sel, sel_impl};
    use std::sync::atomic::{AtomicBool, Ordering};

    #[link(name = "GameController", kind = "framework")]
    unsafe extern "C" {}
    #[link(name = "UIKit", kind = "framework")]
    unsafe extern "C" {}

    #[derive(Default)]
    pub struct Pad {
        pub input: Input,
        pub start: bool,
        pub any: bool,
    }

    static MENU_WAS_DOWN: AtomicBool = AtomicBool::new(false);

    fn pressed(element: *mut Object) -> bool {
        if element.is_null() {
            return false;
        }
        let p: BOOL = unsafe { msg_send![element, isPressed] };
        p != NO
    }

    /// The first connected controller with a full set of buttons.
    pub fn read() -> Pad {
        let mut pad = Pad::default();
        unsafe {
            let all: *mut Object = msg_send![class!(GCController), controllers];
            let n: usize = msg_send![all, count];
            for i in 0..n {
                let c: *mut Object = msg_send![all, objectAtIndex: i];
                let g: *mut Object = msg_send![c, extendedGamepad];
                if g.is_null() {
                    continue;
                }
                let dpad: *mut Object = msg_send![g, dpad];
                let stick: *mut Object = msg_send![g, leftThumbstick];
                let dir = |name: &str| -> bool {
                    let (d, s): (*mut Object, *mut Object) = match name {
                        "up" => (msg_send![dpad, up], msg_send![stick, up]),
                        "down" => (msg_send![dpad, down], msg_send![stick, down]),
                        "left" => (msg_send![dpad, left], msg_send![stick, left]),
                        _ => (msg_send![dpad, right], msg_send![stick, right]),
                    };
                    pressed(d) || pressed(s)
                };
                let i = &mut pad.input;
                i.up = dir("up");
                i.down = dir("down");
                i.left = dir("left");
                i.right = dir("right");
                let a: *mut Object = msg_send![g, buttonA];
                let b: *mut Object = msg_send![g, buttonB];
                let rt: *mut Object = msg_send![g, rightTrigger];
                let rs: *mut Object = msg_send![g, rightShoulder];
                i.fire = pressed(a) || pressed(b) || pressed(rt) || pressed(rs);
                let x: *mut Object = msg_send![g, buttonX];
                i.select = pressed(x);
                let menu: *mut Object = msg_send![g, buttonMenu];
                let down = pressed(menu);
                pad.start = down && !MENU_WAS_DOWN.swap(down, Ordering::Relaxed);
                if !down {
                    MENU_WAS_DOWN.store(false, Ordering::Relaxed);
                }
                pad.any = i.up || i.down || i.left || i.right || i.fire || i.select || down;
                break;
            }
        }
        pad
    }

    /// A firm tap from the phone's vibration motor.
    pub fn bump() {
        unsafe {
            let g: *mut Object = msg_send![class!(UIImpactFeedbackGenerator), alloc];
            // UIImpactFeedbackStyleHeavy
            let g: *mut Object = msg_send![g, initWithStyle: 2isize];
            let _: () = msg_send![g, impactOccurred];
            let _: () = msg_send![g, release];
        }
    }
}
