//! The river: what is on screen, and how new stretches of it are made.
//!
//! The screen shows six horizontal bands, each 32 lines tall. Each band has
//! its own bank shape and at most one object. As the plane flies, the bands
//! scroll down; when the top band has fully appeared, a new band is made
//! above it and the bottom one is dropped.
//!
//! New bands come from a random number generator that always starts from
//! the same seed, so the river is the same in every game. A section is 16
//! river widths (32 bands) and ends with a bridge. The generator's state at
//! each bridge is saved, so after a crash the river is rebuilt from the
//! last bridge.

use crate::objects::{Kind, Object};
use crate::sprites;

/// Bands on screen. Index 0 is the bottom band, 5 the top one.
pub const BANDS: usize = 6;
/// River widths in a section (each lasts two bands).
pub const WIDTHS_PER_SECTION: u8 = 16;
/// The narrow river at each bridge.
const BRIDGE_WIDTH: u8 = 12;

/// The river's random number generator (xorshift).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Seed(pub u32);

impl Seed {
    /// Every game starts from this seed.
    pub const START: Seed = Seed(0x2F6B_1A93);

    pub fn step(&mut self) {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.0 = x;
    }

    /// A number from 0 to n - 1, taken from bits `shift` and up.
    fn below(self, shift: u32, n: u32) -> u32 {
        (self.0 >> shift) % n
    }
}

/// The shape of the land in one band.
///
/// The screen is 40 blocks of 4 pixels across, and the land is mirrored
/// around the centre. Without an island, the banks reach `4 + width` blocks
/// in from each edge. With an island, the banks are just the 4 edge blocks
/// and the island reaches `width` blocks out from the centre. Within a
/// band, the width slides one block per row from `from` (bottom) to `to`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Land {
    pub from: u8,
    pub to: u8,
    pub island: bool,
}

impl Land {
    /// Width at `row` (0 = bottom row, 15 = top row; each row is 2 lines).
    pub fn width_at(&self, row: u8) -> u8 {
        let (from, to) = (self.from, self.to);
        if to > from {
            to.min(from + 1 + row)
        } else if to < from {
            from.min(to + 1 + (15 - row))
        } else {
            to
        }
    }

    /// Blocks of land from each edge, and from the centre, at `row`.
    pub fn row(&self, row: u8) -> (u8, u8) {
        let w = self.width_at(row);
        if self.island { (4, w) } else { (4 + w, 0) }
    }

    /// Is pixel column `x` (0-159) land at `row`?
    pub fn is_land(&self, row: u8, x: u8) -> bool {
        let block = if x < 80 { x / 4 } else { (159 - x) / 4 };
        let (edge, centre) = self.row(row);
        block < edge || block >= 20u8.saturating_sub(centre)
    }

    /// The water that is open along the whole band: (left, right) pixel
    /// ranges, the right one empty when there is no island.
    fn open_water(&self) -> [(u8, u8); 2] {
        let w = self.from.max(self.to);
        if self.island {
            [(16, 80 - 4 * w), (80 + 4 * w, 144)]
        } else {
            let bank = 4 * (4 + w);
            [(bank, 160 - bank), (0, 0)]
        }
    }

    /// Bank that is land along the whole band, in pixels from each edge.
    fn bank(&self) -> u8 {
        if self.island { 16 } else { 4 * (4 + self.from.min(self.to)) }
    }
}

/// One 32-line band of the river.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Band {
    pub land: Land,
    /// The band with the road and bridge at the end of a section.
    pub bridge: bool,
    /// Banks alternate between two greens, section by section.
    pub dark_banks: bool,
    pub object: Object,
}

impl Band {
    fn new(dark_banks: bool) -> Band {
        Band { land: Land { from: BRIDGE_WIDTH, to: BRIDGE_WIDTH, island: false }, bridge: false, dark_banks, object: Object::NONE }
    }
}

/// Islands open in steps so the land never jumps: the banks first open
/// fully (`Opening`), then the island grows from the centre (`Present`),
/// then it shrinks to nothing (`Closing`) before the banks come back.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Island {
    None,
    Opening,
    Present,
    Closing,
}

impl Island {
    /// True while the land is drawn as an island in the middle.
    pub fn shown(self) -> bool {
        matches!(self, Island::Present | Island::Closing)
    }
}

