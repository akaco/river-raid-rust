//! Lets a simple autopilot play, to check the game runs without breaking.

use delta_run::game::{Game, Input, Phase};
use delta_run::objects::Kind;
use delta_run::screen::{band_at, object_line, row_of};

/// Steers towards the middle of the open water ahead and keeps firing.
fn autopilot(g: &Game, frame: u32) -> Input {
    // water that stays open from a little ahead down to the plane
    // (and keep clear of anything that isn't a fuel tank)
    let mut blocked = [false; 160];
    for y in 120..200u8 {
        let (k, dy) = band_at(&g.river, y);
        let band = &g.river.bands[k];
        for x in 0..160u8 {
            blocked[x as usize] |= band.land.is_land(row_of(dy), x);
        }
        if let Some((art, _)) = object_line(&g.river, k, dy)
            && band.object.kind != Kind::FuelDepot
            && y >= 150
        {
            for x in band.object.pixels(art) {
                for d in 0..10u8 {
                    blocked[(x as usize + 160 + d as usize - 5) % 160] = true; // (a little room either side)
                }
            }
        }
    }
    let open = |x: u8| !blocked[x as usize];
    let plane = g.plane.x + 3;
    // the nearest water to the plane, and its middle
    let mut best = None;
    let mut x = 0u16;
    while x < 160 {
        if open(x as u8) {
            let start = x;
            while x < 160 && open(x as u8) {
                x += 1;
            }
            let run = (start as i32, x as i32);
            let mid = |(a, b): (i32, i32)| (a + b) / 2;
            if best.is_none_or(|b| (mid(run) - plane as i32).abs() < (mid(b) - plane as i32).abs()) {
                best = Some(run);
            }
        }
        x += 1;
    }
    let mut target = best.map_or(plane as i32, |(a, b)| (a + b) / 2);
    // low on fuel: fly over the nearest tank ahead
    if g.fuel < 0x9000 {
        let tank = (100..182u8).rev().find_map(|y| {
            let (k, _) = band_at(&g.river, y);
            let object = g.river.bands[k].object;
            (object.kind == Kind::FuelDepot).then(|| object.x as i32 + 4)
        });
        if let Some(t) = tank.filter(|t| best.is_some_and(|(a, b)| a <= t - 4 && t + 4 < b)) {
            target = t;
        }
    }
    // don't shoot the fuel tank straight ahead
    let fuel_ahead = (60..182u8).rev().find_map(|y| {
        let (k, dy) = band_at(&g.river, y);
        let object = g.river.bands[k].object;
        let art = object_line(&g.river, k, dy)?.0;
        object.pixels(art).any(|x| x == plane).then_some(object.kind == Kind::FuelDepot)
    });
    Input {
        left: (plane as i32) > target + 2,
        right: (plane as i32) < target - 2,
        down: true,
        fire: frame.is_multiple_of(2) && fuel_ahead != Some(true),
        reset: g.phase == Phase::GameOver && frame.is_multiple_of(600),
        ..Input::default()
    }
}

#[test]
fn the_game_runs() {
    let mut g = Game::new();
    let mut furthest = 0;
    for frame in 0..60_000 {
        let input = autopilot(&g, frame);
        g.frame(input);
        furthest = furthest.max(g.river.section);
    }
    assert!(furthest >= 3, "the autopilot only reached section {furthest}");
}
