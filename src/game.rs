//! The game: plane, fuel, score, lives, players, and what happens each frame.

use crate::collide::{self, Collisions};
use crate::objects::Kind;
use crate::palette::{self, Colour};
use crate::river::{BANDS, River, Seed};
use crate::sound::{Effect, Sound};

/// Controls for one frame: a joystick, fire, and the game switches.
#[derive(Clone, Copy, Default, Debug)]
pub struct Input {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    pub fire: bool,
    /// Game switches: RESET starts a game, SELECT picks 1 or 2 players.
    pub reset: bool,
    pub select: bool,
    /// Missiles fly straight instead of following the plane.
    pub straight_missiles: bool,
    /// In a two-player game, let these controls fly for player 2 as well
    /// (otherwise player 2 needs controls of their own).
    pub one_joystick: bool,
}

/// What the plane is doing.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Phase {
    Flying,
    /// The plane waits for the player to move the stick or fire.
    Ready,
    /// The river rolls in after a crash; frames left until `Ready`.
    Starting(u8),
    /// Exploding; frames left until the river is rebuilt.
    Crashed(u8),
    GameOver,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PlaneSprite {
    Hidden,
    Straight,
    BankedLeft,
    BankedRight,
    Crashed,
}

#[derive(Clone, Debug)]
pub struct Plane {
    pub x: u8,
    /// Fraction of a pixel, for smooth sideways acceleration.
    pub sub_x: u8,
    /// Sideways acceleration; grows while the stick is held left or right.
    pub steer: u8,
    /// Scrolling speed in 256ths of a line per half frame: $40 slow, $80
    /// cruising, $FE fast.
    pub speed: u8,
    pub sprite: PlaneSprite,
}

pub const MISSILE_READY: u8 = 0xB4;
const PLANE_LEFTMOST: u8 = 1;
const PLANE_RIGHTMOST: u8 = 151;

#[derive(Clone, Debug)]
pub struct Missile {
    pub x: u8,
    /// Height above the plane, in lines; `MISSILE_READY` when not flying.
    pub y: u8,
}

impl Missile {
    pub fn flying(&self) -> bool {
        self.y < 0xA2
    }
}

/// Spare planes, as the lives digit shows them.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Lives {
    Spare(u8),
    /// A new game: blank until the first plane takes off, then 3.
    Starting,
    /// No spare planes (shown blank).
    Last,
}

impl Lives {
    /// The digit shown, if any.
    pub fn digit(self) -> Option<u8> {
        match self {
            Lives::Spare(n) => Some(n),
            _ => None,
        }
    }

    fn gain(&mut self) {
        *self = match *self {
            Lives::Spare(9) => Lives::Spare(9),
            Lives::Spare(n) => Lives::Spare(n + 1),
            _ => Lives::Spare(1),
        };
    }

    /// One plane is used. Returns false if there was none.
    fn take(&mut self) -> bool {
        *self = match *self {
            Lives::Spare(0) => return false,
            Lives::Spare(1) => Lives::Last,
            Lives::Spare(n) => Lives::Spare(n - 1),
            Lives::Starting => Lives::Spare(3),
            Lives::Last => Lives::Last,
        };
        true
    }
}

/// The six score digits as shown: blank, 0-9, or 10 for "!" (the score
/// went past 999,999).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Score {
    pub digits: [Option<u8>; 6],
}

impl Score {
    /// All blank but the last digit (a new game shows 0, the title screen
    /// shows the game number).
    pub fn showing(last: u8) -> Score {
        Score { digits: [None, None, None, None, None, Some(last)] }
    }