#[derive(Clone, Debug)]
pub struct River {
    pub bands: [Band; BANDS],
    /// How many lines of the top band have scrolled into view (0-31).
    pub scroll_line: u8,
    /// Fraction of a line scrolled, in 256ths.
    pub scroll_fraction: u8,

    pub seed: Seed,
    /// Section number (1, 2, ...).
    pub section: u8,
    /// Seed saved at the last bridge passed, and at the one before.
    pub checkpoint: Seed,
    pub previous_checkpoint: Seed,

    /// River widths still to come in this section.
    pub widths_left: u8,
    /// Each width lasts two bands; this counts them.
    pub width_bands_left: u8,
    pub width: u8,
    pub previous_width: u8,
    pub island: Island,
    /// Set when the section's bridge has been destroyed (it counts bridges
    /// hit since the bridge band was made).
    pub bridges_hit: u8,
}

impl River {
    pub fn new() -> River {
        let mut r = River {
            bands: [Band::new(true); BANDS],
            scroll_line: 0,
            scroll_fraction: 0,
            seed: Seed::START,
            section: 1,
            checkpoint: Seed::START,
            previous_checkpoint: Seed::START,
            widths_left: 0,
            width_bands_left: 0,
            width: 0,
            previous_width: 0,
            island: Island::None,
            bridges_hit: 0,
        };
        r.clear_screen();
        r
    }

    /// An empty screen: a narrow straight river, nothing on it. The
    /// generator is left where it is.
    pub fn clear_screen(&mut self) {
        let dark = self.section % 2 == 1;
        self.bands = [Band::new(dark); BANDS];
        self.scroll_line = 0;
        self.scroll_fraction = 0;
        // one extra width: the section starts with the bands on screen
        self.widths_left = WIDTHS_PER_SECTION + 1;
        self.width_bands_left = 1;
        self.width = BRIDGE_WIDTH;
        self.previous_width = BRIDGE_WIDTH;
        self.island = Island::None;
        self.bridges_hit = 0;
    }

    /// Rebuilds the screen after a crash (or at the start of a game): a
    /// narrow river with a bridge band at the top, and the generator back at
    /// the last checkpoint.
    pub fn restart_at_checkpoint(&mut self) {
        self.clear_screen();
        self.bands[BANDS - 1].bridge = true;
        self.seed = self.checkpoint;
    }

    /// Scrolls the river by `speed` (in 256ths of a line, applied twice per
    /// frame; 254 and above is exactly two lines a frame).
    pub fn scroll(&mut self, speed: u8) {
        for _ in 0..2 {
            if speed < 0xFE {
                let (f, carry) = self.scroll_fraction.overflowing_add(speed);
                self.scroll_fraction = f;
                if !carry {
                    continue;
                }
            }
            self.scroll_line += 1;
            if self.scroll_line >= 32 {
                self.add_band();
            }
        }
    }

    /// A section is finished: save the generator as the new checkpoint.
    pub fn next_section(&mut self) {
        self.previous_checkpoint = self.checkpoint;
        self.checkpoint = self.seed;
        self.section = self.section.saturating_add(1);
    }

    /// The plane crashed before reaching the bridge on screen: the section
    /// it was flying in has not been passed after all.
    pub fn back_to_previous_section(&mut self) {
        self.section = self.section.saturating_sub(1).max(1);
        self.checkpoint = self.previous_checkpoint;
    }

    /// A new band appears at the top.
    fn add_band(&mut self) {
        self.scroll_line = 0;
        let mut band = Band::new(self.bands[BANDS - 1].dark_banks);
        self.bands.copy_within(1.., 0);
        self.previous_width = self.width;
        self.seed.step();

        self.width_bands_left -= 1;
        if self.width_bands_left != 0 {
            // second band of the same width
            if self.widths_left == 1 {
                // the last band of the section holds the bridge; the bank
                // colour changes from here
                self.bridges_hit = 0;
                band.bridge = true;
                band.dark_banks = self.section.is_multiple_of(2);
            }
        } else {
            self.widths_left -= 1;
            if self.widths_left == 0 {
                self.next_section();
                self.widths_left = WIDTHS_PER_SECTION;
            }
            self.width = self.choose_width();
            self.width_bands_left = 2;
        }
        band.land = Land { from: self.previous_width, to: self.width, island: self.island.shown() };
        band.object = self.choose_object(&band);
        self.bands[BANDS - 1] = band;
    }

