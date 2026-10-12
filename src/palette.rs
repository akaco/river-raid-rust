//! Colours, as 0xRRGGBB.

pub type Colour = u32;

pub const BLACK: Colour = 0x000000;
pub const WATER: Colour = 0x2A52BE;
/// The water flashes this colour when a bridge blows up.
pub const FLASH: Colour = 0xB03030;
pub const BANK: Colour = 0x4F9A3A;
pub const BANK_DARK: Colour = 0x3D7F2C;
pub const ROAD: Colour = 0x5A5A5A;
pub const ROAD_EDGE: Colour = 0x3E3E3E;
pub const ROAD_LINE: Colour = 0xE8D44D;

pub const STATUS: Colour = 0x24242C;
pub const STATUS_TEXT: Colour = 0xE6E6E6;
pub const NEEDLE: Colour = 0xFFB000;

pub const PLAYER_1: Colour = 0x141414;
/// Player 1's score and spare planes (black would not show on the status bar).
pub const PLAYER_1_TEXT: Colour = 0xF4F4F4;
pub const PLAYER_2: Colour = 0x7FE3FF;

// The pictures' colours are in assets/sprites.txt.