    /// Adds `amount` to digit `place` (3 = hundreds, 4 = tens), carrying
    /// as needed. Returns (extra plane earned, score overflowed).
    fn add(&mut self, place: usize, amount: u8) -> (bool, bool) {
        if place == 3 && self.digits[4].is_none() {
            self.digits[4] = Some(0);
        }
        let mut extra_plane = false;
        let mut i = place;
        let mut add = amount;
        loop {
            if i == 1 {
                // every 10,000 points
                extra_plane = true;
            }
            let sum = self.digits[i].unwrap_or(0) + add;
            if sum < 10 {
                self.digits[i] = Some(sum);
                return (extra_plane, false);
            }
            self.digits[i] = Some(sum - 10);
            add = 1;
            if i == 0 {
                self.digits = [Some(10); 6];
                return (extra_plane, true);
            }
            i -= 1;
        }
    }
}

/// Which digit a kind's points go to, and how many.
fn points(kind: Kind) -> (usize, u8) {
    match kind {
        Kind::Jet => (3, 1),
        Kind::Bridge => (3, 5),
        Kind::Helicopter(_) => (4, 6),
        Kind::Ship => (4, 3),
        Kind::FuelDepot => (4, 8),
        _ => (4, 0),
    }
}

/// The other player's progress in a two-player game, swapped in on their turn.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Waiting {
    pub section: u8,
    pub checkpoint: Seed,
    pub lives: Lives,
    pub score: Score,
}

#[derive(Clone, Debug)]
pub struct Game {
    pub two_players: bool,
    /// Whose turn it is (0 or 1).
    pub player: u8,
    pub phase: Phase,
    pub river: River,
    pub plane: Plane,
    pub missile: Missile,
    /// 0-$FFFF; the gauge shows the top byte. Burns $20 a frame.
    pub fuel: u16,
    pub lives: Lives,
    pub score: Score,
    pub waiting: Waiting,
    pub sound: Sound,

    /// Counts down every frame.
    pub frame_counter: u8,
    /// 8-bit random numbers (shift register, taps at bits 7 and 4).
    pub random: u8,
    /// The stick now (low 4 bits) and last frame (high 4 bits); bits are
    /// right, left, down, up from high to low, and 1 means released.
    pub stick: u8,
    pub select_timer: u8,
    /// Frames of red flashing after a bridge blows up.
    pub flash: u8,
    /// The background flashes red this frame.
    pub flashing: bool,
    /// What touched what in this frame's picture.
    pub collisions: Collisions,
    started: bool,
    input: Input,
}

impl Game {
    /// The game just started up: the title screen.
    pub fn new() -> Game {
        let mut g = Game {
            two_players: false,
            player: 0,
            phase: Phase::GameOver,
            river: River::new(),
            plane: Plane { x: 0, sub_x: 0, steer: 0, speed: 0, sprite: PlaneSprite::Hidden },
            missile: Missile { x: 0, y: 0 },
            fuel: 0,
            lives: Lives::Starting,
            score: Score::showing(0),
            waiting: Waiting { section: 1, checkpoint: Seed::START, lives: Lives::Starting, score: Score::showing(0) },
            sound: Sound::default(),
            frame_counter: 0,
            random: 0,
            stick: 0,
            select_timer: 0,
            flash: 0,
            flashing: false,
            collisions: Collisions::default(),
            started: false,
            input: Input::default(),
        };
        g.new_game();
        // the title screen: shows game 1, with no planes to fly
        g.random = 1;
        g.lives = Lives::Spare(0);
        g.score = Score::showing(1);
        g
    }

    /// Plays one frame (1/60 s).
    pub fn frame(&mut self, input: Input) {
        self.start_frame(input);
        self.finish_frame();
    }

    /// The first part of a frame: game switches and scrolling. After
    /// this the game is in the state the frame's picture shows.
    pub fn start_frame(&mut self, input: Input) {
        self.input = input;
        if self.started {
            self.game_switches();
        }
        self.started = true;
        self.flashing = self.flash & 1 != 0;
        self.flash = self.flash.saturating_sub(1);
    }

