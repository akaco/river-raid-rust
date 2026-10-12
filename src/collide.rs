//! What touches what. Things collide when their drawn pixels overlap, so
//! a near miss is a miss.

use crate::game::Game;
use crate::river::BANDS;
use crate::screen::{self, RIVER_BOTTOM, RIVER_TOP, WIDTH};

#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub struct Collisions {
    /// The band whose object the plane touched (the lowest, if several).
    pub plane_touched: Option<usize>,
    pub plane_on_bank: bool,
    /// The band whose object the missile hit (the lowest, if several).
    pub missile_hit: Option<usize>,
    pub missile_on_bank: bool,
    /// Each band's object touches the bank (moving things turn round).
    pub object_on_bank: [bool; BANDS],
}

pub fn check(game: &Game) -> Collisions {
    let river = &game.river;
    let mut c = Collisions::default();
    let lowest = |a: Option<usize>, b: usize| Some(a.map_or(b, |a| a.min(b)));
    for y in RIVER_TOP..=RIVER_BOTTOM {
        let (k, dy) = screen::band_at(river, y);
        let band = &river.bands[k];
        let row = screen::row_of(dy);
        let land = |x: u8| band.land.is_land(row, x);

        let mut object = [false; WIDTH as usize];
        if let Some((art, _)) = screen::object_line(river, k, dy) {
            for x in band.object.pixels(art) {
                object[x as usize] = true;
                c.object_on_bank[k] |= land(x);
            }
        }
        for x in screen::plane_pixels(game, y) {
            c.plane_on_bank |= land(x);
            if object[x as usize] {
                c.plane_touched = lowest(c.plane_touched, k);
            }
        }
        if let Some(x) = screen::missile_pixel(game, y) {
            c.missile_on_bank |= land(x);
            if object[x as usize] {
                c.missile_hit = lowest(c.missile_hit, k);
            }
        }
    }
    c
}
