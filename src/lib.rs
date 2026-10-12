//! Delta Run: fly up a river, shoot what is in the way, and keep the tank
//! full. A small game in the style of the early 1980s, written in plain
//! Rust: the river, its objects, the plane, fuel and score are structs, and
//! the picture is drawn with rectangles.
//!
//! - `river`: banks, islands and bridges, and how new river is made
//! - `objects`: jets, helicopters, boats, fuel tanks, houses
//! - `game`: the plane, fuel, score, lives and the rules for one frame
//! - `collide`: what touches what
//! - `screen`: where things are on screen, and drawing them
//! - `sound`, `synth`: sound effects and a tiny synthesizer
//! - `sprites`, `palette`: pictures and colours

/// The game's name, as the window and the status bar show it.
pub const GAME_NAME: &str = "DELTA RUN";

pub mod collide;
pub mod game;
pub mod objects;
pub mod palette;
pub mod river;
pub mod screen;
pub mod sound;
pub mod sprites;
pub mod synth;
