//! Where everything is on screen, and drawing it.
//!
//! The picture is 160 pixels wide (each pixel is drawn twice as wide as
//! tall). The river takes lines 39-199 and the status bar lines 200-237.

use crate::game::{Game, Phase, PlaneSprite};
use crate::palette::{self, Colour};
use crate::river::{BANDS, Band, River};
use crate::sprites::{self, Sprite};

pub const WIDTH: u8 = 160;
pub const RIVER_TOP: u8 = 39;
pub const RIVER_BOTTOM: u8 = 199;
pub const STATUS_TOP: u8 = 200;
pub const STATUS_BOTTOM: u8 = 237;
pub const BAND_LINES: u8 = 32;

/// Anything that can fill rectangles.
pub trait Canvas {
    fn rect(&mut self, x: u8, y: u8, w: u8, h: u8, colour: Colour);
}

/// The band showing river line `y`, and the line within it (0 = top).
pub fn band_at(river: &River, y: u8) -> (usize, u8) {
    // the top band's top line is above the screen while it scrolls in
    let d = y - (RIVER_TOP - BAND_LINES + 1 + river.scroll_line);
    (BANDS - 1 - (d / BAND_LINES) as usize, d % BAND_LINES)
}

/// Land row (0 = bottom, 15 = top) of line `dy` of a band.
pub fn row_of(dy: u8) -> u8 {
    15 - dy / 2
}

/// The sprite an object shows on line `dy` of its band, and that line of it.
pub fn object_line(river: &River, band: usize, dy: u8) -> Option<(&'static Sprite, &'static str)> {
    let sprite = river.bands[band].object.kind.sprite()?;
    Some((sprite, sprite_line(sprite, dy)?))
}

fn sprite_line(sprite: &'static Sprite, dy: u8) -> Option<&'static str> {
    let i = dy.checked_sub(sprite.first_line)? as usize;
    sprite.lines.get(i).map(String::as_str)
}

fn plane_sprite(s: PlaneSprite) -> Option<&'static Sprite> {
    match s {
        PlaneSprite::Hidden => None,
        PlaneSprite::Straight => Some(sprites::get("plane")),
        PlaneSprite::BankedLeft => Some(sprites::get("plane_left")),
        PlaneSprite::BankedRight => Some(sprites::get("plane_right")),
        PlaneSprite::Crashed => Some(sprites::get("plane_crashed")),
    }
}

/// The plane's pixels on line `y`.
pub fn plane_pixels(game: &Game, y: u8) -> impl Iterator<Item = u8> {
    let art = plane_sprite(game.plane.sprite).and_then(|s| sprite_line(s, y.checked_sub(sprites::PLANE_TOP)?));
    let x0 = game.plane.x;
    art.into_iter()
        .flat_map(move |art| art.bytes().enumerate().filter(|(_, b)| *b != b'.').map(move |(i, _)| (x0 as usize + i) as u8 % WIDTH))
}

/// The missile's pixel on line `y`, if it is there. It is 8 lines tall and
/// flies up from the plane's nose.
pub fn missile_pixel(game: &Game, y: u8) -> Option<u8> {
    let top = 193 - game.missile.y as i16;
    let y = y as i16;
    let visible = y > RIVER_TOP as i16 && y < sprites::PLANE_TOP as i16 && y >= top && y < top + 8;
    let x = game.missile.x;
    (visible && x < WIDTH).then_some(x)
}

fn bank_colour(band: &Band, row: u8) -> Colour {
    if band.bridge && (4..=11).contains(&row) {
        // the road to the bridge
        match row {
            4 | 11 => palette::ROAD_EDGE,
            8 => palette::ROAD_LINE,
            _ => palette::ROAD,
        }
    } else if band.dark_banks {
        palette::BANK_DARK
    } else {
        palette::BANK
    }
}

/// Draws one frame.
pub fn draw(game: &Game, c: &mut impl Canvas) {
    let river = &game.river;

    // river
    let water = if game.flashing { palette::FLASH } else { palette::WATER };
    c.rect(0, RIVER_TOP, WIDTH, RIVER_BOTTOM - RIVER_TOP + 1, water);
    for y in RIVER_TOP..=RIVER_BOTTOM {
        let (k, dy) = band_at(river, y);
        let band = &river.bands[k];
        let row = row_of(dy);
        let (edge, centre) = band.land.row(row);
        let colour = bank_colour(band, row);
        let edge_px = edge * 4;
        c.rect(0, y, edge_px, 1, colour);
        c.rect(WIDTH - edge_px, y, edge_px, 1, colour);
        if centre > 0 {
            c.rect(80 - centre * 4, y, centre * 8, 1, colour);
        }
        if let Some((sprite, art)) = object_line(river, k, dy) {
            for (x, letter) in band.object.pixels(art) {
                c.rect(x, y, 1, 1, sprite.colour(letter));
            }
        }
        if let Some(x) = missile_pixel(game, y) {
            c.rect(x, y, 1, 1, game.plane_colour());
        }
        for x in plane_pixels(game, y) {
            c.rect(x, y, 1, 1, game.plane_colour());
        }
    }

    // status bar: score, fuel gauge, spare planes and the title
    c.rect(0, STATUS_TOP, WIDTH, 1, palette::BLACK);
    c.rect(0, STATUS_TOP + 1, WIDTH, STATUS_BOTTOM - STATUS_TOP, palette::STATUS);
    let player = game.score_colour();
    for (i, digit) in game.score.digits.iter().enumerate() {
        if let Some(d) = digit {
            let ch = if *d < 10 { (b'0' + d) as char } else { '!' };
            draw_glyph(c, 57 + 8 * i as u8, 203, ch, player);
        }
    }
    draw_gauge(c, game.fuel);
    if game.phase != Phase::GameOver
        && let Some(d) = game.lives.digit()
    {
        draw_glyph(c, 20, 228, (b'0' + d) as char, player);
    }
    draw_text(c, 228, crate::GAME_NAME, palette::STATUS_TEXT);
}

/// The fuel gauge: a box from E (empty) to F (full) with a moving needle.
fn draw_gauge(c: &mut impl Canvas, fuel: u16) {
    let (left, top, w, h) = (52, 212, 56, 14);
    let ink = palette::STATUS_TEXT;
    c.rect(left, top, w, 1, ink);
    c.rect(left, top + h - 1, w, 1, ink);
    c.rect(left, top, 1, h, ink);
    c.rect(left + w - 1, top, 1, h, ink);
    for tick in [66, 79, 93] {
        c.rect(tick, top + 1, 1 + (tick == 79) as u8, 2, ink);
    }
    let needle = 61 + ((fuel >> 8) * 36 / 255) as u8;
    c.rect(needle, top + 3, 2, 9, palette::NEEDLE);
    draw_glyph(c, left + 3, top + 4, 'E', ink);
    draw_glyph(c, left + w - 8, top + 4, 'F', ink);
}

/// Text centred across the screen.
fn draw_text(c: &mut impl Canvas, y: u8, text: &str, colour: Colour) {
    let width = text.chars().count() as u8 * 6 - 1;
    for (i, ch) in text.chars().enumerate() {
        draw_glyph(c, (WIDTH - width) / 2 + 6 * i as u8, y, ch, colour);
    }
}

fn draw_glyph(c: &mut impl Canvas, x: u8, y: u8, ch: char, colour: Colour) {
    for (j, line) in sprites::glyph(ch).iter().enumerate() {
        for (i, b) in line.bytes().enumerate() {
            if b == b'#' {
                c.rect(x + i as u8, y + j as u8, 1, 1, colour);
            }
        }
    }
}
