//! Draws the app icon and the Google Play feature graphic from the game's
//! own pictures and colours, as a PPM image on standard output:
//!
//! `cargo run --example store_art -- icon > icon.ppm` (1024 x 1024)
//! `cargo run --example store_art -- adaptive > adaptive.ppm` (1024 x 1024,
//! plane smaller, for Android launchers that cut the icon to a circle)
//! `cargo run --example store_art -- feature > feature.ppm` (1024 x 500)
//!
//! `store/make-art.sh` runs all three and makes every size the stores and
//! the app projects need.

use delta_run::palette::{self, Colour};
use delta_run::sprites::{self, Sprite};

struct Image {
    w: usize,
    h: usize,
    px: Vec<Colour>,
}

impl Image {
    fn rect(&mut self, x: usize, y: usize, w: usize, h: usize, c: Colour) {
        for yy in y..(y + h).min(self.h) {
            for xx in x..(x + w).min(self.w) {
                self.px[yy * self.w + xx] = c;
            }
        }
    }

    /// Draws a sprite with its top-left corner at (x, y). Each sprite pixel
    /// is `2 * unit` wide and `unit` tall, as in the game.
    fn sprite(&mut self, s: &Sprite, x: usize, y: usize, unit: usize, colour: Option<Colour>) {
        for (row, line) in s.lines.iter().enumerate() {
            for (col, &b) in line.as_bytes().iter().enumerate() {
                if b != b'.' {
                    let c = colour.unwrap_or_else(|| s.colour(b));
                    self.rect(x + col * 2 * unit, y + row * unit, 2 * unit, unit, c);
                }
            }
        }
    }

    /// Writes `text` in the game's font, centred on `cx`.
    fn text(&mut self, text: &str, cx: usize, y: usize, unit: usize, c: Colour) {
        let w = text.len() * 6 * unit - unit;
        let mut x = cx - w / 2;
        for ch in text.chars() {
            if ch != ' ' {
                for (row, line) in sprites::glyph(ch).iter().enumerate() {
                    for (col, b) in line.bytes().enumerate() {
                        if b == b'#' {
                            self.rect(x + col * unit, y + row * unit, unit, unit, c);
                        }
                    }
                }
            }
            x += 6 * unit;
        }
    }

    /// River running up the picture: water between two banks.
    fn river(&mut self, bank: usize) {
        let edge = bank / 8;
        self.rect(0, 0, self.w, self.h, palette::WATER);
        self.rect(0, 0, bank, self.h, palette::BANK);
        self.rect(self.w - bank, 0, bank, self.h, palette::BANK);
        self.rect(bank - edge, 0, edge, self.h, palette::BANK_DARK);
        self.rect(self.w - bank, 0, edge, self.h, palette::BANK_DARK);
    }

    fn print_ppm(&self) {
        print!("P3\n{} {}\n255\n", self.w, self.h);
        for c in &self.px {
            println!("{} {} {}", c >> 16, (c >> 8) & 0xFF, c & 0xFF);
        }
    }
}

/// The plane flying up the middle of the river with a shot in front of it.
/// `unit` sets the size of everything.
fn icon(unit: usize) -> Image {
    let size = 1024;
    let mut img = Image { w: size, h: size, px: vec![0; size * size] };
    img.river(size * 3 / 16);
    let plane = sprites::get("plane");
    let pw = plane.width() as usize * 2 * unit;
    // plane and shot together, centred; the sprite's last column is empty
    let (px, py) = ((size - pw) / 2 + unit, (size - 15 * unit) / 2 + 4 * unit);
    img.sprite(plane, px, py, unit, Some(palette::PLAYER_1));
    // the shot leaves from the nose (column 3 of the sprite)
    img.rect(px + 3 * 2 * unit, py - 4 * unit, 2 * unit, 3 * unit, palette::ROAD_LINE);
    img
}

fn feature() -> Image {
    let (w, h) = (1024, 500);
    let mut img = Image { w, h, px: vec![palette::WATER; w * h] };
    img.rect(0, 0, w, 60, palette::BANK);
    img.rect(0, 60, w, 8, palette::BANK_DARK);
    img.rect(0, h - 60, w, 60, palette::BANK);
    img.rect(0, h - 68, w, 8, palette::BANK_DARK);
    img.text(delta_run::GAME_NAME, w / 2, 120, 16, palette::STATUS_TEXT);
    let plane = sprites::get("plane");
    img.sprite(plane, w / 2 - 7 * 10 + 10, 270, 10, Some(palette::PLAYER_1));
    let jet = sprites::get("jet");
    img.sprite(jet, 140, 300, 6, None);
    let ship = sprites::get("ship");
    img.sprite(ship, w - 140 - ship.width() as usize * 12, 320, 6, None);
    img
}

fn main() {
    match std::env::args().nth(1).as_deref() {
        Some("icon") => icon(36).print_ppm(),
        Some("adaptive") => icon(24).print_ppm(),
        Some("feature") => feature().print_ppm(),
        _ => eprintln!("usage: store_art icon|adaptive|feature > picture.ppm"),
    }
}
