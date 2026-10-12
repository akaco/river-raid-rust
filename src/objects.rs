//! Things in the river: enemies, fuel depots, houses, bridges, explosions.

use crate::sprites::{self, Sprite};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    None,
    /// Explosion frames 1-3, then nothing.
    Explosion(u8),
    Jet,
    /// Helicopter with its rotor in one of two positions.
    Helicopter(u8),
    Ship,
    Bridge,
    House,
    FuelDepot,
}

impl Kind {
    /// Something solid: it can be shot or crashed into.
    pub fn is_solid(self) -> bool {
        !matches!(self, Kind::None | Kind::Explosion(_))
    }

    /// Flying over it refuels the plane.
    pub fn refuels(self) -> bool {
        self == Kind::FuelDepot
    }

    /// Points for shooting it.
    pub fn points(self) -> u32 {
        match self {
            Kind::Jet => 100,
            Kind::Helicopter(_) => 60,
            Kind::Ship => 30,
            Kind::Bridge => 500,
            Kind::FuelDepot => 80,
            _ => 0,
        }
    }

    pub fn sprite(self) -> Option<&'static Sprite> {
        Some(match self {
            Kind::None | Kind::Explosion(0) => return None,
            Kind::Explosion(1) => sprites::get("explosion_1"),
            Kind::Explosion(2) => sprites::get("explosion_2"),
            Kind::Explosion(_) => sprites::get("explosion_3"),
            Kind::Jet => sprites::get("jet"),
            Kind::Helicopter(0) => sprites::get("helicopter_1"),
            Kind::Helicopter(_) => sprites::get("helicopter_2"),
            Kind::Ship => sprites::get("ship"),
            Kind::Bridge => sprites::get("bridge"),
            Kind::House => sprites::get("house"),
            Kind::FuelDepot => sprites::get("fuel"),
        })
    }

    /// Animation step: helicopter rotors turn every 2 frames, explosions
    /// move on every 16 frames. `frame` is the game's frame counter.
    pub fn animate(self, frame: u8) -> Kind {
        match self {
            Kind::Helicopter(r) if frame & 1 == 0 => Kind::Helicopter(r ^ 1),
            Kind::Explosion(n) if frame & 0x0F == 0 => {
                if n >= 3 {
                    Kind::None
                } else {
                    Kind::Explosion(n + 1)
                }
            }
            k => k,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Object {
    pub kind: Kind,
    /// Horizontal position, 0-159 (the screen wraps round).
    pub x: u8,
    pub facing_left: bool,
    /// Ships and helicopters start still and set off at random moments.
    pub moving: bool,
    /// Already turned round at the bank it is touching.
    pub turned: bool,
}

impl Object {
    pub const NONE: Object = Object { kind: Kind::None, x: 0, facing_left: false, moving: false, turned: false };

    /// A new object whose picture starts at column `x`.
    pub fn new(kind: Kind, x: u8, facing_left: bool) -> Object {
        Object { kind, x, facing_left, moving: false, turned: false }
    }

    /// Width of its picture in pixels.
    pub fn width(&self) -> u8 {
        self.kind.sprite().map_or(0, |s| s.width())
    }

    /// One pixel along, wrapping round the screen.
    pub fn step(&mut self) {
        self.x = if self.facing_left {
            if self.x == 0 { 159 } else { self.x - 1 }
        } else if self.x >= 159 {
            0
        } else {
            self.x + 1
        };
    }

    /// Moves it for one frame. Jets always fly; ships and helicopters move
    /// on even frames once they have set off, turning round at the banks.
    pub fn update(&mut self, frame: u8, touching_bank: bool) {
        match self.kind {
            Kind::Jet => self.step(),
            Kind::Helicopter(_) | Kind::Ship if frame & 1 == 0 && self.moving => {
                if !touching_bank {
                    self.turned = false;
                } else if !self.turned {
                    self.facing_left = !self.facing_left;
                    self.turned = true;
                }
                self.step();
            }
            _ => {}
        }
    }

    /// The screen pixels drawn by sprite line `art`, with their letters.
    pub fn pixels<'a>(&self, art: &'a str) -> impl Iterator<Item = (u8, u8)> + 'a {
        let (x0, left, w) = (self.x as usize, self.facing_left, art.len());
        art.bytes().enumerate().filter(|(_, b)| *b != b'.').map(move |(i, b)| {
            let col = if left { w - 1 - i } else { i };
            (((x0 + col) % 160) as u8, b)
        })
    }
}
