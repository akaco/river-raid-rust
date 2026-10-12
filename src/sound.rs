//! Sound effects. Each frame the game sets two voices (see `synth.rs`):
//! voice 0 plays the engine, the low-fuel alarm, refuelling and crashes;
//! voice 1 plays the missile and explosions.

use crate::synth::{Voice, Wave};

/// A sound on voice 0 that overrides the engine for a while.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Effect {
    Crash,
    OutOfFuel,
    TankFull,
    Refuelling,
}

#[derive(Clone, Debug, Default)]
pub struct Sound {
    pub effect: Option<Effect>,
    /// Frames left of the effect.
    pub timer: u8,
    /// Frames left of the missile's whistle.
    pub shot: u8,
    /// Frames left of the explosion noise.
    pub explosion: u8,
    /// Counts frames for the low-fuel alarm.
    pub alarm: u8,
    /// The voices, as last set.
    pub out: [Voice; 2],
}

const SHOT_FRAMES: f32 = 15.0;
const EXPLOSION_FRAMES: f32 = 23.0;

impl Sound {
    pub fn start(&mut self, effect: Effect, frames: u8) {
        self.effect = Some(effect);
        self.timer = frames;
    }

    /// Sets the voices for this frame.
    pub fn update(&mut self, flying: bool, speed: u8, fuel: u16, stick_up: bool, stick_down: bool) {
        self.out[0] = if let Some(effect) = self.effect {
            let t = self.timer as f32;
            let voice = match effect {
                Effect::Crash => Voice { wave: Wave::Noise, pitch: 900.0, volume: t / 31.0 },
                Effect::OutOfFuel => Voice { wave: Wave::Square, pitch: 120.0 + 8.0 * t, volume: t / 40.0 },
                Effect::TankFull => Voice { wave: Wave::Square, pitch: 1320.0, volume: 0.25 },
                Effect::Refuelling => Voice { wave: Wave::Square, pitch: 440.0 + 60.0 * (8.0 - t), volume: 0.2 },
            };
            self.timer = self.timer.saturating_sub(1);
            if self.timer == 0 {
                self.effect = None;
            }
            voice
        } else if !flying {
            Voice::default()
        } else if fuel < 0x4000 && self.low_fuel_beep() {
            Voice { wave: Wave::Square, pitch: 880.0, volume: 0.25 }
        } else {
            // the engine hums higher as the plane goes faster
            let volume = if stick_up {
                0.22
            } else if stick_down {
                0.12
            } else {
                0.16
            };
            Voice { wave: Wave::Square, pitch: 45.0 + 0.4 * speed as f32, volume }
        };

        self.out[1] = if self.explosion != 0 {
            self.explosion -= 1;
            self.shot = 0;
            Voice { wave: Wave::Noise, pitch: 2500.0, volume: 0.6 * self.explosion as f32 / EXPLOSION_FRAMES }
        } else if self.shot != 0 {
            // a falling whistle
            self.shot -= 1;
            Voice { wave: Wave::Square, pitch: 700.0 + 60.0 * self.shot as f32, volume: 0.15 * self.shot as f32 / SHOT_FRAMES }
        } else {
            Voice::default()
        };
    }

    /// Below a quarter tank the alarm beeps, half a second on and off.
    fn low_fuel_beep(&mut self) -> bool {
        self.alarm = self.alarm.wrapping_add(1);
        self.alarm % 32 < 16
    }
}