    /// The rest of the frame: what the picture shows touching, then
    /// everything moves.
    pub fn finish_frame(&mut self) {
        self.collisions = collide::check(self);
        if self.phase == Phase::GameOver {
            self.lives = Lives::Starting;
        }
        self.timers();
        self.update_objects();
        self.read_stick();
        self.update_plane();
        let flying = self.phase == Phase::Flying;
        let (up, down) = (self.stick & 1 == 0, self.stick & 2 == 0);
        self.sound.update(flying, self.plane.speed, self.fuel, up, down);
    }

    /// The colour of the plane and its missile.
    pub fn plane_colour(&self) -> Colour {
        if self.player == 0 { palette::PLAYER_1 } else { palette::PLAYER_2 }
    }

    /// The colour of the current player's score and spare planes.
    pub fn score_colour(&self) -> Colour {
        if self.player == 0 { palette::PLAYER_1_TEXT } else { palette::PLAYER_2 }
    }

    /// RESET and SELECT, then scrolling.
    fn game_switches(&mut self) {
        if self.input.reset {
            self.player = self.two_players as u8;
            self.sound = Sound { out: self.sound.out, ..Sound::default() };
            self.new_game();
            return;
        }
        if self.input.select {
            // switches game on the first frame, then every 31 frames held
            self.select_timer = self.select_timer.wrapping_sub(1);
            if self.select_timer & 0x80 != 0 {
                self.two_players = !self.two_players;
                let g = self.two_players as u8;
                self.player = g;
                self.score = Score::showing(g + 1);
                self.waiting.score = Score::showing(g + 1);
                self.game_over();
                self.select_timer = 0x1E;
            }
        } else {
            self.select_timer = 0x82;
        }
        match self.phase {
            Phase::Crashed(_) | Phase::GameOver => {}
            Phase::Ready => self.plane.sprite = PlaneSprite::Straight,
            Phase::Flying | Phase::Starting(_) => self.river.scroll(self.plane.speed),
        }
    }

    fn new_game(&mut self) {
        self.score = Score::showing(0);
        self.waiting.score = Score::showing(0);
        self.river.section = 1;
        self.river.checkpoint = Seed::START;
        self.lives = Lives::Starting;
        self.waiting = Waiting { section: 1, checkpoint: Seed::START, lives: Lives::Starting, score: self.waiting.score };
        self.start_positions();
        // the first plane is put in place next frame, like after a crash
        self.phase = Phase::Crashed(0);
    }

    /// Plane, fuel and screen as at the start of a life.
    fn start_positions(&mut self) {
        self.river.clear_screen();
        self.flash = 0;
        self.plane = Plane { x: 0x4C, sub_x: 0, steer: 0, speed: 0xFE, sprite: PlaneSprite::Hidden };
        self.missile.y = MISSILE_READY;
        self.fuel = 0xFFFF;
    }

    fn game_over(&mut self) {
        self.frame_counter = 0xFF;
        self.phase = Phase::GameOver;
        self.missile.y = MISSILE_READY;
    }

    /// Two-player games: the other player's turn.
    fn swap_players(&mut self) {
        if !self.two_players {
            return;
        }
        self.player ^= 1;
        let w = &mut self.waiting;
        std::mem::swap(&mut self.river.section, &mut w.section);
        std::mem::swap(&mut self.river.checkpoint, &mut w.checkpoint);
        std::mem::swap(&mut self.lives, &mut w.lives);
        std::mem::swap(&mut self.score, &mut w.score);
    }

    /// Frame counter, random numbers, and setting objects off.
    fn timers(&mut self) {
        self.frame_counter = self.frame_counter.wrapping_sub(1);
        if self.frame_counter == 0 && self.phase == Phase::GameOver {
            // the title screen alternates between both players' scores
            self.swap_players();
        }

        let r = self.random;
        self.random = (r << 1) | (((r << 3) ^ r) >> 7);
        if self.frame_counter & 0x0F == 0 {
            // every 16 frames, one of the lower five bands' objects sets off
            let mut band = (self.random & 7) as usize;
            if band >= 5 {
                band -= 5;
            }
            self.river.bands[band].object.moving = true;
        }
    }