    /// The next river width (land, so larger is narrower water), and
    /// whether an island starts or ends.
    fn choose_width(&mut self) -> u8 {
        match self.widths_left {
            // narrow at the bridge
            1 => {
                self.island = Island::None;
                return BRIDGE_WIDTH;
            }
            // just before the bridge: close any island
            2 => {
                if self.island == Island::Present {
                    self.island = Island::Closing;
                    return 0;
                }
                self.island = Island::None;
                return self.random_width();
            }
            // the first stretch after a bridge is calm
            n if n >= WIDTHS_PER_SECTION - 1 => return 8,
            _ => {}
        }
        match self.island {
            Island::Opening => {
                self.island = Island::Present;
                return self.random_width();
            }
            Island::Closing => {
                self.island = Island::None;
                return self.random_width();
            }
            _ => {}
        }
        // from section 2, one width in five opens or closes an island
        if self.section >= 2 && self.seed.below(0, 5) == 0 {
            self.island = if self.island == Island::Present { Island::Closing } else { Island::Opening };
            return 0;
        }
        self.random_width()
    }

    /// The river narrows as the sections go by.
    fn random_width(&self) -> u8 {
        let limit = if self.island.shown() { 12 } else { 13 };
        let narrowest = (5 + self.section).min(limit);
        2 + self.seed.below(8, (narrowest - 1) as u32) as u8
    }

    /// What the new band holds, and where.
    fn choose_object(&self, band: &Band) -> Object {
        let seed = self.seed;
        if band.bridge {
            let x = 80 - sprites::get("bridge").width() / 2;
            return Object::new(Kind::Bridge, x, false);
        }

        // Enemies get more likely in later sections; otherwise a fuel
        // tank, or a house on the bank.
        let enemy_chance = (45 + 3 * self.section as u32).min(80);
        let roll = seed.below(12, 100);
        if roll >= enemy_chance {
            let fuel = roll < enemy_chance + (100 - enemy_chance) * 2 / 3;
            return if fuel { self.place(Kind::FuelDepot, band) } else { self.house(band) };
        }
        let pick = seed.below(20, 8);
        let kind = match pick {
            0 | 1 if self.section >= 3 => Kind::Jet,
            0..=3 => Kind::Ship,
            _ => Kind::Helicopter(0),
        };
        if kind == Kind::Jet {
            // jets fly across the whole screen
            let x = seed.below(4, 160) as u8;
            return Object::new(kind, x, seed.below(28, 2) == 0);
        }
        let object = self.place(kind, band);
        if object.kind == Kind::None && kind == Kind::Ship {
            // no room for a boat: a helicopter instead
            return self.place(Kind::Helicopter(0), band);
        }
        object
    }

    /// Puts an object somewhere on the open water, or nothing if it does
    /// not fit.
    fn place(&self, kind: Kind, band: &Band) -> Object {
        let seed = self.seed;
        let water = band.land.open_water();
        let side = if water[1].1 > water[1].0 && seed.below(29, 2) == 1 { water[1] } else { water[0] };
        let (left, right) = (side.0 + 2, side.1.saturating_sub(2));
        let width = Object::new(kind, 0, false).width();
        if right < left + width {
            return Object::NONE;
        }
        let x = left + seed.below(4, (right - left - width + 1) as u32) as u8;
        let facing_left = kind != Kind::FuelDepot && seed.below(28, 2) == 0;
        Object::new(kind, x, facing_left)
    }

    /// A house on the left or right bank, or nothing if the bank is too
    /// narrow for it.
    fn house(&self, band: &Band) -> Object {
        let seed = self.seed;
        let width = sprites::get("house").width();
        let Some(room) = band.land.bank().checked_sub(width) else {
            return Object::NONE;
        };
        let x = seed.below(4, room as u32 + 1) as u8;
        let x = if seed.below(29, 2) == 0 { x } else { 160 - width - x };
        Object::new(Kind::House, x, seed.below(28, 2) == 0)
    }
}

impl Default for River {
    fn default() -> Self {
        River::new()
    }
}