    fn update_objects(&mut self) {
        let can_move = self.phase == Phase::Flying && self.river.section >= 2;
        for i in (0..BANDS).rev() {
            let o = &mut self.river.bands[i].object;
            o.kind = o.kind.animate(self.frame_counter);
            if can_move {
                o.update(self.frame_counter, self.collisions.object_on_bank[i]);
            }
        }
    }

    /// Reads the stick; moving it (or firing) launches a waiting plane.
    fn read_stick(&mut self) {
        let i = self.input;
        let ours = self.player == 0 || i.one_joystick;
        let held = |b: bool| ours && b;
        let released = (!held(i.right) as u8) << 3 | (!held(i.left) as u8) << 2 | (!held(i.down) as u8) << 1 | !held(i.up) as u8;
        self.stick = (self.stick << 4) | released;
        if (released != 0x0F || held(i.fire)) && self.phase == Phase::Ready {
            self.plane.speed = 0x40;
            self.phase = Phase::Flying;
        }
    }

    fn update_plane(&mut self) {
        if self.phase != Phase::Flying {
            self.count_down();
            return;
        }
        let c = self.collisions;
        if let Some(band) = c.plane_touched {
            let kind = self.river.bands[band].object.kind;
            if kind.is_solid() {
                if !kind.refuels() {
                    if kind == Kind::Bridge {
                        self.river.bridges_hit += 1;
                    }
                    self.crash(Effect::Crash, 0x1F);
                    self.destroy(band, 2);
                    return;
                }
                // refuel: 2 units a frame
                let effect = if self.fuel >= 0xFE00 {
                    self.fuel = 0xFFFF;
                    Effect::TankFull
                } else {
                    self.fuel += 0x200;
                    Effect::Refuelling
                };
                if self.sound.effect != Some(effect) {
                    self.sound.start(effect, 8);
                }
            }
        }
        if c.plane_on_bank {
            self.crash(Effect::Crash, 0x1F);
            return;
        }
        if self.fuel < 0x20 {
            self.crash(Effect::OutOfFuel, 0x23);
            return;
        }
        self.fuel -= 0x20;
        self.steer();
        self.change_speed();
        if let Some(band) = c.missile_hit.filter(|&b| self.river.bands[b].object.kind.is_solid()) {
            self.destroy(band, 1);
            return;
        }
        self.move_missile(c.missile_on_bank);
    }

    /// Crashed, starting and game-over timers.
    fn count_down(&mut self) {
        match self.phase {
            Phase::Crashed(0) => {
                self.phase = Phase::Starting(0x4F);
                self.respawn();
            }
            Phase::Crashed(n) => self.phase = Phase::Crashed(n - 1),
            Phase::Starting(1) => {
                // the new plane is taken from the spares
                if !self.lives.take() {
                    self.game_over();
                    return;
                }
                self.phase = Phase::Ready;
            }
            Phase::Starting(n) => self.phase = Phase::Starting(n - 1),
            _ => {}
        }
    }

    /// After a crash: the other player's turn (if they have planes left),
    /// and the river rebuilt from the last bridge.
    fn respawn(&mut self) {
        self.plane.sprite = PlaneSprite::Hidden;
        if self.waiting.lives != Lives::Last {
            self.swap_players();
        }
        if self.lives == Lives::Last {
            self.game_over();
            return;
        }
        self.start_positions();
        self.river.restart_at_checkpoint();
    }

    fn crash(&mut self, effect: Effect, frames: u8) {
        self.sound.start(effect, frames);
        let bands = &self.river.bands;
        if bands[0].dark_banks != bands[BANDS - 1].dark_banks {
            // A section ends on screen. The checkpoint moved on when the
            // bridge band was made; if the bridge was not reached, undo that.
            let bridge_on_top = bands[BANDS - 1].bridge;
            if self.river.bridges_hit != 0 {
                if bridge_on_top {
                    self.river.next_section();
                }
            } else if !bridge_on_top {
                self.river.back_to_previous_section();
            }
        }
        self.plane.sprite = PlaneSprite::Crashed;
        self.phase = Phase::Crashed(0x5F);
        self.missile.y = MISSILE_READY;
    }

    /// The object in `band` is hit; it becomes explosion frame `frame`.
    fn destroy(&mut self, band: usize, frame: u8) {
        let o = &mut self.river.bands[band].object;
        let kind = o.kind;
        // the explosion is centred where the object was
        let width = o.width();
        o.kind = Kind::Explosion(frame);
        o.x = ((o.x as i16 + (width as i16 - o.width() as i16) / 2).rem_euclid(160)) as u8;
        self.sound.explosion = 0x17;
        if kind == Kind::Bridge {
            self.flash = 0x17;
            self.river.bridges_hit += 1;
        }
        let (place, amount) = points(kind);
        let (extra_plane, overflow) = self.score.add(place, amount);
        if extra_plane {
            self.lives.gain();
        }
        if overflow {
            self.lives = Lives::Last;
            self.crash(Effect::Crash, 0x1F);
            return;
        }
        self.missile.y = MISSILE_READY;
    }

    /// Left and right, with acceleration and sub-pixel movement.
    fn steer(&mut self) {
        let s = self.stick;
        let p = &mut self.plane;
        // stick bits are 1 when released: bit 3 right, bit 2 left
        let (left, right) = (s & 0x04 == 0, s & 0x08 == 0);
        if left == right {
            p.steer = 0;
            p.sub_x = 0;
            p.sprite = PlaneSprite::Straight;
            return;
        }
        p.sprite = if left { PlaneSprite::BankedLeft } else { PlaneSprite::BankedRight };
        // a fresh push moves at once, then the plane speeds up sideways
        if s & 0xC0 == 0xC0 {
            p.sub_x = 0xFF;
        }
        p.steer = p.steer.saturating_add(8);
        let (sub, carry) = p.sub_x.overflowing_add(p.steer);
        p.sub_x = sub;
        let step = carry as u8 + (p.steer == 0xFF) as u8;
        p.x = if left { p.x.saturating_sub(step).max(PLANE_LEFTMOST) } else { (p.x + step).min(PLANE_RIGHTMOST) };
    }

    /// Up speeds up, down slows down, centred drifts back to cruising.
    fn change_speed(&mut self) {
        let up = self.stick & 1 == 0;
        let down = self.stick & 2 == 0;
        let v = self.plane.speed;
        let faster = v.checked_add(2);
        let slower = if v > 0x40 { Some(v - 2) } else { None };
        let new = if up {
            faster
        } else if down {
            slower
        } else if v < 0x80 {
            faster
        } else if v == 0x80 {
            None
        } else {
            slower
        };
        if let Some(v) = new {
            self.plane.speed = v;
        }
    }

    fn move_missile(&mut self, hit_bank: bool) {
        let m = &mut self.missile;
        if hit_bank {
            m.y = MISSILE_READY;
            return;
        }
        let y;
        if m.flying() {
            y = m.y + 6;
            if self.input.straight_missiles {
                m.y = y;
                return;
            }
        } else {
            let fire = self.input.fire && (self.player == 0 || self.input.one_joystick);
            if !fire {
                m.y = MISSILE_READY;
                return;
            }
            self.sound.shot = 0x0F;
            y = 0x0D;
        }
        // launched from the plane, and (unless straight missiles are on) steered with it
        m.x = self.plane.x + 3;
        m.y = y;
    }
}

impl Default for Game {
    fn default() -> Self {
        Game::new()
    }
}
